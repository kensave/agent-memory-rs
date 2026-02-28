# Interface Design - SOLID Principles

**Date**: 2026-01-31  
**Status**: Design Complete  
**Location**: `<project-root>/src/traits/`

---

## Overview

This document describes the trait-based interface design for the Agent Memory Management System, following SOLID principles.

---

## SOLID Principles Applied

### 1. Single Responsibility Principle (SRP)
Each trait has ONE reason to change:
- `MemoryStore` - only storage operations (CRUD)
- `MemoryRetriever` - only retrieval/search operations
- `EmbeddingService` - only embedding generation

### 2. Open/Closed Principle (OCP)
- Traits are open for extension (new implementations)
- Closed for modification (trait definitions stable)
- Example: Can add `FastEmbedder`, `OpenAIEmbedder`, `LocalEmbedder` without changing trait

### 3. Liskov Substitution Principle (LSP)
- Any implementation of a trait can be substituted
- Contracts defined by trait methods must be honored
- Example: Any `EmbeddingService` can replace another without breaking code

### 4. Interface Segregation Principle (ISP)
- Clients depend only on methods they use
- `MemoryStore` and `MemoryRetriever` are separate (not one fat interface)
- Services can implement only needed traits

### 5. Dependency Inversion Principle (DIP)
- High-level modules depend on abstractions (traits), not concrete types
- Example: `MemoryManager` depends on `MemoryStore` trait, not specific implementation
- Enables dependency injection and testing

---

## Trait Definitions

### 1. MemoryStore

**Purpose**: Core CRUD operations for memory storage

**Responsibilities**:
- Store new memories
- Retrieve memories by ID
- Update existing memories
- Delete memories
- Batch operations

**Generic Types**:
- `Memory`: The memory type (Episode, Procedure, etc.)
- `Id`: The identifier type (i64, UUID, etc.)

**Methods**:
```rust
async fn store(&self, memory: Self::Memory) -> Result<Self::Id>
async fn get(&self, id: Self::Id) -> Result<Option<Self::Memory>>
async fn update(&self, id: Self::Id, memory: Self::Memory) -> Result<()>
async fn delete(&self, id: Self::Id) -> Result<()>
async fn store_batch(&self, memories: Vec<Self::Memory>) -> Result<Vec<Self::Id>>
```

**Implementations**:
- `EpisodicMemoryStore` (stores Episode)

---

### 2. MemoryRetriever

**Purpose**: Search and retrieval operations

**Responsibilities**:
- Search by query
- Filter by time range
- Filter by conversation

**Generic Types**:
- `Memory`: The memory type
- `Query`: Query type (string, embedding, etc.)
- `Filters`: Filter criteria
- `Result`: Search result with scores

**Methods**:
```rust
async fn search(&self, query: Self::Query, filters: Self::Filters) -> Result<Vec<Self::Result>>
async fn get_by_time_range(&self, start: String, end: String, filters: Self::Filters) -> Result<Vec<Self::Memory>>
async fn get_by_conversation(&self, conversation_id: String) -> Result<Vec<Self::Memory>>
```

**Implementations**:
- `HybridRetrievalEngine` (BM25 + vector search)
- `VectorRetriever` (pure vector search)
- `KeywordRetriever` (pure BM25 search)

---

### 3. EmbeddingService

**Purpose**: Generate embeddings for text

**Responsibilities**:
- Single text embedding
- Batch text embedding
- Report embedding dimensions

**Methods**:
```rust
async fn embed(&self, text: &str) -> Result<Vec<f32>>
async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>
fn dimensions(&self) -> usize
```

**Implementations**:
- `FastEmbedder` (existing - BERT MiniLM)
- `OpenAIEmbedder` (future - OpenAI API)
- `LocalLlamaEmbedder` (future - local LLM)

**Benefits**:
- Easy to swap embedding models
- Testable with mock embedders
- Can use different models for different memory types

---

## Data Transfer Objects (DTOs)

### Episode
```rust
pub struct Episode {
    pub id: Option<i64>,
    pub workspace_id: i64,
    pub agent_id: Option<i64>,
    pub timestamp: String,
    pub conversation_id: Option<String>,
    pub event_type: String,
    pub context: serde_json::Value,
    pub outcome: Option<String>,
    pub valence: Option<f64>,
    pub archived: bool,
    pub created_at: Option<String>,
}
```

**Purpose**: Store specific interaction events with full context

**Fields**:
- `event_type`: "user_query", "tool_execution", "error", etc.
- `context`: Full JSON context (request, response, state)
- `outcome`: Result of the interaction
- `valence`: Emotional valence (-1.0 negative, +1.0 positive)
- `archived`: Whether episode is archived

---

## Dependency Injection Pattern

### Example: MemoryManager Implementation

```rust
pub struct MemoryManager<S, R, E>
where
    S: MemoryStore,
    R: MemoryRetriever,
    E: EmbeddingService,
{
    episode_store: Arc<S>,
    retriever: Arc<R>,
    embedder: Arc<E>,
}

impl<S, R, E> MemoryManager<S, R, E>
where
    S: MemoryStore,
    R: MemoryRetriever,
    E: EmbeddingService,
{
    pub fn new(
        episode_store: Arc<S>,
        retriever: Arc<R>,
        embedder: Arc<E>,
    ) -> Self {
        Self {
            episode_store,
            retriever,
            embedder,
        }
    }
}
```

**Benefits**:
- Testable with mock implementations
- Flexible - can swap implementations
- Follows DIP - depends on abstractions

---

## Testing Strategy

### Unit Tests
- Test each trait implementation in isolation
- Use mock implementations for dependencies
- Example: Test `EpisodicMemoryStore` with mock database

### Integration Tests
- Test trait implementations together
- Example: Test `MemoryManager` with real stores

### Contract Tests
- Verify trait implementations honor contracts
- Example: All `MemoryStore` implementations must handle batch operations

---

## Migration from Existing Code

### Existing Code
```rust
pub struct MemorySystem {
    store: MemoryStore,
    embedder: FastEmbedder,
}
```

### New Code (with traits)
```rust
pub struct MemorySystem<S, E>
where
    S: MemoryStore,
    E: EmbeddingService,
{
    store: Arc<S>,
    embedder: Arc<E>,
}
```

**Benefits**:
- Existing code continues to work
- New code uses trait abstractions
- Gradual migration possible

---

## Next Steps

1. ✅ Define traits (DONE)
2. ✅ Define DTOs (DONE)
3. ⏭️ Implement database migrations (Task 2)
4. ⏭️ Implement trait for existing `MemoryStore`
5. ⏭️ Implement new memory stores (Episodic, Procedural)
6. ⏭️ Implement services using traits

---

## Files Created

1. `<project-root>/src/traits/mod.rs` - Module exports
2. `<project-root>/src/traits/memory_store.rs` - MemoryStore trait
3. `<project-root>/src/traits/retriever.rs` - MemoryRetriever trait
4. `<project-root>/src/traits/embedder.rs` - EmbeddingService trait
5. `/Users/kenneth/workspace/memory-rs/src/traits/consolidation.rs` - ConsolidationEngine trait
6. `/Users/kenneth/workspace/memory-rs/src/models/mod.rs` - Models module
7. `/Users/kenneth/workspace/memory-rs/src/models/dtos.rs` - DTOs

---

## References

- SOLID Principles: https://en.wikipedia.org/wiki/SOLID
- Rust Traits: https://doc.rust-lang.org/book/ch10-02-traits.html
- Dependency Injection in Rust: https://github.com/Mcat12/shaku
- Async Trait: https://docs.rs/async-trait/
