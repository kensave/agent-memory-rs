# Memory-RS API Documentation

> **Note**: This document is outdated. Please refer to the comprehensive documentation in:
> - **[docs/README.md](docs/README.md)** - Complete API reference with new memory system
> - **[docs/MCP_AUTO_CONSOLIDATION.md](docs/MCP_AUTO_CONSOLIDATION.md)** - MCP server usage
> - **[docs/interface-design.md](docs/interface-design.md)** - SOLID architecture and traits

## Quick Links

### New Memory System (v2)
- **MemoryManager**: Unified facade for all memory operations
- **EpisodicMemoryStore**: Raw event storage
- **SemanticMemoryStore**: Knowledge with confidence tracking
- **ProceduralMemoryStore**: Workflow tracking
- **HybridRetrievalEngine**: BM25 + Vector search
- **ConsolidationEngine**: Pattern extraction and synopsis generation

### Legacy API (v1)

The following API is maintained for backward compatibility but deprecated in favor of the new memory system.

## Table of Contents

1. [Rust API](#rust-api)
2. [MCP Protocol API](#mcp-protocol-api)
3. [Database Schema](#database-schema)
4. [Error Handling](#error-handling)

## Rust API

### WorkspaceManager

Manages multiple workspace databases.

```rust
pub struct WorkspaceManager
```

#### Methods

##### `new(model_type: ModelType) -> Result<Self>`

Creates a new workspace manager with default base directory (`~/.memory-rs/workspaces/`).

**Parameters:**
- `model_type`: Embedding model to use (MiniLM, Nomic, BgeSmall)

**Returns:** `Result<WorkspaceManager>`

**Example:**
```rust
let manager = WorkspaceManager::new(ModelType::MiniLM)?;
```

##### `with_base_dir<P: AsRef<Path>>(base_dir: P, model_type: ModelType) -> Result<Self>`

Creates workspace manager with custom base directory.

##### `get_or_create_workspace(&self, workspace_name: &str) -> Result<MemorySystem>`

Gets existing workspace or creates new one.

**Returns:** `MemorySystem` instance for the workspace

##### `list_workspaces(&self) -> Result<Vec<String>>`

Lists all available workspaces.

##### `delete_workspace(&self, workspace_name: &str) -> Result<()>`

Deletes a workspace and all its data.

##### `workspace_exists(&self, workspace_name: &str) -> bool`

Checks if workspace exists.

##### `detect_workspace_from_cwd() -> Option<String>`

Detects workspace name from current working directory.

---

### MemorySystem

High-level API for memory operations.

```rust
pub struct MemorySystem
```

#### Methods

##### `new<P: AsRef<Path>>(db_path: P, model_type: ModelType) -> Result<Self>`

Creates new memory system with specified database path.

##### `learn(&self, memory: &Memory) -> Result<i64>`

Stores a memory with automatic embedding generation.

**Parameters:**
- `memory`: Memory struct with text and metadata

**Returns:** Memory ID

**Example:**
```rust
let memory = Memory {
    id: None,
    workspace_id: 1,
    agent_id: None,
    text: "Important information".to_string(),
    tags: Some("important".to_string()),
    importance_score: 0.8,
    access_count: 0,
    last_accessed: None,
    conversation_id: None,
    parent_memory_id: None,
    user_feedback: None,
    created_at: None,
    updated_at: None,
};

let memory_id = system.learn(&memory)?;
```

##### `learn_batch(&self, memories: &[Memory]) -> Result<Vec<i64>>`

Stores multiple memories efficiently in batch.

**Returns:** Vector of memory IDs

##### `search(&self, query: &str, filters: &SearchFilters, limit: usize) -> Result<Vec<SearchResult>>`

Searches memories using semantic similarity and filters.

**Parameters:**
- `query`: Search query text
- `filters`: Search filters (workspace, agent, importance, etc.)
- `limit`: Maximum number of results

**Returns:** Vector of search results with scores

**Example:**
```rust
let filters = SearchFilters {
    workspace_id: Some(1),
    min_importance: Some(0.5),
    ..Default::default()
};

let results = system.search("query text", &filters, 10)?;
```

##### `get_memory(&self, memory_id: i64) -> Result<Option<Memory>>`

Retrieves a specific memory by ID.

##### `update_memory(&self, memory_id: i64, memory: &Memory) -> Result<()>`

Updates an existing memory.

##### `delete_memory(&self, memory_id: i64) -> Result<()>`

Deletes a memory and its embedding.

##### `database(&self) -> &Database`

Gets reference to underlying database.

---

### Memory

Represents a memory entry.

```rust
pub struct Memory {
    pub id: Option<i64>,
    pub workspace_id: i64,
    pub agent_id: Option<i64>,
    pub text: String,
    pub tags: Option<String>,
    pub importance_score: f64,
    pub access_count: i64,
    pub last_accessed: Option<String>,
    pub conversation_id: Option<String>,
    pub parent_memory_id: Option<i64>,
    pub user_feedback: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
```

**Fields:**
- `id`: Memory ID (None for new memories)
- `workspace_id`: Workspace this memory belongs to
- `agent_id`: Optional agent ID for private memories
- `text`: Memory content (required)
- `tags`: Comma-separated tags
- `importance_score`: Score 0.0-1.0 (default 0.5)
- `access_count`: Number of times accessed
- `last_accessed`: Last access timestamp
- `conversation_id`: Group related memories
- `parent_memory_id`: Create memory hierarchies
- `user_feedback`: Optional user feedback
- `created_at`: Creation timestamp (auto-generated)
- `updated_at`: Last update timestamp (auto-generated)

---

### SearchFilters

Filters for memory search.

```rust
pub struct SearchFilters {
    pub workspace_id: Option<i64>,
    pub agent_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub min_importance: Option<f64>,
    pub max_importance: Option<f64>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub conversation_id: Option<String>,
}
```

**Example:**
```rust
let filters = SearchFilters {
    workspace_id: Some(1),
    agent_id: Some(5),
    min_importance: Some(0.7),
    conversation_id: Some("conv-123".to_string()),
    ..Default::default()
};
```

---

### SearchResult

Search result with scores.

```rust
pub struct SearchResult {
    pub memory: Memory,
    pub similarity_score: f64,
    pub combined_score: f64,
}
```

**Fields:**
- `memory`: The memory entry
- `similarity_score`: Cosine similarity (0.0-1.0)
- `combined_score`: Hybrid score (70% similarity + 30% importance)

---

### ModelType

Available embedding models.

```rust
pub enum ModelType {
    MiniLM,    // 384 dimensions, fast
    Nomic,     // 768 dimensions, high quality
    BgeSmall,  // 384 dimensions, balanced
}
```

---

## MCP Protocol API

### JSON-RPC 2.0 Format

All requests and responses follow JSON-RPC 2.0 specification.

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "id": <number or string>,
  "method": "<method_name>",
  "params": <object or null>
}
```

**Response Format:**
```json
{
  "jsonrpc": "2.0",
  "id": <same as request>,
  "result": <object>,
  "error": <error object or null>
}
```

**Error Format:**
```json
{
  "jsonrpc": "2.0",
  "id": <same as request>,
  "error": {
    "code": <integer>,
    "message": "<error message>",
    "data": <optional additional data>
  }
}
```

### Methods

#### initialize

Initialize MCP server connection.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "memory-rs",
      "version": "0.1.0"
    }
  }
}
```

#### tools/list

List available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": null
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "learn",
        "description": "Store a new memory with embedding",
        "inputSchema": { /* JSON Schema */ }
      },
      {
        "name": "search",
        "description": "Search memories by semantic similarity",
        "inputSchema": { /* JSON Schema */ }
      }
    ]
  }
}
```

#### tools/call

Execute a tool.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "learn",
    "arguments": {
      "text": "Memory text",
      "workspace_id": 1
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "memory_id": 42,
    "status": "success"
  }
}
```

