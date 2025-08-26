#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if we have the GitHub CLI
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ GitHub CLI (gh) is required but not installed.${NC}"
    echo -e "${YELLOW}💡 Install it with: brew install gh${NC}"
    exit 1
fi

# Check if we're logged in to GitHub CLI
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ You need to log in to GitHub CLI first.${NC}"
    echo -e "${YELLOW}💡 Run: gh auth login${NC}"
    exit 1
fi

echo -e "${GREEN}🚀 Automated Release Creator${NC}"

# Get the version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}📦 Current version: $VERSION${NC}"

# Ask if we want to create a release for this version
echo -e "${BLUE}❓ Create release for version v$VERSION? (y/n)${NC}"
read -r CREATE_RELEASE

if [[ "$CREATE_RELEASE" != "y" && "$CREATE_RELEASE" != "Y" ]]; then
    echo -e "${YELLOW}ℹ️  Release cancelled.${NC}"
    exit 0
fi

# Build the binaries
echo -e "${GREEN}🔨 Building release binaries...${NC}"
./build_macos_release.sh

if [ ! -d "./release-macos" ] || [ ! -f "./release-macos/checksums.txt" ]; then
    echo -e "${RED}❌ Build failed or release directory not found.${NC}"
    exit 1
fi

echo -e "${GREEN}📋 Binaries built successfully!${NC}"

# Create or check if tag exists
TAG_NAME="v$VERSION"
if git tag -l | grep -q "^$TAG_NAME$"; then
    echo -e "${YELLOW}⚠️  Tag $TAG_NAME already exists.${NC}"
else
    echo -e "${BLUE}🏷️  Creating tag $TAG_NAME...${NC}"
    git tag "$TAG_NAME"
    git push origin "$TAG_NAME"
fi

# Create release notes
RELEASE_NOTES_FILE="release_notes_tmp.md"
cat > "$RELEASE_NOTES_FILE" << EOF
# Rune VCS $VERSION

## 🎯 Alpha Release

This is an alpha release of Rune VCS - a modern, intelligent version control system.

### 📦 Available Downloads

- **macOS (Apple Silicon)**: \`rune-$VERSION-aarch64-apple-darwin.tar.gz\`
- **macOS (Intel)**: \`rune-$VERSION-x86_64-apple-darwin.tar.gz\`

### 🔐 Checksums

\`\`\`
$(cat "./release-macos/checksums.txt")
\`\`\`

### 📋 Installation

#### Homebrew (Coming Soon)
\`\`\`bash
brew tap CaptainOtto/rune-vcs
brew install rune
\`\`\`

#### Manual Installation
1. Download the appropriate archive for your system
2. Extract: \`tar -xzf rune-$VERSION-<your-platform>.tar.gz\`
3. Move to PATH: \`mv rune /usr/local/bin/\`

### ⚠️ Alpha Notice

This is an alpha release for testing and feedback. Not recommended for production use.

---
*Built with ❤️ by the Rune VCS team*
EOF

echo -e "${BLUE}📝 Creating GitHub release...${NC}"

# Create the release
if gh release create "$TAG_NAME" \
    --title "Rune VCS $VERSION" \
    --notes-file "$RELEASE_NOTES_FILE" \
    --prerelease \
    ./release-macos/*.tar.gz \
    ./release-macos/checksums.txt; then
    
    echo -e "${GREEN}🎉 Release created successfully!${NC}"
    echo -e "${BLUE}🔗 View release: https://github.com/Johan-Ott/rune-vcs/releases/tag/$TAG_NAME${NC}"
    
    # Clean up
    rm -f "$RELEASE_NOTES_FILE"
    
    echo -e "${YELLOW}📋 Next steps:${NC}"
    echo -e "${YELLOW}  1. Update Homebrew formula${NC}"
    echo -e "${YELLOW}  2. Test the release binaries${NC}"
    echo -e "${YELLOW}  3. Share with other projects/AI systems${NC}"
    
else
    echo -e "${RED}❌ Failed to create release.${NC}"
    rm -f "$RELEASE_NOTES_FILE"
    exit 1
fi
