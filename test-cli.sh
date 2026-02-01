#!/bin/bash
# Test script for agent-memory-cli

set -e

CLI="./target/release/agent-memory-cli"
DB="test-memory.db"

echo "🧪 Testing Agent Memory CLI"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Clean up
rm -f $DB
export DATABASE_URL=$DB

echo ""
echo "1️⃣  Testing stats (should show empty workspace)"
$CLI stats --workspace 1

echo ""
echo "2️⃣  Testing query (should find no results)"
$CLI query --workspace 1 "rust programming" --limit 5

echo ""
echo "3️⃣  Testing synopsis (should show no synopsis)"
$CLI synopsis --workspace 1 --date 2026-02-01

echo ""
echo "✅ All CLI commands executed successfully!"
echo ""
echo "Note: Store command requires workspace to be created via MCP server first."
echo "To test full workflow:"
echo "  1. Start MCP server: ./target/release/agent-memory-mcp test-workspace"
echo "  2. Use @memory/learn to store memories"
echo "  3. Use CLI commands to query and manage"
