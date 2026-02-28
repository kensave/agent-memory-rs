# Usage Documentation Moved

This file has been removed. Please refer to:

- **[README.md](README.md)** - Quick start guide
- **[docs/README.md](docs/README.md)** - Complete usage guide

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [MCP Server Usage](#mcp-server-usage)
4. [Programmatic API](#programmatic-api)
5. [Workspace Management](#workspace-management)
6. [Advanced Features](#advanced-features)
7. [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo package manager

### Build from Source

```bash
git clone https://github.com/yourusername/memory-rs
cd memory-rs
cargo build --release
```

The compiled binary will be in `target/release/`.

### Run Tests

```bash
cargo test
```

All 28 tests should pass.

## Quick Start

### 1. Start the MCP Server

```bash
# Use default workspace
cargo run --example mcp_server

# Use specific workspace
cargo run --example mcp_server my-project
```

The server will:
- Create workspace directory at `~/.memory-rs/workspaces/my-project/`
- Initialize SQLite database with schema
- Load embedding model (MiniLM by default)
- Listen on stdin for JSON-RPC requests

### 2. Send Your First Learn Request

Create a file `learn_request.json`:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "learn",
    "arguments": {
      "text": "Rust is a systems programming language that runs blazingly fast",
      "workspace_id": 1,
      "importance_score": 0.8,
      "tags": "rust,programming,systems"
    }
  }
}
```

Send it to the server:
```bash
cat learn_request.json | cargo run --example mcp_server my-project
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "memory_id": 1,
    "status": "success"
  }
}
```

### 3. Search Your Memories

Create `search_request.json`:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "fast programming language",
      "workspace_id": 1,
      "limit": 5
    }
  }
}
```

Send it:
```bash
cat search_request.json | cargo run --example mcp_server my-project
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "results": [
      {
        "memory_id": 1,
        "text": "Rust is a systems programming language that runs blazingly fast",
        "similarity_score": 0.89,
        "combined_score": 0.86,
        "importance_score": 0.8,
        "tags": "rust,programming,systems",
        "created_at": "2026-01-30T23:00:00Z"
      }
    ],
    "count": 1
  }
}
```

## MCP Server Usage

### Available Methods

#### 1. Initialize

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

Response includes server capabilities and version.

#### 2. List Tools

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": null
}
```

Returns available tools (learn, search) with their schemas.

#### 3. Call Tool

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "learn",
    "arguments": {
      "text": "Your memory text here",
      "workspace_id": 1
    }
  }
}
```

### Learn Tool Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| text | string | Yes | The text to remember |
| workspace_id | integer | Yes | Workspace ID (usually 1) |
| agent_id | integer | No | Optional agent ID for private memories |
| tags | string | No | Comma-separated tags |
| importance_score | number | No | Score 0-1 (default 0.5) |
| conversation_id | string | No | Group related memories |

### Search Tool Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| query | string | Yes | Search query text |
| workspace_id | integer | No | Filter by workspace |
| agent_id | integer | No | Filter by agent |
| min_importance | number | No | Minimum importance score |
| max_importance | number | No | Maximum importance score |
| conversation_id | string | No | Filter by conversation |
| limit | integer | No | Max results (default 10, max 100) |

## Programmatic API

### Using Memory-RS as a Library

Add to your `Cargo.toml`:
```toml
[dependencies]
memory-rs = { path = "../memory-rs" }
```

### Basic Example

```rust
use memory_rs::{WorkspaceManager, ModelType, storage::Memory};
use anyhow::Result;

fn main() -> Result<()> {
    // Create workspace manager
    let manager = WorkspaceManager::new(ModelType::MiniLM)?;
    
    // Get or create workspace
    let system = manager.get_or_create_workspace("my-app")?;
    
    // Get workspace ID
    let workspace_id: i64 = system.database().connection()
        .query_row("SELECT id FROM workspaces WHERE name = 'my-app'", [], |row| row.get(0))?;
    
    // Learn a memory
    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Important fact to remember".to_string(),
        tags: Some("important,fact".to_string()),
        importance_score: 0.9,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        created_at: None,
        updated_at: None,
    };
    
    let memory_id = system.learn(&memory)?;
    println!("Stored memory with ID: {}", memory_id);
    
    // Search memories
    use memory_rs::storage::SearchFilters;
    
    let filters = SearchFilters {
        workspace_id: Some(workspace_id),
        min_importance: Some(0.5),
        ..Default::default()
    };
    
    let results = system.search("important information", &filters, 10)?;
    
    for result in results {
        println!("Found: {} (score: {:.2})", 
            result.memory.text, 
            result.combined_score
        );
    }
    
    Ok(())
}
```

### Batch Learning

