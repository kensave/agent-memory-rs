use crate::models::dtos::{Episode, Procedure, Synopsis};
use crate::services::consolidation_engine::ConsolidationEngine;
use crate::services::decay_manager::DecayManager;
use crate::services::episodic_store::EpisodicMemoryStore;
use crate::services::hybrid_retrieval::{HybridRetrievalEngine, HybridSearchResult};
use crate::services::procedural_store::ProceduralMemoryStore;
use crate::storage::database::Database;
use crate::storage::memory_store::{Memory, MemoryStore as SemanticStore};
use crate::traits::consolidation::ConsolidationEngine as ConsolidationTrait;
use crate::traits::memory_store::MemoryStore;
use anyhow::Result;

pub struct MemoryManager {
    pub(crate) db: Database,
    episodic: EpisodicMemoryStore,
    procedural: ProceduralMemoryStore,
    semantic: SemanticStore,
    retrieval: HybridRetrievalEngine,
    consolidation: ConsolidationEngine,
    decay: DecayManager,
}

impl MemoryManager {
    pub fn new(db: Database) -> Self {
        Self {
            episodic: EpisodicMemoryStore::new(db.clone()),
            procedural: ProceduralMemoryStore::new(db.clone()),
            semantic: SemanticStore::new(db.clone()),
            retrieval: HybridRetrievalEngine::new(db.clone()),
            consolidation: ConsolidationEngine::new(db.clone()),
            decay: DecayManager::new(db.clone()),
            db,
        }
    }

    pub async fn store_episode(&self, episode: Episode) -> Result<i64> {
        self.episodic.store(episode).await
    }

    pub async fn store_procedure(&self, procedure: Procedure) -> Result<i64> {
        self.procedural.store(procedure).await
    }

    pub fn store_knowledge(&self, memory: &Memory) -> Result<i64> {
        self.semantic.insert_memory(memory)
    }

    pub fn retrieve(&self, query: &str, workspace_id: i64, limit: usize) -> Result<Vec<HybridSearchResult>> {
        self.retrieval.hybrid_search(query, workspace_id, limit)
    }

    pub fn retrieve_hierarchical(&self, query: &str, workspace_id: i64, max_results: usize) -> Result<Vec<HybridSearchResult>> {
        let mut results = Vec::new();
        
        // Level 1: Semantic memory (high confidence)
        let semantic = self.retrieval.search_by_type(query, workspace_id, "semantic", max_results / 2)?;
        results.extend(semantic);
        
        // Level 2: Recent episodes
        let episodic = self.retrieval.search_by_type(query, workspace_id, "episodic", max_results / 4)?;
        results.extend(episodic);
        
        // Level 3: Procedures
        let procedural = self.retrieval.search_by_type(query, workspace_id, "procedural", max_results / 4)?;
        results.extend(procedural);
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results);
        Ok(results)
    }

    pub fn get_synopsis(&self, workspace_id: i64, date: &str) -> Result<Option<Synopsis>> {
        self.db.execute(|conn| {
            let result = conn.query_row(
                "SELECT date, workspace_id, agent_id, summary, key_insights,
                        new_knowledge_ids, new_procedure_ids, stats, created_at
                 FROM daily_synopsis
                 WHERE workspace_id = ? AND date = ?",
                rusqlite::params![workspace_id, date],
                |row| {
                    Ok(Synopsis {
                        date: row.get(0)?,
                        workspace_id: row.get(1)?,
                        agent_id: row.get(2)?,
                        summary: row.get(3)?,
                        key_insights: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                        new_knowledge_ids: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                        new_procedure_ids: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                        stats: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                        created_at: row.get(8)?,
                    })
                }
            );
            
            match result {
                Ok(synopsis) => Ok(Some(synopsis)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    pub async fn consolidate(&self, date: String) -> Result<Synopsis> {
        self.consolidation.consolidate_daily(date).await
    }

    pub async fn prune(&self, workspace_id: i64, dry_run: bool) -> Result<(usize, usize, usize)> {
        let episodes = self.decay.archive_episodes(workspace_id, 0.3, dry_run).await?;
        let knowledge = self.decay.prune_low_confidence(workspace_id, 0.4, dry_run).await?;
        let procedures = self.decay.remove_inactive_procedures(workspace_id, 90, dry_run).await?;
        Ok((episodes.len(), knowledge.len(), procedures.len()))
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
            
            let procedures: i64 = conn.query_row(
                "SELECT COUNT(*) FROM procedures WHERE workspace_id = ?",
                rusqlite::params![workspace_id],
                |row| row.get(0)
            )?;
            
            Ok(MemoryStats {
                active_episodes: episodes as usize,
                archived_episodes: archived as usize,
                knowledge_count: knowledge as usize,
                procedure_count: procedures as usize,
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub active_episodes: usize,
    pub archived_episodes: usize,
    pub knowledge_count: usize,
    pub procedure_count: usize,
}
