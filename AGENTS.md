# Agent Memory Architecture

## Codebase Overview

**Size:** Large (300+ files, 6,000+ LOC)  
**Language:** Rust  
**Components:** 200+ functions, 40+ structs/classes  
**Tests:** 29 tests

### Key Modules

- **Services** (6 files, ~1,200 LOC) - Core memory operations
- **Storage** (3 files, ~800 LOC) - Database and persistence
- **MCP** (3 files, ~1,000 LOC) - Model Context Protocol server
- **Traits** (5 files, ~110 LOC) - SOLID interfaces
- **CLI** (2 files, ~125 LOC) - Command-line tools

## Architecture Overview

### Core Services (`src/services/`)

**Memory Operations:**
- `EpisodicMemoryStore` - Raw interaction events (250 LOC)
- `HybridRetrievalEngine` - BM25 + vector search (253 LOC)

**Facade:**
- `MemoryManager` - Unified API (140 LOC)

### Storage Layer (`src/storage/`)

- `Database` - SQLite with migrations (224 LOC)
- `MemoryStore` - CRUD + semantic search (360 LOC)
- `schema.rs` - Database initialization (204 LOC)

### MCP Integration (`src/mcp/`)

- `MemoryMcpServer` - MCP protocol server (366 LOC)
- `tools.rs` - Learn/search tools (392 LOC)
- `server.rs` - JSON-RPC server (250 LOC)

### Core Components (`src/`)

- `MemorySystem` - Main orchestrator (285 LOC)
- `FastEmbedder` - BERT embeddings (162 LOC)
- `WorkspaceManager` - Multi-workspace support (234 LOC)
- `ModelDownloader` - Model fetching (63 LOC)

### CLI (`src/cli/`)

- `MemoryCLI` - Command-line interface (123 LOC)

### Traits (`src/traits/`)

SOLID interface definitions:
- `memory_store.rs` - IMemoryStore trait
- `retriever.rs` - IRetriever trait
- `consolidation.rs` - IConsolidationEngine trait
- `decay.rs` - IDecayManager trait
- `embedder.rs` - IEmbedder trait

### Models (`src/models/`)

- `dtos.rs` - Data transfer objects (Episode, Procedure, Synopsis, Pattern)
- `types.rs` - Enums (ModelType, QuantizationType)

### Tests (`tests/`)

44 integration tests covering:
- Memory lifecycle
- Store operations (episodic, procedural, semantic)
- Consolidation and pattern extraction
- Hybrid retrieval
- CLI commands
- Health monitoring

## Overview

Memory-RS provides episodic memory management for AI agents. Episodes are stored with vector embeddings and retrieved using hybrid search (BM25 + vector similarity). Memories are workspace-scoped for project isolation.

## Memory Types

### Episodic Memory
Raw interaction events with full context:
- Event type, timestamp, conversation ID
- Outcome and valence (emotional value)
- Full context as JSON
- Vector embeddings for semantic search

## End-to-End Flow

### 1. Workspace Initialization

```
Agent starts → MCP Server initializes → Ready
```

**Storage Location:**
```
~/.memory-rs/workspaces/
  ├── project-a.db           # Workspace database
  ├── project-b.db           # Workspace database
  └── default.db             # Default workspace
```

### 2. Learning Flow (Episode Storage)

```
Agent learns → Episode created → Stored in episodic memory
```

**Example:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "learn",
    "arguments": {
      "text": "User prefers minimal code",
      "importance_score": 0.9,
      "tags": "user-preference"
    }
  }
}
```

### 3. Retrieval Flow (Hybrid Search)

```
Agent queries → Hybrid search (BM25 + Vector) → Ranked results
```

**Example:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "coding preferences",
      "limit": 10
    }
  }
}
```

**What happens:**
1. Query → BM25 keyword search + Vector similarity
2. Cosine distance retrieval on episode embeddings
3. Return top N results

## Memory Lifecycle

### Continuous Operation
- Episodes stored as events occur
- Vector embeddings generated for semantic search
- Hybrid search (BM25 + vector) for retrieval
- Workspace isolation maintains context boundaries

## Memory Scoping

### Workspace-Scoped (Default)

Memories are isolated per workspace by default:

```rust
// Agent in /path/to/project-a
learn("User likes TypeScript") → stored in project-a/memory.db

// Agent in /path/to/project-b  
search("programming preferences") → searches only project-b/memory.db
```

### Configuration: Scope Control

MCP server can be configured to control memory scope:

```rust
// Config options
pub enum MemoryScope {
    Workspace,      // Default: current workspace only
    Global,         // Search across all workspaces
    WorkspaceFirst, // Workspace + global fallback
}
```

