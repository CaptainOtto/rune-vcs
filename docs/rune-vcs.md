# rune-vcs.md — Complete Guide to Rune Version Control System

**Rune VCS** — Modern, scalable version control system  
**Carved in Rust. Forged for Worlds.**

---

## 🎯 Overview

rune-vcs är ett modernt versionshanteringssystem designat för att ersätta Git med förbättrad prestanda, AI-integration och smidigare workflow för moderna utvecklingsteam.

### Key Features

- 🚀 **Performance**: Smart delta compression, predictive caching
- 🧠 **AI-Powered**: Intelligent repository analysis and insights
- 📦 **LFS Built-in**: Automatic large file handling
- 🔐 **Security**: GPG signing, audit trails
- 🌐 **Modern Protocol**: QUIC/HTTP3 support
- 📊 **Analytics**: Built-in repository metrics

---

## 🚀 Installation

### macOS (Homebrew)

```bash
brew tap johan-ott/rune-vcs
brew install rune-vcs
```

### From Source

```bash
git clone https://github.com/Johan-Ott/rune-vcs
cd rune-vcs
cargo build --release
sudo cp target/release/rune /usr/local/bin/rune-vcs
```

### Verify Installation

```bash
rune-vcs --version
rune-vcs doctor  # System diagnostics
```

---

## 🌟 Quick Start

### Initialize Repository

```bash
# Create new repository
rune-vcs init

# Clone existing repository
rune-vcs clone <remote-url>

# Configure user
rune-vcs config --global user.name "Your Name"
rune-vcs config --global user.email "your.email@example.com"
```

### Basic Workflow

```bash
# Check status
rune-vcs status

# Add files
rune-vcs add file.txt
rune-vcs add .  # Add all files

# Review changes
rune-vcs diff
rune-vcs diff --staged

# Commit changes
rune-vcs commit -m "feat: add new feature"

# View history
rune-vcs log
rune-vcs log --oneline
```

---

## 🔄 Branch Management

### Creating and Switching Branches

```bash
# Create new branch
rune-vcs branch feature/new-feature

# Switch to branch
rune-vcs checkout feature/new-feature

# Create and switch in one command
rune-vcs checkout -b feature/another-feature

# List branches
rune-vcs branch --list
rune-vcs branch -a  # Include remote branches
```

### Merging

```bash
# Merge branch into current
rune-vcs merge feature/new-feature

# Interactive merge with conflict resolution
rune-vcs merge --tool feature/new-feature

# Abort merge
rune-vcs merge --abort
```

### Cleaning Up

```bash
# Delete local branch
rune-vcs branch --delete feature/completed-feature

# Delete remote branch
rune-vcs push origin --delete feature/completed-feature
```

---

## 🌐 Remote Operations

### Remote Management

```bash
# Add remote
rune-vcs remote add origin <url>

# List remotes
rune-vcs remote list

# Remove remote
rune-vcs remote remove origin

# Change remote URL
rune-vcs remote set-url origin <new-url>
```

### Syncing Changes

```bash
# Fetch changes from remote
rune-vcs fetch origin

# Pull changes (fetch + merge)
rune-vcs pull origin main

# Push changes
rune-vcs push origin main

# Push all branches
rune-vcs push --all origin
```

---

## 📦 Large File Support (LFS)

### Automatic LFS

```bash
# Track file types automatically
rune-vcs lfs track "*.psd" "*.zip" "*.mp4"

# Check LFS status
rune-vcs lfs status

# Migrate existing large files
rune-vcs lfs migrate --min-size 50MB

# List tracked patterns
rune-vcs lfs track --list
```

### Manual LFS Operations

```bash
# Force add file to LFS
rune-vcs lfs add large-file.zip

# Pull LFS files
rune-vcs lfs pull

# Push LFS files
rune-vcs lfs push origin main
```

---

## 🧠 AI Intelligence Features

### Repository Analysis

```bash
# Analyze code quality and patterns
rune-vcs intelligence analyze

# Predict potential issues
rune-vcs intelligence predict

# Generate insights report
rune-vcs intelligence report --format=html
```

### Smart Suggestions

```bash
# Get commit message suggestions
rune-vcs intelligence suggest-commit

# Analyze branch naming patterns
rune-vcs intelligence analyze-branches

# Performance optimization suggestions
rune-vcs intelligence optimize
```

---

## 🔐 Security Features

### GPG Signing

```bash
# Configure GPG key
rune-vcs config --global user.signingkey <key-id>

# Sign commits
rune-vcs commit --gpg-sign -m "secure: implement authentication"

# Verify signatures
rune-vcs verify-commit HEAD
rune-vcs log --show-signature
```

### Security Audit

```bash
# Audit repository security
rune-vcs security audit

# Check for sensitive data
rune-vcs security scan

# Generate security report
rune-vcs security report
```

