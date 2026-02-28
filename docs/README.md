# Memory-RS: Agent Memory Management System

## Overview

A comprehensive memory management system for AI agents with episodic, semantic, and procedural memory types, intelligent consolidation, and decay mechanisms.

## Documentation

- **[Design Rationale](DESIGN_RATIONALE.md)** - Design decisions, formulas, algorithms, and research
- **[MCP Auto-Consolidation](MCP_AUTO_CONSOLIDATION.md)** - How auto-consolidation works
- **[Interface Design](interface-design.md)** - SOLID principles and trait design
- **[Schema](schema-extensions-v2.md)** - Database schema and migrations

## Architecture

### Core Components

```
MemoryManager (Facade)
    ├── EpisodicMemoryStore      - Raw interaction events
    ├── HybridRetrievalEngine    - BM25 + Vector search
    ├── ConsolidationEngine      - Pattern extraction
    ├── PatternExtractor         - Identifies recurring themes
    └── SynopsisGenerator        - Daily summaries
```

### Memory Types

1. **Episodic Memory**: Stores specific events with full context
   - Event type, timestamp, conversation ID
   - Outcome, valence (emotional value)
   - Vector embeddings for semantic search

2. **Consolidated Memories**: Extracted patterns and themes
   - Created through consolidation process
   - Pattern extraction from episodes
   - Daily synopsis generation

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

// Consolidate daily
let synopsis = manager.consolidate("2026-01-31".to_string()).await?;
```

### CLI Commands

```bash
# Start MCP server
cargo run --bin agent-memory-mcp my-workspace

# View statistics
memory-cli stats --workspace 1

# Query memories
memory-cli query "rust programming" --limit 10

# View daily synopsis
memory-cli synopsis --date 2026-01-31

# Consolidate memories
memory-cli consolidate --date 2026-01-31

# Prune old memories
memory-cli prune --dry-run
```

## Key Features

### 1. Hierarchical Retrieval

Retrieves memories in priority order:
- Semantic memory (50% of results)
- Recent episodes (25%)
- Procedures (25%)

```rust
let results = manager.retrieve_hierarchical(query, workspace_id, 10)?;
```

### 2. Daily Consolidation

Automatically extracts patterns and generates synopsis:

```rust
let synopsis = manager.consolidate(date).await?;
// Returns: summary, key insights, new knowledge, new procedures
```

### 3. Intelligent Decay

Composite scoring formula:
```
score = (recency × 0.3) + (relevance × 0.4) + (utility × 0.3)
```

Archives low-scoring memories automatically.

### 4. Hybrid Search

Combines BM25 keyword search with vector similarity:
- RRF (Reciprocal Rank Fusion) for result merging
- Type-specific search strategies
- Configurable result limits

## Database Schema

### Core Tables

- `workspaces` - Project isolation
- `agents` - Multi-agent support
- `memories` - Semantic knowledge
- `episodes` - Episodic events
- `procedures` - Learned workflows
- `daily_synopsis` - Consolidated summaries

### Vector Tables

- `vec0` - Memory embeddings (384 dims)
- `vec_episodes` - Episode embeddings
- `vec_procedures` - Procedure embeddings
- `vec_synopsis` - Synopsis embeddings

## API Reference

### MemoryManager

**Storage**:
- `store_episode(episode)` → `i64`
- `store_procedure(procedure)` → `i64`
- `store_knowledge(memory)` → `i64`

**Retrieval**:
- `retrieve(query, workspace_id, limit)` → `Vec<HybridSearchResult>`
- `retrieve_hierarchical(query, workspace_id, max)` → `Vec<HybridSearchResult>`
- `get_synopsis(workspace_id, date)` → `Option<Synopsis>`

**Management**:
- `consolidate(date)` → `Synopsis`
- `prune(workspace_id, dry_run)` → `(usize, usize, usize)`
- `get_memory_stats(workspace_id)` → `MemoryStats`

### ContextInjectionService

Prepares memory context for LLM consumption:

```rust
let service = ContextInjectionService::new(manager);
let context = service.prepare_context(query, workspace_id, token_budget)?;
```

Budget allocation:
- Synopsis: 25%
- Semantic: 40%
- Episodic: 25%
- Procedural: 10%

## Performance

### Benchmarks

- Episode storage: ~5ms
- Hybrid search: ~20ms (1000 memories)
- Consolidation: ~2s (100 episodes)
- Synopsis generation: ~500ms

### Optimization Tips

1. **Batch Operations**: Use batch methods for bulk inserts
2. **Index Usage**: Ensure indexes on timestamp, workspace_id
3. **Archival**: Run consolidation nightly to keep active set small
4. **Token Budget**: Adjust context budget based on LLM limits

## Testing

### Run All Tests

```bash
cargo test --quiet
```

### Integration Tests

- `test_episodic_store` - Episode CRUD
- `test_semantic_extensions` - Knowledge tracking
- `test_consolidation_engine` - Full pipeline
- `test_hybrid_retrieval` - BM25 + vector search
- `test_full_pipeline` - End-to-end workflow

Total: 29 tests

## SOLID Principles

### Single Responsibility
Each service has one clear purpose:
- `EpisodicMemoryStore` - Episode storage only
- `PatternExtractor` - Pattern analysis only
- `ConsolidationEngine` - Consolidation only

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
- `ConsolidationEngine` - Consolidation only
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

// Store as semantic memory
manager.store_knowledge(&memory)?;

// Or store as episode
manager.store_episode(episode).await?;

// Retrieve with hybrid search
let results = manager.retrieve(query, workspace_id, limit)?;
```

### Backward Compatibility

The old `memories` table is now semantic memory. Existing data works without migration.

## Troubleshooting

### Common Issues

**Issue**: Old tests failing after schema changes
**Solution**: Update tests to use new Memory struct fields (source_episodes, confidence, last_validated)

**Issue**: Consolidation taking too long
**Solution**: Reduce episode count via more aggressive archival thresholds

**Issue**: Low health score
**Solution**: Run consolidation to extract patterns and increase confidence scores

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
