# LoCoMo Benchmark for memory-rs

Retrieval-only evaluation of memory-rs on the LoCoMo (Long Conversation Memory) benchmark.

## What it tests

- **Long-term conversational memory**: 10 conversations with ~300 turns each (9K tokens avg)
- **Question types**: Single-hop, Multi-hop, Temporal, Commonsense, Adversarial
- **Metrics**: Recall@1/3/5/10 and MRR (Mean Reciprocal Rank)

## What this benchmark does NOT test

- ❌ Daily synopsis generation (consolidation pipeline)
- ❌ Memory decay and archival
- ❌ Episodic → Semantic memory conversion
- ❌ LLM answer generation

This is a **pure retrieval test**: Can the system find relevant dialog turns when given a question?

## Prerequisites

1. Rust toolchain installed
2. Memory-rs repository cloned

## Setup

1. Clone LoCoMo dataset:
```bash
# Clone to your home directory (default location)
cd ~
git clone https://github.com/snap-research/LoCoMo.git

# Or clone anywhere and set LOCOMO_DATA_PATH
git clone https://github.com/snap-research/LoCoMo.git /path/to/LoCoMo
export LOCOMO_DATA_PATH=/path/to/LoCoMo/data/locomo10.json
```

2. Build benchmark tools from memory-rs root:
```bash
cd /path/to/memory-rs
cargo build --release --manifest-path benchmarks/locomo/Cargo.toml
```

## Usage

All commands run from memory-rs root directory.

### Load a conversation into memory

```bash
./benchmarks/locomo/target/release/locomo-loader <conversation_index>
```

Example:
```bash
./benchmarks/locomo/target/release/locomo-loader 0  # Load conv-26 (419 turns)
```

This will:
- Create workspace `locomo-conv-0`
- Load BgeSmall model (384-dim embeddings)
- Store each dialog turn as a memory with tags

### Evaluate retrieval performance

```bash
./benchmarks/locomo/target/release/locomo-eval <conversation_index>
```

Example:
```bash
./benchmarks/locomo/target/release/locomo-eval 0  # Evaluate conv-26
```

This will:
- Load the workspace
- For each question, search memory (top-10 results)
- Check if evidence dialog IDs appear in results
- Calculate Recall@K and MRR metrics

### Evaluate with LLM (using Kiro - no token costs!)

```bash
./benchmarks/locomo/eval_with_kiro.sh <conversation_index>
```

Example:
```bash
./benchmarks/locomo/eval_with_kiro.sh 0  # Evaluate conv-26 with LLM
```

This will:
- Create a temporary Kiro agent with only memory MCP server
- For each question, use Kiro to search memory and generate answer
- Compare generated answer with ground truth
- Calculate accuracy (expected: 70-75% vs 68% retrieval-only)

**Note:** This uses Kiro's sub-agent feature for free LLM inference. No API costs!

### Run full benchmark (all 10 conversations)

```bash
for i in {0..9}; do
  echo "Loading conversation $i..."
  ./benchmarks/locomo/target/release/locomo-loader $i
  echo "Evaluating conversation $i..."
  ./benchmarks/locomo/target/release/locomo-eval $i
done
```

## Results (conv-26)

Using BgeSmall model (384-dim embeddings):

```
Overall:
  Questions: 197
  Recall@1:  0.284 (28.4%)
  Recall@3:  0.492 (49.2%)
  Recall@5:  0.599 (59.9%)
  Recall@10: 0.680 (68.0%)  ⭐
  MRR:       0.416

By Category:
  Multi-hop:    89.2% @ Recall@10 (best)
  Single-hop:   62.5% @ Recall@10
  Temporal:     63.6% @ Recall@10
  Commonsense:  67.1% @ Recall@10
  Adversarial:  57.4% @ Recall@10 (weakest)
```

## Comparison with Other Systems

| System | Score | Notes |
|--------|-------|-------|
| EverMemOS | 80.1% | SOTA, uses LLM |
| Zep | 75.1% | Commercial tool |
| Letta | 74.0% | GPT-4o-mini + filesystem |
| **memory-rs** | **68.0%** | **Retrieval-only, $0 cost** |
| Mem0 | 68.5% | Knowledge graph |

**Key difference:** Other systems use LLMs to generate answers. memory-rs only tests retrieval (finding context), making it harder to compare directly.

## Interpretation

- **Recall@10 = 68%**: Memory system finds relevant context in top-10 results 68% of the time
- **Strong at multi-hop reasoning (89%)**: Excellent at connecting related facts across conversation
- **Weaker at adversarial questions (57%)**: Trick questions that require careful reasoning are harder
- **Zero cost**: No LLM API calls, runs entirely locally

## Cleanup

Delete benchmark workspaces:
```bash
rm -rf ~/.memory-rs/workspaces/locomo-conv-*
```

## Technical Details

### What gets stored

Each dialog turn becomes a memory:
```
Text: "[D1:3] Caroline: Hey Mel! Good to see you! How have you been?"
Tags: "locomo,conv-26,session_1,D1:3"
Embedding: 384-dim vector (BgeSmall)
```

### What gets tested

For each question:
1. Search memory with question text
2. Extract dialog IDs from top-10 results
3. Check if evidence IDs appear in results
4. Calculate metrics

### Why no consolidation?

LoCoMo tests **retrieval from raw conversation history**, not consolidated knowledge. The consolidation pipeline (daily synopsis, decay, episodic→semantic) is designed for long-running agents, not static conversation datasets.

To test consolidation, you'd need a different benchmark that:
- Simulates multi-day agent interactions
- Tests knowledge extraction over time
- Measures memory efficiency (tokens saved)

## Future Work

- [ ] Add LLM answer generation (would likely reach 70-75%)
- [ ] Test with larger models (Nomic 768-dim)
- [ ] Run on all 10 conversations
- [ ] Compare with/without consolidation pipeline
- [ ] Benchmark query latency
