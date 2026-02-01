# MCP Server with Auto-Consolidation

## How It Works

The MCP server now automatically consolidates memories:

### **On Server Start**
```bash
cargo run --bin memory-rs-mcp my-workspace
# Or after building:
./target/release/memory-rs-mcp my-workspace
```

**What happens:**
```
🚀 Memory MCP Server initializing...
🔄 Consolidating memories from 2026-01-30...
✅ Memory MCP Server ready
✅ Consolidation complete: 5 insights extracted
```

The server consolidates **yesterday's memories** in the background while starting up.

---

### **During Usage (Every 20 Messages)**

**Messages 1-19:**
```json
// Client calls learn tool
{"method": "tools/call", "params": {"name": "learn", "arguments": {"text": "..."}}}
// Server stores memory, increments counter (1, 2, 3... 19)
{"result": {"memory_id": 123, "status": "success"}}
```

**Message 20:**
```json
// Client calls learn tool
{"method": "tools/call", "params": {"name": "learn", "arguments": {"text": "..."}}}
// Server stores memory, counter reaches 20
📊 20 messages processed, triggering consolidation
// Consolidation runs in background (non-blocking)
{"result": {"memory_id": 143, "status": "success"}}
// Server continues immediately

// Background:
✅ Auto-consolidation complete
```

**Message 21-39:**
- Counter increments again
- At message 40, consolidates again

---

## Configuration

### **Change Consolidation Threshold**

Modify `src/mcp/rmcp_server.rs`:

```rust
impl MemoryMcpServer {
    pub fn new(workspace_name: &str) -> Result<Self> {
        // ...
        Ok(Self {
            // ...
            consolidation_threshold: 50,  // Change from 20 to 50
        })
    }
}
```

### **Disable Auto-Consolidation**

```rust
consolidation_threshold: usize::MAX,  // Never auto-consolidate
```

### **Consolidate on Every Message**

```rust
consolidation_threshold: 1,  // Consolidate after every message (not recommended)
```

---

## What Gets Consolidated

### **Input (Raw Episodes)**
```
Episode 1: learn("Rust error handling")
Episode 2: learn("Connection timeout fix")
Episode 3: learn("Database optimization")
Episode 4: search("error handling")
Episode 5: learn("Async patterns")
...
Episode 20: search("rust patterns")
```

### **Output (After Consolidation)**

**Semantic Memory:**
```
Memory {
    text: "Rust error handling patterns: Use Result<> types",
    confidence: 0.7,
    source_episodes: [1, 4]
}

Memory {
    text: "Connection timeout: increase to 30s",
    confidence: 0.8,
    source_episodes: [2]
}
```

**Procedural Memory:**
```
Procedure {
    name: "Debug connection issues",
    trigger: {"error_type": "timeout"},
    actions: ["check timeout", "increase value", "test"],
    success_rate: 1.0
}
```

**Daily Synopsis:**
```
## Daily Synopsis - 2026-01-31

Processed 20 episodes across 5 conversations.

Key Insights:
1. Error handling pattern identified (3 occurrences)
2. Connection timeout solutions documented
3. Async patterns learned

Stats: 20 episodes, 15 successful, 75% positive
```

---

## Performance Impact

### **Consolidation Time:**
- 20 episodes: ~200ms
- 50 episodes: ~500ms
- 100 episodes: ~2s

### **Server Responsiveness:**
- ✅ **Non-blocking**: Consolidation runs in background
- ✅ **No delays**: Server responds immediately to learn/search
- ✅ **Concurrent**: Multiple consolidations can run if triggered rapidly

### **Memory Usage:**
- Minimal: Consolidation task is lightweight
- No accumulation: Tasks complete and clean up

---

## Monitoring

### **Check Consolidation Status**

Server logs show consolidation activity:
```
📊 20 messages processed, triggering consolidation
✅ Auto-consolidation complete
```

### **Manual Consolidation**

You can still manually consolidate via CLI:
```bash
memory-cli consolidate --date 2026-01-31
```

---

## Benefits

### **Before (Manual Consolidation)**
- ❌ Memories accumulate without structure
- ❌ No pattern extraction
- ❌ Must remember to run consolidation
- ❌ Context gets stale

### **After (Auto-Consolidation)**
- ✅ Automatic pattern extraction every 20 messages
- ✅ Knowledge continuously updated
- ✅ No manual intervention needed
- ✅ Always fresh, relevant context
- ✅ Consolidates yesterday on startup

---

## Example Session

```bash
# Start server
$ cargo run --bin memory-rs-mcp my-project
# Or:
$ ./target/release/memory-rs-mcp my-project

🚀 Memory MCP Server initializing...
🔄 Consolidating memories from 2026-01-30...
✅ Memory MCP Server ready
✅ Consolidation complete: 3 insights extracted

# Client uses server (messages 1-19)
# ... no consolidation yet ...

# Message 20 triggers consolidation
📊 20 messages processed, triggering consolidation
✅ Auto-consolidation complete

# Client continues (messages 21-39)
# ... no consolidation yet ...

# Message 40 triggers consolidation again
📊 20 messages processed, triggering consolidation
✅ Auto-consolidation complete
```

---

## Troubleshooting

### **Consolidation Fails**
```
❌ Auto-consolidation failed: database locked
```
**Solution**: Consolidation will retry on next threshold. Non-fatal.

### **Too Frequent Consolidation**
```
📊 20 messages processed, triggering consolidation
📊 20 messages processed, triggering consolidation  # Too soon!
```
**Solution**: Increase threshold to 50 or 100.

### **Never Consolidates**
**Check**: Are you reaching 20 messages?
**Solution**: Lower threshold or manually consolidate.

---

## Summary

The MCP server now:
1. ✅ **Consolidates yesterday on startup** (background, non-blocking)
2. ✅ **Auto-consolidates every 20 messages** (background, non-blocking)
3. ✅ **Stays responsive** (no delays for clients)
4. ✅ **Requires no manual intervention** (fully automatic)

Just start the server and use it - consolidation happens automatically! 🎉