### Learn Tool

**Input Schema:**
```typescript
{
  text: string;              // Required: Memory text
  workspace_id: number;      // Required: Workspace ID
  agent_id?: number;         // Optional: Agent ID
  tags?: string;             // Optional: Comma-separated tags
  importance_score?: number; // Optional: 0.0-1.0 (default 0.5)
  conversation_id?: string;  // Optional: Conversation grouping
}
```

**Output Schema:**
```typescript
{
  memory_id: number;  // ID of stored memory
  status: string;     // "success"
}
```

**Validation:**
- `text` cannot be empty
- `workspace_id` must be valid integer
- `importance_score` must be between 0.0 and 1.0

### Search Tool

**Input Schema:**
```typescript
{
  query: string;              // Required: Search query
  workspace_id?: number;      // Optional: Filter by workspace
  agent_id?: number;          // Optional: Filter by agent
  min_importance?: number;    // Optional: Minimum importance
  max_importance?: number;    // Optional: Maximum importance
  conversation_id?: string;   // Optional: Filter by conversation
  limit?: number;             // Optional: Max results (default 10, max 100)
}
```

**Output Schema:**
```typescript
{
  results: Array<{
    memory_id: number;
    text: string;
    similarity_score: number;  // 0.0-1.0
    combined_score: number;    // 0.0-1.0
    importance_score: number;  // 0.0-1.0
    tags: string | null;
    created_at: string | null;
  }>;
  count: number;  // Number of results
}
```

