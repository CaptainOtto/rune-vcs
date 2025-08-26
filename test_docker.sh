#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🐳 Docker Test Script${NC}"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed or not in PATH${NC}"
    echo -e "${YELLOW}💡 Install Docker to test the container build${NC}"
    echo -e "${BLUE}ℹ️  You can still commit the Dockerfile fixes and test on GitHub Actions${NC}"
    exit 1
fi

echo -e "${BLUE}🔨 Building Docker image...${NC}"

# Build the image
if docker build -t rune-vcs:test .; then
    echo -e "${GREEN}✅ Docker image built successfully!${NC}"
    
    echo -e "${BLUE}🧪 Testing Docker image...${NC}"
    
    # Test basic functionality
    if docker run --rm rune-vcs:test rune --version; then
        echo -e "${GREEN}✅ Version command works!${NC}"
    else
        echo -e "${RED}❌ Version command failed${NC}"
    fi
    
    # Test help
    if docker run --rm rune-vcs:test rune --help; then
        echo -e "${GREEN}✅ Help command works!${NC}"
    else
        echo -e "${RED}❌ Help command failed${NC}"
    fi
    
    echo -e "${GREEN}🎉 Docker tests completed!${NC}"
    echo -e "${YELLOW}💡 To clean up: docker rmi rune-vcs:test${NC}"
    
else
    echo -e "${RED}❌ Docker build failed${NC}"
    exit 1
fi
