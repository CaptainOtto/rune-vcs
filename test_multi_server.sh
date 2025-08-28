#!/bin/bash

# Rune VCS Multi-Server Test Script
# Tests remote operations and Docker deployment

set -e

echo "🚀 Starting Rune VCS Multi-Server Test"
echo "======================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
SERVER1_API="http://localhost:7421"
SERVER2_API="http://localhost:8421"
SERVER1_SHRINE="http://localhost:7420"
SERVER2_SHRINE="http://localhost:8420"

# Function to check server health
check_server_health() {
    local server_url=$1
    local server_name=$2
    
    echo -n "Checking $server_name health... "
    if curl -s -f "$server_url/sync/info" > /dev/null; then
        echo -e "${GREEN}✓ Healthy${NC}"
        return 0
    else
        echo -e "${RED}✗ Unhealthy${NC}"
        return 1
    fi
}

# Function to test repository sync
test_repository_sync() {
    local source_server=$1
    local target_server=$2
    
    echo "Testing sync from $source_server to $target_server..."
    
    # Create test commit data
    local test_commit=$(cat <<EOF
{
    "commits": [
        {
            "hash": "test-$(date +%s)",
            "message": "Test commit for multi-server sync",
            "author": "test@rune.dev",
            "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
            "parent": null,
            "files": [
                {
                    "path": "test-sync.txt",
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
    
    # Push to source server
    echo -n "  Pushing to source server... "
    if curl -s -X POST "$source_server/sync/push" \
        -H "Content-Type: application/json" \
        -d "$test_commit" > /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Pull from target server (simulation)
    echo -n "  Testing pull endpoint... "
    if curl -s -X POST "$target_server/sync/pull" \
        -H "Content-Type: application/json" \
        -d '{"branch": "main", "since_commit": null}' > /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Function to test LFS operations
test_lfs_operations() {
    local server_url=$1
    local server_name=$2
    
    echo "Testing LFS operations on $server_name..."
    
    # Test LFS upload
    echo -n "  Testing LFS upload... "
    local lfs_data=$(cat <<EOF
{
    "oid": "test-oid-$(date +%s)",
    "chunk": "chunk1",
    "data": [72, 101, 108, 108, 111]
}
EOF
    )
    
    if curl -s -X POST "$server_url/lfs/upload" \
        -H "Content-Type: application/json" \
        -d "$lfs_data" > /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
    
    # Test LFS download
    echo -n "  Testing LFS download... "
    local download_data=$(cat <<EOF
{
    "oid": "test-oid",
    "chunk": "chunk1"
}
EOF
    )
    
    if curl -s -X POST "$server_url/lfs/download" \
        -H "Content-Type: application/json" \
        -d "$download_data" > /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Function to test load balancer
test_load_balancer() {
    echo "Testing load balancer..."
    
    # Add entries to /etc/hosts if running locally
    if [ -f "/etc/hosts" ]; then
        echo -n "  Checking DNS entries... "
        if grep -q "api.rune.local" /etc/hosts; then
            echo -e "${GREEN}✓ Found${NC}"
        else
            echo -e "${YELLOW}⚠ Add to /etc/hosts: 127.0.0.1 api.rune.local lfs.rune.local${NC}"
        fi
    fi
    
    # Test nginx health
    echo -n "  Testing nginx health... "
    if curl -s -f "http://localhost/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠ Nginx not accessible on port 80${NC}"
    fi
}

# Function to run Docker Compose setup
setup_docker_environment() {
    echo "Setting up Docker environment..."
    
    if ! command -v docker-compose &> /dev/null; then
        echo -e "${RED}✗ docker-compose not found${NC}"
        return 1
    fi
    
    echo "  Starting multi-server environment..."
    docker-compose -f docker-compose.multi-server.yml up -d
    
    echo "  Waiting for services to be healthy..."
    sleep 30
    
    # Check if services are running
    echo -n "  Checking Docker services... "
    if docker-compose -f docker-compose.multi-server.yml ps | grep -q "Up"; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

# Function to cleanup
cleanup() {
    echo "Cleaning up..."
    docker-compose -f docker-compose.multi-server.yml down -v 2>/dev/null || true
}

# Main test execution
main() {
    echo "📋 Starting comprehensive test suite..."
    echo
    
    # Setup Docker environment
    if [ "${SKIP_DOCKER:-false}" != "true" ]; then
        setup_docker_environment
        trap cleanup EXIT
    fi
    
    # Wait for services to stabilize
    echo "⏳ Waiting for services to stabilize..."
    sleep 10
    
    # Test server health
    echo "🏥 Testing server health..."
    check_server_health "$SERVER1_API" "Server 1" || exit 1
    check_server_health "$SERVER2_API" "Server 2" || exit 1
    
    echo
    
    # Test repository sync
    echo "🔄 Testing repository synchronization..."
    test_repository_sync "$SERVER1_API" "$SERVER2_API" || exit 1
    test_repository_sync "$SERVER2_API" "$SERVER1_API" || exit 1
    
    echo
    
    # Test LFS operations
    echo "📦 Testing LFS operations..."
    test_lfs_operations "$SERVER1_SHRINE" "Server 1 Shrine" || exit 1
    test_lfs_operations "$SERVER2_SHRINE" "Server 2 Shrine" || exit 1
    
    echo
    
    # Test load balancer
    echo "⚖️ Testing load balancer..."
    test_load_balancer
    
    echo
    echo -e "${GREEN}✅ All tests completed successfully!${NC}"
    echo
    echo "🔗 Access points:"
    echo "  - Server 1 API: $SERVER1_API"
    echo "  - Server 2 API: $SERVER2_API"
    echo "  - Server 1 Shrine: $SERVER1_SHRINE"
    echo "  - Server 2 Shrine: $SERVER2_SHRINE"
    echo "  - Load Balancer: http://localhost (if nginx running)"
    echo "  - Monitoring: http://localhost:9090 (Prometheus)"
    echo
    echo "📚 Next steps for Sunday testing:"
    echo "  1. Test with your actual server setup"
    echo "  2. Configure authentication tokens"
    echo "  3. Test real repository operations"
    echo "  4. Verify cross-server synchronization"
}

# Handle script arguments
case "${1:-test}" in
    "setup")
        setup_docker_environment
        ;;
    "cleanup")
        cleanup
        ;;
    "test")
        main
        ;;
    *)
        echo "Usage: $0 [setup|test|cleanup]"
        exit 1
        ;;
esac
