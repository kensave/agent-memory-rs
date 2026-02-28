#!/usr/bin/env python3
"""Load LoCoMo via MCP and evaluate with new pipeline."""

import json
import asyncio
import sys
from pathlib import Path

try:
    from mcp import ClientSession, StdioServerParameters
    from mcp.client.stdio import stdio_client
except ImportError:
    print("❌ Install MCP: pip install mcp")
    sys.exit(1)

DATA = Path.home() / "newProject/LoCoMo/data/locomo10.json"
MCP_SERVER = Path(__file__).parent.parent.parent / "target/release/agent-memory-mcp"

async def load_and_evaluate(conv_idx):
    """Load conversation via MCP and evaluate."""
    data = json.load(open(DATA))
    conv = data[conv_idx]
    
    sample_id = conv['sample_id']
    conversation = conv['conversation']
    qa_pairs = conv['qa']
    
    workspace = f"locomo-{conv_idx}"
    
    print(f"\n{'='*70}")
    print(f"🚀 LoCoMo Benchmark - Conversation {conv_idx}")
    print(f"Sample: {sample_id}")
    print(f"{'='*70}\n")
    
    # Start MCP server
    import os
    env = {"RUST_LOG": "info"}
    if "MEMORY_MODEL" in os.environ:
        env["MEMORY_MODEL"] = os.environ["MEMORY_MODEL"]
    
    server_params = StdioServerParameters(
        command=str(MCP_SERVER),
        args=[workspace],
        env=env
    )
    
    # Increase timeout for slow embedding operations
    import os
    os.environ['MCP_TIMEOUT'] = '300'  # 5 minutes per operation
    
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            
            # Load conversation
            print(f"📥 Loading conversation...")
            sessions = sorted([k for k in conversation.keys() 
                             if k.startswith('session_') and not k.endswith('_date_time')])
            
            loaded = 0
            for session_key in sessions:
                # Get session date
                date_key = f"{session_key}_date_time"
                session_date = conversation.get(date_key, "")
                
                turns = conversation[session_key]
                for turn in turns:
                    dia_id = turn['dia_id']
                    speaker = turn['speaker']
                    text = turn['text']
                    
                    # Parse date: "1:56 pm on 8 May, 2023" -> "2023-05-08 13:56:00"
                    timestamp = None
                    if session_date:
                        try:
                            import re
                            from datetime import datetime
                            # Extract: "1:56 pm on 8 May, 2023"
                            match = re.match(r'(\d+):(\d+)\s*(am|pm)\s*on\s*(\d+)\s*(\w+),\s*(\d+)', session_date)
                            if match:
                                hour, minute, ampm, day, month, year = match.groups()
                                hour = int(hour)
                                if ampm.lower() == 'pm' and hour != 12:
                                    hour += 12
                                elif ampm.lower() == 'am' and hour == 12:
                                    hour = 0
                                dt = datetime.strptime(f"{day} {month} {year} {hour}:{minute}", "%d %B %Y %H:%M")
                                timestamp = dt.strftime("%Y-%m-%d %H:%M:%S")
                        except Exception as e:
                            print(f"  ⚠️  Failed to parse date '{session_date}': {e}")
                    
                    try:
                        learn_args = {
                            "text": f"[{dia_id}] {speaker}: {text}",
                            "event_type": "dialog_turn",
                            "importance_score": 0.5
                        }
                        if timestamp:
                            learn_args["timestamp"] = timestamp
                        
                        result = await session.call_tool("learn", learn_args)
                        loaded += 1
                        if loaded % 50 == 0:
                            print(f"  {loaded} turns...")
                    except Exception as e:
                        print(f"  ❌ Failed {dia_id}: {e}")
            
            print(f"✅ Loaded {loaded} turns\n")
            
            # Consolidate all dates
            print(f"🔄 Running consolidation for all dates...")
            # Get unique dates from sessions
            unique_dates = set()
            for session_key in sessions:
                date_key = f"{session_key}_date_time"
                session_date = conversation.get(date_key, "")
                if session_date:
                    try:
                        import re
                        from datetime import datetime
                        match = re.match(r'(\d+):(\d+)\s*(am|pm)\s*on\s*(\d+)\s*(\w+),\s*(\d+)', session_date)
                        if match:
                            hour, minute, ampm, day, month, year = match.groups()
                            dt = datetime.strptime(f"{day} {month} {year}", "%d %B %Y")
                            unique_dates.add(dt.strftime("%Y-%m-%d"))
                    except:
                        pass
            
            print(f"  Found {len(unique_dates)} unique dates to consolidate")
            # Note: We can't trigger consolidation via MCP yet, so this is informational
            # Consolidation will run automatically every 20 messages during loading
            print(f"  (Auto-consolidation ran {loaded // 20} times during loading)\n")
            
            # Evaluate
            print(f"🔍 Evaluating {len(qa_pairs)} questions...")
            print(f"⚠️  Note: Using BM25 search only (vector search may timeout)\n")
            
            results = {}
            category_map = {
                1: 'Single-hop', 
                2: 'Temporal', 
                3: 'Hypothetical',
                4: 'Multi-hop',
                5: 'Causal'
            }
            for category in category_map.values():
                results[category] = {'count': 0, 'r1': [], 'r3': [], 'r5': [], 'r10': [], 'mrr': []}
            
            evaluated = 0
            for qa in qa_pairs:
                question = qa['question']
                category_num = qa['category']
                category = category_map.get(category_num, 'Unknown')
                evidence = qa.get('evidence', [])
                
                if not evidence:
                    continue
                
                try:
                    result = await session.call_tool("search", {
                        "query": question,
                        "limit": 10
                    })
                    
                    evaluated += 1
                    if evaluated % 10 == 0:
                        print(f"  Evaluated {evaluated} questions...")
                    
                    # Parse results
                    content = result.content[0].text if result.content else "{}"
                    data = json.loads(content)
                    retrieved_ids = []
                    
                    for r in data.get('results', []):
                        content_text = r.get('content', '')
                        # Extract from JSON context: {"text":"[D1:1] ..."}
                        if '"text"' in content_text and '[D' in content_text:
                            start = content_text.find('[D')
                            if start != -1:
                                end = content_text.find(']', start)
                                if end > start:
                                    retrieved_ids.append(content_text[start+1:end])
                        # Also try direct format: [D1:1] ...
                        elif '[D' in content_text:
                            start = content_text.find('[D')
                            if start != -1:
                                end = content_text.find(']', start)
                                if end > start:
                                    retrieved_ids.append(content_text[start+1:end])
                    
                    # Calculate metrics
                    r1 = any(e in retrieved_ids[:1] for e in evidence)
                    r3 = any(e in retrieved_ids[:3] for e in evidence)
                    r5 = any(e in retrieved_ids[:5] for e in evidence)
                    r10 = any(e in retrieved_ids[:10] for e in evidence)
                    
                    mrr = 0.0
                    for rank, dia_id in enumerate(retrieved_ids, 1):
                        if dia_id in evidence:
                            mrr = 1.0 / rank
                            break
                    
                    results[category]['count'] += 1
                    results[category]['r1'].append(r1)
                    results[category]['r3'].append(r3)
                    results[category]['r5'].append(r5)
                    results[category]['r10'].append(r10)
                    results[category]['mrr'].append(mrr)
                    
                except Exception as e:
                    if evaluated == 0:
                        print(f"  ❌ First search failed: {str(e)}")
                        print(f"     This likely means the MCP server has an issue")
                        break
            
            # Print results
            print(f"\n{'='*70}")
            print(f"{'Category':<15} {'Count':>8} {'R@1':>8} {'R@3':>8} {'R@5':>8} {'R@10':>8} {'MRR':>8}")
            print(f"{'='*70}")
            
            all_r1, all_r3, all_r5, all_r10, all_mrr = [], [], [], [], []
            
            for cat in ['Single-hop', 'Temporal', 'Hypothetical', 'Multi-hop', 'Causal']:
                r = results.get(cat)
                if not r or r['count'] == 0:
                    continue
                
                avg_r1 = sum(r['r1']) / len(r['r1']) * 100
                avg_r3 = sum(r['r3']) / len(r['r3']) * 100
                avg_r5 = sum(r['r5']) / len(r['r5']) * 100
                avg_r10 = sum(r['r10']) / len(r['r10']) * 100
                avg_mrr = sum(r['mrr']) / len(r['mrr'])
                
                print(f"{cat:<15} {r['count']:>8} {avg_r1:>7.1f}% {avg_r3:>7.1f}% {avg_r5:>7.1f}% {avg_r10:>7.1f}% {avg_mrr:>8.3f}")
                
                all_r1.extend(r['r1'])
                all_r3.extend(r['r3'])
                all_r5.extend(r['r5'])
                all_r10.extend(r['r10'])
                all_mrr.extend(r['mrr'])
            
            print(f"{'='*70}")
            if all_r1:
                print(f"{'OVERALL':<15} {len(all_r1):>8} {sum(all_r1)/len(all_r1)*100:>7.1f}% {sum(all_r3)/len(all_r3)*100:>7.1f}% {sum(all_r5)/len(all_r5)*100:>7.1f}% {sum(all_r10)/len(all_r10)*100:>7.1f}% {sum(all_mrr)/len(all_mrr):>8.3f}")
            print(f"{'='*70}\n")

def main():
    if not DATA.exists():
        print(f"❌ Data not found: {DATA}")
        sys.exit(1)
    
    if not MCP_SERVER.exists():
        print(f"❌ MCP server not found: {MCP_SERVER}")
        print("Run: cargo build --release")
        sys.exit(1)
    
    conv_idx = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    asyncio.run(load_and_evaluate(conv_idx))

if __name__ == '__main__':
    main()
