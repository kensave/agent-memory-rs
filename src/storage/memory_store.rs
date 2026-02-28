use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use super::Database;

// Helper function to generate current timestamp
pub fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn is_zero(n: &i64) -> bool { *n == 0 }
fn is_empty_vec(v: &[i64]) -> bool { v.is_empty() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub workspace_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<i64>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    pub importance_score: f64,
    #[serde(skip_serializing_if = "is_zero")]
    pub access_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_accessed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_memory_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_feedback: Option<String>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub source_episodes: Vec<i64>,
    pub confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub workspace_id: Option<i64>,
    pub agent_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub min_importance: Option<f64>,
    pub max_importance: Option<f64>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub memory: Memory,
    pub similarity_score: f64,
    pub combined_score: f64,
}

/// Semantic memory store (existing memories table)
pub struct MemoryStore {
    db: Database,
}

impl MemoryStore {
    pub fn new(db: Database) -> Self {
        MemoryStore { db }
    }

    pub fn insert_memory(&self, memory: &Memory) -> Result<i64> {
        self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO memories (workspace_id, agent_id, text, tags, importance_score, 
                 conversation_id, parent_memory_id, user_feedback, source_episodes, confidence)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    memory.workspace_id,
                    memory.agent_id,
                    memory.text,
                    memory.tags,
                    memory.importance_score,
                    memory.conversation_id,
                    memory.parent_memory_id,
                    memory.user_feedback,
                    serde_json::to_string(&memory.source_episodes)?,
                    memory.confidence,
                ],
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn insert_embedding(&self, memory_id: i64, embedding: &[f32]) -> Result<()> {
        self.db.execute(|conn| {
            let bytes: Vec<u8> = embedding.iter()
                .flat_map(|f| f.to_le_bytes())
                .collect();
            
            conn.execute(
                "INSERT INTO vec0 (memory_id, embedding) VALUES (?1, vec_f32(?2))",
                params![memory_id, bytes],
            )?;
            Ok(())
        })
    }

