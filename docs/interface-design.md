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
- `ConsolidationEngine` - only consolidation orchestration

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
- Example: `ConsolidationEngine` depends on `MemoryStore` trait, not specific implementation
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

### 4. ConsolidationEngine

**Purpose**: Orchestrate memory consolidation

**Responsibilities**:
- Daily consolidation workflow
- Pattern extraction
- Synopsis generation

**Generic Types**:
- `Synopsis`: Synopsis type
- `Pattern`: Pattern type

**Methods**:
```rust
async fn consolidate_daily(&self, date: String) -> Result<Self::Synopsis>
async fn extract_patterns(&self, episode_ids: Vec<i64>) -> Result<Vec<Self::Pattern>>
async fn generate_synopsis(&self, date: String) -> Result<Self::Synopsis>
```

**Implementations**:
- `DefaultConsolidationEngine` (main implementation)
- `TestConsolidationEngine` (for testing)

**Dependencies** (via DIP):
- `MemoryStore` (to retrieve episodes)
- `MemoryRetriever` (to search patterns)
- `EmbeddingService` (to generate synopsis embeddings)

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

### Procedure
```rust
pub struct Procedure {
    pub id: Option<i64>,
    pub workspace_id: i64,
    pub name: String,
    pub trigger_conditions: serde_json::Value,
    pub action_sequence: serde_json::Value,
    pub success_rate: f64,
    pub usage_count: i64,
    pub last_used: Option<String>,
    pub learned_from: Vec<i64>,
    pub created_at: Option<String>,
}
```

**Purpose**: Store learned workflows and action sequences

**Fields**:
- `trigger_conditions`: JSON conditions that trigger this procedure
- `action_sequence`: JSON array of actions to execute
- `success_rate`: 0.0 to 1.0 success rate
- `usage_count`: Number of times used
- `learned_from`: Episode IDs that led to this procedure

---

### Synopsis
```rust
pub struct Synopsis {
    pub date: String,
    pub workspace_id: i64,
    pub agent_id: Option<i64>,
    pub summary: String,
    pub key_insights: Vec<String>,
    pub new_knowledge_ids: Vec<i64>,
    pub new_procedure_ids: Vec<i64>,
    pub stats: serde_json::Value,
    pub created_at: Option<String>,
}
```

**Purpose**: Daily consolidated summary

**Fields**:
- `summary`: Natural language summary of the day
- `key_insights`: Top 5 insights extracted
- `new_knowledge_ids`: IDs of new semantic memories created
- `new_procedure_ids`: IDs of new procedures learned
- `stats`: JSON with metrics (conversations, tasks, success rate)

---

### Pattern
```rust
pub struct Pattern {
    pub pattern_type: String,
    pub description: String,
    pub frequency: i64,
    pub confidence: f64,
    pub source_episodes: Vec<i64>,
}
```

**Purpose**: Extracted pattern from episodic memories

**Fields**:
- `pattern_type`: "user_preference", "workflow", "error_pattern", etc.
- `description`: Natural language description
- `frequency`: Number of occurrences
- `confidence`: 0.0 to 1.0 confidence score
- `source_episodes`: Episode IDs supporting this pattern

---

### CompositeScore
```rust
pub struct CompositeScore {
    pub recency: f64,
    pub relevance: f64,
    pub utility: f64,
    pub combined: f64,
}
```

**Purpose**: Memory importance score components

**Formula**:
```
combined = (recency × 0.3) + (relevance × 0.4) + (utility × 0.3)
```

---

## Dependency Injection Pattern

### Example: ConsolidationEngine Implementation

```rust
pub struct DefaultConsolidationEngine<S, R, E>
where
    S: MemoryStore,
    R: MemoryRetriever,
    E: EmbeddingService,
{
    episode_store: Arc<S>,
    retriever: Arc<R>,
    embedder: Arc<E>,
}

impl<S, R, E> DefaultConsolidationEngine<S, R, E>
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
- Example: Test `ConsolidationEngine` with real stores

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
