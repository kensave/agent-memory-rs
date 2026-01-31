use anyhow::Result;
use memory_rs::mcp::MemoryMcpServer;
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

    tracing::info!("Using workspace: {}", workspace_name);

    // Create and serve the server via stdio
    let service = MemoryMcpServer::new(&workspace_name)?
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
