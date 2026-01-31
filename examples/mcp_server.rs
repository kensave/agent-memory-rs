use memory_rs::{WorkspaceManager, ModelType, mcp::{McpServer, MemoryTools}};
use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr only (stdout is for JSON-RPC)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Get workspace name from args or use default
    let workspace_name = std::env::args()
        .nth(1)
        .or_else(|| WorkspaceManager::detect_workspace_from_cwd())
        .unwrap_or_else(|| "default".to_string());

    // Initialize workspace manager and memory system
    let manager = WorkspaceManager::new(ModelType::MiniLM)?;
    let memory_system = manager.get_or_create_workspace(&workspace_name)?;

    // Create MCP tools
    let tools = MemoryTools::new(memory_system);

    // Create and run MCP server
    let server = McpServer::new("memory-rs", "0.1.0");
    
    server.run(move |request| {
        tools.handle_request(&request.method, request.params)
    }).await?;

    Ok(())
}
