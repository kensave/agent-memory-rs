use serde::{Deserialize, Serialize};

/// Episodic memory - stores specific interaction events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Option<i64>,
    pub workspace_id: i64,
    pub agent_id: Option<i64>,
    pub timestamp: String,
    pub conversation_id: Option<String>,
    pub event_type: String,
    pub context: serde_json::Value,
    pub outcome: Option<String>,
    pub valence: Option<f64>, // -1.0 to 1.0
    pub archived: bool,
    pub created_at: Option<String>,
}

/// Procedural memory - stores workflows and action sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: Option<i64>,
    pub workspace_id: i64,
    pub name: String,
    pub trigger_conditions: serde_json::Value,
    pub action_sequence: serde_json::Value,
    pub success_rate: f64,
    pub usage_count: i64,
    pub last_used: Option<String>,
    pub learned_from: Vec<i64>, // episode IDs
    pub created_at: Option<String>,
}

/// Daily synopsis - consolidated daily summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synopsis {
    pub date: String,
    pub workspace_id: i64,
    pub agent_id: Option<i64>,
    pub summary: String,
    pub key_insights: Vec<String>,
    pub new_knowledge_ids: Vec<i64>,
    pub new_procedure_ids: Vec<i64>,
    pub stats: serde_json::Value,
    pub created_at: Option<String>,
}

impl Default for Synopsis {
    fn default() -> Self {
        Self {
            date: String::new(),
            workspace_id: 0,
            agent_id: None,
            summary: String::new(),
            key_insights: Vec::new(),
            new_knowledge_ids: Vec::new(),
            new_procedure_ids: Vec::new(),
            stats: serde_json::json!({}),
            created_at: None,
        }
    }
}

/// Pattern extracted from episodic memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: String,
    pub description: String,
    pub frequency: i64,
    pub confidence: f64,
    pub source_episodes: Vec<i64>,
}

/// Composite score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeScore {
    pub recency: f64,
    pub relevance: f64,
    pub utility: f64,
    pub combined: f64,
}
