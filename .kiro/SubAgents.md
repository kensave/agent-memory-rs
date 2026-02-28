# SubAgents Collaboration Guidelines

## Agent Identities

- **Roberto (Coder)** - coding-agent - Handles implementation, testing, debugging
- **Jonathan (Researcher)** - research-agent - Handles research, documentation, analysis

## Core Collaboration Rules

### 1. Git Worktrees - MANDATORY

**Always use git worktrees for parallel work:**

```bash
# Roberto creates worktree for feature
git worktree add ../memory-rs-feature feature-branch

# Jonathan creates worktree for research
git worktree add ../memory-rs-research research-branch
```

**Why:** Prevents conflicts, allows simultaneous work on different branches.

### 2. Memory System - USE CONSTANTLY

**CRITICAL: Memory is your communication channel. Use it every 5-10 minutes.**

#### Learning (Store Information)
```
@memory/learn {
  "text": "[Roberto] Implemented consolidation fix in src/services/consolidation_engine.rs. Ready for Jonathan to document.",
  "importance_score": 0.8,
  "tags": "status-update,consolidation,ready-for-docs"
}
```

#### Searching (Check Updates)
```
@memory/search {
  "query": "Jonathan status updates",
  "limit": 5
}
```

**Memory is shared across ALL agents** - Roberto can read Jonathan's memories and vice versa.

#### What to Store in Memory

**Status Updates:**
- `[Roberto] Started working on X`
- `[Roberto] Completed X, blocked on Y`
- `[Jonathan] Found documentation for Z`

**Questions:**
- `[Roberto] Question for Jonathan: What's the best approach for X?`
- `[Jonathan] Question for Roberto: Does the API support Y?`

**Decisions:**
- `[Roberto] Decision: Using approach A because B`
- `[Jonathan] Decision: Documenting X before Y`

**Blockers:**
- `[Roberto] Blocked: Need research on X before implementing`
- `[Jonathan] Blocked: Need code example for X`

**Completions:**
- `[Roberto] Completed: Feature X is ready for testing`
- `[Jonathan] Completed: Documentation for X is done`

### 3. Commit Changes - ALWAYS

**Commit after every meaningful change:**

```bash
# Roberto commits code
git add .
git commit -m "[Roberto] Implement feature X - ready for Jonathan to document"
git push origin feature-branch

# Jonathan commits docs
git add .
git commit -m "[Jonathan] Document feature X based on Roberto's implementation"
git push origin research-branch
```

**Commit message format:** `[AgentName] Action - Context`

### 4. Project Workspace Structure

```
.kiro/
├── agents/
│   ├── coding-agent.json
│   └── research-agent.json
└── projects/
    └── memory-rs/
        ├── status.md          # Current status
        ├── tasks.md           # Task breakdown
        ├── decisions.md       # Design decisions
        └── communication.md   # Agent messages
```

## Collaboration Workflow

### Starting Work

**Roberto (Coder):**
1. Check memory for Jonathan's updates: `@memory/search "Jonathan"`
2. Create worktree: `git worktree add ../memory-rs-feature feature-name`
3. Store status: `@memory/learn "[Roberto] Starting work on X in worktree memory-rs-feature"`

**Jonathan (Researcher):**
1. Check memory for Roberto's updates: `@memory/search "Roberto"`
2. Create worktree: `git worktree add ../memory-rs-research research-name`
3. Store status: `@memory/learn "[Jonathan] Starting research on X in worktree memory-rs-research"`

### During Work (Every 10-15 minutes)

1. **Check memory** for partner's updates
2. **Store progress** in memory
3. **Commit changes** with descriptive message
4. **Update** `.kiro/projects/memory-rs/status.md`

### Asking Questions

**Roberto asks Jonathan:**
```
@memory/learn {
  "text": "[Roberto] QUESTION for Jonathan: What's the recommended approach for implementing feature X? I'm considering A vs B.",
  "importance_score": 0.9,
  "tags": "question,roberto-to-jonathan,feature-x"
}
```

