use anyhow::Result;
use clap::Parser;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, ErrorCode, ErrorData, ListToolsResult,
    PaginatedRequestParam, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_client::StreamableHttpClientWorker;
use rmcp::{transport::stdio, ServerHandler, ServiceExt};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "agent-memory-proxy",
    about = "Stdio-to-HTTP proxy for remote Memory MCP server"
)]
struct Args {
    /// Remote server URL (e.g. http://192.168.1.100:8230/mcp)
    #[arg(long)]
    remote: String,
}

type RemoteClient = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

struct ProxyServer {
    client: Arc<RemoteClient>,
}

impl ProxyServer {
    async fn connect(url: &str) -> Result<Self> {
        let worker = StreamableHttpClientWorker::<reqwest::Client>::new_simple(url);
        let client = ().serve(worker).await?;
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

impl ServerHandler for ProxyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some("Memory-RS proxy: forwards to remote server.".to_string()),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        self.client.list_tools(request).await.map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Remote error: {}", e),
                None,
            )
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.client.call_tool(request).await.map_err(|e| {
            ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Remote error: {}", e),
                None,
            )
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();
    tracing::info!("Connecting to remote: {}", args.remote);

    let proxy = ProxyServer::connect(&args.remote).await?;
    tracing::info!("Connected, starting stdio server");

    let service = proxy.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
