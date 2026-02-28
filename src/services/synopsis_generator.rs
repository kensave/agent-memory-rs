use crate::models::dtos::{Episode, Synopsis};
use crate::services::pattern_extractor::PatternExtractor;
use crate::storage::database::Database;
use anyhow::Result;
use rusqlite::params;
use std::collections::HashMap;

pub struct DailySynopsisGenerator {
    db: Database,
    extractor: PatternExtractor,
}

impl DailySynopsisGenerator {
    pub fn new(db: Database) -> Self {
        Self {
            extractor: PatternExtractor::new(db.clone()),
            db,
        }
    }

    pub fn generate_synopsis(&self, workspace_id: i64, date: &str) -> Result<Synopsis> {
        let episodes = self.get_episodes_for_date(workspace_id, date)?;
        
        let grouped = self.group_episodes_by_context(&episodes);
        let insights = self.extract_top_insights(&episodes, 5)?;
        let summary = self.generate_summary(&episodes, &grouped);
        let stats = self.calculate_daily_stats(&episodes);

        Ok(Synopsis {
            date: date.to_string(),
            workspace_id,
            agent_id: None,
            summary,
            key_insights: insights,
            new_knowledge_ids: vec![],
            new_procedure_ids: vec![],
            stats,
            created_at: None,
        })
    }

    fn get_episodes_for_date(&self, workspace_id: i64, date: &str) -> Result<Vec<Episode>> {
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, timestamp, conversation_id, event_type,
                        context, outcome, valence, archived, created_at
                 FROM episodes 
                 WHERE workspace_id = ? AND DATE(timestamp) = ? AND archived = 0"
            )?;
            
            let rows = stmt.query_map(params![workspace_id, date], |row| {
                Ok(Episode {
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
            })?;
            
            Ok(rows.filter_map(Result::ok).collect())
        })
    }

    fn group_episodes_by_context<'a>(&self, episodes: &'a [Episode]) -> HashMap<String, Vec<&'a Episode>> {
        let mut groups: HashMap<String, Vec<&'a Episode>> = HashMap::new();
        
        for episode in episodes {
            let key = episode.conversation_id.clone()
                .unwrap_or_else(|| episode.event_type.clone());
            groups.entry(key).or_default().push(episode);
        }
        
        groups
    }

    fn extract_top_insights(&self, episodes: &[Episode], limit: usize) -> Result<Vec<String>> {
        let patterns = self.extractor.extract_recurring_patterns(episodes)?;
        
        let mut insights: Vec<String> = patterns.iter()
            .take(limit)
            .map(|p| p.description.clone())
            .collect();

        let positive_count = episodes.iter()
            .filter(|e| e.valence.is_some_and(|v| v > 0.5))
            .count();
        
        if positive_count > episodes.len() / 2 {
            insights.push(format!("High success rate: {}% positive outcomes", 
                (positive_count * 100) / episodes.len()));
        }

        Ok(insights)
    }

    fn generate_summary(&self, episodes: &[Episode], grouped: &HashMap<String, Vec<&Episode>>) -> String {
        let total = episodes.len();
        let conversations = grouped.len();
        let event_types: std::collections::HashSet<_> = episodes.iter()
            .map(|e| &e.event_type)
            .collect();

        format!(
            "Processed {} episodes across {} conversations. Event types: {}. ",
            total,
            conversations,
            event_types.len()
        )
    }

    fn calculate_daily_stats(&self, episodes: &[Episode]) -> serde_json::Value {
        let total = episodes.len();
        let with_outcome = episodes.iter().filter(|e| e.outcome.is_some()).count();
        let positive = episodes.iter()
            .filter(|e| e.valence.is_some_and(|v| v > 0.0))
            .count();

        serde_json::json!({
            "total_episodes": total,
            "with_outcome": with_outcome,
            "positive_valence": positive,
            "success_rate": if with_outcome > 0 { 
                positive as f64 / with_outcome as f64 
            } else { 
                0.0 
            }
        })
    }

    pub fn store_synopsis(&self, synopsis: &Synopsis) -> Result<i64> {
        use crate::storage::memory_store::MemoryStore;
        
        // Store in daily_synopsis table
        let synopsis_id = self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO daily_synopsis 
                 (date, workspace_id, agent_id, summary, key_insights, 
                  new_knowledge_ids, new_procedure_ids, stats, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))",
                params![
                    synopsis.date,
                    synopsis.workspace_id,
                    synopsis.agent_id,
                    synopsis.summary,
                    serde_json::to_string(&synopsis.key_insights)?,
                    serde_json::to_string(&synopsis.new_knowledge_ids)?,
                    serde_json::to_string(&synopsis.new_procedure_ids)?,
                    synopsis.stats.to_string(),
                ]
            )?;
            Ok(conn.last_insert_rowid())
        })?;
        
        // Store as searchable semantic memory with 'synopsis' tag
        let memory_store = MemoryStore::new(self.db.clone());
        let synopsis_text = format!(
            "Daily Synopsis ({}): {} Key insights: {}",
            synopsis.date,
            synopsis.summary,
            synopsis.key_insights.join(", ")
        );
        
        let memory = crate::storage::Memory {
            id: None,
            workspace_id: synopsis.workspace_id,
            agent_id: synopsis.agent_id,
            text: synopsis_text,
            tags: Some("synopsis".to_string()),
            importance_score: 0.9,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            source_episodes: vec![],
            confidence: 0.9,
            last_validated: None,
            created_at: None,
            updated_at: None,
        };
        
        let _ = memory_store.insert_memory(&memory);
        
        Ok(synopsis_id)
    }
}
