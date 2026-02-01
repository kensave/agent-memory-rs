use anyhow::Result;
use async_trait::async_trait;

/// Core trait for memory storage operations (CRUD)
/// Follows Single Responsibility Principle - only handles storage
#[async_trait]
pub trait MemoryStore: Send + Sync {
    type Memory;
    type Id;

    /// Store a new memory
    async fn store(&self, memory: Self::Memory) -> Result<Self::Id>;

    /// Retrieve memory by ID
    async fn get(&self, id: Self::Id) -> Result<Option<Self::Memory>>;

    /// Update existing memory
    async fn update(&self, id: Self::Id, memory: Self::Memory) -> Result<()>;

    /// Delete memory
    async fn delete(&self, id: Self::Id) -> Result<()>;

    /// Batch store for efficiency
    async fn store_batch(&self, memories: Vec<Self::Memory>) -> Result<Vec<Self::Id>>;
}
