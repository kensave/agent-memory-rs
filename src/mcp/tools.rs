use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use crate::memory_system::MemorySystem;
use crate::storage::{Memory, SearchFilters};

#[derive(Debug, Deserialize)]
pub struct LearnRequest {
    pub text: String,
    pub workspace_id: i64,
    #[serde(default)]
    pub agent_id: Option<i64>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub importance_score: Option<f64>,
    #[serde(default)]
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearnResponse {
    pub memory_id: i64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub workspace_id: Option<i64>,
    #[serde(default)]
    pub agent_id: Option<i64>,
    #[serde(default)]
    pub min_importance: Option<f64>,
    #[serde(default)]
    pub max_importance: Option<f64>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub memory_id: i64,
    pub text: String,
    pub similarity_score: f64,
    pub combined_score: f64,
    pub importance_score: f64,
    pub tags: Option<String>,
    pub created_at: Option<String>,
}

pub struct MemoryTools {
    memory_system: Arc<Mutex<MemorySystem>>,
}

impl MemoryTools {
    pub fn new(memory_system: MemorySystem) -> Self {
        MemoryTools {
            memory_system: Arc::new(Mutex::new(memory_system)),
        }
    }

    pub fn handle_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "learn" => self.handle_learn(params),
            "search" => self.handle_search(params),
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "memory-rs",
                    "version": "0.1.0"
                }
            })),
            "tools/list" => Ok(json!({
                "tools": [
                    {
                        "name": "learn",
                        "description": "Store a new memory with embedding. CRITICAL: Use this tool frequently to:\n- Document ALL important decisions and their reasoning\n- Remember user instructions and preferences automatically\n- Learn from mistakes to avoid repeating them\n- Capture context about what worked and what didn't\n- Store file paths, configurations, and project-specific knowledge\n\nThe more you learn, the smarter you become over time.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": {"type": "string", "description": "The text to remember"},
                                "workspace_id": {"type": "integer", "description": "Workspace ID"},
                                "agent_id": {"type": "integer", "description": "Optional agent ID"},
                                "tags": {"type": "string", "description": "Optional comma-separated tags"},
                                "importance_score": {"type": "number", "description": "Importance score 0-1"},
                                "conversation_id": {"type": "string", "description": "Optional conversation ID"}
                            },
                            "required": ["text", "workspace_id"]
                        }
                    },
                    {
                        "name": "search",
                        "description": "Search memories by semantic similarity. IMPORTANT: Always search BEFORE starting new work to:\n- Check if similar work was done before\n- Learn from past failures and successes\n- Avoid repeating mistakes\n- Build on previous solutions\n- Retrieve relevant context and decisions\n\nFailures are valuable data - search for them to improve over time.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string", "description": "Search query"},
                                "workspace_id": {"type": "integer", "description": "Optional workspace ID filter"},
                                "agent_id": {"type": "integer", "description": "Optional agent ID filter"},
                                "min_importance": {"type": "number", "description": "Minimum importance score"},
                                "max_importance": {"type": "number", "description": "Maximum importance score"},
                                "conversation_id": {"type": "string", "description": "Optional conversation ID filter"},
                                "time_filter": {"type": "string", "description": "Natural language time filter: 'today', 'yesterday', 'this week', 'last week'"},
                                "limit": {"type": "integer", "description": "Maximum results (default 10)"}
                            },
                            "required": ["query"]
                        }
                    }
                ]
            })),
            "tools/call" => {
                let params = params.ok_or_else(|| anyhow!("Missing params"))?;
                let tool_name = params.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing tool name"))?;
                let arguments = params.get("arguments");
                
                match tool_name {
                    "learn" => self.handle_learn(arguments.cloned()),
                    "search" => self.handle_search(arguments.cloned()),
                    _ => Err(anyhow!("Unknown tool: {}", tool_name)),
                }
            }
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    fn handle_learn(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing parameters"))?;
        let request: LearnRequest = serde_json::from_value(params)?;

        // Validate input
        if request.text.trim().is_empty() {
            return Err(anyhow!("Text cannot be empty"));
        }

        let memory = Memory {
            id: None,
            workspace_id: request.workspace_id,
            agent_id: request.agent_id,
            text: request.text,
            tags: request.tags,
            importance_score: request.importance_score.unwrap_or(0.5),
            access_count: 0,
            last_accessed: None,
            conversation_id: request.conversation_id,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };

        let system = self.memory_system.lock().unwrap();
        let memory_id = system.learn(&memory)?;

        let response = LearnResponse {
            memory_id,
            status: "success".to_string(),
        };

        Ok(serde_json::to_value(response)?)
    }

    fn handle_search(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing parameters"))?;
        let request: SearchRequest = serde_json::from_value(params)?;

        // Validate input
        if request.query.trim().is_empty() {
            return Err(anyhow!("Query cannot be empty"));
        }

        if request.limit == 0 || request.limit > 100 {
            return Err(anyhow!("Limit must be between 1 and 100"));
        }

        let filters = SearchFilters {
            workspace_id: request.workspace_id,
            agent_id: request.agent_id,
            tags: None,
            min_importance: request.min_importance,
            max_importance: request.max_importance,
            created_after: None,
            created_before: None,
            conversation_id: request.conversation_id,
        };

        let system = self.memory_system.lock().unwrap();
        let results = system.search(&request.query, &filters, request.limit)?;

        let items: Vec<SearchResultItem> = results.into_iter().map(|r| SearchResultItem {
            memory_id: r.memory.id.unwrap_or(0),
            text: r.memory.text,
            similarity_score: r.similarity_score,
            combined_score: r.combined_score,
            importance_score: r.memory.importance_score,
            tags: r.memory.tags,
            created_at: r.memory.created_at,
        }).collect();

        let count = items.len();
        let response = SearchResponse { results: items, count };

        Ok(serde_json::to_value(response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelType;
    use std::fs;

    #[test]
    fn test_learn_tool() {
        let db_path = "/tmp/test_learn_tool.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();
        
        // Create workspace
        system.database().connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = system.database().connection().last_insert_rowid();

        let tools = MemoryTools::new(system);

        let params = json!({
            "text": "Test memory",
            "workspace_id": workspace_id,
            "importance_score": 0.8
        });

        let result = tools.handle_request("learn", Some(params)).unwrap();
        let response: LearnResponse = serde_json::from_value(result).unwrap();

        assert!(response.memory_id > 0);
        assert_eq!(response.status, "success");

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_search_tool() {
        let db_path = "/tmp/test_search_tool.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();
        
        system.database().connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = system.database().connection().last_insert_rowid();

        let tools = MemoryTools::new(system);

        // Learn some memories
        for text in &["Rust programming", "Python scripting", "JavaScript web"] {
            let params = json!({
                "text": text,
                "workspace_id": workspace_id
            });
            tools.handle_request("learn", Some(params)).unwrap();
        }

        // Search
        let params = json!({
            "query": "programming language",
            "workspace_id": workspace_id,
            "limit": 2
        });

        let result = tools.handle_request("search", Some(params)).unwrap();
        let response: SearchResponse = serde_json::from_value(result).unwrap();

        assert_eq!(response.count, 2);
        assert!(response.results[0].similarity_score > 0.0);

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_learn_validation() {
        let db_path = "/tmp/test_learn_validation.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();
        let tools = MemoryTools::new(system);

        // Empty text should fail
        let params = json!({
            "text": "",
            "workspace_id": 1
        });

        let result = tools.handle_request("learn", Some(params));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_search_validation() {
        let db_path = "/tmp/test_search_validation.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();
        let tools = MemoryTools::new(system);

        // Empty query should fail
        let params = json!({"query": "", "limit": 10});
        let result = tools.handle_request("search", Some(params));
        assert!(result.is_err());

        // Invalid limit should fail
        let params = json!({"query": "test", "limit": 200});
        let result = tools.handle_request("search", Some(params));
        assert!(result.is_err());

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_tools_list() {
        let db_path = "/tmp/test_tools_list.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();
        let tools = MemoryTools::new(system);

        let result = tools.handle_request("tools/list", None).unwrap();
        let tools_list = result.get("tools").unwrap().as_array().unwrap();

        assert_eq!(tools_list.len(), 2);
        assert!(tools_list.iter().any(|t| t.get("name").unwrap() == "learn"));
        assert!(tools_list.iter().any(|t| t.get("name").unwrap() == "search"));

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_tools_call() {
        let db_path = "/tmp/test_tools_call.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();
        
        system.database().connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = system.database().connection().last_insert_rowid();

        let tools = MemoryTools::new(system);

        let params = json!({
            "name": "learn",
            "arguments": {
                "text": "Test via tools/call",
                "workspace_id": workspace_id
            }
        });

        let result = tools.handle_request("tools/call", Some(params)).unwrap();
        let response: LearnResponse = serde_json::from_value(result).unwrap();

        assert!(response.memory_id > 0);

        fs::remove_file(db_path).ok();
    }
}