---

## 🔧 Advanced Operations

### Interactive Rebase

```bash
# Interactive rebase last 3 commits
rune-vcs rebase -i HEAD~3

# Rebase onto another branch
rune-vcs rebase main

# Continue rebase after resolving conflicts
rune-vcs rebase --continue

# Abort rebase
rune-vcs rebase --abort
```

### Cherry-picking

```bash
# Cherry-pick single commit
rune-vcs cherry-pick <commit-hash>

# Cherry-pick range of commits
rune-vcs cherry-pick-range <start-commit>..<end-commit>

# Cherry-pick without committing
rune-vcs cherry-pick --no-commit <commit-hash>
```

### Stashing

```bash
# Stash current changes
rune-vcs stash

# Stash with message
rune-vcs stash save "work in progress"

# List stashes
rune-vcs stash list

# Apply stash
rune-vcs stash apply
rune-vcs stash apply stash@{1}

# Pop stash (apply and remove)
rune-vcs stash pop

# Drop stash
rune-vcs stash drop stash@{0}
```

---

## 📊 Repository Management

### Status and Information

```bash
# Detailed status
rune-vcs status --verbose

# Show repository information
rune-vcs show HEAD
rune-vcs show <commit-hash>

# Repository statistics
rune-vcs stats
rune-vcs stats --detailed
```

### Cleaning and Maintenance

```bash
# Clean untracked files
rune-vcs clean

# Clean ignored files
rune-vcs clean --ignored

# Garbage collection
rune-vcs gc

# Optimize repository
rune-vcs optimize
```

---

## 🎨 Customization

### Configuration

```bash
# Global configuration
rune-vcs config --global core.editor vim
rune-vcs config --global core.autocrlf false

# Repository-specific configuration
rune-vcs config user.name "Project Specific Name"

# List all configuration
rune-vcs config --list

# Edit configuration file
rune-vcs config --edit
```

### Aliases

```bash
# Create aliases
rune-vcs config --global alias.st status
rune-vcs config --global alias.co checkout
rune-vcs config --global alias.br branch
rune-vcs config --global alias.ci commit

# Use aliases
rune-vcs st  # Same as rune-vcs status
rune-vcs co main  # Same as rune-vcs checkout main
```

---

## 🔍 Troubleshooting

### Common Issues

```bash
# Reset to last commit (keep changes)
rune-vcs reset --soft HEAD~1

# Reset to last commit (discard changes)
rune-vcs reset --hard HEAD

# Undo last commit but keep changes staged
rune-vcs reset --mixed HEAD~1

# Show what changed in commit
rune-vcs show <commit-hash>

# Find when something was introduced
rune-vcs bisect start
rune-vcs bisect bad HEAD
rune-vcs bisect good <good-commit>
```

### Recovery Operations

```bash
# Restore deleted file
rune-vcs checkout HEAD -- deleted-file.txt

# Recover from reflog
rune-vcs reflog
rune-vcs checkout <reflog-entry>

# Fsck repository integrity
rune-vcs fsck

# Repair repository
rune-vcs repair
```

---

## 📈 Performance Features

### Smart Caching

```bash
# Enable predictive caching
rune-vcs config performance.predictive-cache true

# Cache statistics
rune-vcs performance stats

# Clear cache
rune-vcs performance clear-cache
```

### Compression

```bash
# Configure compression level
rune-vcs config compression.level 6

# Compress repository
rune-vcs compress

# Compression statistics
rune-vcs compression stats
```

---

## 🔌 Integration

### Hooks

```bash
# Install hooks
rune-vcs hooks install

# List available hooks
rune-vcs hooks list

# Create custom hook
rune-vcs hooks create pre-commit

# Test hooks
rune-vcs hooks test pre-commit
```

### API Server

```bash
# Start local API server
rune-vcs api start --port 8080

# API endpoints
curl http://localhost:8080/status
curl http://localhost:8080/log
curl http://localhost:8080/branches
```

---

## 🛠️ Submodules

### Basic Submodule Operations

```bash
# Add submodule
rune-vcs submodule add <url> <path>

# Initialize submodules
rune-vcs submodule init

# Update submodules
rune-vcs submodule update

# Update to latest
rune-vcs submodule update --remote
```

### Advanced Submodule Management

```bash
# Remove submodule
rune-vcs submodule remove <path>

# Status of all submodules
rune-vcs submodule status

# Execute command in all submodules
rune-vcs submodule foreach "git status"
```

---

## 📚 Documentation and Help

### Getting Help

```bash
# General help
rune-vcs help
rune-vcs --help

# Command-specific help
rune-vcs help commit
rune-vcs commit --help

# Documentation
rune-vcs docs

# Examples
rune-vcs examples

# Tutorial
rune-vcs tutorial
```

