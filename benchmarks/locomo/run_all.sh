#!/bin/bash
# Run LoCoMo benchmark on all 10 conversations

echo "🚀 Running LoCoMo Benchmark on All 10 Conversations"
echo "===================================================="
echo ""

for i in {0..9}; do
    echo "📊 Conversation $i..."
    python3 benchmarks/locomo/run_mcp_benchmark.py $i 2>&1 | tail -13
    echo ""
done

echo "✅ Complete!"