```rust
use memory_rs::storage::Memory;

let memories: Vec<Memory> = vec![
    Memory { /* ... */ },
    Memory { /* ... */ },
    Memory { /* ... */ },
];

let memory_ids = system.learn_batch(&memories)?;
println!("Stored {} memories", memory_ids.len());
```

## Workspace Management

### Creating Workspaces

```rust
use memory_rs::{WorkspaceManager, ModelType};

let manager = WorkspaceManager::new(ModelType::MiniLM)?;

// Create workspace
let system1 = manager.get_or_create_workspace("project-a")?;
let system2 = manager.get_or_create_workspace("project-b")?;
```

### Listing Workspaces

```rust
let workspaces = manager.list_workspaces()?;
for workspace in workspaces {
    println!("Workspace: {}", workspace);
}
```

### Deleting Workspaces

```rust
manager.delete_workspace("old-project")?;
```

### Workspace Isolation

Each workspace has its own:
- SQLite database file
- Memory entries
- Embeddings
- Workspace and agent configurations

Memories in one workspace cannot be accessed from another workspace.

## Advanced Features

### Agent Scoping

Create private memories for specific agents:

```rust
// Create agent
system.database().connection().execute(
    "INSERT INTO agents (workspace_id, name) VALUES (?1, 'assistant')",
    [workspace_id],
)?;
let agent_id = system.database().connection().last_insert_rowid();

// Create agent-specific memory
let memory = Memory {
    workspace_id,
    agent_id: Some(agent_id),
    text: "Private agent memory".to_string(),
    // ... other fields
};

system.learn(&memory)?;
```

### Conversation Tracking

Group related memories:

```rust
let memory1 = Memory {
    conversation_id: Some("conv-123".to_string()),
    text: "First message in conversation".to_string(),
    // ... other fields
};

let memory2 = Memory {
    conversation_id: Some("conv-123".to_string()),
    text: "Second message in conversation".to_string(),
    // ... other fields
};

system.learn(&memory1)?;
system.learn(&memory2)?;

// Search within conversation
let filters = SearchFilters {
    conversation_id: Some("conv-123".to_string()),
    ..Default::default()
};

let results = system.search("query", &filters, 10)?;
```

### Memory Hierarchies

Create parent-child relationships:

```rust
let parent_id = system.learn(&parent_memory)?;

let child_memory = Memory {
    parent_memory_id: Some(parent_id),
    text: "Child memory referencing parent".to_string(),
    // ... other fields
};

system.learn(&child_memory)?;
```

### Importance Scoring

Use importance scores to prioritize memories:

```rust
let filters = SearchFilters {
    min_importance: Some(0.7), // Only high-importance memories
    ..Default::default()
};

let results = system.search("query", &filters, 10)?;
```

The hybrid search combines:
- 70% semantic similarity
- 30% importance score

## Troubleshooting

### Server Won't Start

**Problem**: Server fails to start with database error

**Solution**: Check that `~/.memory-rs/workspaces/` directory is writable:
```bash
mkdir -p ~/.memory-rs/workspaces
chmod 755 ~/.memory-rs/workspaces
```

### Embedding Model Not Found

**Problem**: "Model not found" error

**Solution**: The first run downloads models from HuggingFace. Ensure internet connection and wait for download to complete. Models are cached in `~/.cache/huggingface/`.

### Search Returns No Results

**Problem**: Search returns empty results

**Solution**: 
1. Verify memories were stored: Check `memory_id` from learn response
2. Ensure `workspace_id` matches in search filters
3. Try broader search query
4. Check importance score filters aren't too restrictive

### JSON-RPC Parse Errors

**Problem**: "Parse error" in response

**Solution**:
1. Ensure JSON is valid (use `jq` to validate)
2. Check all required fields are present
3. Verify `jsonrpc` field is exactly `"2.0"`
4. Ensure proper line endings (single `\n`)

### Performance Issues

**Problem**: Slow search with many memories

**Solution**:
1. Use more specific filters (workspace_id, agent_id)
2. Reduce search limit
3. Consider using importance score filters
4. For >100K memories, consider database optimization (Task 8)

### Workspace Not Found

**Problem**: Cannot access workspace

**Solution**:
```bash
# List available workspaces
ls ~/.memory-rs/workspaces/

# Recreate workspace
cargo run --example mcp_server workspace-name
```

## Best Practices

1. **Use Descriptive Tags**: Tag memories for easier filtering
2. **Set Importance Scores**: Prioritize critical information (0.7-1.0)
3. **Group Conversations**: Use conversation_id for related memories
4. **Workspace Per Project**: Isolate memories by project/context
5. **Regular Backups**: Backup `~/.memory-rs/workspaces/` directory
6. **Monitor Database Size**: Large databases (>1GB) may need optimization

## Next Steps

- Explore the [API Documentation](API.md)
- Check out [Examples](examples/)
- Read about [Architecture](ARCHITECTURE.md)
- Contribute to the project!
