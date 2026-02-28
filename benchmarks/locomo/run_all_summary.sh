#!/bin/bash
# Run LoCoMo benchmark on all 10 conversations and aggregate results

echo "🚀 Running LoCoMo Benchmark on All 10 Conversations"
echo "===================================================="

declare -a r10_scores

for i in {0..9}; do
    echo "📊 Conversation $i..."
    output=$(python3 benchmarks/locomo/run_mcp_benchmark.py $i 2>&1)
    
    # Extract R@10 from OVERALL line
    r10=$(echo "$output" | grep "^OVERALL" | awk '{print $6}' | tr -d '%')
    r10_scores+=($r10)
    
    echo "   R@10: ${r10}%"
done

echo ""
echo "=" | head -c 70
echo ""
echo "Summary:"
echo "=" | head -c 70
echo ""

for i in {0..9}; do
    printf "  Conv %d: %5.1f%%\n" $i ${r10_scores[$i]}
done

# Calculate average
total=0
for score in "${r10_scores[@]}"; do
    total=$(echo "$total + $score" | bc)
done
avg=$(echo "scale=1; $total / 10" | bc)

echo "=" | head -c 70
echo ""
printf "  Average: %5.1f%%\n" $avg
echo "=" | head -c 70
echo ""
