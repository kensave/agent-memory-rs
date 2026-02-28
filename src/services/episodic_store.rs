use anyhow::Result;
use async_trait::async_trait;
use rusqlite::params;
use serde_json;
use std::sync::{Arc, Mutex};

use crate::embedder::FastEmbedder;
use crate::models::Episode;
use crate::storage::Database;
use crate::traits::MemoryStore;

pub struct EpisodicMemoryStore {
    db: Database,
    embedder: Option<Arc<Mutex<FastEmbedder>>>,
}

impl EpisodicMemoryStore {
    pub fn new(db: Database) -> Self {
        Self { db, embedder: None }
    }
    
    pub fn with_embedder(db: Database, embedder: Arc<Mutex<FastEmbedder>>) -> Self {
        Self { db, embedder: Some(embedder) }
    }
}

#[async_trait]
impl MemoryStore for EpisodicMemoryStore {
    type Memory = Episode;
    type Id = i64;

    async fn store(&self, memory: Self::Memory) -> Result<Self::Id> {
        let context_json = serde_json::to_string(&memory.context)?;
        
        // Generate embedding from context
        let embedding = if let Some(ref embedder) = self.embedder {
            Some(embedder.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire embedder lock"))?
                .embed(&context_json)?)
        } else {
            None
        };
        
        self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO episodes (workspace_id, agent_id, timestamp, conversation_id, event_type, context, outcome, valence, archived)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    memory.workspace_id,
                    memory.agent_id,
                    memory.timestamp,
                    memory.conversation_id,
                    memory.event_type,
                    context_json,
                    memory.outcome,
                    memory.valence,
                    memory.archived as i32,
                ],
            )?;
            
            let episode_id = conn.last_insert_rowid();
            
            // Store embedding in vec0 format (reuse existing table with negative IDs)
            if let Some(emb) = embedding {
                let embedding_blob: Vec<u8> = emb.iter()
                    .flat_map(|&f| f.to_le_bytes())
                    .collect();
                
                // Use negative ID to distinguish episodes from semantic memories
                conn.execute(
                    "INSERT INTO vec0 (memory_id, embedding) VALUES (?1, ?2)",
                    params![-episode_id, embedding_blob],
                )?;
            }
            
            Ok(episode_id)
        })
    }

    async fn get(&self, id: Self::Id) -> Result<Option<Self::Memory>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, timestamp, conversation_id, event_type, context, outcome, valence, archived, created_at
                 FROM episodes WHERE id = ?1"
            )?;
            
            let result = stmt.query_row([id], |row| {
                let context_str: String = row.get(6)?;
                let context = serde_json::from_str(&context_str).unwrap_or(serde_json::json!({}));
                let archived_int: i32 = row.get(9)?;
                
                Ok(Episode {
                    id: Some(row.get(0)?),
                    workspace_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    timestamp: row.get(3)?,
                    conversation_id: row.get(4)?,
                    event_type: row.get(5)?,
                    context,
                    outcome: row.get(7)?,
                    valence: row.get(8)?,
                    archived: archived_int != 0,
                    created_at: row.get(10)?,
                })
            });
            
            match result {
                Ok(episode) => Ok(Some(episode)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    async fn update(&self, id: Self::Id, memory: Self::Memory) -> Result<()> {
        self.db.execute(|conn| {
            let context_json = serde_json::to_string(&memory.context)?;
            
            conn.execute(
                "UPDATE episodes SET event_type = ?1, context = ?2, outcome = ?3, valence = ?4, archived = ?5
                 WHERE id = ?6",
                params![
                    memory.event_type,
                    context_json,
                    memory.outcome,
                    memory.valence,
                    memory.archived as i32,
                    id,
                ],
            )?;
            
            Ok(())
        })
    }

    async fn delete(&self, id: Self::Id) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute("DELETE FROM episodes WHERE id = ?1", [id])?;
            // Episodes stored in vec0 with negative IDs
            conn.execute("DELETE FROM vec0 WHERE memory_id = ?1", [-id])?;
            Ok(())
        })
    }

    async fn store_batch(&self, memories: Vec<Self::Memory>) -> Result<Vec<Self::Id>> {
        self.db.execute(|conn| {
            let mut ids = Vec::new();
            let tx = conn.unchecked_transaction()?;
            
            for memory in memories {
                let context_json = serde_json::to_string(&memory.context)?;
                
                tx.execute(
                    "INSERT INTO episodes (workspace_id, agent_id, timestamp, conversation_id, event_type, context, outcome, valence, archived)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        memory.workspace_id,
                        memory.agent_id,
                        memory.timestamp,
                        memory.conversation_id,
                        memory.event_type,
                        context_json,
                        memory.outcome,
                        memory.valence,
                        memory.archived as i32,
                    ],
                )?;
                
                ids.push(tx.last_insert_rowid());
            }
            
            tx.commit()?;
            Ok(ids)
        })
    }
}

// Additional methods specific to episodic memory
impl EpisodicMemoryStore {
    pub fn get_by_time_range(&self, workspace_id: i64, start: &str, end: &str) -> Result<Vec<Episode>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, timestamp, conversation_id, event_type, context, outcome, valence, archived, created_at
                 FROM episodes 
                 WHERE workspace_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3 AND archived = 0
                 ORDER BY timestamp DESC"
            )?;
            
            let episodes = stmt.query_map(params![workspace_id, start, end], |row| {
                let context_str: String = row.get(6)?;
                let context = serde_json::from_str(&context_str).unwrap_or(serde_json::json!({}));
                let archived_int: i32 = row.get(9)?;
                
                Ok(Episode {
                    id: Some(row.get(0)?),
                    workspace_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    timestamp: row.get(3)?,
                    conversation_id: row.get(4)?,
                    event_type: row.get(5)?,
                    context,
                    outcome: row.get(7)?,
                    valence: row.get(8)?,
                    archived: archived_int != 0,
                    created_at: row.get(10)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;
            
            Ok(episodes)
        })
    }
    
    pub fn get_by_conversation(&self, conversation_id: &str) -> Result<Vec<Episode>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, timestamp, conversation_id, event_type, context, outcome, valence, archived, created_at
                 FROM episodes 
                 WHERE conversation_id = ?1
                 ORDER BY timestamp ASC"
            )?;
            
            let episodes = stmt.query_map([conversation_id], |row| {
                let context_str: String = row.get(6)?;
                let context = serde_json::from_str(&context_str).unwrap_or(serde_json::json!({}));
                let archived_int: i32 = row.get(9)?;
                
                Ok(Episode {
                    id: Some(row.get(0)?),
                    workspace_id: row.get(1)?,
                    agent_id: row.get(2)?,
                    timestamp: row.get(3)?,
                    conversation_id: row.get(4)?,
                    event_type: row.get(5)?,
                    context,
                    outcome: row.get(7)?,
                    valence: row.get(8)?,
                    archived: archived_int != 0,
                    created_at: row.get(10)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;
            
            Ok(episodes)
        })
    }
    
    pub fn archive(&self, id: i64) -> Result<()> {
        self.db.execute(|conn| {
            conn.execute("UPDATE episodes SET archived = 1 WHERE id = ?1", [id])?;
            Ok(())
        })
    }
}
