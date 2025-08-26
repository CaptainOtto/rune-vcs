#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🍺 Updating Homebrew Tap Repository${NC}"

# Configuration
TAP_REPO_URL="https://github.com/Johan-Ott/homebrew-rune-vcs.git"
TAP_DIR="../homebrew-rune-vcs"

# Get current version and checksums from our local release
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}📦 Current version: $VERSION${NC}"

# Check if we have release files
if [ ! -f "./release-macos/checksums.txt" ]; then
    echo -e "${RED}❌ Release files not found. Please run ./build_macos_release.sh first.${NC}"
    exit 1
fi

# Extract checksums
ARM64_FILE="rune-v$VERSION-aarch64-apple-darwin.tar.gz"
X64_FILE="rune-v$VERSION-x86_64-apple-darwin.tar.gz"

ARM64_SHA=$(grep "$ARM64_FILE" ./release-macos/checksums.txt | cut -d' ' -f1)
X64_SHA=$(grep "$X64_FILE" ./release-macos/checksums.txt | cut -d' ' -f1)

if [ -z "$ARM64_SHA" ] || [ -z "$X64_SHA" ]; then
    echo -e "${RED}❌ Could not find checksums for both architectures.${NC}"
    echo -e "${YELLOW}Available checksums:${NC}"
    cat ./release-macos/checksums.txt
    exit 1
fi

echo -e "${GREEN}✅ Found checksums:${NC}"
echo -e "${BLUE}  ARM64: $ARM64_SHA${NC}"
echo -e "${BLUE}  x86_64: $X64_SHA${NC}"

# Clone or update the tap repository
if [ -d "$TAP_DIR" ]; then
    echo -e "${BLUE}📁 Updating existing tap repository...${NC}"
    cd "$TAP_DIR"
    git pull origin master
else
    echo -e "${BLUE}📁 Cloning tap repository...${NC}"
    git clone "$TAP_REPO_URL" "$TAP_DIR"
    cd "$TAP_DIR"
fi

# Create the updated formula
echo -e "${BLUE}📋 Creating updated formula...${NC}"

cat > "Formula/rune-vcs.rb" << EOF
class RuneVcs < Formula
  desc "Modern, intelligent version control system with virtual workspaces and draft commits"
  homepage "https://github.com/Johan-Ott/rune-vcs"
  url "https://github.com/Johan-Ott/rune-vcs/releases/download/v$VERSION/rune-v$VERSION-x86_64-apple-darwin.tar.gz"
  sha256 "$X64_SHA"
  license "Apache-2.0"
  version "$VERSION"

  on_arm do
    url "https://github.com/Johan-Ott/rune-vcs/releases/download/v$VERSION/rune-v$VERSION-aarch64-apple-darwin.tar.gz"
    sha256 "$ARM64_SHA"
  end

  def install
    bin.install "rune" => "rune-vcs"
  end

  test do
    # Test basic version command
    assert_match version.to_s, shell_output("#{bin}/rune-vcs --version")
    
    # Test that the doctor command works
    system "#{bin}/rune-vcs", "doctor"
  end
end
EOF

echo -e "${GREEN}✅ Formula updated${NC}"

# Show the formula
echo -e "${YELLOW}📋 Updated formula:${NC}"
cat "Formula/rune-vcs.rb"

# Git operations
git add Formula/rune-vcs.rb

if git diff --staged --quiet; then
    echo -e "${YELLOW}ℹ️  No changes to commit${NC}"
else
    git commit -m "feat: update rune-vcs to v$VERSION

- Update to Rune VCS v$VERSION
- Switch to pre-built binaries for faster installation
- Support both ARM64 and x86_64 macOS architectures
- Include proper SHA256 checksums for security
- Remove Rust dependency (no longer building from source)
- Add doctor command test for better validation"
    
    echo -e "${GREEN}✅ Changes committed${NC}"
    
    echo -e "${BLUE}📤 Pushing changes...${NC}"
    git push origin master
    echo -e "${GREEN}✅ Changes pushed to GitHub${NC}"
fi

echo
echo -e "${GREEN}🎉 Homebrew tap updated successfully!${NC}"
echo -e "${YELLOW}📋 Your tap is now ready at: https://github.com/Johan-Ott/homebrew-rune-vcs${NC}"
echo

echo -e "${YELLOW}📋 To test the installation:${NC}"
echo -e "${BLUE}  # Add the tap (if not already added)${NC}"
echo -e "${BLUE}  brew tap Johan-Ott/rune-vcs${NC}"
echo
echo -e "${BLUE}  # Update the tap${NC}"
echo -e "${BLUE}  brew update${NC}"
echo
echo -e "${BLUE}  # Install or upgrade Rune${NC}"
echo -e "${BLUE}  brew install rune-vcs${NC}"
echo -e "${BLUE}  # or if already installed:${NC}"
echo -e "${BLUE}  brew upgrade rune-vcs${NC}"
echo
echo -e "${BLUE}  # Test the installation${NC}"
echo -e "${BLUE}  rune-vcs --version${NC}"
echo -e "${BLUE}  rune-vcs doctor${NC}"

echo
echo -e "${YELLOW}💡 For your AI/other projects:${NC}"
echo -e "${BLUE}  The Homebrew formula now references release v$VERSION${NC}"
echo -e "${BLUE}  Installation is faster (no compilation required)${NC}"
echo -e "${BLUE}  Both ARM64 and Intel Macs are supported${NC}"

# Return to original directory
cd - > /dev/null