**Usage:**

```bash
# Workspace-scoped (default)
cargo run --bin mcp_server

# Global mode
cargo run --bin mcp_server -- --scope global

# Workspace-first with global fallback
cargo run --bin mcp_server -- --scope workspace-first
```

### Search with Scope Override

```json
{
  "name": "search",
  "arguments": {
    "query": "user preferences",
    "scope": "global",  // Override default
    "limit": 10
  }
}
```

## Agent Isolation

Agents can have private memories within a workspace:

```json
{
  "name": "learn",
  "arguments": {
    "text": "Agent-specific context",
    "agent_id": 5,  // Private to this agent
    "importance_score": 0.8
  }
}
```

**Scoping hierarchy:**
1. Agent-private memories (agent_id set)
2. Workspace-shared memories (agent_id null)
3. Global memories (if enabled)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                      AI Agent                            │
└────────────────────┬────────────────────────────────────┘
                     │ JSON-RPC (stdio)
┌────────────────────▼────────────────────────────────────┐
│                  MCP Server                              │
│  ┌──────────────────────────────────────────┐           │
│  │  Scope: Workspace (default)              │           │
│  │  - Workspace detection                   │           │
│  │  - Memory isolation                      │           │
│  │  - Optional global access                │           │
│  └──────────────────────────────────────────┘           │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              Workspace Manager                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Workspace A  │  │ Workspace B  │  │   Global     │  │
│  │  memory.db   │  │  memory.db   │  │  memory.db   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Use Cases

### 1. Project-Specific Context

**Scenario:** Agent working on multiple projects

```
Project A: "Use React for frontend"
Project B: "Use Vue for frontend"
```

Workspace scoping prevents context bleeding.

### 2. Global Preferences

**Scenario:** User preferences across all projects

```bash
# Start with global scope
mcp_server --scope global
```

```json
{
  "name": "learn",
  "arguments": {
    "text": "User prefers minimal code",
    "scope": "global"
  }
}
```

### 3. Hybrid Approach

**Scenario:** Workspace-first with global fallback

```
1. Search workspace for project-specific context
2. If insufficient, search global memories
3. Combine results with workspace-first ranking
```

## Implementation Details

### Workspace Detection

```rust
pub fn detect_workspace() -> Result<String> {
    // 1. Check environment variable
    if let Ok(ws) = env::var("MEMORY_WORKSPACE") {
        return Ok(ws);
    }
    
    // 2. Use current directory
    let cwd = env::current_dir()?;
    Ok(cwd.file_name()?.to_string())
}
```

### Scope Configuration

```rust
pub struct McpConfig {
    pub scope: MemoryScope,
    pub workspace_name: Option<String>,
    pub enable_global: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            scope: MemoryScope::Workspace,
            workspace_name: None,
            enable_global: false,
        }
    }
}
```

### Search with Scope

```rust
pub async fn search_with_scope(
    query: &str,
    scope: MemoryScope,
    filters: SearchFilters,
) -> Result<Vec<SearchResult>> {
    match scope {
        MemoryScope::Workspace => {
            // Search current workspace only
            search_workspace(query, filters).await
        }
        MemoryScope::Global => {
            // Search all workspaces
            search_all_workspaces(query, filters).await
        }
        MemoryScope::WorkspaceFirst => {
            // Try workspace first, fallback to global
            let mut results = search_workspace(query, filters.clone()).await?;
            if results.len() < filters.limit {
                let global = search_global(query, filters).await?;
                results.extend(global);
            }
            Ok(results)
        }
    }
}
```

## Best Practices

1. **Default to workspace-scoped** - Prevents context pollution
2. **Use global sparingly** - Only for true cross-project preferences
3. **Tag appropriately** - Use tags to organize memories
4. **Set importance scores** - Higher for critical context
5. **Use conversation_id** - Group related memories
6. **Agent isolation** - Use agent_id for private context

## Configuration Examples

### Workspace-Only (Default)

```bash
cargo run --bin mcp_server
```

### Global Access

```bash
cargo run --bin mcp_server -- --scope global
```

### Custom Workspace

```bash
MEMORY_WORKSPACE=my-project cargo run --bin mcp_server
```

### Hybrid Mode

```bash
cargo run --bin mcp_server -- --scope workspace-first --enable-global
```

## Future Enhancements

- [ ] Workspace sharing between agents
- [ ] Memory export/import between workspaces
- [ ] Workspace-level configuration files
- [ ] Memory sync across machines
- [ ] Workspace templates
