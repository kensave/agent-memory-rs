use crate::models::dtos::{Pattern, Synopsis};
use crate::services::pattern_extractor::PatternExtractor;
use crate::services::synopsis_generator::DailySynopsisGenerator;
use crate::storage::database::Database;
use crate::storage::memory_store::{Memory, MemoryStore};
use crate::traits::consolidation::ConsolidationEngine as ConsolidationEngineTrait;
use anyhow::Result;
use async_trait::async_trait;
use rusqlite::params;

pub struct ConsolidationEngine {
    db: Database,
    pattern_extractor: PatternExtractor,
    synopsis_generator: DailySynopsisGenerator,
    memory_store: MemoryStore,
}

impl ConsolidationEngine {
    pub fn new(db: Database) -> Self {
        Self {
            pattern_extractor: PatternExtractor::new(db.clone()),
            synopsis_generator: DailySynopsisGenerator::new(db.clone()),
            memory_store: MemoryStore::new(db.clone()),
            db,
        }
    }

    async fn update_semantic_memory(&self, workspace_id: i64, patterns: &[Pattern]) -> Result<Vec<i64>> {
        let mut knowledge_ids = Vec::new();
        
        for pattern in patterns {
            if pattern.confidence > 0.6 {
                // Boost importance of related existing memories
                for episode_id in &pattern.source_episodes {
                    let _ = self.db.execute(|conn| {
                        let count = conn.execute(
                            "UPDATE memories 
                             SET importance_score = MIN(importance_score * 1.2, 1.0)
                             WHERE ? = ANY(source_episodes)",
                            params![episode_id]
                        )?;
                        Ok(count)
                    });
                }
                
                // Store pattern as high-confidence semantic memory
                let memory = Memory {
                    id: None,
                    workspace_id,
                    agent_id: None,
                    text: pattern.description.clone(),
                    tags: Some(pattern.pattern_type.clone()),
                    importance_score: pattern.confidence.max(0.8),
                    access_count: 0,
                    last_accessed: None,
                    conversation_id: None,
                    parent_memory_id: None,
                    user_feedback: None,
                    source_episodes: pattern.source_episodes.clone(),
                    confidence: pattern.confidence.max(0.8),
                    last_validated: None,
                    created_at: None,
                    updated_at: None,
                };
                
                let id = self.memory_store.insert_memory(&memory)?;
                knowledge_ids.push(id);
            }
        }
        
        Ok(knowledge_ids)
    }


    async fn mark_episodes_for_archival(&self, workspace_id: i64, date: &str) -> Result<usize> {
        self.db.execute(|conn| {
            let count = conn.execute(
                "UPDATE episodes 
                 SET archived = 1 
                 WHERE workspace_id = ? AND DATE(timestamp) = ? AND archived = 0",
                params![workspace_id, date]
            )?;
            Ok(count)
        })
    }
}

#[async_trait]
impl ConsolidationEngineTrait for ConsolidationEngine {
    type Synopsis = Synopsis;
    type Pattern = Pattern;

    async fn consolidate_daily(&self, date: String) -> Result<Self::Synopsis> {
        let workspaces: Vec<i64> = self.db.execute(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM workspaces")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            Ok(rows.filter_map(Result::ok).collect())
        })?;

        let first_workspace = workspaces.first().copied().unwrap_or(1);
        let mut first_synopsis = None;

        for workspace_id in &workspaces {
            let patterns = self.pattern_extractor.extract_all_patterns(*workspace_id)?;
            
            let knowledge_ids = self.update_semantic_memory(*workspace_id, &patterns).await?;
            
            let mut synopsis = self.synopsis_generator.generate_synopsis(*workspace_id, &date)?;
            synopsis.new_knowledge_ids = knowledge_ids;
            
            self.synopsis_generator.store_synopsis(&synopsis)?;
            
            // Save first workspace synopsis for return
            if *workspace_id == first_workspace {
                first_synopsis = Some(synopsis);
            }
            
            self.mark_episodes_for_archival(*workspace_id, &date).await?;
        }

        // Return the synopsis that was generated before archival
        Ok(first_synopsis.unwrap_or_else(|| {
            // Fallback if no workspaces (shouldn't happen)
            self.synopsis_generator.generate_synopsis(first_workspace, &date).unwrap_or_default()
        }))
    }

    async fn extract_patterns(&self, episode_ids: Vec<i64>) -> Result<Vec<Self::Pattern>> {
        let episodes = self.db.execute(|conn| {
            let mut all_episodes = Vec::new();
            for id in episode_ids {
                if let Ok(episode) = conn.query_row(
                    "SELECT id, workspace_id, agent_id, timestamp, conversation_id, event_type,
                            context, outcome, valence, archived, created_at
                     FROM episodes WHERE id = ?",
                    params![id],
                    |row| {
                        Ok(crate::models::dtos::Episode {
                            id: Some(row.get(0)?),
                            workspace_id: row.get(1)?,
                            agent_id: row.get(2)?,
                            timestamp: row.get(3)?,
                            conversation_id: row.get(4)?,
                            event_type: row.get(5)?,
                            context: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                            outcome: row.get(7)?,
                            valence: row.get(8)?,
                            archived: row.get::<_, i64>(9)? != 0,
                            created_at: row.get(10)?,
                        })
                    }
                ) {
                    all_episodes.push(episode);
                }
            }
            Ok(all_episodes)
        })?;

        self.pattern_extractor.extract_recurring_patterns(&episodes)
    }

    async fn generate_synopsis(&self, date: String) -> Result<Self::Synopsis> {
        let workspace_id = self.db.execute(|conn| {
            Ok(conn.query_row("SELECT id FROM workspaces LIMIT 1", [], |row| row.get(0))?)
        })?;
        
        self.synopsis_generator.generate_synopsis(workspace_id, &date)
    }
}
