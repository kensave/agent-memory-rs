# Memory-RS: Agent Memory Management System

A comprehensive memory management system for AI agents with episodic, semantic, and procedural memory types, intelligent consolidation, and decay mechanisms. Built in Rust with SOLID principles.

## 🎯 Features

### Memory Types
- **Episodic Memory**: Raw interaction events with full context, timestamps, and valence
- **Semantic Memory**: Distilled knowledge with confidence scores and source tracking
- **Procedural Memory**: Learned workflows with success rates and usage tracking

### Core Capabilities
- **Intelligent Consolidation**: Automatic pattern extraction and daily synopsis generation
- **Hybrid Search**: BM25 keyword + vector semantic search with RRF fusion
- **Hierarchical Retrieval**: Multi-level memory access (synopsis → semantic → episodic → archived)
- **Smart Decay**: Composite scoring with automatic archival (recency×0.3 + relevance×0.4 + utility×0.3)
- **MCP Server**: Auto-consolidation on startup and every N messages
- **CLI Tools**: 5 commands for memory operations (consolidate, synopsis, stats, prune, query)
- **Health Monitoring**: System metrics and health scoring

### Technical
- **Persistent Storage**: SQLite with sqlite-vec extension (384-dim embeddings)
- **Thread-Safe**: Arc<Mutex<Connection>> pattern, no lifetimes
- **SOLID Architecture**: 5 traits with clean separation of concerns
- **Test Coverage**: 44 integration tests, full lifecycle testing
- **Performance**: Episode storage ~5ms, Search ~20ms, Consolidation ~2s

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/yourusername/memory-rs
cd memory-rs
cargo build --release

# Install binaries
cargo install --path .
```

### MCP Server

Start the MCP server (auto-consolidates on startup):

```bash
# Development
cargo run --bin memory-rs-mcp my-workspace

# Production
./target/release/memory-rs-mcp my-workspace
```

Add to your AI agent's MCP configuration (e.g., Claude Desktop `config.json`):

```json
{
  "mcpServers": {
    "memory-rs": {
      "command": "/path/to/memory-rs-mcp",
      "args": ["my-workspace"]
    }
  }
}
```

### CLI Usage

```bash
# View daily synopsis
memory-cli synopsis --date 2026-01-31

# Query memories
memory-cli query "rust programming" --limit 10

# Check system health
memory-cli stats --workspace 1

# Consolidate memories
memory-cli consolidate --date 2026-01-31

# Prune old memories
memory-cli prune --workspace 1 --dry-run
```

## 📚 Documentation

- **[Complete API Reference](docs/README.md)** - Full API documentation with examples
- **[MCP Auto-Consolidation](docs/MCP_AUTO_CONSOLIDATION.md)** - How auto-consolidation works
- **[Architecture](docs/architecture-redesign.md)** - System architecture and design decisions
- **[Interface Design](docs/interface-design.md)** - SOLID principles and trait design
- **[Schema](docs/schema-extensions-v2.md)** - Database schema and migrations

## 🏗️ Architecture

### Memory Hierarchy (5 Levels)

```
Level 1: Working Memory (Current Session)
    ↓
Level 2: Daily Synopsis (Compressed daily summary)
    ↓
Level 3: Semantic Memory (Distilled knowledge)
    ↓
Level 4: Episodic Memory (Recent events, last 7-30 days)
    ↓
Level 5: Archived Episodes (Old events, >30 days)
```

### Core Services

```
MemoryManager (Facade)
    ├── EpisodicMemoryStore    - Raw interaction events
    ├── SemanticMemoryStore    - Distilled knowledge
    ├── ProceduralMemoryStore  - Learned workflows
    ├── HybridRetrievalEngine  - BM25 + Vector search
    ├── ConsolidationEngine    - Pattern extraction & synopsis
    └── DecayManager           - Intelligent archival
```

### Auto-Consolidation

The MCP server automatically:
1. **On startup**: Consolidates yesterday's memories (background, non-blocking)
2. **Every 20 messages**: Triggers consolidation (configurable, background)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test test_lifecycle

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- 44 integration tests
- Full lifecycle tests (store → consolidate → retrieve → decay → archive)
- Hierarchical retrieval tests
- CLI tests
- Health monitoring tests

## 🔧 Development

### Project Structure

```
src/
├── services/          # 12 core services
├── storage/           # Database and memory store
├── traits/            # 5 SOLID traits
├── models/            # DTOs and types
├── cli/               # CLI commands
└── mcp/               # MCP server

tests/                 # 13 integration test files
docs/                  # 5 documentation files
```

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build MCP server only
cargo build --bin memory-rs-mcp --release
```

## 📊 Performance

- **Episode storage**: ~5ms
- **Hybrid search**: ~20ms (1000 memories)
- **Consolidation**: ~2s (100 episodes)
- **Synopsis generation**: ~500ms
- **All operations**: Non-blocking

## 🤝 Contributing

1. Follow SOLID principles
2. Write minimal, focused code
3. Add tests for new features
4. Update documentation
5. Run `cargo test` before committing

## 📝 License

MIT OR Apache-2.0

## 🙏 Acknowledgments

Built with:
- Rust 🦀
- SQLite + sqlite-vec
- Candle (ML framework)
- MCP Protocol

---

**Status**: Production-ready ✅
**Tests**: 44 passing ✅
**Documentation**: Complete ✅
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
