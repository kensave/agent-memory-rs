# Memory-RS: Persistent Memory System with MCP Server

A high-performance, SQLite-backed persistent memory system with semantic search capabilities, exposed via the Model Context Protocol (MCP). Built in Rust for speed, safety, and reliability.

## 🎯 Features

- **Persistent Storage**: SQLite database with sqlite-vec extension for vector embeddings
- **Semantic Search**: Hybrid search combining cosine similarity with metadata filtering
- **MCP Protocol**: JSON-RPC 2.0 over stdio for easy integration with AI assistants
- **Workspace Isolation**: Multi-database support with per-workspace memory isolation
- **Agent Scoping**: Shared workspace memories + optional private agent memories
- **Rich Metadata**: Tags, importance scores, conversation tracking, user feedback
- **Embedding Models**: Support for MiniLM, Nomic, and BGE-small models
- **Test-Driven**: 28 comprehensive tests ensuring reliability

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/yourusername/memory-rs
cd memory-rs
cargo build --release

# Install binary to PATH
cargo install --path .
```

### MCP Configuration

Add to your AI agent's MCP configuration (e.g., Claude Desktop `config.json`):

```json
{
  "mcpServers": {
    "memory-rs": {
      "command": "memory-rs",
      "args": ["--scope", "workspace"]
    }
  }
}
```

**Scope Options:**

```json
// Workspace-only (default) - memories isolated per project
"args": ["--scope", "workspace"]

// Global - access memories across all workspaces
"args": ["--scope", "global"]

// Workspace-first - search workspace, fallback to global
"args": ["--scope", "workspace-first"]

// Custom workspace name
"args": ["my-project-name"]
```

**Full Example:**

```json
{
  "mcpServers": {
    "memory-rs": {
      "command": "memory-rs",
      "args": ["--scope", "workspace-first", "my-project"]
    }
  }
}
```

### Running Manually

```bash
# Start server for default workspace
cargo run --bin mcp_server

# Start server for specific workspace
cargo run --bin mcp_server my-project

# With scope configuration
cargo run --bin mcp_server -- --scope global
```

### Basic Usage

The MCP server communicates via stdio using JSON-RPC 2.0. Here's how to interact with it:

#### Learn (Store a Memory)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "learn",
    "arguments": {
      "text": "Rust is a systems programming language focused on safety and performance",
      "workspace_id": 1,
      "importance_score": 0.8,
      "tags": "rust,programming"
    }
  }
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "memory_id": 42,
    "status": "success"
  }
}
```

#### Search (Query Memories)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "programming languages",
      "workspace_id": 1,
      "limit": 5
    }
  }
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "results": [
      {
        "memory_id": 42,
        "text": "Rust is a systems programming language...",
        "similarity_score": 0.92,
        "combined_score": 0.88,
        "importance_score": 0.8,
        "tags": "rust,programming",
        "created_at": "2026-01-30T22:00:00Z"
      }
    ],
    "count": 1
  }
}
```

## 📚 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Tool                             │
└──────────────────────────┬──────────────────────────────────┘
                           │ stdio (JSON-RPC 2.0)
┌──────────────────────────▼──────────────────────────────────┐
│                      MCP Server                              │
│  ┌────────────────┐  ┌────────────────┐                     │
│  │  Learn Tool    │  │  Search Tool   │                     │
│  └────────┬───────┘  └────────┬───────┘                     │
└───────────┼──────────────────┼─────────────────────────────┘
            │                  │
┌───────────▼──────────────────▼─────────────────────────────┐
│                    Memory System                             │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │  FastEmbedder    │  │  Memory Store    │                │
│  │  (MiniLM/Nomic)  │  │  (SQLite+vec)    │                │
│  └──────────────────┘  └──────────────────┘                │
└─────────────────────────────────────────────────────────────┘
            │                  │
┌───────────▼──────────────────▼─────────────────────────────┐
│              Workspace Manager                               │
│  ~/.memory-rs/workspaces/                                    │
│    ├── project-a/memory.db                                   │
│    ├── project-b/memory.db                                   │
│    └── project-c/memory.db                                   │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

1. **Storage Layer** (`src/storage/`)
   - `schema.rs`: Database schema with sqlite-vec integration
   - `memory_store.rs`: CRUD operations and vector search

2. **Memory System** (`src/memory_system.rs`)
   - High-level API combining embedder and storage
   - Atomic learn and search operations

3. **MCP Server** (`src/mcp/`)
   - `server.rs`: JSON-RPC 2.0 stdio transport
   - `tools.rs`: Learn and Search tool implementations

4. **Workspace Manager** (`src/workspace.rs`)
   - Multi-database support
   - Workspace isolation and management

5. **Embedder** (`src/embedder.rs`)
   - FastEmbedder with multiple model support
   - Mock fallback for testing

## 🔧 Configuration

### Embedding Models

Choose your embedding model based on your needs:

| Model | Dimensions | Speed | Quality |
|-------|-----------|-------|---------|
| MiniLM | 384 | Fast | Good |
| BGE-small | 384 | Medium | Better |
| Nomic | 768 | Slower | Best |

Configure in code:
```rust
use memory_rs::{WorkspaceManager, ModelType};

