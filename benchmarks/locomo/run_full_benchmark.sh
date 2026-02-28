#!/bin/bash
# Run full LoCoMo benchmark (all 10 conversations)

set -e

RESULTS_DIR="benchmarks/locomo/results"
mkdir -p "$RESULTS_DIR"

echo "LoCoMo Full Benchmark"
echo "====================="
echo ""
echo "This will:"
echo "  - Load 10 conversations (~4,000 dialog turns)"
echo "  - Evaluate ~1,500 questions"
echo "  - Take approximately 30 minutes"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 0
fi

echo ""
echo "Starting benchmark..."
echo ""

# Run all 10 conversations
for i in {0..9}; do
    echo "[$((i+1))/10] Processing conversation $i..."
    
    # Load conversation
    echo "  Loading..."
    ./benchmarks/locomo/target/release/locomo-loader $i 2>&1 | grep -E "Loading|Loaded|✅"
    
    # Evaluate
    echo "  Evaluating..."
    ./benchmarks/locomo/target/release/locomo-eval $i > "$RESULTS_DIR/conv-$i.txt" 2>&1
    
    # Extract overall score
    SCORE=$(grep "Recall@10:" "$RESULTS_DIR/conv-$i.txt" | tail -1 | awk '{print $2}')
    echo "  Result: Recall@10 = $SCORE"
    echo ""
done

echo ""
echo "Aggregating results..."
echo ""

# Aggregate results
python3 << 'PYTHON'
import re
from pathlib import Path

results_dir = Path("benchmarks/locomo/results")
scores = []
category_scores = {
    'Single-hop': [],
    'Multi-hop': [],
    'Temporal': [],
    'Commonsense': [],
    'Adversarial': []
}

for i in range(10):
    result_file = results_dir / f"conv-{i}.txt"
    if not result_file.exists():
        continue
    
    content = result_file.read_text()
    
    # Extract overall Recall@10
    match = re.search(r'Overall:.*?Recall@10:\s+([\d.]+)', content, re.DOTALL)
    if match:
        scores.append(float(match.group(1)))
    
    # Extract category scores
    for category in category_scores.keys():
        pattern = f'{category}:.*?Recall@10:\\s+([\\d.]+)'
        match = re.search(pattern, content, re.DOTALL)
        if match:
            category_scores[category].append(float(match.group(1)))

# Print results
print("="*60)
print("FINAL RESULTS - Full LoCoMo Benchmark")
print("="*60)
print()
print(f"Conversations tested: {len(scores)}/10")
print(f"Average Recall@10: {sum(scores)/len(scores):.3f} ({sum(scores)/len(scores)*100:.1f}%)")
print()
print("By Category:")
for category, vals in category_scores.items():
    if vals:
        avg = sum(vals)/len(vals)
        print(f"  {category:15s}: {avg:.3f} ({avg*100:.1f}%)")
print()
print("Individual Conversations:")
for i, score in enumerate(scores):
    print(f"  conv-{i}: {score:.3f} ({score*100:.1f}%)")
print()
print("="*60)
print("Comparison:")
print("  memory-rs (retrieval-only): {:.1f}%".format(sum(scores)/len(scores)*100))
print("  Mem0 (with LLM):            68.5%")
print("  Letta (with LLM):           74.0%")
print("  Zep (with LLM):             75.1%")
print("  EverMemOS (with LLM):       80.1%")
print("="*60)

# Save summary
summary_file = results_dir / "summary.txt"
with open(summary_file, 'w') as f:
    f.write(f"LoCoMo Benchmark Results\n")
    f.write(f"========================\n\n")
    f.write(f"Model: BgeSmall (384-dim embeddings)\n")
    f.write(f"Method: Retrieval-only (no LLM)\n")
    f.write(f"Conversations: {len(scores)}/10\n\n")
    f.write(f"Average Recall@10: {sum(scores)/len(scores):.3f}\n\n")
    f.write(f"By Category:\n")
    for category, vals in category_scores.items():
        if vals:
            f.write(f"  {category}: {sum(vals)/len(vals):.3f}\n")
    f.write(f"\nIndividual Results:\n")
    for i, score in enumerate(scores):
        f.write(f"  conv-{i}: {score:.3f}\n")

print(f"\nResults saved to: {results_dir}/")
PYTHON

echo ""
echo "✅ Benchmark complete!"
echo ""
echo "Results saved in: benchmarks/locomo/results/"
echo "  - Individual: conv-0.txt through conv-9.txt"
echo "  - Summary: summary.txt"
