use anyhow::Result;
use async_trait::async_trait;

/// Trait for memory decay and archival operations
/// Handles scoring, pruning, and archival
#[async_trait]
pub trait DecayManager: Send + Sync {
    /// Calculate composite score for a memory
    fn calculate_score(&self, recency: f64, relevance: f64, utility: f64) -> f64;

    /// Archive low-scoring memories
    async fn archive_low_scoring(&self, threshold: f64, dry_run: bool) -> Result<Vec<i64>>;

    /// Prune redundant memories
    async fn prune_redundant(&self, similarity_threshold: f64, dry_run: bool) -> Result<Vec<i64>>;

    /// Remove unused procedures
    async fn remove_unused(&self, days_inactive: i64, dry_run: bool) -> Result<Vec<i64>>;
}
