use crate::services::composite_score::CompositeScoreCalculator;
use crate::storage::database::Database;
use crate::storage::memory_store::MemoryStore;
use crate::traits::decay::DecayManager as DecayManagerTrait;
use anyhow::Result;
use async_trait::async_trait;
use rusqlite::params;

pub struct DecayManager {
    db: Database,
    calculator: CompositeScoreCalculator,
    memory_store: MemoryStore,
}

impl DecayManager {
    pub fn new(db: Database) -> Self {
        Self {
            memory_store: MemoryStore::new(db.clone()),
            calculator: CompositeScoreCalculator::new(),
            db,
        }
    }

    pub async fn archive_episodes(&self, workspace_id: i64, threshold: f64, dry_run: bool) -> Result<Vec<i64>> {
        let episodes = self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp FROM episodes 
                 WHERE workspace_id = ? AND archived = 0"
            )?;
            
            let rows = stmt.query_map(params![workspace_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        let mut to_archive = Vec::new();
        for (id, timestamp) in episodes {
            let recency = self.calculator.calculate_recency(&timestamp)?;
            if recency < threshold {
                to_archive.push(id);
            }
        }

        if !dry_run && !to_archive.is_empty() {
            for id in &to_archive {
                self.db.execute(|conn| {
                    conn.execute("UPDATE episodes SET archived = 1 WHERE id = ?", params![id])?;
                    Ok(())
                })?;
            }
        }

        Ok(to_archive)
    }

    pub async fn prune_low_confidence(&self, workspace_id: i64, threshold: f64, dry_run: bool) -> Result<Vec<i64>> {
        let memories = self.memory_store.get_by_confidence_threshold(workspace_id, 0.0)?;
        
        let to_prune: Vec<i64> = memories.iter()
            .filter(|m| m.confidence < threshold)
            .filter_map(|m| m.id)
            .collect();

        if !dry_run && !to_prune.is_empty() {
            for id in &to_prune {
                self.db.execute(|conn| {
                    conn.execute("DELETE FROM memories WHERE id = ?", params![id])?;
                    conn.execute("DELETE FROM vec0 WHERE memory_id = ?", params![id])?;
                    Ok(())
                })?;
            }
        }

        Ok(to_prune)
    }

    pub async fn remove_inactive_procedures(&self, workspace_id: i64, days: i64, dry_run: bool) -> Result<Vec<i64>> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff_date.format("%Y-%m-%d %H:%M:%S").to_string();

        let to_remove = self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM procedures 
                 WHERE workspace_id = ? AND (last_used IS NULL OR last_used < ?)"
            )?;
            
            let rows = stmt.query_map(params![workspace_id, cutoff_str], |row| {
                row.get::<_, i64>(0)
            })?;
            
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        if !dry_run && !to_remove.is_empty() {
            for id in &to_remove {
                self.db.execute(|conn| {
                    conn.execute("DELETE FROM procedures WHERE id = ?", params![id])?;
                    Ok(())
                })?;
            }
        }

        Ok(to_remove)
    }
}

#[async_trait]
impl DecayManagerTrait for DecayManager {
    fn calculate_score(&self, recency: f64, relevance: f64, utility: f64) -> f64 {
        self.calculator.calculate_composite(recency, relevance, utility).combined
    }

    async fn archive_low_scoring(&self, threshold: f64, dry_run: bool) -> Result<Vec<i64>> {
        let workspaces = self.db.execute(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM workspaces")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        let mut all_archived = Vec::new();
        for ws_id in workspaces {
            let archived = self.archive_episodes(ws_id, threshold, dry_run).await?;
            all_archived.extend(archived);
        }

        Ok(all_archived)
    }

    async fn prune_redundant(&self, similarity_threshold: f64, dry_run: bool) -> Result<Vec<i64>> {
        let workspaces = self.db.execute(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM workspaces")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        let mut all_pruned = Vec::new();
        for ws_id in workspaces {
            let pruned = self.prune_low_confidence(ws_id, similarity_threshold, dry_run).await?;
            all_pruned.extend(pruned);
        }

        Ok(all_pruned)
    }

    async fn remove_unused(&self, days_inactive: i64, dry_run: bool) -> Result<Vec<i64>> {
        let workspaces = self.db.execute(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM workspaces")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        let mut all_removed = Vec::new();
        for ws_id in workspaces {
            let removed = self.remove_inactive_procedures(ws_id, days_inactive, dry_run).await?;
            all_removed.extend(removed);
        }

        Ok(all_removed)
    }
}
