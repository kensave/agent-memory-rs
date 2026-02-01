use crate::models::dtos::{Episode, Pattern};
use crate::storage::database::Database;
use anyhow::Result;
use rusqlite::params;
use std::collections::HashMap;

pub struct PatternExtractor {
    db: Database,
}

impl PatternExtractor {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn extract_recurring_patterns(&self, episodes: &[Episode]) -> Result<Vec<Pattern>> {
        let mut event_counts: HashMap<String, Vec<i64>> = HashMap::new();
        
        for episode in episodes {
            event_counts
                .entry(episode.event_type.clone())
                .or_insert_with(Vec::new)
                .push(episode.id.unwrap_or(0));
        }

        let mut patterns = Vec::new();
        for (event_type, episode_ids) in event_counts {
            if episode_ids.len() >= 2 {
                patterns.push(Pattern {
                    pattern_type: "recurring_event".to_string(),
                    description: format!("Recurring event: {}", event_type),
                    frequency: episode_ids.len() as i64,
                    confidence: (episode_ids.len() as f64 / episodes.len() as f64).min(1.0),
                    source_episodes: episode_ids,
                });
            }
        }

        Ok(patterns)
    }

    pub fn extract_user_preferences(&self, episodes: &[Episode]) -> Result<Vec<Pattern>> {
        let mut preferences = Vec::new();
        let mut positive_outcomes: HashMap<String, i64> = HashMap::new();

        for episode in episodes {
            if let Some(valence) = episode.valence {
                if valence > 0.5 {
                    if let Some(outcome) = &episode.outcome {
                        *positive_outcomes.entry(outcome.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        for (outcome, count) in positive_outcomes {
            if count >= 2 {
                preferences.push(Pattern {
                    pattern_type: "user_preference".to_string(),
                    description: format!("Preferred outcome: {}", outcome),
                    frequency: count,
                    confidence: 0.7,
                    source_episodes: vec![],
                });
            }
        }

        Ok(preferences)
    }

    pub fn extract_successful_workflows(&self, episodes: &[Episode]) -> Result<Vec<Pattern>> {
        let mut workflows = Vec::new();
        let mut sequences: HashMap<String, i64> = HashMap::new();

        let mut sorted_episodes = episodes.to_vec();
        sorted_episodes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for window in sorted_episodes.windows(2) {
            if let Some(conv_id) = &window[0].conversation_id {
                if window[1].conversation_id.as_ref() == Some(conv_id) {
                    let sequence = format!("{} -> {}", window[0].event_type, window[1].event_type);
                    *sequences.entry(sequence).or_insert(0) += 1;
                }
            }
        }

        for (sequence, count) in sequences {
            if count >= 2 {
                workflows.push(Pattern {
                    pattern_type: "workflow".to_string(),
                    description: format!("Common workflow: {}", sequence),
                    frequency: count,
                    confidence: 0.6,
                    source_episodes: vec![],
                });
            }
        }

        Ok(workflows)
    }

    pub fn cluster_similar_episodes(&self, workspace_id: i64, _similarity_threshold: f64) -> Result<Vec<Pattern>> {
        let episodes = self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, event_type FROM episodes WHERE workspace_id = ? AND archived = 0"
            )?;
            
            let rows = stmt.query_map(params![workspace_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        let mut clusters: HashMap<String, Vec<i64>> = HashMap::new();
        for (id, event_type) in episodes {
            clusters.entry(event_type).or_insert_with(Vec::new).push(id);
        }

        let mut patterns = Vec::new();
        for (event_type, episode_ids) in clusters {
            if episode_ids.len() >= 3 {
                patterns.push(Pattern {
                    pattern_type: "cluster".to_string(),
                    description: format!("Event cluster: {}", event_type),
                    frequency: episode_ids.len() as i64,
                    confidence: 0.8,
                    source_episodes: episode_ids,
                });
            }
        }

        Ok(patterns)
    }

    pub fn extract_all_patterns(&self, workspace_id: i64) -> Result<Vec<Pattern>> {
        let episodes = self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, workspace_id, agent_id, timestamp, conversation_id, event_type, 
                        context, outcome, valence, archived, created_at
                 FROM episodes WHERE workspace_id = ? AND archived = 0"
            )?;
            
            let rows = stmt.query_map(params![workspace_id], |row| {
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
            
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        let mut all_patterns = Vec::new();
        all_patterns.extend(self.extract_recurring_patterns(&episodes)?);
        all_patterns.extend(self.extract_user_preferences(&episodes)?);
        all_patterns.extend(self.extract_successful_workflows(&episodes)?);
        all_patterns.extend(self.cluster_similar_episodes(workspace_id, 0.8)?);

        Ok(all_patterns)
    }
}
