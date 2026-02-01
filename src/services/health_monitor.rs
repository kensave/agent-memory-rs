use crate::services::memory_manager::MemoryManager;
use anyhow::Result;

pub struct HealthMonitor {
    manager: MemoryManager,
}

#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub total_memories: usize,
    pub active_ratio: f64,
    pub avg_confidence: f64,
    pub recent_activity: usize,
    pub health_score: f64,
}

impl HealthMonitor {
    pub fn new(manager: MemoryManager) -> Self {
        Self { manager }
    }

    pub fn calculate_metrics(&self, workspace_id: i64) -> Result<HealthMetrics> {
        let stats = self.manager.get_memory_stats(workspace_id)?;
        
        let total_episodes = stats.active_episodes + stats.archived_episodes;
        let active_ratio = if total_episodes > 0 {
            stats.active_episodes as f64 / total_episodes as f64
        } else {
            0.0
        };

        let avg_confidence = self.calculate_avg_confidence(workspace_id)?;
        let recent_activity = stats.active_episodes;

        // Health score: weighted combination
        let health_score = (active_ratio * 0.3) + (avg_confidence * 0.4) + 
                          (if recent_activity > 0 { 0.3 } else { 0.0 });

        Ok(HealthMetrics {
            total_memories: total_episodes + stats.knowledge_count + stats.procedure_count,
            active_ratio,
            avg_confidence,
            recent_activity,
            health_score,
        })
    }

    fn calculate_avg_confidence(&self, workspace_id: i64) -> Result<f64> {
        self.manager.db.execute(|conn| {
            let result: Result<f64, rusqlite::Error> = conn.query_row(
                "SELECT AVG(confidence) FROM memories WHERE workspace_id = ?",
                rusqlite::params![workspace_id],
                |row| row.get(0)
            );
            Ok(result.unwrap_or(0.5))
        })
    }

    pub fn check_health(&self, workspace_id: i64) -> Result<String> {
        let metrics = self.calculate_metrics(workspace_id)?;
        
        let status = if metrics.health_score > 0.7 {
            "HEALTHY"
        } else if metrics.health_score > 0.4 {
            "MODERATE"
        } else {
            "NEEDS ATTENTION"
        };

        Ok(format!(
            "Memory Health: {} ({:.1}%)\n\
             Total Memories: {}\n\
             Active Ratio: {:.1}%\n\
             Avg Confidence: {:.2}\n\
             Recent Activity: {}",
            status,
            metrics.health_score * 100.0,
            metrics.total_memories,
            metrics.active_ratio * 100.0,
            metrics.avg_confidence,
            metrics.recent_activity
        ))
    }
}