**Validation:**
- `query` cannot be empty
- `limit` must be between 1 and 100

## Database Schema

### Tables

#### workspaces
```sql
CREATE TABLE workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### agents
```sql
CREATE TABLE agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    UNIQUE(workspace_id, name)
);
```

#### memories
```sql
CREATE TABLE memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL,
    agent_id INTEGER,
    text TEXT NOT NULL,
    tags TEXT,
    importance_score REAL DEFAULT 0.5,
    access_count INTEGER DEFAULT 0,
    last_accessed TEXT,
    conversation_id TEXT,
    parent_memory_id INTEGER,
    user_feedback TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_memory_id) REFERENCES memories(id) ON DELETE SET NULL
);
```

#### vec0 (Virtual Table)
```sql
CREATE VIRTUAL TABLE vec0 USING vec0(
    memory_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]  -- or FLOAT[768] for Nomic
);
```

### Indexes

```sql
CREATE INDEX idx_memories_workspace ON memories(workspace_id);
CREATE INDEX idx_memories_agent ON memories(agent_id);
CREATE INDEX idx_memories_importance ON memories(importance_score);
CREATE INDEX idx_memories_created ON memories(created_at);
CREATE INDEX idx_memories_conversation ON memories(conversation_id);
```

## Error Handling

### Rust Errors

All functions return `Result<T, anyhow::Error>`. Common errors:

- **Database Error**: SQLite operation failed
- **Embedding Error**: Model loading or embedding generation failed
- **Validation Error**: Invalid input parameters
- **IO Error**: File system operations failed

**Example:**
```rust
match system.learn(&memory) {
    Ok(memory_id) => println!("Success: {}", memory_id),
    Err(e) => eprintln!("Error: {}", e),
}
```

### MCP Errors

JSON-RPC error codes:

| Code | Meaning | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Invalid JSON-RPC format |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Server internal error |

**Example Error Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Text cannot be empty",
    "data": null
  }
}
```

## Performance Considerations

### Embedding Generation

- **MiniLM**: ~300ms per embedding (real model)
- **Mock**: ~20μs per embedding (testing)
- **Batch**: More efficient for multiple memories

### Search Performance

- **<1K memories**: Sub-10ms
- **1K-10K memories**: 10-100ms
- **10K-100K memories**: 100ms-1s
- **>100K memories**: Consider optimization (indexing, quantization)

### Database Size

- **Text**: ~1KB per memory (average)
- **Embedding**: 1.5KB (384d) or 3KB (768d)
- **Total**: ~2.5KB per memory (MiniLM)
- **100K memories**: ~250MB database

## Best Practices

1. **Batch Operations**: Use `learn_batch()` for multiple memories
2. **Filter Early**: Use workspace_id and agent_id filters
3. **Limit Results**: Don't request more than needed
4. **Index Usage**: Filters use indexes for fast queries
5. **Connection Reuse**: WorkspaceManager reuses connections
6. **Error Handling**: Always handle Result types properly

## Examples

See [USAGE.md](USAGE.md) for complete examples and [examples/](examples/) directory for code samples.
