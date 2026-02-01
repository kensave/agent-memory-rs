use anyhow::Result;
use agent_memory_rs::mcp::MemoryMcpServer;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with stderr output
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Memory-RS MCP Server");

    // Get workspace name from args, or auto-detect from current directory
    let workspace_name = std::env::args()
        .nth(1)
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        })
        .unwrap_or_else(|| "default".to_string());

    // Get model type from environment variable or default to BGE-Small
    let model_type = std::env::var("MEMORY_MODEL")
        .ok()
        .and_then(|m| match m.to_lowercase().as_str() {
            "minilm" => Some(agent_memory_rs::ModelType::MiniLM),
            "nomic" => Some(agent_memory_rs::ModelType::Nomic),
            "bge" | "bge-small" => Some(agent_memory_rs::ModelType::BgeSmall),
            _ => None,
        })
        .unwrap_or(agent_memory_rs::ModelType::BgeSmall);

    tracing::info!("Using workspace: {}", workspace_name);
    tracing::info!("Using model: {:?}", model_type);

    // Create server
    let server = MemoryMcpServer::new(&workspace_name, model_type)?;
    
    tracing::info!("Memory MCP Server ready");
    
    // Serve via stdio
    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
