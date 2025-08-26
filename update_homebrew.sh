#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🍺 Homebrew Formula Updater${NC}"

# Get the version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}📦 Version: $VERSION${NC}"

# Check if release files exist
if [ ! -f "./release-macos/checksums.txt" ]; then
    echo -e "${RED}❌ Release files not found. Run ./build_macos_release.sh first.${NC}"
    exit 1
fi

# Extract checksums
ARM64_FILE="rune-$VERSION-aarch64-apple-darwin.tar.gz"
X64_FILE="rune-$VERSION-x86_64-apple-darwin.tar.gz"

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

# Update the Homebrew formula
FORMULA_FILE="tap_template/Formula/rune.rb"

cat > "$FORMULA_FILE" << EOF
class Rune < Formula
  desc "Modern, intelligent version control system"
  homepage "https://github.com/Johan-Ott/rune-vcs"
  url "https://github.com/Johan-Ott/rune-vcs/releases/download/v$VERSION/rune-$VERSION-x86_64-apple-darwin.tar.gz"
  sha256 "$X64_SHA"
  license "Apache-2.0"
  version "$VERSION"

  on_arm do
    url "https://github.com/Johan-Ott/rune-vcs/releases/download/v$VERSION/rune-$VERSION-aarch64-apple-darwin.tar.gz"
    sha256 "$ARM64_SHA"
  end

  depends_on "rust" => :build

  def install
    bin.install "rune"
  end

  test do
    system "#{bin}/rune", "--version"
  end
end
EOF

echo -e "${GREEN}📝 Updated Homebrew formula: $FORMULA_FILE${NC}"

# Show the formula content
echo -e "${YELLOW}📋 Formula content:${NC}"
cat "$FORMULA_FILE"

echo
echo -e "${YELLOW}📋 Next steps for Homebrew:${NC}"
echo -e "${YELLOW}  1. Copy this formula to your Homebrew tap repository${NC}"
echo -e "${YELLOW}  2. Commit and push to your tap repository${NC}"
echo -e "${YELLOW}  3. Test installation: brew install your-tap/rune${NC}"

# Ask if we want to commit the formula
echo
echo -e "${BLUE}❓ Commit the updated formula to this repository? (y/n)${NC}"
read -r COMMIT_FORMULA

if [[ "$COMMIT_FORMULA" == "y" || "$COMMIT_FORMULA" == "Y" ]]; then
    git add "$FORMULA_FILE"
    git commit -m "feat: update Homebrew formula for v$VERSION

- Update to version $VERSION
- Add ARM64 and x86_64 macOS support
- Include SHA256 checksums for verification"
    
    echo -e "${GREEN}✅ Formula committed to repository!${NC}"
    echo -e "${YELLOW}💡 Don't forget to push: git push origin main${NC}"
fi

echo -e "${GREEN}🎉 Homebrew formula update complete!${NC}"
