#!/usr/bin/env python3
"""Quick LoCoMo benchmark with new infrastructure."""

import json
import subprocess
import sys
import time
from pathlib import Path
from collections import defaultdict

CLI = Path(__file__).parent.parent.parent / "target/release/agent-memory-cli"
DATA = Path.home() / "newProject/LoCoMo/data/locomo10.json"

def load_conversation(conv_data, workspace_id):
    """Load conversation using store command."""
    sample_id = conv_data['sample_id']
    conversation = conv_data['conversation']
    
    sessions = sorted([k for k in conversation.keys() 
                      if k.startswith('session_') and not k.endswith('_date_time')])
    
    print(f"\n📥 Loading {sample_id}: {len(sessions)} sessions")
    
    loaded = 0
    for session_key in sessions:
        turns = conversation[session_key]
        for turn in turns:
            dia_id = turn['dia_id']
            speaker = turn['speaker']
            text = turn['text']
            
            context = f"[{dia_id}] {speaker}: {text}"
            
            cmd = [
                str(CLI), 'store',
                '--workspace', str(workspace_id),
                '--type', 'user_input',
                '--context', context
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode != 0:
                print(f"  ❌ Failed {dia_id}")
                continue
            
            loaded += 1
            if loaded % 50 == 0:
                print(f"  {loaded} turns...")
    
    print(f"✅ Loaded {loaded} turns")
    return loaded

def search_memory(workspace_id, query, limit=10):
    """Search and extract dialog IDs."""
    cmd = [str(CLI), 'query', '--workspace', str(workspace_id), query, '--limit', str(limit)]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        return []
    
    dia_ids = []
    for line in result.stdout.split('\n'):
        if '[D' in line:
            start = line.find('[D')
            if start != -1:
                end = line.find(']', start)
                if end != -1:
                    dia_ids.append(line[start+1:end])
    
    return dia_ids

def calculate_metrics(retrieved, evidence):
    """Calculate recall@k and MRR."""
    if not evidence:
        return None, None, None, None, None
    
    r1 = any(e in retrieved[:1] for e in evidence)
    r3 = any(e in retrieved[:3] for e in evidence)
    r5 = any(e in retrieved[:5] for e in evidence)
    r10 = any(e in retrieved[:10] for e in evidence)
    
    mrr = 0.0
    for rank, dia_id in enumerate(retrieved, 1):
        if dia_id in evidence:
            mrr = 1.0 / rank
            break
    
    return r1, r3, r5, r10, mrr

def evaluate_conversation(conv_data, workspace_id):
    """Evaluate retrieval."""
    qa_pairs = conv_data['qa']
    
    print(f"\n🔍 Evaluating {len(qa_pairs)} questions")
    
    results = defaultdict(lambda: {'count': 0, 'r1': [], 'r3': [], 'r5': [], 'r10': [], 'mrr': []})
    
    for qa in qa_pairs:
        question = qa['question']
        category = qa['category']
        evidence = qa.get('evidence', [])
        
        if not evidence:
            continue
        
        retrieved = search_memory(workspace_id, question, 10)
        r1, r3, r5, r10, mrr = calculate_metrics(retrieved, evidence)
        
        if r1 is not None:
            results[category]['count'] += 1
            results[category]['r1'].append(r1)
            results[category]['r3'].append(r3)
            results[category]['r5'].append(r5)
            results[category]['r10'].append(r10)
            results[category]['mrr'].append(mrr)
    
    return results

def print_results(results):
    """Print formatted results."""
    print(f"\n{'='*70}")
    print(f"{'Category':<20} {'Count':>8} {'R@1':>8} {'R@3':>8} {'R@5':>8} {'R@10':>8} {'MRR':>8}")
    print(f"{'='*70}")
    
    all_r1, all_r3, all_r5, all_r10, all_mrr = [], [], [], [], []
    
    for cat in sorted(results.keys()):
        r = results[cat]
        if r['count'] == 0:
            continue
        
        avg_r1 = sum(r['r1']) / len(r['r1']) * 100
        avg_r3 = sum(r['r3']) / len(r['r3']) * 100
        avg_r5 = sum(r['r5']) / len(r['r5']) * 100
        avg_r10 = sum(r['r10']) / len(r['r10']) * 100
        avg_mrr = sum(r['mrr']) / len(r['mrr'])
        
        print(f"{cat:<20} {r['count']:>8} {avg_r1:>7.1f}% {avg_r3:>7.1f}% {avg_r5:>7.1f}% {avg_r10:>7.1f}% {avg_mrr:>8.3f}")
        
        all_r1.extend(r['r1'])
        all_r3.extend(r['r3'])
        all_r5.extend(r['r5'])
        all_r10.extend(r['r10'])
        all_mrr.extend(r['mrr'])
    
    print(f"{'='*70}")
    if all_r1:
        print(f"{'OVERALL':<20} {len(all_r1):>8} {sum(all_r1)/len(all_r1)*100:>7.1f}% {sum(all_r3)/len(all_r3)*100:>7.1f}% {sum(all_r5)/len(all_r5)*100:>7.1f}% {sum(all_r10)/len(all_r10)*100:>7.1f}% {sum(all_mrr)/len(all_mrr):>8.3f}")
    print(f"{'='*70}\n")

def main():
    if not DATA.exists():
        print(f"❌ Data not found: {DATA}")
        sys.exit(1)
    
    data = json.load(open(DATA))
    
    # Use first conversation for quick benchmark
    conv_idx = 0
    conv = data[conv_idx]
    workspace_id = 1
    
    print(f"\n🚀 LoCoMo Benchmark - Conversation {conv_idx}")
    print(f"Sample: {conv['sample_id']}")
    
    # Create workspace
    subprocess.run([str(CLI), 'workspace', 'create', '--name', 'locomo-test', '--path', '/tmp/locomo'], 
                   capture_output=True)
    
    # Load
    start = time.time()
    count = load_conversation(conv, workspace_id)
    load_time = time.time() - start
    print(f"⏱️  Load time: {load_time:.1f}s ({count/load_time:.1f} turns/sec)")
    
    # Evaluate
    start = time.time()
    results = evaluate_conversation(conv, workspace_id)
    eval_time = time.time() - start
    print(f"⏱️  Eval time: {eval_time:.1f}s")
    
    # Results
    print_results(results)

if __name__ == '__main__':
    main()