**Jonathan responds:**
```
@memory/learn {
  "text": "[Jonathan] ANSWER to Roberto's question: Approach A is better because of reasons X, Y, Z. See research in docs/feature-x.md",
  "importance_score": 0.9,
  "tags": "answer,jonathan-to-roberto,feature-x"
}
```

### Handoffs

**Roberto finishes implementation:**
```bash
git commit -m "[Roberto] Complete feature X implementation - ready for Jonathan to document"
git push

@memory/learn {
  "text": "[Roberto] HANDOFF: Feature X is complete in src/feature.rs. Ready for Jonathan to write documentation. Key points: does A, B, C.",
  "importance_score": 1.0,
  "tags": "handoff,roberto-to-jonathan,feature-x,ready-for-docs"
}
```

**Jonathan picks up:**
```
@memory/search {
  "query": "Roberto handoff ready for docs",
  "limit": 3
}

@memory/learn {
  "text": "[Jonathan] ACKNOWLEDGED: Starting documentation for feature X based on Roberto's implementation",
  "importance_score": 0.9,
  "tags": "acknowledgment,jonathan-to-roberto,feature-x"
}
```

## Communication Patterns

### Pattern 1: Parallel Work
- Roberto: Implements feature A
- Jonathan: Documents feature B (already implemented)
- Both check memory every 10 minutes for blockers

### Pattern 2: Sequential Work
- Jonathan: Researches approach for feature X
- Jonathan: Stores findings in memory with "ready-for-implementation" tag
- Roberto: Searches memory, implements based on research
- Roberto: Stores completion with "ready-for-docs" tag
- Jonathan: Documents implementation

### Pattern 3: Collaborative Problem Solving
- Roberto: Hits blocker, stores question in memory
- Jonathan: Searches memory, finds question, researches solution
- Jonathan: Stores answer in memory
- Roberto: Searches memory, finds answer, continues work

## Memory Search Strategies

**Check partner's recent activity:**
```
@memory/search "Jonathan last 30 minutes"
@memory/search "Roberto status"
```

**Find specific information:**
```
@memory/search "consolidation implementation details"
@memory/search "benchmark results"
```

**Check for questions/blockers:**
```
@memory/search "QUESTION for Roberto"
@memory/search "BLOCKED"
```

**Find handoffs:**
```
@memory/search "HANDOFF ready"
@memory/search "ready-for-docs"
```

## Best Practices

1. **Memory First** - Before starting work, check memory for updates
2. **Frequent Updates** - Store status every 10-15 minutes
3. **Clear Tags** - Use consistent tags: status-update, question, answer, handoff, blocked
4. **Descriptive Messages** - Include context, not just "done with X"
5. **Commit Often** - Small, frequent commits with clear messages
6. **Worktree Discipline** - Never work in same worktree simultaneously
7. **Acknowledge Handoffs** - Always acknowledge when picking up work
8. **Update Status File** - Keep `.kiro/projects/memory-rs/status.md` current

## Emergency Protocols

**If Roberto is blocked:**
```
@memory/learn {
  "text": "[Roberto] URGENT BLOCKER: Cannot proceed with X because Y. Need Jonathan's input ASAP.",
  "importance_score": 1.0,
  "tags": "urgent,blocker,roberto-to-jonathan"
}
```

**If Jonathan finds critical issue:**
```
@memory/learn {
  "text": "[Jonathan] CRITICAL: Found issue in X that affects Y. Roberto should review before continuing.",
  "importance_score": 1.0,
  "tags": "critical,issue,jonathan-to-roberto"
}
```

## Success Metrics

- Memory updates every 10-15 minutes
- Commits every 20-30 minutes
- Zero merge conflicts (thanks to worktrees)
- Questions answered within 1 hour
- Handoffs acknowledged within 30 minutes

## Remember

**You are a team. Memory is your communication channel. Use it constantly.**
