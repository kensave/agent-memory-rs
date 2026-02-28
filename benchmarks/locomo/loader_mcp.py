#!/usr/bin/env python3
"""Load LoCoMo conversations into memory-rs via MCP server."""

import json
import sys
from pathlib import Path

# Use the MCP tools directly via the running server
try:
    from mcp import ClientSession, StdioServerParameters
    from mcp.client.stdio import stdio_client
except ImportError:
    print("❌ MCP client not installed. Install with: pip install mcp")
    sys.exit(1)

LOCOMO_DATA = Path.home() / "newProject/LoCoMo/data/locomo10.json"

async def load_conversation_mcp(conv_data, workspace_name):
    """Load conversation using MCP server."""
    sample_id = conv_data['sample_id']
    conversation = conv_data['conversation']
    
    sessions = sorted([k for k in conversation.keys() 
                      if k.startswith('session_') and not k.endswith('_date_time')])
    
    print(f"\nLoading {sample_id}: {len(sessions)} sessions")
    
    # Connect to MCP server
    server_params = StdioServerParameters(
        command="<project-root>/target/release/agent-memory-mcp",
        args=[workspace_name]
    )
    
    memories_loaded = 0
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            
            for session_key in sessions:
                session_num = session_key.replace('session_', '')
                turns = conversation[session_key]
                
                for turn in turns:
                    dia_id = turn['dia_id']
                    speaker = turn['speaker']
                    text = turn['text']
                    
                    memory_text = f"[{dia_id}] {speaker}: {text}"
                    
                    # Call learn tool
                    result = await session.call_tool(
                        "learn",
                        arguments={
                            "text": memory_text,
                            "tags": f"locomo,{sample_id},session_{session_num},{dia_id}"
                        }
                    )
                    
                    memories_loaded += 1
                    if memories_loaded % 50 == 0:
                        print(f"  Loaded {memories_loaded} turns...")
    
    print(f"✅ Loaded {memories_loaded} turns from {sample_id}")
    return memories_loaded

def main():
    import asyncio
    
    if len(sys.argv) < 2:
        print("Usage: python3 loader_mcp.py <conversation_index>")
        print("  conversation_index: 0-9")
        sys.exit(1)
    
    data = json.load(open(LOCOMO_DATA))
    idx = int(sys.argv[1])
    
    if idx < 0 or idx >= len(data):
        print(f"❌ Index must be 0-{len(data)-1}")
        sys.exit(1)
    
    workspace = f"locomo-conv-{idx}"
    asyncio.run(load_conversation_mcp(data[idx], workspace))

if __name__ == '__main__':
    main()
