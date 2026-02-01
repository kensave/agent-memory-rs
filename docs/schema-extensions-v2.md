# Database Schema Extensions - Version 2

**Date**: 2026-01-31  
**Migration**: v1 → v2  
**Status**: Complete

---

## Overview

Extended the existing memory-rs database schema with three new memory types:
1. **Episodes** - Episodic memory for interaction events
2. **Procedures** - Procedural memory for workflows
3. **Daily Synopsis** - Consolidated daily summaries

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

### 2. procedures

**Purpose**: Store learned workflows and action sequences

```sql
CREATE TABLE procedures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    trigger_conditions TEXT NOT NULL,
    action_sequence TEXT NOT NULL,
    success_rate REAL DEFAULT 0.0,
    usage_count INTEGER DEFAULT 0,
    last_used TEXT,
    learned_from TEXT DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);
```

**Indexes**:
- `idx_procedures_workspace` - Filter by workspace
- `idx_procedures_name` - Search by name
- `idx_procedures_last_used` - Sort by recency (DESC)

**Fields**:
- `trigger_conditions`: JSON string with trigger conditions
- `action_sequence`: JSON array of actions to execute
- `success_rate`: 0.0 to 1.0 (updated after each use)
- `usage_count`: Number of times procedure was executed
- `learned_from`: JSON array of episode IDs that led to this procedure

---

### 3. daily_synopsis

**Purpose**: Store consolidated daily summaries

```sql
CREATE TABLE daily_synopsis (
    date TEXT NOT NULL,
    workspace_id INTEGER NOT NULL,
    agent_id INTEGER,
    summary TEXT NOT NULL,
    key_insights TEXT DEFAULT '[]',
    new_knowledge_ids TEXT DEFAULT '[]',
    new_procedure_ids TEXT DEFAULT '[]',
    stats TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (date, workspace_id, agent_id),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
);
```

**Indexes**:
- `idx_synopsis_date` - Time-based queries (DESC)
- `idx_synopsis_workspace` - Filter by workspace

**Fields**:
- `date`: Date in YYYY-MM-DD format
- `summary`: Natural language summary of the day
- `key_insights`: JSON array of top insights
- `new_knowledge_ids`: JSON array of memory IDs created that day
- `new_procedure_ids`: JSON array of procedure IDs learned that day
- `stats`: JSON object with metrics (conversations, tasks, success_rate)

**Composite Primary Key**: (date, workspace_id, agent_id) ensures one synopsis per agent per day

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

### vec_procedures
```sql
CREATE VIRTUAL TABLE vec_procedures USING vec0(
    procedure_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);
```
**Purpose**: Vector embeddings for procedure semantic search

### vec_synopsis
```sql
CREATE VIRTUAL TABLE vec_synopsis USING vec0(
    synopsis_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);
```
**Purpose**: Vector embeddings for synopsis semantic search

**Note**: All use 384 dimensions to match BERT MiniLM model

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
DROP TABLE IF EXISTS procedures;
DROP TABLE IF EXISTS daily_synopsis;
DROP TABLE IF EXISTS vec_episodes;
DROP TABLE IF EXISTS vec_procedures;
DROP TABLE IF EXISTS vec_synopsis;
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

### 3. Composite Primary Key for Synopsis

**Decision**: (date, workspace_id, agent_id)  
**Rationale**: Ensures one synopsis per agent per day; natural key

### 4. Partial Index for Episodes

**Decision**: Index only non-archived episodes  
**Rationale**: Most queries target active episodes; saves space

### 5. Valence Range Check

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

### Get Active Procedures
```sql
SELECT * FROM procedures 
WHERE workspace_id = ? 
ORDER BY last_used DESC NULLS LAST;
```

### Get Daily Synopsis
```sql
SELECT * FROM daily_synopsis 
WHERE workspace_id = ? AND agent_id = ? 
ORDER BY date DESC 
LIMIT 7;
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
    assert!(tables.contains(&"procedures".to_string()));
    assert!(tables.contains(&"daily_synopsis".to_string()));
    
    // Verify schema version
    let version: i32 = /* query schema_version */;
    assert_eq!(version, 2);
}
```

**Status**: ✅ Test passing

---

## Next Steps

1. ✅ Schema migration complete
2. ⏭️ Implement EpisodicMemoryStore (Task 3)
3. ⏭️ Implement ProceduralMemoryStore (Task 4)
4. ⏭️ Implement synopsis generation (Task 9)

---

## Files Modified

- `/Users/kenneth/workspace/memory-rs/src/storage/schema.rs` - Added migrate_to_v2()
- `/Users/kenneth/workspace/memory-rs/tests/test_schema_migration.rs` - Integration test

---

## References

- SQLite JSON Functions: https://www.sqlite.org/json1.html
- sqlite-vec Documentation: https://github.com/asg017/sqlite-vec
- Schema Versioning: https://www.sqlite.org/pragma.html#pragma_user_version
