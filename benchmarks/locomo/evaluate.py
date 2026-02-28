#!/usr/bin/env python3
"""Evaluate memory-rs retrieval on LoCoMo QA benchmark."""

import json
import subprocess
import sys
from pathlib import Path
from collections import defaultdict

MEMORY_CLI = Path(__file__).parent.parent.parent / "target/release/agent-memory-cli"
LOCOMO_DATA = Path.home() / "newProject/LoCoMo/data/locomo10.json"

def search_memory(workspace, query, limit=10):
    """Search memory and return dialog IDs from results."""
    cmd = [
        str(MEMORY_CLI),
        'query',
        '--workspace', workspace,
        query,
        '--limit', str(limit)
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        return []
    
    # Extract dialog IDs from results
    dia_ids = []
    for line in result.stdout.split('\n'):
        if '[D' in line:
            # Extract [D1:3] format
            start = line.find('[D')
            if start != -1:
                end = line.find(']', start)
                if end != -1:
                    dia_id = line[start+1:end]
                    dia_ids.append(dia_id)
    
    return dia_ids

def calculate_recall_at_k(retrieved, evidence, k):
    """Check if any evidence appears in top-k retrieved."""
    if not evidence:
        return None
    top_k = retrieved[:k]
    return any(e in top_k for e in evidence)

def calculate_mrr(retrieved, evidence):
    """Calculate Mean Reciprocal Rank."""
    if not evidence:
        return None
    
    for rank, dia_id in enumerate(retrieved, 1):
        if dia_id in evidence:
            return 1.0 / rank
    return 0.0

def evaluate_conversation(conv_data, conv_idx):
    """Evaluate retrieval for one conversation."""
    sample_id = conv_data['sample_id']
    workspace = f'locomo-conv-{conv_idx}'
    qa_pairs = conv_data['qa']
    
    print(f"\n{'='*60}")
    print(f"Evaluating {sample_id} ({len(qa_pairs)} questions)")
    print(f"{'='*60}")
    
    results = {
        'sample_id': sample_id,
        'total_questions': len(qa_pairs),
        'by_category': defaultdict(lambda: {
            'count': 0,
            'recall@1': [],
            'recall@3': [],
            'recall@5': [],
            'recall@10': [],
            'mrr': []
        })
    }
    
    for i, qa in enumerate(qa_pairs):
        question = qa['question']
        category = qa['category']
        evidence = qa.get('evidence', [])
        
        if not evidence:
            continue
        
        # Search memory
        retrieved = search_memory(workspace, question, limit=10)
        
        # Calculate metrics
        r1 = calculate_recall_at_k(retrieved, evidence, 1)
        r3 = calculate_recall_at_k(retrieved, evidence, 3)
        r5 = calculate_recall_at_k(retrieved, evidence, 5)
        r10 = calculate_recall_at_k(retrieved, evidence, 10)
        mrr = calculate_mrr(retrieved, evidence)
        
        cat_results = results['by_category'][category]
        cat_results['count'] += 1
        if r1 is not None: cat_results['recall@1'].append(r1)
        if r3 is not None: cat_results['recall@3'].append(r3)
        if r5 is not None: cat_results['recall@5'].append(r5)
        if r10 is not None: cat_results['recall@10'].append(r10)
        if mrr is not None: cat_results['mrr'].append(mrr)
        
        if (i + 1) % 20 == 0:
            print(f"  Processed {i+1}/{len(qa_pairs)} questions...")
    
    return results

def print_results(all_results):
    """Print aggregated results."""
    print(f"\n{'='*60}")
    print("FINAL RESULTS")
    print(f"{'='*60}")
    
    # Category names
    cat_names = {
        '1': 'Single-hop',
        '2': 'Multi-hop',
        '3': 'Temporal',
        '4': 'Commonsense',
        '5': 'Adversarial'
    }
    
    # Aggregate across all conversations
    aggregated = defaultdict(lambda: {
        'recall@1': [],
        'recall@3': [],
        'recall@5': [],
        'recall@10': [],
        'mrr': []
    })
    
    for result in all_results:
        for cat, metrics in result['by_category'].items():
            aggregated[cat]['recall@1'].extend(metrics['recall@1'])
            aggregated[cat]['recall@3'].extend(metrics['recall@3'])
            aggregated[cat]['recall@5'].extend(metrics['recall@5'])
            aggregated[cat]['recall@10'].extend(metrics['recall@10'])
            aggregated[cat]['mrr'].extend(metrics['mrr'])
    
    # Print by category
    for cat in sorted(aggregated.keys()):
        metrics = aggregated[cat]
        cat_name = cat_names.get(cat, f'Category {cat}')
        
        print(f"\n{cat_name}:")
        print(f"  Questions: {len(metrics['recall@1'])}")
        if metrics['recall@1']:
            print(f"  Recall@1:  {sum(metrics['recall@1'])/len(metrics['recall@1']):.3f}")
            print(f"  Recall@3:  {sum(metrics['recall@3'])/len(metrics['recall@3']):.3f}")
            print(f"  Recall@5:  {sum(metrics['recall@5'])/len(metrics['recall@5']):.3f}")
            print(f"  Recall@10: {sum(metrics['recall@10'])/len(metrics['recall@10']):.3f}")
            print(f"  MRR:       {sum(metrics['mrr'])/len(metrics['mrr']):.3f}")
    
    # Overall
    all_r1 = [v for m in aggregated.values() for v in m['recall@1']]
    all_r3 = [v for m in aggregated.values() for v in m['recall@3']]
    all_r5 = [v for m in aggregated.values() for v in m['recall@5']]
    all_r10 = [v for m in aggregated.values() for v in m['recall@10']]
    all_mrr = [v for m in aggregated.values() for v in m['mrr']]
    
    print(f"\nOverall:")
    print(f"  Questions: {len(all_r1)}")
    print(f"  Recall@1:  {sum(all_r1)/len(all_r1):.3f}")
    print(f"  Recall@3:  {sum(all_r3)/len(all_r3):.3f}")
    print(f"  Recall@5:  {sum(all_r5)/len(all_r5):.3f}")
    print(f"  Recall@10: {sum(all_r10)/len(all_r10):.3f}")
    print(f"  MRR:       {sum(all_mrr)/len(all_mrr):.3f}")

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 evaluate.py <conversation_index>")
        print("  conversation_index: 0-9 (or 'all')")
        sys.exit(1)
    
    data = json.load(open(LOCOMO_DATA))
    conv_idx = sys.argv[1]
    
    if conv_idx == 'all':
        all_results = []
        for i, conv in enumerate(data):
            result = evaluate_conversation(conv, i)
            all_results.append(result)
        print_results(all_results)
    else:
        idx = int(conv_idx)
        result = evaluate_conversation(data[idx], idx)
        print_results([result])

if __name__ == '__main__':
    main()
