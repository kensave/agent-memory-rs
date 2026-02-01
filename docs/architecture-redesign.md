# Memory System Architecture - Redesign

## Current State Analysis

### Existing Architecture
```
MemorySystem
    ├── Database (owns Connection)
    │   └── Connection (not thread-safe)
    ├── FastEmbedder
    └── MemoryStore<'a> (borrows &Connection)
```

**Problems:**
1. `Connection` is not `Send + Sync` (uses `RefCell` internally)
2. `MemoryStore` has lifetime `'a` tied to `Database`
3. Cannot share `Database` across threads
4. Traits require `Send + Sync` but current design doesn't support it

---

## Redesigned Architecture

### Core Principle
**Single Source of Truth**: `Database` owns the connection and provides thread-safe access

### Class Diagram
```
┌─────────────────────────────────────────────────────────────┐
│                        Database                              │
│  - conn: Arc<Mutex<Connection>>                             │
│  + new(path) -> Self                                        │
│  + connection() -> Arc<Mutex<Connection>>                   │
│  + execute<F>(&self, f: F) -> Result<T>                    │
│    where F: FnOnce(&Connection) -> Result<T>               │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ provides connection to
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   MemoryStore (Trait)                        │
│  type Memory, type Id                                       │
│  + store(memory) -> Result<Id>                              │
│  + get(id) -> Result<Option<Memory>>                        │
│  + update(id, memory) -> Result<()>                         │
│  + delete(id) -> Result<()>                                 │
│  + store_batch(memories) -> Result<Vec<Id>>                 │
└─────────────────────────────────────────────────────────────┘
                              △
                              │ implements
                ┌─────────────┼─────────────┐
                │             │             │
┌───────────────┴──┐  ┌──────┴──────┐  ┌──┴──────────────┐
│ EpisodicStore    │  │ProcedureStore│  │ SemanticStore   │
│ - db: Database   │  │- db: Database│  │ - db: Database  │
└──────────────────┘  └──────────────┘  └─────────────────┘
```

### Key Design Decisions

#### 1. Database as Connection Manager
```rust
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    // Helper method for safe execution
    pub fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }
}
```

**Benefits:**
- Single lock point
- Clean API for stores
- Thread-safe by design
- No lifetime issues

#### 2. Stores Own Database Reference
```rust
pub struct EpisodicStore {
    db: Database,  // Cheap clone (Arc internally)
}

impl EpisodicStore {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}
```

**Benefits:**
- No lifetimes
- Stores are `Send + Sync`
- Can be passed across threads
- Clean ownership model

#### 3. Trait Implementation
```rust
#[async_trait]
impl MemoryStore for EpisodicStore {
    type Memory = Episode;
    type Id = i64;
    
    async fn store(&self, memory: Self::Memory) -> Result<Self::Id> {
        self.db.execute(|conn| {
            // Use conn here
            conn.execute(...)?;
            Ok(conn.last_insert_rowid())
        })
    }
}
```

**Benefits:**
- Clean separation
- No manual locking in store code
- Consistent pattern
- Easy to test with mock Database

---

## Implementation Plan

### Step 1: Refactor Database
```rust
// src/storage/database.rs (new file)
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> { ... }
    
    pub fn execute<F, T>(&self, f: F) -> Result<T>
    where F: FnOnce(&Connection) -> Result<T>
    { ... }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn)
        }
    }
}
```

### Step 2: Update MemoryStore (existing semantic)
```rust
// src/storage/semantic_store.rs (rename from memory_store.rs)
pub struct SemanticStore {
    db: Database,
}

impl SemanticStore {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    pub fn insert_memory(&self, memory: &Memory) -> Result<i64> {
        self.db.execute(|conn| {
            conn.execute(...)?;
            Ok(conn.last_insert_rowid())
        })
    }
}
```

### Step 3: Implement EpisodicStore
```rust
// src/services/episodic_store.rs
pub struct EpisodicStore {
    db: Database,
}

#[async_trait]
impl MemoryStore for EpisodicStore {
    type Memory = Episode;
    type Id = i64;
    
    async fn store(&self, memory: Self::Memory) -> Result<Self::Id> {
        self.db.execute(|conn| {
            let context_json = serde_json::to_string(&memory.context)?;
            conn.execute(
                "INSERT INTO episodes (...) VALUES (...)",
                params![...]
            )?;
            Ok(conn.last_insert_rowid())
        })
    }
}
```

### Step 4: Update MemorySystem
```rust
pub struct MemorySystem {
    db: Database,
    embedder: FastEmbedder,
    semantic_store: SemanticStore,
    episodic_store: EpisodicStore,
}

impl MemorySystem {
    pub fn new<P: AsRef<Path>>(db_path: P, model_type: ModelType) -> Result<Self> {
        let db = Database::new(db_path)?;
        let embedder = FastEmbedder::with_model(model_type)?;
        
        Ok(MemorySystem {
            semantic_store: SemanticStore::new(db.clone()),
            episodic_store: EpisodicStore::new(db.clone()),
            db,
            embedder,
        })
    }
}
```

---

## Migration Strategy

### Phase 1: Extract Database (Non-breaking)
1. Create `src/storage/database.rs` with new `Database` struct
2. Keep old `schema.rs` `Database` as `DatabaseV1`
3. Add `impl From<DatabaseV1> for Database`

### Phase 2: Refactor Stores (Breaking but internal)
1. Rename `memory_store.rs` → `semantic_store.rs`
2. Update `SemanticStore` to use new `Database`
3. Create `EpisodicStore` with new pattern
4. Create `ProceduralStore` with new pattern

### Phase 3: Update MemorySystem (Breaking but clean)
1. Update `MemorySystem` to use new stores
2. Update MCP tools to use new API
3. Update tests

### Phase 4: Schema Extensions
1. Add episodes, procedures, synopsis tables
2. Migrate to v2 schema
3. All stores use same Database instance

---

## File Structure
```
src/
├── storage/
│   ├── mod.rs
│   ├── database.rs          (NEW - connection manager)
│   ├── semantic_store.rs    (RENAMED from memory_store.rs)
│   └── schema.rs            (UPDATED - just SQL)
├── services/
│   ├── mod.rs
│   ├── episodic_store.rs    (NEW)
│   ├── procedural_store.rs  (NEW)
│   └── ...
├── traits/
│   └── ... (already done)
├── models/
│   └── ... (already done)
└── memory_system.rs         (UPDATED)
```

---

## Benefits of This Approach

1. **Thread Safety**: `Database` is `Send + Sync` by design
2. **No Lifetimes**: Stores own `Database` (cheap clone)
3. **Clean API**: `db.execute(|conn| ...)` pattern
4. **Testable**: Easy to mock `Database`
5. **Consistent**: All stores follow same pattern
6. **Scalable**: Easy to add new store types
7. **SOLID**: Each store has single responsibility

---

## Next Steps

1. Create `src/storage/database.rs` with new design
2. Update schema migrations in `schema.rs`
3. Refactor existing `MemoryStore` → `SemanticStore`
4. Implement `EpisodicStore` with trait
5. Update `MemorySystem` to use new architecture
6. Update tests

This is a proper architectural refactor, not a minimal patch!
