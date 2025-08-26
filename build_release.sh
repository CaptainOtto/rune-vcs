#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting local release build...${NC}"

# Get the version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}📦 Building version: $VERSION${NC}"

# Create release directory
RELEASE_DIR="./release"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Function to build for a target
build_target() {
    local target=$1
    local os_name=$2
    
    echo -e "${YELLOW}🔨 Building for $target...${NC}"
    
    # Install target if not already installed
    rustup target add "$target" 2>/dev/null || true
    
    # Build the binary
    if cargo build --release --target "$target"; then
        # Find the binary
        if [ -f "target/$target/release/rune" ]; then
            BINARY_PATH="target/$target/release/rune"
        elif [ -f "target/$target/release/rune.exe" ]; then
            BINARY_PATH="target/$target/release/rune.exe"
        else
            echo -e "${RED}❌ Binary not found for $target${NC}"
            return 1
        fi
        
        # Create archive name
        ARCHIVE_NAME="rune-$VERSION-$target.tar.gz"
        
        # Create tar.gz archive
        echo -e "${YELLOW}📦 Creating archive: $ARCHIVE_NAME${NC}"
        tar -czf "$RELEASE_DIR/$ARCHIVE_NAME" -C "$(dirname "$BINARY_PATH")" "$(basename "$BINARY_PATH")"
        
        echo -e "${GREEN}✅ Successfully built $ARCHIVE_NAME${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to build for $target${NC}"
        return 1
    fi
}

# Build for different targets
TARGETS=(
    "aarch64-apple-darwin:macos-arm64"
    "x86_64-apple-darwin:macos-x86_64"
    "x86_64-unknown-linux-gnu:linux-x86_64"
    "aarch64-unknown-linux-gnu:linux-arm64"
)

echo -e "${YELLOW}🎯 Building for all targets...${NC}"

SUCCESS_COUNT=0
TOTAL_COUNT=${#TARGETS[@]}

for target_info in "${TARGETS[@]}"; do
    IFS=':' read -r target os_name <<< "$target_info"
    if build_target "$target" "$os_name"; then
        ((SUCCESS_COUNT++))
    fi
done

echo
echo -e "${YELLOW}📋 Generating checksums...${NC}"

# Generate checksums
cd "$RELEASE_DIR"
if ls *.tar.gz 1> /dev/null 2>&1; then
    for file in *.tar.gz; do
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            shasum -a 256 "$file" >> checksums.txt
        else
            # Linux
            sha256sum "$file" >> checksums.txt
        fi
    done
    echo -e "${GREEN}✅ Checksums generated${NC}"
else
    echo -e "${RED}❌ No archives found to checksum${NC}"
fi

cd ..

echo
echo -e "${GREEN}🎉 Release build complete!${NC}"
echo -e "${YELLOW}📊 Built $SUCCESS_COUNT out of $TOTAL_COUNT targets${NC}"
echo -e "${YELLOW}📁 Release files in: $RELEASE_DIR${NC}"

if [ -d "$RELEASE_DIR" ]; then
    echo -e "${YELLOW}📋 Generated files:${NC}"
    ls -la "$RELEASE_DIR/"
    
    if [ -f "$RELEASE_DIR/checksums.txt" ]; then
        echo
        echo -e "${YELLOW}🔐 Checksums:${NC}"
        cat "$RELEASE_DIR/checksums.txt"
    fi
fi

echo
if [ $SUCCESS_COUNT -eq $TOTAL_COUNT ]; then
    echo -e "${GREEN}🎯 All builds successful! Ready for release.${NC}"
else
    echo -e "${YELLOW}⚠️  Some builds failed. Check the output above.${NC}"
fi