### Guides and Examples

```bash
# View workflow examples
rune-vcs examples workflow

# Git migration guide
rune-vcs guide migrate-from-git

# Best practices
rune-vcs guide best-practices
```

---

## 🎯 Git Migration

### Migration Commands

```bash
# Convert Git repository to Rune
rune-vcs migrate from-git /path/to/git-repo

# Import Git history
rune-vcs import git-history

# Compare with Git
rune-vcs compare-with-git
```

### Git Compatibility

| Git Command           | Rune VCS Equivalent        |
| --------------------- | -------------------------- |
| `git init`            | `rune-vcs init`            |
| `git add .`           | `rune-vcs add .`           |
| `git commit -m "msg"` | `rune-vcs commit -m "msg"` |
| `git status`          | `rune-vcs status`          |
| `git log`             | `rune-vcs log`             |
| `git branch`          | `rune-vcs branch`          |
| `git checkout`        | `rune-vcs checkout`        |
| `git merge`           | `rune-vcs merge`           |
| `git push`            | `rune-vcs push`            |
| `git pull`            | `rune-vcs pull`            |
| `git clone`           | `rune-vcs clone`           |

---

## 📋 Conventional Commits

### Commit Types

```bash
# Features
rune-vcs commit -m "feat: add user authentication"
rune-vcs commit -m "feat(api): implement REST endpoints"

# Bug fixes
rune-vcs commit -m "fix: resolve memory leak in parser"
rune-vcs commit -m "fix(ui): button alignment issue"

# Performance
rune-vcs commit -m "perf: optimize database queries"

# Refactoring
rune-vcs commit -m "refactor: extract utility functions"

# Documentation
rune-vcs commit -m "docs: update API documentation"

# Tests
rune-vcs commit -m "test: add unit tests for auth module"

# Breaking changes
rune-vcs commit -m "feat!: redesign API interface"
```

---

## 🔄 Workflows

### Feature Development Workflow

```bash
# 1. Start from main
rune-vcs checkout main
rune-vcs pull origin main

# 2. Create feature branch
rune-vcs checkout -b feature/new-feature

# 3. Develop and commit
rune-vcs add .
rune-vcs commit -m "feat: implement new feature"

# 4. Push feature branch
rune-vcs push origin feature/new-feature

# 5. Merge back to main
rune-vcs checkout main
rune-vcs merge feature/new-feature
rune-vcs push origin main

# 6. Clean up
rune-vcs branch --delete feature/new-feature
```

### Hotfix Workflow

```bash
# 1. Create hotfix from main
rune-vcs checkout main
rune-vcs checkout -b hotfix/critical-fix

# 2. Fix and commit
rune-vcs add .
rune-vcs commit -m "fix: resolve critical security issue"

# 3. Merge to main
rune-vcs checkout main
rune-vcs merge hotfix/critical-fix

# 4. Tag release
rune-vcs tag v1.0.1 -m "Hotfix release"

# 5. Push everything
rune-vcs push origin main --tags
```

---

## 📊 Performance Monitoring

### Performance Statistics

```bash
# Repository performance overview
rune-vcs performance overview

# Detailed timing information
rune-vcs performance timing

# Memory usage statistics
rune-vcs performance memory

# Network performance
rune-vcs performance network
```

### Optimization

```bash
# Optimize repository for performance
rune-vcs optimize --aggressive

# Benchmark operations
rune-vcs benchmark

# Profile repository operations
rune-vcs profile
```

---

## 🌟 Best Practices

### Repository Organization

- Use clear, descriptive branch names (`feature/user-auth`, `fix/memory-leak`)
- Keep commits small and focused
- Write meaningful commit messages using conventional commits
- Use branches for all non-trivial changes
- Regularly sync with remote repositories

### Performance Tips

- Enable predictive caching for frequently accessed repositories
- Use LFS for binary files larger than 10MB
- Regularly run `rune-vcs optimize` on large repositories
- Configure appropriate compression levels for your use case

### Security Recommendations

- Enable GPG signing for sensitive repositories
- Regularly audit repository security with `rune-vcs security audit`
- Use SSH keys for remote authentication
- Keep rune-vcs updated to the latest version

---

## 🔗 Resources

### Official Links

- **Homepage**: https://github.com/Johan-Ott/rune-vcs
- **Documentation**: `rune-vcs docs`
- **Issues**: https://github.com/Johan-Ott/rune-vcs/issues
- **Releases**: https://github.com/Johan-Ott/rune-vcs/releases

### Community

- **Discussions**: GitHub Discussions
- **Stack Overflow**: Tag `rune-vcs`
- **Discord**: [Community Server]

---

## 📄 License

MIT License - see LICENSE file for details.

---

_Last updated: 2025-08-26_  
_Rune VCS version: 0.2.5_
