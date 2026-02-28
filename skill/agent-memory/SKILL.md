---
name: agent-memory
description: Persistent memory system for AI agents with episodic and semantic memory. Use when Claude needs to (1) Store information for future sessions (learn), (2) Recall previous conversations or decisions (search), (3) Build long-term context about users, projects, or patterns, (4) Remember preferences, workflows, or domain knowledge across conversations. Enables stateful agents that improve over time.
---

# Agent Memory

Persistent memory system enabling stateful AI agents through learn and search operations.

## When to Use

**Learn** - Store information for future recall:
- User preferences and patterns
- Project context and decisions
- Workflows and procedures
- Domain knowledge and schemas

**Search** - Retrieve relevant memories:
- Before starting tasks (check prior work)
- When user references past context
- To maintain consistency across sessions
- To leverage accumulated knowledge

## Quick Start

### Store a Memory

```
@memory/learn with:
{
  "text": "User Kenneth prefers TypeScript over JavaScript for new projects",
  "tags": "user-preference, typescript",
  "importance_score": 0.8
}
```

### Search Memories

```
@memory/search with:
{
  "query": "Kenneth's language preferences",
  "limit": 5
}
```

## Memory Types

The system automatically organizes memories into two types:

**Episodic** - Specific interaction events
- Raw conversation context
- Task outcomes and results
- Temporal sequences
- Stored as episodes with vector embeddings

**Semantic** - Distilled knowledge
- Facts and concepts
- User preferences
- Domain expertise
- Created through consolidation (importance > 0.7) or pattern extraction

## Best Practices

### What to Learn

**DO learn:**
- User preferences ("prefers dark mode")
- Project context ("working on CV site in /path/to/workspace/NewCV")
- Decisions made ("chose Next.js 14 for performance")
- Patterns discovered ("user typically works evenings PST")
- Domain knowledge ("company uses PostgreSQL for production")

**DON'T learn:**
- Temporary information (current time, weather)
- Publicly available facts (language syntax, API docs)
- Sensitive data (passwords, API keys)
- Redundant information (already stored)

### Importance Scoring

- **0.9-1.0**: Critical preferences, key decisions
- **0.7-0.8**: Important context, frequent patterns
- **0.5-0.6**: Useful information, occasional patterns
- **0.3-0.4**: Minor details, rare occurrences

### Tagging Strategy

Use descriptive, searchable tags:
- `user-preference` - User choices and preferences
- `project-context` - Project-specific information
- `workflow` - Procedures and patterns
- `decision` - Important decisions made
- `domain-knowledge` - Technical or business knowledge

Combine tags for specificity: `user-preference, typescript, code-style`

### Search Strategy

**Before tasks:**
```
Search: "project setup and preferences"
→ Retrieve context before starting work
```

**During conversations:**
```
Search: "previous discussions about authentication"
→ Maintain consistency with past decisions
```

**For patterns:**
```
Search: "user's typical workflow"
→ Anticipate needs and preferences
```

## Workflow Patterns

### Pattern 1: New Project Setup

1. **Search** for user preferences and patterns
2. Apply learned preferences to setup
3. **Learn** project-specific decisions
4. **Learn** new patterns discovered

### Pattern 2: Continuing Work

1. **Search** for project context
2. Review previous decisions and state
3. Continue work with full context
4. **Learn** new developments

### Pattern 3: Problem Solving

1. **Search** for similar past problems
2. Apply learned solutions
3. **Learn** new solution if novel
4. **Learn** what worked/didn't work

## Configuration

The memory system uses BGE-Small embeddings by default for optimal quality/speed balance.

**Model options** (via `MEMORY_MODEL` env var):
- `bge` (default) - Best quality/speed, 384 dims
- `nomic` - Long context (8K tokens), 768 dims
- `minilm` - Fastest, 384 dims

**Note:** Changing models requires separate workspaces due to dimension differences.

## Advanced Features

### Auto-Consolidation

The system automatically:
- Consolidates yesterday's memories on first use
- Extracts patterns every 20 messages
- Generates daily synopsis
- Archives low-value memories

### Hierarchical Retrieval

Search automatically queries across:
- Daily synopsis (recent summary)
- Semantic memory (distilled knowledge)
- Episodic memory (specific events)
- Procedural memory (workflows)

Results ranked by: `(similarity × 0.7) + (importance × 0.3)`

### Memory Health

The system tracks:
- Active vs archived memories
- Retrieval accuracy
- Learning rate
- Memory health ratio

## Troubleshooting

**No results found:**
- Try broader search terms
- Check if information was actually learned
- Verify workspace is correct

**Irrelevant results:**
- Use more specific search terms
- Add context to query
- Increase importance scores when learning

**Slow performance:**
- Reduce search limit
- Use more specific queries
- Consider switching to `minilm` model

## Examples

### Example 1: User Preference

```
Learn:
{
  "text": "Kenneth prefers minimal code comments, relying on self-documenting code with clear variable names",
  "tags": "user-preference, code-style, comments",
  "importance_score": 0.8
}

Search: "Kenneth's code style preferences"
→ Returns preference about comments
```

### Example 2: Project Context

```
Learn:
{
  "text": "NewCV project uses Next.js 14, TypeScript, Tailwind CSS. Deployed on Vercel. Database: PostgreSQL on Supabase.",
  "tags": "project-context, newcv, tech-stack",
  "importance_score": 0.9
}

Search: "NewCV tech stack"
→ Returns complete project setup
```

### Example 3: Workflow Pattern

```
Learn:
{
  "text": "When refactoring, Kenneth prefers: 1) Write tests first, 2) Refactor incrementally, 3) Run tests after each change",
  "tags": "workflow, refactoring, testing",
  "importance_score": 0.7
}

Search: "refactoring workflow"
→ Returns step-by-step procedure
```

## Integration

The memory system integrates with:
- MCP servers (Model Context Protocol)
- CLI tools for manual operations
- Background consolidation services

See [repository](https://github.com/yourusername/agent-memory-rs) for setup details.
