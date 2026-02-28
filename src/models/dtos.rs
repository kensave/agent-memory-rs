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
