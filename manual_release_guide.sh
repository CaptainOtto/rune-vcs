#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}📋 Manual Release Instructions${NC}"

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}📦 Version: $VERSION${NC}"

if [ ! -d "./release-macos" ]; then
    echo -e "${RED}❌ Release directory not found. Please run ./build_macos_release.sh first.${NC}"
    exit 1
fi

echo -e "${BLUE}📁 Release files ready:${NC}"
ls -la ./release-macos/

echo
echo -e "${YELLOW}📋 Manual Release Steps:${NC}"
echo -e "${BLUE}1. Go to: https://github.com/Johan-Ott/rune-vcs/releases/new${NC}"
echo -e "${BLUE}2. Tag: v$VERSION${NC}"
echo -e "${BLUE}3. Title: Rune VCS $VERSION${NC}"
echo -e "${BLUE}4. Mark as pre-release: ✓ (since this is an alpha)${NC}"
echo -e "${BLUE}5. Upload these files:${NC}"
echo -e "${BLUE}   - ./release-macos/rune-$VERSION-aarch64-apple-darwin.tar.gz${NC}"
echo -e "${BLUE}   - ./release-macos/rune-$VERSION-x86_64-apple-darwin.tar.gz${NC}"
echo -e "${BLUE}   - ./release-macos/checksums.txt${NC}"

echo
echo -e "${YELLOW}📋 Release Notes Template:${NC}"
cat << EOF

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

#### Homebrew
\`\`\`bash
brew tap Johan-Ott/rune-vcs
brew install rune-vcs
\`\`\`

#### Manual Installation
1. Download the appropriate archive for your system
2. Extract: \`tar -xzf rune-$VERSION-<your-platform>.tar.gz\`
3. Move to PATH: \`mv rune /usr/local/bin/\`

### 🧪 Testing

\`\`\`bash
rune --version
rune doctor
rune --help
\`\`\`

### ⚠️ Alpha Notice

This is an alpha release for testing and feedback. Not recommended for production use.

---
*Built with ❤️ by the Rune VCS team*

EOF

echo
echo -e "${GREEN}🎉 After creating the release manually, test with:${NC}"
echo -e "${BLUE}brew uninstall rune-vcs${NC}"
echo -e "${BLUE}brew install rune-vcs${NC}"
