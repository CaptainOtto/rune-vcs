# Getting Started with Rune

## What is Rune?

Rune is a next-generation version control system designed to be faster, more intelligent, and easier to use than traditional VCS tools. It combines the best features of Git with modern performance optimizations and user-friendly interfaces.

## Key Features

- **Lightning Fast**: Built in Rust for maximum performance
- **Intelligent Analysis**: AI-powered insights for your code
- **User Friendly**: Intuitive commands and helpful guidance
- **Git Compatible**: Easy migration from existing Git repositories
- **Advanced Features**: Built-in LFS, delta compression, and more

## Installation

### From Package Managers

#### macOS (Homebrew)

```bash
brew tap rune-vcs/tap
brew install rune
```

#### Windows (Scoop)

```bash
scoop bucket add rune https://github.com/rune-vcs/scoop-bucket.git
scoop install rune
```

#### Linux (Snap)

```bash
snap install rune-vcs
```

### From Source

```bash
git clone https://github.com/rune-vcs/rune.git
cd rune
cargo build --release
```

## First Steps

### Creating a new repository

```bash
mkdir my-project
cd my-project
rune init
echo "# My Project" > README.md
rune add README.md
rune commit -m "Initial commit"
```

### Daily development

```bash
# Check status
rune status

# Add and commit changes
rune add .
rune commit -m "Add new feature"

# View history
rune log
```

### Working with branches

```bash
# Create and switch to a new branch
rune branch feature/new-functionality
rune checkout feature/new-functionality

# Make changes and commit
rune add .
rune commit -m "Implement new feature"

# Switch back and merge
rune checkout main
rune merge feature/new-functionality
```

## Migrating from Git

Rune provides seamless migration from Git repositories:

```bash
# Clone a Git repository as a Rune repository
rune clone https://github.com/user/repo.git

# Or convert an existing Git repository
cd existing-git-repo
rune migrate --from-git
```

## Getting Help

- Use `rune help` for command overview
- Use `rune help <command>` for specific command help
- Use `rune docs` to open documentation
- Use `rune examples` to see common workflows
- Use `rune tutorial` for interactive learning

## What's Next?

1. Try the basic tutorial: `rune tutorial basics`
2. Explore examples: `rune examples list`
3. Read the complete documentation: `rune docs serve`
4. Learn about advanced features: `rune tutorial advanced`
