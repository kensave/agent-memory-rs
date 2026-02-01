use crate::models::dtos::{Pattern, Procedure, Synopsis};
use crate::services::pattern_extractor::PatternExtractor;
use crate::services::procedural_store::ProceduralMemoryStore;
use crate::services::synopsis_generator::DailySynopsisGenerator;
use crate::storage::database::Database;
use crate::storage::memory_store::{Memory, MemoryStore};
use crate::traits::consolidation::ConsolidationEngine as ConsolidationEngineTrait;
use crate::traits::memory_store::MemoryStore as MemoryStoreTrait;
use anyhow::Result;
use async_trait::async_trait;
use rusqlite::params;
use serde_json::json;

pub struct ConsolidationEngine {
    db: Database,
    pattern_extractor: PatternExtractor,
    synopsis_generator: DailySynopsisGenerator,
    procedural_store: ProceduralMemoryStore,
    memory_store: MemoryStore,
}

impl ConsolidationEngine {
    pub fn new(db: Database) -> Self {
        Self {
            pattern_extractor: PatternExtractor::new(db.clone()),
            synopsis_generator: DailySynopsisGenerator::new(db.clone()),
            procedural_store: ProceduralMemoryStore::new(db.clone()),
            memory_store: MemoryStore::new(db.clone()),
            db,
        }
    }

    async fn update_semantic_memory(&self, workspace_id: i64, patterns: &[Pattern]) -> Result<Vec<i64>> {
        let mut knowledge_ids = Vec::new();
        
        for pattern in patterns {
            if pattern.confidence > 0.6 {
                let memory = Memory {
                    id: None,
                    workspace_id,
                    agent_id: None,
                    text: pattern.description.clone(),
                    tags: Some(pattern.pattern_type.clone()),
                    importance_score: pattern.confidence,
                    access_count: 0,
                    last_accessed: None,
                    conversation_id: None,
                    parent_memory_id: None,
                    user_feedback: None,
                    source_episodes: pattern.source_episodes.clone(),
                    confidence: pattern.confidence,
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

    async fn update_procedural_memory(&self, workspace_id: i64, patterns: &[Pattern]) -> Result<Vec<i64>> {
        let mut procedure_ids = Vec::new();
        
        for pattern in patterns {
            if pattern.pattern_type == "workflow" && pattern.frequency >= 2 {
                let procedure = Procedure {
                    id: None,
                    workspace_id,
                    name: pattern.description.clone(),
                    trigger_conditions: json!({"pattern_type": pattern.pattern_type}),
                    action_sequence: json!({"description": pattern.description}),
                    success_rate: pattern.confidence,
                    usage_count: pattern.frequency,
                    last_used: None,
                    learned_from: pattern.source_episodes.clone(),
                    created_at: None,
                };
                
                let id = self.procedural_store.store(procedure).await?;
                procedure_ids.push(id);
            }
        }
        
        Ok(procedure_ids)
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

        for workspace_id in &workspaces {
            let patterns = self.pattern_extractor.extract_all_patterns(*workspace_id)?;
            
            let knowledge_ids = self.update_semantic_memory(*workspace_id, &patterns).await?;
            let procedure_ids = self.update_procedural_memory(*workspace_id, &patterns).await?;
            
            let mut synopsis = self.synopsis_generator.generate_synopsis(*workspace_id, &date)?;
            synopsis.new_knowledge_ids = knowledge_ids;
            synopsis.new_procedure_ids = procedure_ids;
            
            self.synopsis_generator.store_synopsis(&synopsis)?;
            
            self.mark_episodes_for_archival(*workspace_id, &date).await?;
        }

        let synopsis = self.synopsis_generator.generate_synopsis(first_workspace, &date)?;
        
        Ok(synopsis)
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
