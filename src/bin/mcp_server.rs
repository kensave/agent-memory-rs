use agent_memory_rs::mcp::MemoryMcpServer;
use anyhow::Result;
use clap::Parser;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpService,
};
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "agent-memory-mcp", about = "Memory MCP server")]
struct Args {
    /// Workspace name (auto-detected from cwd if omitted)
    workspace: Option<String>,

    /// Run as standalone HTTP server (e.g. --http 0.0.0.0:8230)
    #[arg(long)]
    http: Option<String>,
}

fn resolve_workspace(name: Option<String>) -> String {
    name.or_else(|| {
        std::env::current_dir().ok().map(|p| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            p.to_string_lossy().hash(&mut hasher);
            let hash = hasher.finish();
            let dir_name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "root".to_string());
            format!("{:08x}-{}", hash, dir_name)
        })
    })
    .unwrap_or_else(|| "default".to_string())
}

fn resolve_model() -> agent_memory_rs::ModelType {
    std::env::var("MEMORY_MODEL")
        .ok()
        .and_then(|m| match m.to_lowercase().as_str() {
            "minilm" => Some(agent_memory_rs::ModelType::MiniLM),
            "nomic" => Some(agent_memory_rs::ModelType::Nomic),
            "bge" | "bge-small" => Some(agent_memory_rs::ModelType::BgeSmall),
            _ => None,
        })
        .unwrap_or(agent_memory_rs::ModelType::BgeSmall)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();
    let workspace_name = resolve_workspace(args.workspace);
    let model_type = resolve_model();

    tracing::info!("workspace: {}, model: {:?}", workspace_name, model_type);

    if let Some(bind_addr) = args.http {
        // Standalone HTTP server mode
        let service = StreamableHttpService::new(
            move || {
                MemoryMcpServer::new(&workspace_name, model_type)
                    .map_err(|e| std::io::Error::other(format!("{}", e)))
            },
            LocalSessionManager::default().into(),
            Default::default(),
        );
        let router = axum::Router::new().nest_service("/mcp", service);
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        tracing::info!("HTTP server listening on {}", bind_addr);
        axum::serve(listener, router)
            .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
            .await?;
    } else {
        // Stdio mode (launched by Kiro)
        let server = MemoryMcpServer::new(&workspace_name, model_type)?;
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
    }

    Ok(())
}
