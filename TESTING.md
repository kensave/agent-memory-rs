# Testing Agent Memory CLI

## Quick Test

Run the test script to verify all CLI commands:

```bash
./test-cli.sh
```

This tests:
- ✅ `stats` - View memory statistics
- ✅ `query` - Search memories
- ✅ `synopsis` - View daily synopsis
- ✅ `consolidate` - Manual consolidation
- ✅ `prune` - Archive old memories

## Full Workflow Test

To test the complete system including memory storage:

### 1. Start MCP Server

```bash
./target/release/agent-memory-mcp test-workspace
```

### 2. Use MCP Tools

In your AI assistant (with MCP configured):

```
@memory/learn with:
{
  "text": "User prefers TypeScript over JavaScript",
  "tags": "user-preference, typescript",
  "importance_score": 0.8
}
```

### 3. Query with CLI

```bash
./target/release/agent-memory-cli query --workspace 1 "typescript" --limit 5
./target/release/agent-memory-cli stats --workspace 1
```

## Available Commands

### stats
View memory statistics for a workspace:
```bash
agent-memory-cli stats --workspace 1
```

### query
Search memories:
```bash
agent-memory-cli query --workspace 1 "search text" --limit 10
```

### synopsis
View daily synopsis:
```bash
agent-memory-cli synopsis --workspace 1 --date 2026-02-01
```

### consolidate
Manually trigger consolidation:
```bash
agent-memory-cli consolidate --date 2026-02-01
```

### prune
Archive old memories:
```bash
agent-memory-cli prune --workspace 1 --threshold 0.3 --dry-run
```

### store
Store an episode (requires workspace created via MCP):
```bash
agent-memory-cli store \
  --workspace 1 \
  --type "user_query" \
  --context "How do I use async/await?" \
  --outcome "Provided tutorial" \
  --valence 0.8
```

## Notes

- **Workspace Creation**: Workspaces are automatically created when using the MCP server
- **Database**: CLI uses `memory.db` by default in the current directory
- **Store Command**: Requires workspace to exist (created via MCP server first)
- **Read-Only Commands**: `stats`, `query`, `synopsis` work without MCP server

## Troubleshooting

**"FOREIGN KEY constraint failed"**
- Workspace doesn't exist yet
- Solution: Start MCP server first to create workspace

**"No results found"**
- No memories stored yet
- Solution: Use `@memory/learn` via MCP to store memories

**"No synopsis found"**
- No consolidation has run for that date
- Solution: Use `consolidate` command or wait for auto-consolidation
