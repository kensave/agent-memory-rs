use anyhow::Result;
use async_trait::async_trait;

/// Trait for embedding generation
/// Abstraction allows swapping embedding models (Dependency Inversion)
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embedding for single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for batch of texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get embedding dimensions
    fn dimensions(&self) -> usize;
}
