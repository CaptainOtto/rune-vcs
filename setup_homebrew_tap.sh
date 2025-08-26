#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🍺 Homebrew Tap Setup${NC}"

# Configuration
TAP_NAME="rune-vcs"
GITHUB_USERNAME="Johan-Ott"  # Change this to your GitHub username
TAP_REPO_NAME="homebrew-$TAP_NAME"
TAP_DIR="../$TAP_REPO_NAME"

echo -e "${YELLOW}📋 Configuration:${NC}"
echo -e "${BLUE}  Tap name: $TAP_NAME${NC}"
echo -e "${BLUE}  Repository: $GITHUB_USERNAME/$TAP_REPO_NAME${NC}"
echo -e "${BLUE}  Directory: $TAP_DIR${NC}"

# Check if tap directory already exists
if [ -d "$TAP_DIR" ]; then
    echo -e "${YELLOW}⚠️  Tap directory already exists: $TAP_DIR${NC}"
    echo -e "${BLUE}❓ Do you want to update the existing tap? (y/n)${NC}"
    read -r UPDATE_EXISTING
    
    if [[ "$UPDATE_EXISTING" != "y" && "$UPDATE_EXISTING" != "Y" ]]; then
        echo -e "${YELLOW}ℹ️  Exiting without changes.${NC}"
        exit 0
    fi
    
    cd "$TAP_DIR"
    echo -e "${BLUE}📁 Using existing tap directory${NC}"
else
    echo -e "${BLUE}📁 Creating new tap repository...${NC}"
    
    # Create the tap directory structure
    mkdir -p "$TAP_DIR/Formula"
    cd "$TAP_DIR"
    
    # Initialize git repository
    git init
    
    # Create README
    cat > README.md << EOF
# Homebrew Tap for Rune VCS

This is a Homebrew tap for [Rune VCS](https://github.com/$GITHUB_USERNAME/rune-vcs), a modern, intelligent version control system.

## Installation

\`\`\`bash
# Add the tap
brew tap $GITHUB_USERNAME/$TAP_NAME

# Install Rune
brew install rune
\`\`\`

## Usage

\`\`\`bash
# Initialize a new repository
rune init

# Check status
rune status

# Get help
rune --help
\`\`\`

## About Rune VCS

Rune is a modern version control system designed with intelligence and user experience in mind. It offers:

- 🧠 **Intelligent Analysis**: Built-in repository analytics and insights
- 🚀 **Modern CLI**: Intuitive commands with helpful output
- 🔧 **Developer Tools**: LFS, hooks, workspaces, and more
- 📦 **Easy Installation**: Available via Homebrew, Docker, and direct download

## Links

- [Main Repository](https://github.com/$GITHUB_USERNAME/rune-vcs)
- [Documentation](https://github.com/$GITHUB_USERNAME/rune-vcs/tree/main/docs)
- [Releases](https://github.com/$GITHUB_USERNAME/rune-vcs/releases)

EOF
    
    echo -e "${GREEN}✅ Created tap directory structure${NC}"
fi

# Copy the formula
echo -e "${BLUE}📋 Copying formula...${NC}"
cp "../rune-vcs/tap_template/Formula/rune.rb" "Formula/rune.rb"

echo -e "${GREEN}✅ Formula copied${NC}"

# Git operations
git add .

if git diff --staged --quiet; then
    echo -e "${YELLOW}ℹ️  No changes to commit${NC}"
else
    # Get version from formula
    VERSION=$(grep 'version "' Formula/rune.rb | sed 's/.*version "\(.*\)".*/\1/')
    
    git commit -m "feat: update Rune formula to v$VERSION

- Update formula for Rune VCS v$VERSION
- Support both ARM64 and x86_64 macOS
- Include proper SHA256 checksums"
    
    echo -e "${GREEN}✅ Changes committed${NC}"
fi

# Check if remote exists
if ! git remote get-url origin &> /dev/null; then
    echo -e "${YELLOW}💡 To complete the setup:${NC}"
    echo -e "${YELLOW}  1. Create repository on GitHub: https://github.com/new${NC}"
    echo -e "${YELLOW}     Name: $TAP_REPO_NAME${NC}"
    echo -e "${YELLOW}     Description: Homebrew tap for Rune VCS${NC}"
    echo -e "${YELLOW}  2. Add remote: git remote add origin https://github.com/$GITHUB_USERNAME/$TAP_REPO_NAME.git${NC}"
    echo -e "${YELLOW}  3. Push: git push -u origin main${NC}"
else
    echo -e "${BLUE}📤 Pushing changes...${NC}"
    git push origin main
    echo -e "${GREEN}✅ Changes pushed to GitHub${NC}"
fi

echo
echo -e "${GREEN}🎉 Homebrew tap setup complete!${NC}"
echo -e "${YELLOW}📋 Formula location: $(pwd)/Formula/rune.rb${NC}"
echo -e "${YELLOW}📋 To install Rune:${NC}"
echo -e "${BLUE}  brew tap $GITHUB_USERNAME/$TAP_NAME${NC}"
echo -e "${BLUE}  brew install rune${NC}"

# Show current directory
echo -e "${YELLOW}📁 Current directory: $(pwd)${NC}"
echo -e "${YELLOW}📋 Files in tap:${NC}"
ls -la
