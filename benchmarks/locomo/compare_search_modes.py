#!/usr/bin/env python3
"""
Compare pure vector search vs hybrid (BM25 + Vector) on LoCoMo benchmark.
Tests only conversation 0 for quick iteration.
"""

import subprocess
import json
import sys
import os

def run_eval_with_mode(conv_idx, mode):
    """Run evaluation with specified search mode."""
    env = os.environ.copy()
    env["LOCOMO_DATA_PATH"] = "/Users/kenneth/newProject/LoCoMo/data/locomo10.json"
    env["SEARCH_MODE"] = mode  # "vector" or "hybrid"
    
    result = subprocess.run(
        [f"./benchmarks/locomo/target/release/locomo-eval", str(conv_idx)],
        capture_output=True,
        text=True,
        env=env
    )
    
    return result.stdout

def parse_results(output):
    """Extract category results from output."""
    import re
    
    categories = {}
    
    # Parse each category
    for cat in ["Single-hop", "Multi-hop", "Temporal", "Commonsense", "Adversarial", "Overall"]:
        pattern = rf"{cat}:\s+Questions:\s+(\d+)\s+.*?Recall@10:\s+([\d.]+)"
        match = re.search(pattern, output, re.DOTALL)
        if match:
            questions = int(match.group(1))
            recall = float(match.group(2))
            categories[cat] = {"questions": questions, "recall": recall}
    
    return categories

def main():
    conv_idx = 0
    
    print("="*80)
    print("HYBRID vs VECTOR SEARCH COMPARISON")
    print("="*80)
    print(f"\nTesting conversation {conv_idx} (conv-26)")
    print()
    
    # Test vector-only
    print("Running VECTOR-ONLY search...")
    vector_output = run_eval_with_mode(conv_idx, "vector")
    vector_results = parse_results(vector_output)
    
    # Test hybrid
    print("Running HYBRID search (BM25 + Vector)...")
    hybrid_output = run_eval_with_mode(conv_idx, "hybrid")
    hybrid_results = parse_results(hybrid_output)
    
    # Compare results
    print("\n" + "="*80)
    print("RESULTS COMPARISON")
    print("="*80)
    print(f"\n{'Category':<15} {'Vector':<12} {'Hybrid':<12} {'Improvement':<12}")
    print("-"*80)
    
    for cat in ["Single-hop", "Multi-hop", "Temporal", "Commonsense", "Adversarial", "Overall"]:
        if cat in vector_results and cat in hybrid_results:
            v_recall = vector_results[cat]["recall"]
            h_recall = hybrid_results[cat]["recall"]
            improvement = h_recall - v_recall
            
            v_pct = f"{v_recall*100:.1f}%"
            h_pct = f"{h_recall*100:.1f}%"
            imp_pct = f"{improvement*100:+.1f}%"
            
            print(f"{cat:<15} {v_pct:<12} {h_pct:<12} {imp_pct:<12}")
    
    print("\n" + "="*80)
    
    # Highlight temporal improvement
    if "Temporal" in vector_results and "Temporal" in hybrid_results:
        v_temp = vector_results["Temporal"]["recall"] * 100
        h_temp = hybrid_results["Temporal"]["recall"] * 100
        improvement = h_temp - v_temp
        
        print(f"\n⭐ TEMPORAL IMPROVEMENT: {v_temp:.1f}% → {h_temp:.1f}% ({improvement:+.1f}%)")
        
        if improvement > 5:
            print("   ✅ Significant improvement! Hybrid search helps temporal questions.")
        elif improvement > 0:
            print("   ✓ Modest improvement. Hybrid search slightly better.")
        else:
            print("   ⚠️ No improvement. Vector search is sufficient.")

if __name__ == "__main__":
    main()
