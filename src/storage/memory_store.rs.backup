use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Option<i64>,
    pub workspace_id: i64,
    pub agent_id: Option<i64>,
    pub text: String,
    pub tags: Option<String>,
    pub importance_score: f64,
    pub access_count: i64,
    pub last_accessed: Option<String>,
    pub conversation_id: Option<String>,
    pub parent_memory_id: Option<i64>,
    pub user_feedback: Option<String>,
    pub created_at: Option<String>,
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

pub struct MemoryStore<'a> {
    conn: &'a Connection,
}

impl<'a> MemoryStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        MemoryStore { conn }
    }

    pub fn insert_memory(&self, memory: &Memory) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO memories (workspace_id, agent_id, text, tags, importance_score, 
             conversation_id, parent_memory_id, user_feedback)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                memory.workspace_id,
                memory.agent_id,
                memory.text,
                memory.tags,
                memory.importance_score,
                memory.conversation_id,
                memory.parent_memory_id,
                memory.user_feedback,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_embedding(&self, memory_id: i64, embedding: &[f32]) -> Result<()> {
        // Convert to bytes for vec0 storage
        let bytes: Vec<u8> = embedding.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        
        self.conn.execute(
            "INSERT INTO vec0 (memory_id, embedding) VALUES (?1, vec_f32(?2))",
            params![memory_id, bytes],
        )?;
        Ok(())
    }

    pub fn get_memory(&self, memory_id: i64) -> Result<Option<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, agent_id, text, tags, importance_score, access_count,
             last_accessed, conversation_id, parent_memory_id, user_feedback, created_at, updated_at
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
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        }).optional()?;

        Ok(memory)
    }

    pub fn update_memory(&self, memory_id: i64, memory: &Memory) -> Result<()> {
        self.conn.execute(
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
    }

    pub fn delete_memory(&self, memory_id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM vec0 WHERE memory_id = ?1", params![memory_id])?;
        self.conn.execute("DELETE FROM memories WHERE id = ?1", params![memory_id])?;
        Ok(())
    }

    pub fn get_memories_by_workspace(&self, workspace_id: i64) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, agent_id, text, tags, importance_score, access_count,
             last_accessed, conversation_id, parent_memory_id, user_feedback, created_at, updated_at
             FROM memories WHERE workspace_id = ?1 ORDER BY created_at DESC",
        )?;

        let memories = stmt.query_map(params![workspace_id], |row| {
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
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    pub fn get_memories_by_agent(&self, workspace_id: i64, agent_id: i64) -> Result<Vec<Memory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, agent_id, text, tags, importance_score, access_count,
             last_accessed, conversation_id, parent_memory_id, user_feedback, created_at, updated_at
             FROM memories WHERE workspace_id = ?1 AND agent_id = ?2 ORDER BY created_at DESC",
        )?;

        let memories = stmt.query_map(params![workspace_id, agent_id], |row| {
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
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(memories)
    }

    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        filters: &SearchFilters,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Convert query embedding to bytes
        let query_bytes: Vec<u8> = query_embedding.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        
        let mut query = String::from(
            "SELECT m.id, m.workspace_id, m.agent_id, m.text, m.tags, m.importance_score, 
             m.access_count, m.last_accessed, m.conversation_id, m.parent_memory_id, 
             m.user_feedback, m.created_at, m.updated_at,
             vec_distance_cosine(v.embedding, vec_f32(?1)) as distance
             FROM memories m
             JOIN vec0 v ON m.id = v.memory_id
             WHERE 1=1"
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query_bytes)];

        if let Some(workspace_id) = filters.workspace_id {
            query.push_str(" AND m.workspace_id = ?");
            params_vec.push(Box::new(workspace_id));
        }

        if let Some(agent_id) = filters.agent_id {
            query.push_str(" AND m.agent_id = ?");
            params_vec.push(Box::new(agent_id));
        }

        if let Some(min_importance) = filters.min_importance {
            query.push_str(" AND m.importance_score >= ?");
            params_vec.push(Box::new(min_importance));
        }

        if let Some(max_importance) = filters.max_importance {
            query.push_str(" AND m.importance_score <= ?");
            params_vec.push(Box::new(max_importance));
        }

        if let Some(ref created_after) = filters.created_after {
            query.push_str(" AND m.created_at >= ?");
            params_vec.push(Box::new(created_after.clone()));
        }

        if let Some(ref created_before) = filters.created_before {
            query.push_str(" AND m.created_at <= ?");
            params_vec.push(Box::new(created_before.clone()));
        }

        if let Some(ref conversation_id) = filters.conversation_id {
            query.push_str(" AND m.conversation_id = ?");
            params_vec.push(Box::new(conversation_id.clone()));
        }

        query.push_str(" ORDER BY distance ASC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.conn.prepare(&query)?;
        let results = stmt.query_map(params_refs.as_slice(), |row| {
            let memory = Memory {
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
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            };
            let distance: f64 = row.get(13)?;
            let similarity_score = 1.0 - distance;
            
            let combined_score = similarity_score * 0.7 + memory.importance_score * 0.3;
            
            Ok(SearchResult {
                memory,
                similarity_score,
                combined_score,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        let mut results = results;
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;
    use std::fs;

    #[test]
    fn test_insert_and_get_memory() {
        let db_path = "/tmp/test_memory_store.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = db.connection().last_insert_rowid();

        let memory = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: "Test memory".to_string(),
            tags: Some("test,memory".to_string()),
            importance_score: 0.8,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };

        let memory_id = store.insert_memory(&memory).unwrap();
        assert!(memory_id > 0);

        let retrieved = store.get_memory(memory_id).unwrap().unwrap();
        assert_eq!(retrieved.text, "Test memory");
        assert_eq!(retrieved.importance_score, 0.8);

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_insert_embedding() {
        let db_path = "/tmp/test_embedding.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = db.connection().last_insert_rowid();

        let memory = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: "Test".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };

        let memory_id = store.insert_memory(&memory).unwrap();
        let embedding = vec![0.1f32; 384];
        store.insert_embedding(memory_id, &embedding).unwrap();

        let count: i64 = db.connection()
            .query_row("SELECT COUNT(*) FROM vec0 WHERE memory_id = ?1", params![memory_id], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_update_and_delete_memory() {
        let db_path = "/tmp/test_update_delete.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = db.connection().last_insert_rowid();

        let mut memory = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: "Original".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };

        let memory_id = store.insert_memory(&memory).unwrap();
        
        memory.text = "Updated".to_string();
        store.update_memory(memory_id, &memory).unwrap();

        let updated = store.get_memory(memory_id).unwrap().unwrap();
        assert_eq!(updated.text, "Updated");

        store.delete_memory(memory_id).unwrap();
        let deleted = store.get_memory(memory_id).unwrap();
        assert!(deleted.is_none());

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_workspace_and_agent_scoping() {
        let db_path = "/tmp/test_scoping.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('ws1', '/tmp/ws1')",
            [],
        ).unwrap();
        let ws1_id = db.connection().last_insert_rowid();

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('ws2', '/tmp/ws2')",
            [],
        ).unwrap();
        let ws2_id = db.connection().last_insert_rowid();

        db.connection().execute(
            "INSERT INTO agents (workspace_id, name) VALUES (?1, 'agent1')",
            params![ws1_id],
        ).unwrap();
        let agent_id = db.connection().last_insert_rowid();

        let mem1 = Memory {
            id: None,
            workspace_id: ws1_id,
            agent_id: None,
            text: "WS1 shared".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        store.insert_memory(&mem1).unwrap();

        let mem2 = Memory {
            id: None,
            workspace_id: ws1_id,
            agent_id: Some(agent_id),
            text: "WS1 agent1".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        store.insert_memory(&mem2).unwrap();

        let mem3 = Memory {
            id: None,
            workspace_id: ws2_id,
            agent_id: None,
            text: "WS2 shared".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        store.insert_memory(&mem3).unwrap();

        let ws1_memories = store.get_memories_by_workspace(ws1_id).unwrap();
        assert_eq!(ws1_memories.len(), 2);

        let ws2_memories = store.get_memories_by_workspace(ws2_id).unwrap();
        assert_eq!(ws2_memories.len(), 1);

        let agent_memories = store.get_memories_by_agent(ws1_id, agent_id).unwrap();
        assert_eq!(agent_memories.len(), 1);
        assert_eq!(agent_memories[0].text, "WS1 agent1");

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_semantic_search() {
        let db_path = "/tmp/test_semantic_search.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = db.connection().last_insert_rowid();

        for i in 0..5 {
            let memory = Memory {
                id: None,
                workspace_id,
                agent_id: None,
                text: format!("Memory {}", i),
                tags: None,
                importance_score: 0.5,
                access_count: 0,
                last_accessed: None,
                conversation_id: None,
                parent_memory_id: None,
                user_feedback: None,
                created_at: None,
                updated_at: None,
            };
            let memory_id = store.insert_memory(&memory).unwrap();
            
            let mut embedding = vec![0.1f32; 384];
            embedding[0] = i as f32 / 10.0;
            store.insert_embedding(memory_id, &embedding).unwrap();
        }

        // Verify embeddings were inserted
        let count: i64 = db.connection()
            .query_row("SELECT COUNT(*) FROM vec0", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5, "Should have 5 embeddings");

        let query_embedding = vec![0.1f32; 384];
        let filters = SearchFilters::default();
        let results = store.search_similar(&query_embedding, &filters, 3).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].similarity_score >= results[1].similarity_score);

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_filtered_search() {
        let db_path = "/tmp/test_filtered_search.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = db.connection().last_insert_rowid();

        for i in 0..5 {
            let memory = Memory {
                id: None,
                workspace_id,
                agent_id: None,
                text: format!("Memory {}", i),
                tags: None,
                importance_score: (i as f64) / 10.0,
                access_count: 0,
                last_accessed: None,
                conversation_id: None,
                parent_memory_id: None,
                user_feedback: None,
                created_at: None,
                updated_at: None,
            };
            let memory_id = store.insert_memory(&memory).unwrap();
            let embedding = vec![0.1f32; 384];
            store.insert_embedding(memory_id, &embedding).unwrap();
        }

        let query_embedding = vec![0.1f32; 384];
        let filters = SearchFilters {
            workspace_id: Some(workspace_id),
            min_importance: Some(0.2),
            ..Default::default()
        };
        let results = store.search_similar(&query_embedding, &filters, 10).unwrap();

        assert!(results.len() >= 3);
        for result in &results {
            assert!(result.memory.importance_score >= 0.2);
        }

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_hybrid_search_ranking() {
        let db_path = "/tmp/test_hybrid_ranking.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).unwrap();
        let store = MemoryStore::new(db.connection());

        db.connection().execute(
            "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
            [],
        ).unwrap();
        let workspace_id = db.connection().last_insert_rowid();

        let mem1 = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: "High importance".to_string(),
            tags: None,
            importance_score: 0.9,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        let mem1_id = store.insert_memory(&mem1).unwrap();
        let mut emb1 = vec![0.1f32; 384];
        emb1[0] = 0.5;
        store.insert_embedding(mem1_id, &emb1).unwrap();

        let mem2 = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: "Low importance".to_string(),
            tags: None,
            importance_score: 0.1,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        let mem2_id = store.insert_memory(&mem2).unwrap();
        let emb2 = vec![0.1f32; 384];
        store.insert_embedding(mem2_id, &emb2).unwrap();

        let query_embedding = vec![0.1f32; 384];
        let filters = SearchFilters::default();
        let results = store.search_similar(&query_embedding, &filters, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].combined_score > 0.0);
        assert!(results[1].combined_score > 0.0);

        fs::remove_file(db_path).ok();
    }
}
