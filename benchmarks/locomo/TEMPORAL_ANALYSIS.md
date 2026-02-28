# Temporal Question Analysis & Improvement Opportunities

## Current Performance
- **Temporal (Category 3): 44.6% R@10** (weakest category)
- **Multi-hop: 75.4% R@10** (strongest category)
- **Gap: 30.8 percentage points**

## What Are "Temporal" Questions?

Despite the name, these are **inference/prediction questions**, not time-based queries:

### Examples:
1. "Would Caroline pursue writing as a career option?"
2. "What fields would Caroline be likely to pursue in her education?"
3. "Would Melanie be considered an ally to the transgender community?"
4. "What personality traits might Melanie say Caroline has?"

### Pattern Analysis:
- 92% contain "would" (hypothetical reasoning)
- 31% contain "likely" (probability estimation)
- 8% contain "if" (counterfactual reasoning)

### What They Require:
1. **Character understanding** - Find all mentions of the person
2. **Trait inference** - Understand personality, values, preferences
3. **Reasoning** - Predict behavior based on past context

## Current Benchmark Setup

**CRITICAL FINDING:** The benchmark uses **pure vector search**, NOT hybrid retrieval!

```rust
// Current: MemorySystem::search()
memory_system.search(question, &filters, 10)
  └─> store.search_similar(&query_embedding, filters, limit)  // Vector only!
```

The `HybridRetrievalEngine` exists but is **not being used** in the benchmark.

## Low-Hanging Fruit Improvements

### 1. Enable Hybrid Search (BM25 + Vector) ⭐ HIGHEST IMPACT

**Why it helps:**
- BM25 finds exact name matches: "Caroline" → all Caroline mentions
- Vector search finds semantic similarity: "career" → "job", "profession"
- Fusion combines both strengths

**Implementation:**
```rust
// Replace MemorySystem::search() with HybridRetrievalEngine
let hybrid_engine = HybridRetrievalEngine::with_embedder(db, embedder);
let results = hybrid_engine.hybrid_search(question, workspace_id, 10)?;
```

**Expected improvement:** +5-10% on temporal questions

### 2. Add Character/Speaker Filtering

**Why it helps:**
- Questions ask about specific people: "Would Caroline..."
- Filter memories to only those mentioning that person
- Reduces noise from irrelevant context

**Implementation:**
```rust
// Extract person name from question
let person = extract_person_name(question); // "Caroline", "Melanie"

// Add to search filters
filters.tags = Some(format!("speaker:{}", person));
```

**Expected improvement:** +3-5% on temporal questions

### 3. Increase Retrieval Limit (k=10 → k=20)

**Why it helps:**
- Temporal questions need more context to infer traits
- More examples = better understanding of character
- LLM can reason over larger context window

**Implementation:**
```rust
// Retrieve more results for temporal questions
let limit = if is_temporal_question(question) { 20 } else { 10 };
let results = search(question, filters, limit)?;
```

**Expected improvement:** +2-4% on temporal questions

### 4. Add Importance Weighting for Character Mentions

**Why it helps:**
- Direct quotes reveal personality better than descriptions
- First-person statements are more reliable
- Weight results by how directly they reveal traits

**Implementation:**
```rust
// Boost scores for direct character mentions
for result in &mut results {
    if result.text.contains(&person_name) {
        result.score *= 1.2;
    }
}
```

**Expected improvement:** +1-3% on temporal questions

### 5. Add Temporal Context (Conversation Flow)

**Why it helps:**
- Character development happens over time
- Recent mentions may be more relevant
- Conversation structure provides context

**Implementation:**
```rust
// Store session/turn numbers as metadata
memory.metadata = json!({
    "session": session_num,
    "turn": turn_num,
    "speaker": speaker_name
});

// Retrieve nearby turns for context
let context_window = get_surrounding_turns(result.turn_num, window=3);
```

**Expected improvement:** +2-4% on temporal questions

## Comparison: Vector vs Hybrid Search

### Test Plan:
1. Run benchmark with current vector-only search (baseline: 44.6%)
2. Run benchmark with hybrid search (BM25 + Vector)
3. Compare temporal question performance

### Hypothesis:
- **Vector-only:** 44.6% R@10 (current)
- **Hybrid (BM25+Vector):** 50-55% R@10 (predicted)
- **Hybrid + Filtering:** 55-60% R@10 (predicted)

## Implementation Priority

1. **HIGH:** Enable hybrid search (5-10% gain, 1 hour work)
2. **MEDIUM:** Add character filtering (3-5% gain, 2 hours work)
3. **MEDIUM:** Increase retrieval limit (2-4% gain, 30 min work)
4. **LOW:** Importance weighting (1-3% gain, 1 hour work)
5. **LOW:** Temporal context (2-4% gain, 3 hours work)

**Total potential improvement:** 13-26 percentage points
**Target:** 44.6% → 58-70% R@10 on temporal questions

## Next Steps

1. Modify benchmark to use `HybridRetrievalEngine` instead of `MemorySystem::search()`
2. Run comparison: vector-only vs hybrid
3. Implement character filtering if hybrid shows improvement
4. Document results and update benchmark methodology

## Why This Matters

Temporal questions test **inference and reasoning**, which is critical for AI agents:
- Understanding user preferences
- Predicting user needs
- Making recommendations
- Personalizing responses

Improving temporal performance makes the memory system more useful for real-world agent applications.
