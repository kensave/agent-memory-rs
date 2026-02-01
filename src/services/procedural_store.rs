use crate::models::dtos::Procedure;
use crate::storage::database::Database;
use crate::traits::memory_store::MemoryStore;
use anyhow::Result;
use async_trait::async_trait;
use rusqlite::params;

pub struct ProceduralMemoryStore {
    db: Database,
}

impl ProceduralMemoryStore {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn get_by_trigger(&self, workspace_id: i64, trigger: &serde_json::Value) -> Result<Vec<Procedure>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, name, trigger_conditions, action_sequence, 
                        success_rate, usage_count, last_used, learned_from, created_at, updated_at
                 FROM procedures 
                 WHERE workspace_id = ? AND json_extract(trigger_conditions, '$') LIKE ?
                 ORDER BY success_rate DESC, usage_count DESC"
            )?;
            
            let trigger_str = format!("%{}%", trigger.to_string());
            let rows = stmt.query_map(params![workspace_id, trigger_str], |row| {
                Ok(Procedure {
                    id: Some(row.get(0)?),
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    trigger_conditions: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                    action_sequence: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    success_rate: row.get(5)?,
                    usage_count: row.get(6)?,
                    last_used: row.get(7)?,
                    learned_from: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                    created_at: row.get(9)?,
                })
            })?;
            
            Ok(rows.filter_map(Result::ok).collect())
        })
    }

    pub fn update_success_rate(&self, id: i64, success: bool) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE procedures 
                 SET success_rate = (success_rate * usage_count + ?) / (usage_count + 1),
                     updated_at = datetime('now')
                 WHERE id = ?",
                params![if success { 1.0 } else { 0.0 }, id]
            )?;
            Ok(())
        })
    }

    pub fn increment_usage(&self, id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE procedures 
                 SET usage_count = usage_count + 1,
                     last_used = datetime('now'),
                     updated_at = datetime('now')
                 WHERE id = ?",
                params![id]
            )?;
            Ok(())
        })
    }
}

#[async_trait]
impl MemoryStore for ProceduralMemoryStore {
    type Memory = Procedure;
    type Id = i64;

    async fn store(&self, memory: Self::Memory) -> Result<i64> {
        self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO procedures (workspace_id, name, trigger_conditions, action_sequence, 
                                        success_rate, usage_count, last_used, learned_from, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
                params![
                    memory.workspace_id,
                    memory.name,
                    memory.trigger_conditions.to_string(),
                    memory.action_sequence.to_string(),
                    memory.success_rate,
                    memory.usage_count,
                    memory.last_used,
                    serde_json::to_string(&memory.learned_from)?,
                ]
            )?;
            Ok(conn.last_insert_rowid())
        })
    }

    async fn get(&self, id: i64) -> Result<Option<Self::Memory>> {
        self.db.execute(|conn| {
            Ok(conn.query_row(
                "SELECT id, workspace_id, name, trigger_conditions, action_sequence, 
                        success_rate, usage_count, last_used, learned_from, created_at, updated_at
                 FROM procedures WHERE id = ?",
                params![id],
                |row| {
                    Ok(Procedure {
                        id: Some(row.get(0)?),
                        workspace_id: row.get(1)?,
                        name: row.get(2)?,
                        trigger_conditions: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                        action_sequence: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                        success_rate: row.get(5)?,
                        usage_count: row.get(6)?,
                        last_used: row.get(7)?,
                        learned_from: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
                        created_at: row.get(9)?,
                    })
                }
            ).ok())
        })
    }

    async fn update(&self, id: i64, memory: Self::Memory) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute(
                "UPDATE procedures 
                 SET name = ?, trigger_conditions = ?, action_sequence = ?, 
                     success_rate = ?, usage_count = ?, last_used = ?, learned_from = ?,
                     updated_at = datetime('now')
                 WHERE id = ?",
                params![
                    memory.name,
                    memory.trigger_conditions.to_string(),
                    memory.action_sequence.to_string(),
                    memory.success_rate,
                    memory.usage_count,
                    memory.last_used,
                    serde_json::to_string(&memory.learned_from)?,
                    id
                ]
            )?;
            Ok(())
        })
    }

    async fn delete(&self, id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute("DELETE FROM procedures WHERE id = ?", params![id])?;
            Ok(())
        })
    }

    async fn store_batch(&self, memories: Vec<Self::Memory>) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        for memory in memories {
            ids.push(self.store(memory).await?);
        }
        Ok(ids)
    }
}
