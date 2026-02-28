use crate::{WorkspaceManager, ModelType, MemorySystem};
use crate::services::memory_manager::MemoryManager;
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
use tokio::sync::Mutex;

pub struct MemoryMcpServer {
    memory_system: Arc<Mutex<MemorySystem>>,
    memory_manager: Arc<MemoryManager>,
    workspace_id: i64,
    message_count: Arc<Mutex<usize>>,
    consolidation_threshold: usize,
    initialized: Arc<Mutex<bool>>,
}

impl MemoryMcpServer {
    pub fn new(workspace_name: &str, model_type: ModelType) -> Result<Self> {
        let manager = WorkspaceManager::new(model_type)?;
        let memory_system = manager.get_or_create_workspace(workspace_name)?;
        
        // Get the workspace ID
        let workspace_id = memory_system.database().execute(|conn| {
            let id: i64 = conn.query_row(
                "SELECT id FROM workspaces WHERE name = ?1",
                [workspace_name],
                |row| row.get(0),
            )?;
            Ok(id)
        })?;
        
        // Create MemoryManager using the SAME database and embedder
        let db = memory_system.database().clone();
        let embedder = memory_system.embedder();
        let memory_manager = Arc::new(MemoryManager::with_embedder(db, embedder));
        
        Ok(Self {
            memory_system: Arc::new(Mutex::new(memory_system)),
            memory_manager,
            workspace_id,
            message_count: Arc::new(Mutex::new(0)),
            consolidation_threshold: 20, // Default: every 20 messages
            initialized: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Initialize server - consolidates yesterday's memories
    pub async fn initialize(&self) -> Result<()> {
        println!("🚀 Memory MCP Server initializing...");
        
        // Consolidate yesterday in background
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d").to_string();
        
        let manager = Arc::clone(&self.memory_manager);
        tokio::spawn(async move {
            println!("🔄 Consolidating memories from {}...", yesterday);
            match manager.consolidate(yesterday).await {
                Ok(synopsis) => {
                    println!("✅ Consolidation complete: {} insights extracted", 
                             synopsis.key_insights.len());
                }
                Err(e) => {
                    eprintln!("⚠️  Consolidation failed (non-fatal): {}", e);
                }
            }
        });
        
        println!("✅ Memory MCP Server ready");
        Ok(())
    }
    
    /// Ensure model is loaded on first use
    async fn ensure_initialized(&self) -> Result<(), ErrorData> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let log_path = "/tmp/mcp-memory-debug.log";
        let mut log = OpenOptions::new().create(true).append(true).open(log_path).ok();
        
        if let Some(ref mut f) = log {
            let _ = writeln!(f, "[{}] ensure_initialized called", chrono::Local::now().format("%H:%M:%S%.3f"));
        }
        
        let mut init = self.initialized.lock().await;
        
        if let Some(ref mut f) = log {
            let _ = writeln!(f, "[{}] Lock acquired, initialized={}", chrono::Local::now().format("%H:%M:%S%.3f"), *init);
        }
        
        if !*init {
            *init = true;
            drop(init);
            
            if let Some(ref mut f) = log {
                let _ = writeln!(f, "[{}] Starting model load...", chrono::Local::now().format("%H:%M:%S%.3f"));
            }
            
            // Load embedding model
            tracing::info!("Loading embedding model on first use...");
            let system = self.memory_system.lock().await;
            
            if let Some(ref mut f) = log {
                let _ = writeln!(f, "[{}] System lock acquired, calling load_model...", chrono::Local::now().format("%H:%M:%S%.3f"));
            }
            
            system.load_model().map_err(|e| ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Failed to load model: {}", e),
                None,
            ))?;
            
            if let Some(ref mut f) = log {
                let _ = writeln!(f, "[{}] Model loaded successfully!", chrono::Local::now().format("%H:%M:%S%.3f"));
            }
            
            drop(system);
            tracing::info!("Model loaded successfully");
            
            if let Some(ref mut f) = log {
                let _ = writeln!(f, "[{}] Starting background consolidation...", chrono::Local::now().format("%H:%M:%S%.3f"));
            }
            
            // Run initial consolidation in background
            tracing::info!("Running initial consolidation for yesterday");
            let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
                .format("%Y-%m-%d").to_string();
            
            let manager = Arc::clone(&self.memory_manager);
            tokio::spawn(async move {
                match manager.consolidate(yesterday).await {
                    Ok(synopsis) => {
                        tracing::info!("Initial consolidation complete: {} insights", 
                                     synopsis.key_insights.len());
                    }
                    Err(e) => {
                        tracing::warn!("Initial consolidation failed: {}", e);
                    }
                }
            });
        }
        
        if let Some(ref mut f) = log {
            let _ = writeln!(f, "[{}] ensure_initialized complete", chrono::Local::now().format("%H:%M:%S%.3f"));
        }
        
        Ok(())
    }
    
    /// Check if consolidation needed after message
    async fn check_consolidation(&self) {
        let mut count = self.message_count.lock().await;
        *count += 1;
        
        if *count >= self.consolidation_threshold {
            *count = 0;
            drop(count);
            
            tracing::info!("Threshold reached, running consolidation");
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            
            let manager = Arc::clone(&self.memory_manager);
            tokio::spawn(async move {
                match manager.consolidate(today).await {
                    Ok(synopsis) => {
                        tracing::info!("Consolidation complete: {} insights", 
                                     synopsis.key_insights.len());
                    }
                    Err(e) => {
                        tracing::warn!("Consolidation failed: {}", e);
                    }
                }
            });
        }
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
    #[serde(default = "default_event_type")]
    event_type: String,
    #[serde(default)]
    context: Option<serde_json::Value>,
    #[serde(default)]
    timestamp: Option<String>,
}

fn default_event_type() -> String {
    "user_input".to_string()
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
        // Ensure model is loaded BEFORE acquiring any locks
        self.ensure_initialized().await?;
        
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

                let importance = input.importance_score.unwrap_or(0.5);
                
                // Store as Episode
                let episode = crate::models::dtos::Episode {
                    id: None,
                    workspace_id: self.workspace_id,
                    agent_id: input.agent_id,
                    timestamp: input.timestamp.unwrap_or_else(|| chrono::Local::now().to_rfc3339()),
                    conversation_id: input.conversation_id.clone(),
                    event_type: input.event_type,
                    context: input.context.unwrap_or_else(|| serde_json::json!({"text": input.text.clone()})),
                    outcome: None,
                    valence: None,
                    archived: false,
                    created_at: None,
                };
                
                let episode_id = self.memory_manager.store_episode(episode).await
                    .map_err(|e| ErrorData::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("Failed to store episode: {}", e),
                        None,
                    ))?;
                
                // Optionally store in semantic memory if high importance
                let memory_id = if importance > 0.7 {
                    let memory = crate::storage::Memory {
                        id: None,
                        workspace_id: self.workspace_id,
                        agent_id: input.agent_id,
                        text: input.text,
                        tags: input.tags,
                        importance_score: importance,
                        access_count: 0,
                        last_accessed: None,
                        conversation_id: input.conversation_id,
                        parent_memory_id: None,
                        user_feedback: None,
                        source_episodes: vec![episode_id],
                        confidence: 0.8,
                        last_validated: None,
                        created_at: None,
                        updated_at: None,
                    };
                    
                    Some(system.learn(&memory).map_err(|e| ErrorData::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("Failed to learn: {}", e),
                        None,
                    ))?)
                } else {
                    None
                };
                
                // Drop lock before async operation
                drop(system);
                
                // Check if consolidation needed
                self.check_consolidation().await;

                Ok(CallToolResult::success(vec![Content::text(json!({
                    "episode_id": episode_id,
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

                // Drop system lock, use manager for hierarchical retrieval
                drop(system);
                
                let results = self.memory_manager
                    .retrieve_hierarchical(&input.query, self.workspace_id, input.limit.min(100))
                    .map_err(|e| ErrorData::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("Failed to search: {}", e),
                        None,
                    ))?;
                
                // Check if consolidation needed
                self.check_consolidation().await;

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