let manager = WorkspaceManager::new(ModelType::BgeSmall)?;
```

### Workspace Management

Workspaces are stored in `~/.memory-rs/workspaces/` by default:

```rust
use memory_rs::WorkspaceManager;

let manager = WorkspaceManager::new(ModelType::MiniLM)?;

// Create or get workspace
let system = manager.get_or_create_workspace("my-project")?;

// List all workspaces
let workspaces = manager.list_workspaces()?;

// Delete workspace
manager.delete_workspace("old-project")?;
```

## 🧪 Testing

Run all tests:
```bash
cargo test
```

Run specific test suites:
```bash
# Storage tests
cargo test --lib storage

# MCP server tests
cargo test --lib mcp

# Workspace tests
cargo test --lib workspace
```

## 📊 Database Schema

### Tables

**workspaces**
- `id`: Primary key
- `name`: Workspace name (unique)
- `path`: Filesystem path
- `created_at`: Timestamp

**agents**
- `id`: Primary key
- `workspace_id`: Foreign key to workspaces
- `name`: Agent name
- `created_at`: Timestamp

**memories**
- `id`: Primary key
- `workspace_id`: Foreign key to workspaces
- `agent_id`: Optional foreign key to agents
- `text`: Memory content
- `tags`: Comma-separated tags
- `importance_score`: Float 0-1
- `access_count`: Usage tracking
- `last_accessed`: Timestamp
- `conversation_id`: Optional conversation grouping
- `parent_memory_id`: Optional memory hierarchy
- `user_feedback`: Optional feedback text
- `created_at`, `updated_at`: Timestamps

**vec0** (virtual table)
- `memory_id`: Foreign key to memories
- `embedding`: Float vector (384 or 768 dimensions)

### Indexes

- `idx_memories_workspace`: Fast workspace filtering
- `idx_memories_agent`: Fast agent filtering
- `idx_memories_importance`: Importance-based queries
- `idx_memories_created`: Temporal queries
- `idx_memories_conversation`: Conversation grouping

## 🔍 Search Capabilities

### Hybrid Search

Combines semantic similarity (70%) with importance score (30%):

```rust
use memory_rs::storage::SearchFilters;

let filters = SearchFilters {
    workspace_id: Some(1),
    agent_id: Some(5),
    min_importance: Some(0.5),
    max_importance: Some(1.0),
    conversation_id: Some("conv-123".to_string()),
    ..Default::default()
};

let results = system.search("query text", &filters, 10)?;
```

### Filtering Options

- **workspace_id**: Limit to specific workspace
- **agent_id**: Limit to specific agent
- **min_importance / max_importance**: Importance range
- **created_after / created_before**: Date range
- **conversation_id**: Conversation grouping
- **tags**: Tag-based filtering (future)

## 🚦 MCP Protocol

### Available Methods

1. **initialize**: Server initialization
2. **tools/list**: List available tools
3. **tools/call**: Execute a tool
4. **learn**: Store a memory (via tools/call)
5. **search**: Query memories (via tools/call)

### Tool Schemas

#### Learn Tool

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "text": {"type": "string", "description": "The text to remember"},
    "workspace_id": {"type": "integer", "description": "Workspace ID"},
    "agent_id": {"type": "integer", "description": "Optional agent ID"},
    "tags": {"type": "string", "description": "Optional comma-separated tags"},
    "importance_score": {"type": "number", "description": "Importance score 0-1"},
    "conversation_id": {"type": "string", "description": "Optional conversation ID"}
  },
  "required": ["text", "workspace_id"]
}
```

#### Search Tool

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "query": {"type": "string", "description": "Search query"},
    "workspace_id": {"type": "integer", "description": "Optional workspace ID filter"},
    "agent_id": {"type": "integer", "description": "Optional agent ID filter"},
    "min_importance": {"type": "number", "description": "Minimum importance score"},
    "max_importance": {"type": "number", "description": "Maximum importance score"},
    "conversation_id": {"type": "string", "description": "Optional conversation ID filter"},
    "limit": {"type": "integer", "description": "Maximum results (default 10, max 100)"}
  },
  "required": ["query"]
}
```

## 🎓 Examples

See `examples/` directory for complete examples:

- `mcp_server.rs`: Full MCP server implementation
- More examples coming soon!

## 📈 Performance

- **Storage**: SQLite with sqlite-vec for efficient vector operations
- **Embedding**: ~300ms per embedding with real models, ~20μs with mock
- **Search**: Sub-second for <10K memories, optimized for 100K+ scale
- **Memory**: Efficient storage with optional quantization support

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## 📝 License

MIT OR Apache-2.0

## 🙏 Acknowledgments

- [sqlite-vec](https://github.com/asg017/sqlite-vec) for vector search in SQLite
- [Candle](https://github.com/huggingface/candle) for ML inference
- Model Context Protocol by Anthropic

## 📞 Support

- Issues: [GitHub Issues](https://github.com/yourusername/memory-rs/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/memory-rs/discussions)

---

Built with ❤️ in Rust
