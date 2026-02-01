use anyhow::Result;
use async_trait::async_trait;

/// Trait for memory consolidation operations
/// Orchestrates pattern extraction and synopsis generation
#[async_trait]
pub trait ConsolidationEngine: Send + Sync {
    type Synopsis;
    type Pattern;

    /// Consolidate memories for a given date
    async fn consolidate_daily(&self, date: String) -> Result<Self::Synopsis>;

    /// Extract patterns from episodic memories
    async fn extract_patterns(&self, episode_ids: Vec<i64>) -> Result<Vec<Self::Pattern>>;

    /// Generate daily synopsis
    async fn generate_synopsis(&self, date: String) -> Result<Self::Synopsis>;
}
