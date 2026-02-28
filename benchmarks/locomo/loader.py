#!/usr/bin/env python3
"""Load LoCoMo conversations into memory-rs via CLI."""

import json
import subprocess
import sys
from pathlib import Path

MEMORY_CLI = Path(__file__).parent.parent.parent / "target/release/agent-memory-cli"
LOCOMO_DATA = Path.home() / "newProject/LoCoMo/data/locomo10.json"

def load_conversation(conv_data, workspace_name):
    """Load a single conversation into memory-rs."""
    sample_id = conv_data['sample_id']
    conversation = conv_data['conversation']
    
    sessions = sorted([k for k in conversation.keys() 
                      if k.startswith('session_') and not k.endswith('_date_time')])
    
    print(f"\nLoading {sample_id}: {len(sessions)} sessions")
    
    memories_loaded = 0
    for session_key in sessions:
        session_num = session_key.replace('session_', '')
        date_key = f'session_{session_num}_date_time'
        timestamp = conversation.get(date_key, '')
        
        turns = conversation[session_key]
        for turn in turns:
            dia_id = turn['dia_id']
            speaker = turn['speaker']
            text = turn['text']
            
            memory_text = f"[{dia_id}] {speaker}: {text}"
            
            cmd = [
                str(MEMORY_CLI),
                'learn',
                '--workspace', workspace_name,
                '--text', memory_text,
                '--tags', f'locomo,{sample_id},session_{session_num},{dia_id}'
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode != 0:
                print(f"  ❌ Failed {dia_id}: {result.stderr}")
                return None
            
            memories_loaded += 1
            if memories_loaded % 50 == 0:
                print(f"  Loaded {memories_loaded} turns...")
    
    print(f"✅ Loaded {memories_loaded} turns from {sample_id}")
    return memories_loaded

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 loader.py <conversation_index>")
        print("  conversation_index: 0-9 (or 'all')")
        sys.exit(1)
    
    if not LOCOMO_DATA.exists():
        print(f"❌ LoCoMo data not found at {LOCOMO_DATA}")
        sys.exit(1)
    
    data = json.load(open(LOCOMO_DATA))
    conv_idx = sys.argv[1]
    
    if conv_idx == 'all':
        total = 0
        for i, conv in enumerate(data):
            workspace = f'locomo-conv-{i}'
            count = load_conversation(conv, workspace)
            if count:
                total += count
        print(f"\n✅ Total: {total} memories")
    else:
        idx = int(conv_idx)
        if idx < 0 or idx >= len(data):
            print(f"❌ Index must be 0-{len(data)-1}")
            sys.exit(1)
        
        workspace = f'locomo-conv-{idx}'
        load_conversation(data[idx], workspace)

if __name__ == '__main__':
    main()
