# Agent Memory Architecture

## Codebase Overview

**Size:** Large (367 files, 6,455 LOC)  
**Language:** Rust  
**Components:** 228 functions, 45 structs/classes  
**Tests:** 44 integration tests

### Key Modules

- **Services** (11 files, ~1,600 LOC) - Core memory operations
- **Storage** (3 files, ~800 LOC) - Database and persistence
- **MCP** (3 files, ~1,000 LOC) - Model Context Protocol server
- **Traits** (6 files, ~110 LOC) - SOLID interfaces
- **CLI** (2 files, ~125 LOC) - Command-line tools

## Overview

Memory-RS provides a comprehensive memory management system for AI agents with three memory types: episodic (raw events), semantic (distilled knowledge), and procedural (learned workflows). Memories are workspace-scoped with automatic consolidation and intelligent decay.

## Memory Types

### 1. Episodic Memory
Raw interaction events with full context:
- Event type, timestamp, conversation ID
- Outcome and valence (emotional value)
- Full context as JSON
- Archival support (active → archived)

### 2. Semantic Memory
Distilled knowledge with confidence tracking:
- Source episode tracking
- Confidence scores (updated with reinforcement)
- Access count and validation timestamps
- Tags and importance scores

### 3. Procedural Memory
Learned workflows and patterns:
- Trigger conditions
- Action sequences
- Success rate tracking
- Usage count and last used timestamp

## End-to-End Flow

### 1. Workspace Initialization

```
Agent starts → MCP Server initializes → Consolidates yesterday's memories → Ready
```

**Storage Location:**
```
~/.memory-rs/workspaces/
  ├── project-a.db           # Workspace database
  ├── project-b.db           # Workspace database
  └── default.db             # Default workspace
```

### 2. Learning Flow (Episodic Storage)

```
Agent learns → Episode created → Stored in episodic memory → Message counter increments
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

### 3. Consolidation Flow (Automatic)

```
Every 20 messages → Consolidate triggered → Extract patterns → Update semantic/procedural → Generate synopsis
```

**What happens:**
1. PatternExtractor analyzes episodes
2. Recurring patterns → Semantic memory (confidence > 0.6)
3. Successful workflows → Procedural memory (frequency >= 2)
4. DailySynopsisGenerator creates summary
5. Episodes marked for archival

### 4. Retrieval Flow (Hierarchical)

```
Agent queries → Hybrid search (BM25 + Vector) → Hierarchical retrieval → Ranked results
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
2. Hierarchical retrieval:
   - Semantic memory (50% of results)
   - Recent episodes (25% of results)
   - Procedures (25% of results)
3. RRF fusion and composite scoring
4. Return top N results

### 5. Decay Flow (Weekly)

```
Every 7 days → Calculate composite scores → Archive low-scoring → Prune redundant
```

**Composite Score Formula:**
```
score = (recency × 0.3) + (relevance × 0.4) + (utility × 0.3)

recency = exp(-0.1 × days_since_access)
relevance = cosine_similarity(embedding, query)
utility = (access_count × 0.4) + (success_rate × 0.4) + (feedback × 0.2)
```

## Memory Lifecycle

### Week 1: Accumulation
- Episodes stored as events occur
- Message counter tracks activity
- Auto-consolidation every 20 messages

### Week 2-4: Consolidation
- Patterns extracted nightly
- Semantic memory grows
- Procedures refined with success rates
- Daily synopses accumulate

### Month 2+: Optimization
- Low-scoring episodes archived
- High-confidence knowledge retained
- Proven workflows (80%+ success) prioritized
- Context stays relevant

## Memory Scoping

### Workspace-Scoped (Default)

Memories are isolated per workspace by default:

```rust
// Agent in /Users/kenneth/project-a
learn("User likes TypeScript") → stored in project-a/memory.db

// Agent in /Users/kenneth/project-b  
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
