#!/bin/bash
# Test script for agent-memory-mcp dual mode (stdio + HTTP)
set -e

BIN="/home/bestia/agent-memory-rs/target/release/agent-memory-mcp"
PORT=8231
WS="test-dual-mode"

cleanup() {
    kill $SERVER_PID 2>/dev/null || true
    kill $FIFO_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    rm -f /tmp/mcp-test-fifo
}
trap cleanup EXIT

echo "=== Test 1: stdio mode (no MEMORY_HTTP) ==="
RESULT=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | timeout 10 "$BIN" "$WS" 2>/dev/null)
if echo "$RESULT" | grep -q '"protocolVersion"'; then
    echo "✅ stdio works"
else
    echo "❌ stdio failed: $RESULT"
    exit 1
fi

echo ""
echo "=== Test 2: dual mode (stdio + HTTP via MEMORY_HTTP) ==="

# Create a fifo to keep stdin open
mkfifo /tmp/mcp-test-fifo

# Start server: read stdin from fifo, HTTP on PORT
MEMORY_HTTP="127.0.0.1:$PORT" "$BIN" "$WS" < /tmp/mcp-test-fifo 2>/dev/null &
SERVER_PID=$!

# Send initialize handshake through the fifo (non-blocking write)
{
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
    echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
    sleep 60
} > /tmp/mcp-test-fifo &
FIFO_PID=$!

# Wait for HTTP to be ready
for i in $(seq 1 15); do
    if ss -tlnp | grep -q ":$PORT"; then
        break
    fi
    sleep 1
done

if ! ss -tlnp | grep -q ":$PORT"; then
    echo "❌ HTTP server didn't start on port $PORT"
    kill $FIFO_PID 2>/dev/null
    exit 1
fi
echo "✅ HTTP server listening on $PORT"

# Test HTTP initialize
HTTP_RESULT=$(curl -s --max-time 5 -X POST "http://127.0.0.1:$PORT/mcp" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}')

if echo "$HTTP_RESULT" | grep -q '"protocolVersion"'; then
    echo "✅ HTTP initialize works"
else
    echo "❌ HTTP initialize failed: $HTTP_RESULT"
    kill $FIFO_PID 2>/dev/null
    exit 1
fi

kill $FIFO_PID 2>/dev/null

echo ""
echo "=== All tests passed ✅ ==="