    pub fn get_memory(&self, memory_id: i64) -> Result<Option<Memory>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, text, tags, importance_score, access_count,
                 last_accessed, conversation_id, parent_memory_id, user_feedback, source_episodes,
                 confidence, last_validated, created_at, updated_at
                 FROM memories WHERE id = ?1",
            )?;

            let memory = stmt.query_row(params![memory_id], |row| {
                Ok(Memory {
                    id: Some(row.get(0)?),
                    workspace_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    text: row.get(3)?,
                    tags: row.get(4)?,
                    importance_score: row.get(5)?,
                    access_count: row.get(6)?,
                    last_accessed: row.get(7)?,
                    conversation_id: row.get(8)?,
                    parent_memory_id: row.get(9)?,
                    user_feedback: row.get(10)?,
                    source_episodes: row.get::<_, Option<String>>(11)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    confidence: row.get::<_, Option<f64>>(12)?.unwrap_or(0.5),
                    last_validated: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: Some(row.get(15)?),
                })
            }).optional()?;

            Ok(memory)
        })
    }

    pub fn update_memory(&self, memory_id: i64, memory: &Memory) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE memories SET text = ?1, tags = ?2, importance_score = ?3,
                 conversation_id = ?4, parent_memory_id = ?5, user_feedback = ?6,
                 updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?7",
                params![
                    memory.text,
                    memory.tags,
                    memory.importance_score,
                    memory.conversation_id,
                    memory.parent_memory_id,
                    memory.user_feedback,
                    memory_id,
                ],
            )?;
            Ok(())
        })
    }

    pub fn delete_memory(&self, memory_id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute("DELETE FROM vec0 WHERE memory_id = ?1", params![memory_id])?;
            conn.execute("DELETE FROM memories WHERE id = ?1", params![memory_id])?;
            Ok(())
        })
    }

    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        filters: &SearchFilters,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.db.execute(|conn| {
            let bytes: Vec<u8> = query_embedding.iter()
                .flat_map(|f| f.to_le_bytes())
                .collect();

            let mut sql = String::from(
                "SELECT m.id, m.workspace_id, m.agent_id, m.text, m.tags, m.importance_score,
                 m.access_count, m.last_accessed, m.conversation_id, m.parent_memory_id,
                 m.user_feedback, m.source_episodes, m.confidence, m.last_validated,
                 m.created_at, m.updated_at,
                 vec_distance_cosine(v.embedding, vec_f32(?1)) as distance
                 FROM memories m
                 JOIN vec0 v ON m.id = v.memory_id
                 WHERE 1=1"
            );

            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(bytes)];

            if let Some(workspace_id) = filters.workspace_id {
                sql.push_str(" AND m.workspace_id = ?");
                params_vec.push(Box::new(workspace_id));
            }

            if let Some(agent_id) = filters.agent_id {
                sql.push_str(" AND m.agent_id = ?");
                params_vec.push(Box::new(agent_id));
            }

            if let Some(min_importance) = filters.min_importance {
                sql.push_str(" AND m.importance_score >= ?");
                params_vec.push(Box::new(min_importance));
            }

            if let Some(max_importance) = filters.max_importance {
                sql.push_str(" AND m.importance_score <= ?");
                params_vec.push(Box::new(max_importance));
            }

            if let Some(ref conversation_id) = filters.conversation_id {
                sql.push_str(" AND m.conversation_id = ?");
                params_vec.push(Box::new(conversation_id.clone()));
            }

            sql.push_str(" ORDER BY distance ASC LIMIT ?");
            params_vec.push(Box::new(limit as i64));

            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

            let mut stmt = conn.prepare(&sql)?;
            let results = stmt.query_map(params_refs.as_slice(), |row| {
                let distance: f64 = row.get(16)?;  // distance is the last column
                let similarity_score = 1.0 - distance;
                let importance_score: f64 = row.get(5)?;
                let combined_score = similarity_score * 0.7 + importance_score * 0.3;

                Ok(SearchResult {
                    memory: Memory {
                        id: Some(row.get(0)?),
                        workspace_id: row.get(1)?,
                        agent_id: row.get(2)?,
                        text: row.get(3)?,
                        tags: row.get(4)?,
                        importance_score,
                        access_count: row.get(6)?,
                        last_accessed: row.get(7)?,
                        conversation_id: row.get(8)?,
                        parent_memory_id: row.get(9)?,
                        user_feedback: row.get(10)?,
                        source_episodes: row.get::<_, Option<String>>(11)?
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .unwrap_or_default(),
                        confidence: row.get::<_, Option<f64>>(12)?.unwrap_or(0.5),
                        last_validated: row.get(13)?,
                        created_at: row.get(14)?,
                        updated_at: Some(row.get(15)?),
                    },
                    similarity_score,
                    combined_score,
                })
            })?;

            let mut search_results: Vec<SearchResult> = results.collect::<Result<Vec<_>, _>>()?;
            search_results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

            Ok(search_results)
        })
    }

    pub fn track_source_episode(&self, memory_id: i64, episode_id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE memories 
                 SET source_episodes = json_insert(source_episodes, '$[#]', ?),
                     updated_at = datetime('now')
                 WHERE id = ?",
                params![episode_id, memory_id]
            )?;
            Ok(())
        })
    }

    pub fn update_confidence(&self, memory_id: i64, delta: f64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE memories 
                 SET confidence = CASE 
                     WHEN confidence + ? < 0.0 THEN 0.0
                     WHEN confidence + ? > 1.0 THEN 1.0
                     ELSE confidence + ?
                 END,
                 updated_at = datetime('now')
                 WHERE id = ?",
                params![delta, delta, delta, memory_id]
            )?;
            Ok(())
        })
    }

    pub fn validate_knowledge(&self, memory_id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE memories 
                 SET last_validated = datetime('now'),
                     updated_at = datetime('now')
                 WHERE id = ?",
                params![memory_id]
            )?;
            Ok(())
        })
    }

    pub fn increment_access_count(&self, memory_id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE memories 
                 SET access_count = access_count + 1,
                     last_accessed = datetime('now'),
                     updated_at = datetime('now')
                 WHERE id = ?",
                params![memory_id]
            )?;
            Ok(())
        })
    }

    pub fn get_by_confidence_threshold(&self, workspace_id: i64, threshold: f64) -> Result<Vec<Memory>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, text, tags, importance_score, access_count,
                        last_accessed, conversation_id, parent_memory_id, user_feedback,
                        source_episodes, confidence, last_validated, created_at, updated_at
                 FROM memories 
                 WHERE workspace_id = ? AND confidence >= ?
                 ORDER BY confidence DESC"
            )?;
            
            let rows = stmt.query_map(params![workspace_id, threshold], |row| {
                Ok(Memory {
                    id: Some(row.get(0)?),
                    workspace_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    text: row.get(3)?,
                    tags: row.get(4)?,
                    importance_score: row.get(5)?,
                    access_count: row.get(6)?,
                    last_accessed: row.get(7)?,
                    conversation_id: row.get(8)?,
                    parent_memory_id: row.get(9)?,
                    user_feedback: row.get(10)?,
                    source_episodes: row.get::<_, Option<String>>(11)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    confidence: row.get::<_, Option<f64>>(12)?.unwrap_or(0.5),
                    last_validated: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: Some(row.get(15)?),
                })
            })?;
            
            Ok(rows.filter_map(Result::ok).collect())
        })
    }
}
