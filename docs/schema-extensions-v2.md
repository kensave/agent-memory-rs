# Database Schema Extensions - Version 2

**Date**: 2026-01-31  
**Migration**: v1 → v2  
**Status**: Complete

---

## Overview

Extended the existing memory-rs database schema with episodic memory:
1. **Episodes** - Episodic memory for interaction events

---

## New Tables

### 1. episodes

**Purpose**: Store specific interaction events with full context

```sql
CREATE TABLE episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL,
    agent_id INTEGER,
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    conversation_id TEXT,
    event_type TEXT NOT NULL,
    context TEXT NOT NULL,
    outcome TEXT,
    valence REAL CHECK (valence IS NULL OR (valence >= -1.0 AND valence <= 1.0)),
    archived INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
);
```

**Indexes**:
- `idx_episodes_workspace` - Filter by workspace
- `idx_episodes_timestamp` - Time-based queries (DESC for recent first)
- `idx_episodes_conversation` - Group by conversation
- `idx_episodes_archived` - Partial index for active episodes only

**Fields**:
- `event_type`: "user_query", "tool_execution", "error", "success", etc.
- `context`: JSON string with full interaction context
- `outcome`: Result or output of the event
- `valence`: Emotional valence (-1.0 = negative, +1.0 = positive)
- `archived`: 0 = active, 1 = archived

---

## Vector Tables

### vec_episodes
```sql
CREATE VIRTUAL TABLE vec_episodes USING vec0(
    episode_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);
```
**Purpose**: Vector embeddings for episodic memory semantic search

**Note**: Uses 384 dimensions to match BGE-Small model

---

## Migration Strategy

### Automatic Migration

The migration runs automatically when `Database::new()` is called:

```rust
fn apply_migrations(&self) -> Result<()> {
    let current_version = /* get from schema_version table */;
    
    if current_version < 2 && SCHEMA_VERSION >= 2 {
        self.migrate_to_v2()?;
    }
    
    // Record new version
    self.conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [SCHEMA_VERSION],
    )?;
    
    Ok(())
}
```

### Backward Compatibility

- ✅ Existing `memories` table unchanged
- ✅ Existing `vec0` table unchanged
- ✅ All new tables are additive
- ✅ No data migration required
- ✅ Existing code continues to work

### Rollback

To rollback to v1:
```sql
DROP TABLE IF EXISTS episodes;
DROP TABLE IF EXISTS vec_episodes;
DELETE FROM schema_version WHERE version = 2;
```

---

## Design Decisions

### 1. TEXT vs JSON Column Type

**Decision**: Use TEXT for JSON data  
**Rationale**: SQLite doesn't have native JSON type; TEXT with JSON functions is standard

### 2. INTEGER for Boolean

**Decision**: Use INTEGER (0/1) for `archived` flag  
**Rationale**: SQLite doesn't have native BOOLEAN type

### 3. Partial Index for Episodes

**Decision**: Index only non-archived episodes  
**Rationale**: Most queries target active episodes; saves space

### 4. Valence Range Check

**Decision**: CHECK constraint for -1.0 to 1.0  
**Rationale**: Enforces valid emotional valence at database level

---

## Query Patterns

### Get Recent Episodes
```sql
SELECT * FROM episodes 
WHERE workspace_id = ? AND archived = 0 
ORDER BY timestamp DESC 
LIMIT 10;
```

### Get Episodes by Conversation
```sql
SELECT * FROM episodes 
WHERE conversation_id = ? 
ORDER BY timestamp ASC;
```

### Vector Search Episodes
```sql
SELECT e.*, vec_distance_cosine(v.embedding, ?) as distance
FROM episodes e
JOIN vec_episodes v ON e.id = v.episode_id
WHERE e.workspace_id = ? AND e.archived = 0
ORDER BY distance ASC
LIMIT 10;
```

---

## Performance Considerations

### Index Strategy

1. **B-tree indexes** for exact matches and range queries
   - workspace_id, timestamp, date
2. **Partial indexes** for filtered queries
   - archived = 0 (episodes)
3. **Vector indexes** for similarity search
   - HNSW-like structure via sqlite-vec

### Expected Performance

- **Insert episode**: < 10ms
- **Query recent episodes**: < 5ms (with index)
- **Vector search**: < 20ms for 10k episodes
- **Daily synopsis**: < 5ms (composite PK lookup)

### Optimization Tips

1. Use prepared statements for repeated queries
2. Batch inserts for episodes (use transactions)
3. Archive old episodes regularly to keep active set small
4. Use partial index for archived flag

---

## Testing

### Integration Test

```rust
#[test]
fn test_migration_to_v2() {
    let db = Database::new("/tmp/test.db")?;
    
    // Verify tables exist
    let tables: Vec<String> = /* query sqlite_master */;
    assert!(tables.contains(&"episodes".to_string()));
    
    // Verify schema version
    let version: i32 = /* query schema_version */;
    assert_eq!(version, 2);
}
```

**Status**: ✅ Test passing

---

## Next Steps

1. ✅ Schema migration complete
2. ✅ Implement EpisodicMemoryStore

---

## Files Modified

- `<project-root>/src/storage/schema.rs` - Added migrate_to_v2()
- `<project-root>/tests/test_schema_migration.rs` - Integration test

---

## References

- SQLite JSON Functions: https://www.sqlite.org/json1.html
- sqlite-vec Documentation: https://github.com/asg017/sqlite-vec
- Schema Versioning: https://www.sqlite.org/pragma.html#pragma_user_version
