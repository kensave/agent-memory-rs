use crate::{WorkspaceManager, ModelType, MemorySystem};
use anyhow::Result;
use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorCode, ErrorData,
        ListToolsResult, PaginatedRequestParam, ServerCapabilities,
        ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
    ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

pub struct MemoryMcpServer {
    memory_system: Arc<tokio::sync::Mutex<MemorySystem>>,
    workspace_id: i64,
}

impl MemoryMcpServer {
    pub fn new(workspace_name: &str) -> Result<Self> {
        let manager = WorkspaceManager::new(ModelType::MiniLM)?;
        let memory_system = manager.get_or_create_workspace(workspace_name)?;
        
        // Get the workspace ID
        let workspace_id = {
            let conn = memory_system.database().connection();
            conn.query_row(
                "SELECT id FROM workspaces WHERE name = ?1",
                [workspace_name],
                |row| row.get(0),
            )?
        };
        
        Ok(Self {
            memory_system: Arc::new(tokio::sync::Mutex::new(memory_system)),
            workspace_id,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LearnInput {
    text: String,
    #[serde(default)]
    agent_id: Option<i64>,
    #[serde(default)]
    tags: Option<String>,
    #[serde(default)]
    importance_score: Option<f64>,
    #[serde(default)]
    conversation_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchInput {
    query: String,
    #[serde(default)]
    agent_id: Option<i64>,
    #[serde(default)]
    min_importance: Option<f64>,
    #[serde(default)]
    max_importance: Option<f64>,
    #[serde(default)]
    conversation_id: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

impl ServerHandler for MemoryMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some(
                "Memory-RS: Persistent memory with semantic search. \
                All memories are AUTOMATICALLY indexed with MiniLM 384d embeddings. \
                Memories MUST include text content. \
                Memories are scoped to current workspace and persist across sessions.".to_string(),
            ),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = vec![
            Tool {
                name: "learn".into(),
                description: Some("Store a memory with automatic semantic indexing. Text MUST be provided. Text is embedded using MiniLM (384d) and stored in SQLite. All memories are AUTOMATICALLY indexed for semantic search. MAY include optional metadata: tags, importance_score (0-1), conversation_id, agent_id.".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The text to remember"
                        },
                        "agent_id": {
                            "type": "integer",
                            "description": "Optional agent ID"
                        },
                        "tags": {
                            "type": "string",
                            "description": "Optional comma-separated tags"
                        },
                        "importance_score": {
                            "type": "number",
                            "description": "Importance score 0-1 (default 0.5)"
                        },
                        "conversation_id": {
                            "type": "string",
                            "description": "Optional conversation ID"
                        }
                    },
                    "required": ["text"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
            Tool {
                name: "search".into(),
                description: Some("Search memories using semantic similarity. Query MUST be provided. Returns ranked results combining cosine similarity (70%) and importance score (30%). All stored memories are searchable. MAY filter by: agent_id, importance range, conversation_id. Default limit is 10, max 100.".into()),
                input_schema: Arc::new(serde_json::from_value(json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "agent_id": {
                            "type": "integer",
                            "description": "Optional agent ID filter"
                        },
                        "min_importance": {
                            "type": "number",
                            "description": "Minimum importance score"
                        },
                        "max_importance": {
                            "type": "number",
                            "description": "Maximum importance score"
                        },
                        "conversation_id": {
                            "type": "string",
                            "description": "Optional conversation ID filter"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results (default 10, max 100)"
                        }
                    },
                    "required": ["query"]
                })).unwrap()),
                output_schema: None,
                annotations: None,
                icons: None,
                title: None,
            },
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let system = self.memory_system.lock().await;

        match request.name.as_ref() {
            "learn" => {
                let args = request.arguments
                    .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
                let input: LearnInput = serde_json::from_value(serde_json::Value::Object(args))
                    .map_err(|e| ErrorData::new(
                        ErrorCode::INVALID_PARAMS,
                        format!("Invalid input: {}", e),
                        None,
                    ))?;

                let memory = crate::storage::Memory {
                    id: None,
                    workspace_id: self.workspace_id,
                    agent_id: input.agent_id,
                    text: input.text,
                    tags: input.tags,
                    importance_score: input.importance_score.unwrap_or(0.5),
                    access_count: 0,
                    last_accessed: None,
                    conversation_id: input.conversation_id,
                    parent_memory_id: None,
                    user_feedback: None,
                    created_at: None,
                    updated_at: None,
                };

                let memory_id = system
                    .learn(&memory)
                    .map_err(|e| ErrorData::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("Failed to learn: {}", e),
                        None,
                    ))?;

                Ok(CallToolResult::success(vec![Content::text(json!({
                    "memory_id": memory_id,
                    "status": "success"
                }).to_string())]))
            }
            "search" => {
                let args = request.arguments
                    .ok_or_else(|| ErrorData::new(ErrorCode::INVALID_PARAMS, "Missing arguments", None))?;
                let input: SearchInput = serde_json::from_value(serde_json::Value::Object(args))
                    .map_err(|e| ErrorData::new(
                        ErrorCode::INVALID_PARAMS,
                        format!("Invalid input: {}", e),
                        None,
                    ))?;

                let filters = crate::storage::SearchFilters {
                    workspace_id: Some(self.workspace_id),
                    agent_id: input.agent_id,
                    min_importance: input.min_importance,
                    max_importance: input.max_importance,
                    conversation_id: input.conversation_id,
                    ..Default::default()
                };

                let results = system
                    .search(&input.query, &filters, input.limit.min(100))
                    .map_err(|e| ErrorData::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("Failed to search: {}", e),
                        None,
                    ))?;

                Ok(CallToolResult::success(vec![Content::text(json!({
                    "results": results,
                    "count": results.len()
                }).to_string())]))
            }
            _ => Err(ErrorData::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }
}
