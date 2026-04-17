use anyhow::Result;
use async_trait::async_trait;

/// Trait for memory retrieval and search operations
/// Separated from storage (Interface Segregation Principle)
#[async_trait]
pub trait MemoryRetriever: Send + Sync {
    type Memory;
    type Query;
    type Filters;
    type Result;

    /// Search memories by query
    async fn search(&self, query: Self::Query, filters: Self::Filters)
        -> Result<Vec<Self::Result>>;

    /// Get memories by time range
    async fn get_by_time_range(
        &self,
        start: String,
        end: String,
        filters: Self::Filters,
    ) -> Result<Vec<Self::Memory>>;

    /// Get memories by conversation
    async fn get_by_conversation(&self, conversation_id: String) -> Result<Vec<Self::Memory>>;
}
