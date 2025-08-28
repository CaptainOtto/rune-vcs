#!/bin/bash

# Quick Start Script for Rune VCS Remote Operations Testing
# Use this for rapid Sunday testing

set -e

echo "🚀 Rune VCS Remote Operations - Quick Start"
echo "=========================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Test if binary exists
if [ ! -f "./target/release/rune" ]; then
    echo -e "${RED}❌ Binary not found. Building...${NC}"
    cargo build --release
    echo -e "${GREEN}✅ Build complete${NC}"
fi

echo -e "${YELLOW}🔧 Starting two Rune servers for testing...${NC}"

# Start server 1 in background
echo "Starting Server 1 on ports 7421 (API) and 7420 (Shrine)..."
./target/release/rune api --with-shrine --addr 127.0.0.1:7421 --shrine-addr 127.0.0.1:7420 &
SERVER1_PID=$!

# Start server 2 in background  
echo "Starting Server 2 on ports 8421 (API) and 8420 (Shrine)..."
./target/release/rune api --with-shrine --addr 127.0.0.1:8421 --shrine-addr 127.0.0.1:8420 &
SERVER2_PID=$!

# Wait for servers to start
echo "⏳ Waiting for servers to start..."
sleep 5

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up servers...${NC}"
    kill $SERVER1_PID $SERVER2_PID 2>/dev/null || true
    wait $SERVER1_PID $SERVER2_PID 2>/dev/null || true
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set trap for cleanup
trap cleanup EXIT INT

# Test server health
echo -e "${YELLOW}🏥 Testing server health...${NC}"

echo -n "Server 1 health: "
if curl -s -f "http://127.0.0.1:7421/sync/info" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Healthy${NC}"
else
    echo -e "${RED}❌ Not responding${NC}"
    exit 1
fi

echo -n "Server 2 health: "
if curl -s -f "http://127.0.0.1:8421/sync/info" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Healthy${NC}"
else
    echo -e "${RED}❌ Not responding${NC}"
    exit 1
fi

# Test repository sync
echo -e "${YELLOW}🔄 Testing repository sync...${NC}"

# Create test commit data
TEST_COMMIT=$(cat <<EOF
{
    "commits": [
        {
            "hash": "test-$(date +%s)",
            "message": "Test commit from quick start script",
            "author": "test@rune.dev",
            "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
            "parent": null,
            "files": [
                {
                    "path": "quick-test.txt",
                    "operation": "Added",
                    "content_hash": "abc123"
                }
            ]
        }
    ],
    "branch": "main",
    "force": false
}
EOF
)

# Test push to server 1
echo -n "Testing push to Server 1: "
if curl -s -X POST "http://127.0.0.1:7421/sync/push" \
    -H "Content-Type: application/json" \
    -d "$TEST_COMMIT" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Success${NC}"
else
    echo -e "${RED}❌ Failed${NC}"
fi

# Test pull from server 2
echo -n "Testing pull from Server 2: "
if curl -s -X POST "http://127.0.0.1:8421/sync/pull" \
    -H "Content-Type: application/json" \
    -d '{"branch": "main", "since_commit": null}' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Success${NC}"
else
    echo -e "${RED}❌ Failed${NC}"
fi

# Test LFS operations
echo -e "${YELLOW}📦 Testing LFS operations...${NC}"

# Test LFS upload to shrine 1
echo -n "Testing LFS upload to Shrine 1: "
LFS_DATA=$(cat <<EOF
{
    "oid": "test-oid-$(date +%s)",
    "chunk": "chunk1",
    "data": [72, 101, 108, 108, 111]
}
EOF
)

if curl -s -X POST "http://127.0.0.1:7420/lfs/upload" \
    -H "Content-Type: application/json" \
    -d "$LFS_DATA" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Success${NC}"
else
    echo -e "${RED}❌ Failed${NC}"
fi

# Test LFS download from shrine 2
echo -n "Testing LFS download from Shrine 2: "
if curl -s -X POST "http://127.0.0.1:8420/lfs/download" \
    -H "Content-Type: application/json" \
    -d '{"oid": "test-oid", "chunk": "chunk1"}' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Success${NC}"
else
    echo -e "${RED}❌ Failed${NC}"
fi

echo ""
echo -e "${GREEN}🎉 All tests completed!${NC}"
echo ""
echo "📊 Server Information:"
echo "  Server 1 API:    http://127.0.0.1:7421"
echo "  Server 1 Shrine: http://127.0.0.1:7420"
echo "  Server 2 API:    http://127.0.0.1:8421"
echo "  Server 2 Shrine: http://127.0.0.1:8420"
echo ""
echo "🔗 Test Endpoints:"
echo "  curl http://127.0.0.1:7421/sync/info"
echo "  curl http://127.0.0.1:7421/sync/branches"
echo "  curl http://127.0.0.1:7420/locks/list"
echo ""
echo "⏰ Servers will keep running until you press Ctrl+C"
echo "   Use them for manual testing and development"

# Keep script running until interrupted
while true; do
    sleep 1
done
