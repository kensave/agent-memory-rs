use crate::embedder::FastEmbedder;
use crate::models::dtos::Episode;
use crate::services::episodic_store::EpisodicMemoryStore;
use crate::services::hybrid_retrieval::{HybridRetrievalEngine, HybridSearchResult};
use crate::storage::database::Database;
use crate::storage::memory_store::{Memory, MemoryStore as SemanticStore};
use crate::traits::memory_store::MemoryStore;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct MemoryManager {
    pub(crate) db: Database,
    episodic: EpisodicMemoryStore,
    semantic: SemanticStore,
    retrieval: HybridRetrievalEngine,
}

impl MemoryManager {
    /// Create a new MemoryManager with default embedder
    pub fn new(db: Database) -> Self {
        Self {
            episodic: EpisodicMemoryStore::new(db.clone()),
            semantic: SemanticStore::new(db.clone()),
            retrieval: HybridRetrievalEngine::new(db.clone()),
            db,
        }
    }
    
    /// Create a new MemoryManager with custom embedder
    pub fn with_embedder(db: Database, embedder: Arc<Mutex<FastEmbedder>>) -> Self {
        Self {
            episodic: EpisodicMemoryStore::with_embedder(db.clone(), embedder.clone()),
            semantic: SemanticStore::new(db.clone()),
            retrieval: HybridRetrievalEngine::with_embedder(db.clone(), embedder),
            db,
        }
    }

    pub async fn store_episode(&self, episode: Episode) -> Result<i64> {
        self.episodic.store(episode).await
    }

    pub fn store_knowledge(&self, memory: &Memory) -> Result<i64> {
        self.semantic.insert_memory(memory)
    }

    pub fn retrieve(&self, query: &str, workspace_id: i64, limit: usize) -> Result<Vec<HybridSearchResult>> {
        self.retrieval.hybrid_search(query, workspace_id, limit)
    }

    pub fn retrieve_hierarchical(&self, query: &str, workspace_id: i64, max_results: usize) -> Result<Vec<HybridSearchResult>> {
        let mut results = Vec::new();
        
        // Level 1: Semantic memory (BM25 + vector hybrid)
        let semantic = self.retrieval.hybrid_search(query, workspace_id, max_results)?;
        results.extend(semantic);
        
        // Level 2: Recent episodes (vector search)
        let episodic = self.retrieval.search_by_type(query, workspace_id, "episodic", max_results)?;
        results.extend(episodic);
        
        // Sort by score and return top N
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results);

        // Track access on returned results
        self.db.execute(|conn| {
            for r in &results {
                let table = if r.memory_type == "episodic" { "episodes" } else { "memories" };
                let _ = conn.execute(
                    &format!("UPDATE {} SET access_count = access_count + 1, last_accessed = datetime('now') WHERE id = ?1", table),
                    rusqlite::params![r.id],
                );
            }
            Ok(())
        })?;

        Ok(results)
    }

    pub fn get_memory_stats(&self, workspace_id: i64) -> Result<MemoryStats> {
        self.db.execute(|conn| {
            let episodes: i64 = conn.query_row(
                "SELECT COUNT(*) FROM episodes WHERE workspace_id = ? AND archived = 0",
                rusqlite::params![workspace_id],
                |row| row.get(0)
            )?;
            
            let archived: i64 = conn.query_row(
                "SELECT COUNT(*) FROM episodes WHERE workspace_id = ? AND archived = 1",
                rusqlite::params![workspace_id],
                |row| row.get(0)
            )?;
            
            let knowledge: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE workspace_id = ?",
                rusqlite::params![workspace_id],
                |row| row.get(0)
            )?;
            
            Ok(MemoryStats {
                active_episodes: episodes as usize,
                archived_episodes: archived as usize,
                knowledge_count: knowledge as usize,
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub active_episodes: usize,
    pub archived_episodes: usize,
    pub knowledge_count: usize,
}
