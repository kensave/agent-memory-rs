# Memory-RS: Agent Memory Management System

## Overview

Episodic memory system for AI agents with vector search, exposed via MCP server.

## Documentation

- **[Design Rationale](DESIGN_RATIONALE.md)** - Design decisions, formulas, algorithms, and research
- **[Interface Design](interface-design.md)** - SOLID principles and trait design
- **[Schema](schema-extensions-v2.md)** - Database schema and migrations

## Architecture

### Core Components

```
MemoryManager (Facade)
    ├── EpisodicMemoryStore      - Raw interaction events
    └── HybridRetrievalEngine    - BM25 + Vector search
```

### Memory Types

1. **Episodic Memory**: Stores specific events with full context
   - Event type, timestamp, conversation ID
   - Outcome, valence (emotional value)
   - Vector embeddings for semantic search

## Quick Start

### Basic Usage

```rust
use memory_rs::{Database, MemoryManager};
use memory_rs::models::dtos::Episode;

// Initialize
let db = Database::new("memory.db")?;
let manager = MemoryManager::new(db);

// Store episode
let episode = Episode {
    workspace_id: 1,
    event_type: "task".to_string(),
    timestamp: "2026-01-31 10:00:00".to_string(),
    outcome: Some("success".to_string()),
    valence: Some(0.8),
    // ... other fields
};
manager.store_episode(episode).await?;

// Retrieve memories
let results = manager.retrieve("task", workspace_id, 10)?;
```

### CLI Commands

```bash
# Start MCP server
cargo run --bin agent-memory-mcp my-workspace

# View statistics
memory-cli stats --workspace 1

# Query memories
memory-cli query "rust programming" --limit 10

# Prune old memories
memory-cli prune --dry-run
```

## Key Features

### 1. Episode Storage

Store interaction events with full context:
- Event type, timestamp, conversation ID
- Outcome, valence (emotional value)
- Vector embeddings for semantic search

```rust
let episode = Episode::new("user_query", context, Some("outcome"), Some(0.8));
manager.store_episode(episode).await?;
```

### 2. Hybrid Search

Combines BM25 keyword search with vector similarity:
- Cosine distance retrieval on episode embeddings
- BM25 search with proper IDF calculation
- Configurable result limits

```rust
let results = manager.retrieve("rust programming", workspace_id, 10)?;
```

## Database Schema

### Core Tables

- `workspaces` - Project isolation
- `agents` - Multi-agent support
- `episodes` - Episodic events

### Vector Tables

- `vec0` - Episode embeddings (384 dims)

## API Reference

### MemoryManager

**Storage**:
- `store_episode(episode)` → `i64`

**Retrieval**:
- `retrieve(query, workspace_id, limit)` → `Vec<HybridSearchResult>`

**Management**:
- `get_memory_stats(workspace_id)` → `MemoryStats`

## Performance

### Benchmarks

- Episode storage: ~5ms
- Hybrid search: ~20ms (1000 memories)

### Optimization Tips

1. **Batch Operations**: Use batch methods for bulk inserts
2. **Index Usage**: Ensure indexes on timestamp, workspace_id
3. **Token Budget**: Adjust context budget based on LLM limits

## Testing

### Run All Tests

```bash
cargo test --quiet
```

### Integration Tests

- `test_episodic_store` - Episode CRUD
- `test_hybrid_retrieval` - BM25 + vector search
- `test_full_pipeline` - End-to-end workflow

Total: 29 tests

## SOLID Principles

### Single Responsibility
Each service has one clear purpose:
- `EpisodicMemoryStore` - Episode storage only
- `HybridRetrievalEngine` - Search only

### Open/Closed
Extend via traits without modifying existing code:
```rust
impl MemoryStore for CustomStore { ... }
```

### Liskov Substitution
All stores implement `MemoryStore` trait interchangeably.

### Interface Segregation
Clients depend only on methods they use:
- `MemoryStore` - CRUD operations
- `MemoryRetriever` - Search only

### Dependency Inversion
High-level modules depend on abstractions (traits), not concrete implementations.

## Migration from Existing System

### Before (Old System)

```rust
let system = MemorySystem::new(db_path)?;
system.learn(text, workspace_id, tags)?;
let results = system.search(query, workspace_id, limit)?;
```

### After (New System)

```rust
let manager = MemoryManager::new(db);

// Store as episode
manager.store_episode(episode).await?;

// Retrieve with hybrid search
let results = manager.retrieve(query, workspace_id, limit)?;
```

### Backward Compatibility

The system maintains compatibility with existing episode data.

## Troubleshooting

### Common Issues

**Issue**: Old tests failing after schema changes
**Solution**: Update tests to use new Episode struct fields

**Issue**: Search performance degrading
**Solution**: Ensure proper indexing on timestamp and workspace_id fields

## Contributing

### Code Style

- Use minimal, focused implementations
- Follow SOLID principles
- Add tests for new features
- Document public APIs

### Testing

```bash
# Run specific test
cargo test --test test_name

# Run with output
cargo test -- --nocapture

# Check compilation
cargo check
```

## License

MIT

## Authors

Built with SOLID principles and minimal code philosophy.
