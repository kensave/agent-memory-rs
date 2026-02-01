use anyhow::Result;
use crate::{FastEmbedder, ModelType};
use crate::storage::{Database, MemoryStore, Memory, SearchFilters, SearchResult};
use std::path::Path;

pub struct MemorySystem {
    db: Database,
    embedder: FastEmbedder,
}

impl MemorySystem {
    pub fn new<P: AsRef<Path>>(db_path: P, model_type: ModelType) -> Result<Self> {
        let db = Database::new(db_path)?;
        let embedder = FastEmbedder::with_model(model_type)?;
        Ok(MemorySystem { db, embedder })
    }

    pub fn database(&self) -> &Database {
        &self.db
    }

    pub fn learn(&self, memory: &Memory) -> Result<i64> {
        let store = MemoryStore::new(self.db.clone());
        
        // Generate embedding
        let embedding = self.embedder.embed(&memory.text)?;
        
        // Store memory and embedding atomically
        let memory_id = store.insert_memory(memory)?;
        store.insert_embedding(memory_id, &embedding)?;
        
        Ok(memory_id)
    }

    pub fn learn_batch(&self, memories: &[Memory]) -> Result<Vec<i64>> {
        let store = MemoryStore::new(self.db.clone());
        let mut memory_ids = Vec::new();
        
        // Collect texts for batch embedding
        let texts: Vec<&str> = memories.iter().map(|m| m.text.as_str()).collect();
        let embeddings = self.embedder.embed_batch(&texts)?;
        
        // Store all memories and embeddings
        for (memory, embedding) in memories.iter().zip(embeddings.iter()) {
            let memory_id = store.insert_memory(memory)?;
            store.insert_embedding(memory_id, embedding)?;
            memory_ids.push(memory_id);
        }
        
        Ok(memory_ids)
    }

    pub fn search(&self, query: &str, filters: &SearchFilters, limit: usize) -> Result<Vec<SearchResult>> {
        let store = MemoryStore::new(self.db.clone());
        
        // Generate query embedding
        let query_embedding = self.embedder.embed(query)?;
        
        // Search with filters
        let results = store.search_similar(&query_embedding, filters, limit)?;
        
        Ok(results)
    }

    pub fn get_memory(&self, memory_id: i64) -> Result<Option<Memory>> {
        let store = MemoryStore::new(self.db.clone());
        store.get_memory(memory_id)
    }

    pub fn update_memory(&self, memory_id: i64, memory: &Memory) -> Result<()> {
        let store = MemoryStore::new(self.db.clone());
        store.update_memory(memory_id, memory)
    }

    pub fn delete_memory(&self, memory_id: i64) -> Result<()> {
        let store = MemoryStore::new(self.db.clone());
        store.delete_memory(memory_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_learn_and_search() {
        let db_path = "/tmp/test_memory_system.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();

        // Create workspace
        system.db.execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        }).unwrap();
        let workspace_id = system.db.execute(|conn| Ok(conn.last_insert_rowid())).unwrap();

        // Learn some facts
        let facts = vec![
            "Rust is a systems programming language",
            "Python is great for data science",
            "JavaScript runs in browsers",
        ];

        for fact in &facts {
            let memory = Memory {
                id: None,
                workspace_id,
                agent_id: None,
                text: fact.to_string(),
                tags: None,
                importance_score: 0.5,
                access_count: 0,
                last_accessed: None,
                conversation_id: None,
                parent_memory_id: None,
                user_feedback: None,
            source_episodes: vec![],
            confidence: 0.5,
            last_validated: None,
            created_at: None,
                updated_at: None,
            };
            system.learn(&memory).unwrap();
        }

        // Search for programming languages
        let filters = SearchFilters {
            workspace_id: Some(workspace_id),
            ..Default::default()
        };
        let results = system.search("programming language", &filters, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].memory.text.contains("Rust") || results[0].memory.text.contains("JavaScript"));

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_batch_learning() {
        let db_path = "/tmp/test_batch_learning.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();

        let workspace_id = system.db.execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        }).unwrap();

        let memories: Vec<Memory> = (0..5).map(|i| Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: format!("Fact number {}", i),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            source_episodes: vec![],
            confidence: 0.5,
            last_validated: None,
            created_at: None,
            updated_at: None,
        }).collect();

        let memory_ids = system.learn_batch(&memories).unwrap();
        assert_eq!(memory_ids.len(), 5);

        // Verify all memories were stored
        for memory_id in memory_ids {
            let retrieved = system.get_memory(memory_id).unwrap();
            assert!(retrieved.is_some());
        }

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_search_with_filters() {
        let db_path = "/tmp/test_search_filters.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();

        let workspace_id = system.db.execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        }).unwrap();

        // Learn memories with different importance scores
        for i in 0..5 {
            let memory = Memory {
                id: None,
                workspace_id,
                agent_id: None,
                text: format!("Memory with importance {}", i),
                tags: None,
                importance_score: (i as f64) / 10.0,
                access_count: 0,
                last_accessed: None,
                conversation_id: None,
                parent_memory_id: None,
                user_feedback: None,
            source_episodes: vec![],
            confidence: 0.5,
            last_validated: None,
            created_at: None,
                updated_at: None,
            };
            system.learn(&memory).unwrap();
        }

        // Search with importance filter
        let filters = SearchFilters {
            workspace_id: Some(workspace_id),
            min_importance: Some(0.3),
            ..Default::default()
        };
        let results = system.search("memory", &filters, 10).unwrap();

        assert!(results.len() >= 2);
        for result in &results {
            assert!(result.memory.importance_score >= 0.3);
        }

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_embedding_error_handling() {
        let db_path = "/tmp/test_error_handling.db";
        let _ = fs::remove_file(db_path);

        let system = MemorySystem::new(db_path, ModelType::MiniLM).unwrap();

        let workspace_id = system.db.execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')",
                [],
            )?;
            Ok(conn.last_insert_rowid())
        }).unwrap();

        // Empty text should still work (mock embedder handles it)
        let memory = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: "".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            source_episodes: vec![],
            confidence: 0.5,
            last_validated: None,
            created_at: None,
            updated_at: None,
        };
        
        let result = system.learn(&memory);
        assert!(result.is_ok());

        fs::remove_file(db_path).ok();
    }
}
