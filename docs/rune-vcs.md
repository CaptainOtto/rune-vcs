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
- 🔐 **Security**: GPG signing, audit trails, secret scanning
- 🌐 **Modern Protocol**: QUIC/HTTP3 support
- 📊 **Analytics**: Built-in repository metrics
- 🏗️ **Virtual Workspaces**: Focus on repository subsets for large projects
- 📝 **Draft Commits**: Work-in-progress management with checkpoints
- 🩹 **Interactive Staging**: Hunk-level staging with patch mode
- 🔄 **Advanced Operations**: Amend, revert, rebase, cherry-pick
- 🚫 **Smart Ignore**: Template-based ignore management with optimization
- 📚 **Built-in Help**: Interactive tutorials, examples, and documentation
- 🛠️ **Extensible**: Hooks, API server, shell completions

---

## 🚀 Installation

### macOS (Homebrew) - Recommended

```bash
brew tap johan-ott/rune-vcs
brew install rune-vcs
```

### Download Pre-built Binaries

```bash
brew tap johan-ott/rune-vcs
brew install rune-vcs
```

### Download Pre-built Binaries

```bash
# Download for macOS ARM64 (Apple Silicon)
curl -L -o rune-vcs.tar.gz https://github.com/Johan-Ott/rune-vcs/releases/download/v0.3.0-alpha.4/rune-v0.3.0-alpha.4-aarch64-apple-darwin.tar.gz

# Download for macOS x86_64 (Intel)
curl -L -o rune-vcs.tar.gz https://github.com/Johan-Ott/rune-vcs/releases/download/v0.3.0-alpha.4/rune-v0.3.0-alpha.4-x86_64-apple-darwin.tar.gz

# Extract and install
tar -xzf rune-vcs.tar.gz
sudo cp rune /usr/local/bin/rune-vcs
```

### From Source

```bash
git clone https://github.com/Johan-Ott/rune-vcs
cd rune-vcs
cargo build --release
sudo cp target/release/rune-cli /usr/local/bin/rune-vcs
```

### Update Rune VCS

```bash
# Update via Homebrew
brew upgrade rune-vcs

# Check for updates
rune-vcs update --check

# Self-update (when implemented)
rune-vcs update
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

# Add files (with interactive patch mode)
rune-vcs add file.txt
rune-vcs add .  # Add all files
rune-vcs add --patch  # Interactive staging

# Review changes
rune-vcs diff
rune-vcs diff --staged

# Commit changes
rune-vcs commit -m "feat: add new feature"

# Amend last commit
rune-vcs commit --amend -m "feat: updated feature description"
rune-vcs commit --amend --no-edit  # Keep same message

# View history
rune-vcs log
rune-vcs log --oneline

# Show specific commit details
rune-vcs show <commit-hash>
rune-vcs show HEAD
```

---

## � File Operations

### Moving and Removing Files

```bash
# Move or rename files
rune-vcs move old-file.txt new-file.txt
rune-vcs move src/file.rs lib/file.rs

# Remove files from working directory and staging
rune-vcs remove file.txt
rune-vcs remove --cached file.txt  # Remove from staging only

# Show line-by-line file history (blame)
rune-vcs blame file.txt
rune-vcs blame --line-range 10:20 file.txt
```

### Reverting Changes

```bash
# Revert a specific commit
rune-vcs revert <commit-hash>

# Revert without creating a new commit
rune-vcs revert --no-commit <commit-hash>

# Revert merge commit (specify parent)
rune-vcs revert --mainline 1 <merge-commit-hash>
```

### Reset Operations

```bash
# Reset staging area (keep working directory changes)
rune-vcs reset

# Reset to specific commit (keep working directory)
rune-vcs reset <commit-hash>

# Hard reset (DESTRUCTIVE - loses all changes)
rune-vcs reset --hard HEAD

# Reset specific files
rune-vcs reset file.txt
rune-vcs reset --hard file.txt
```

---

## 📦 Draft Commits & Checkpoints

Rune VCS includes a powerful draft system for managing work-in-progress changes and creating checkpoints.

### Creating and Managing Drafts

```bash
# Create a draft from current changes
rune-vcs draft create "work in progress on auth module"

# List all drafts
rune-vcs draft list

# Show draft details
rune-vcs draft show <draft-id>

# Apply a draft to working directory
rune-vcs draft apply <draft-id>

# Update existing draft with current changes
rune-vcs draft update <draft-id>

# Shelve (temporarily store) active changes
rune-vcs draft shelve <draft-id>
```

### Draft Tagging and Organization

```bash
# Add tags to drafts for organization
rune-vcs draft tag <draft-id> "feature" "auth" "wip"

# Remove tags
rune-vcs draft untag <draft-id> "wip"

# Delete drafts
rune-vcs draft delete <draft-id>

# Clean up old drafts
rune-vcs draft cleanup --older-than 30d
```

### Automatic Checkpoints

```bash
# Create automatic checkpoint
rune-vcs draft checkpoint

# Configure automatic checkpointing
rune-vcs config draft.auto-checkpoint true
rune-vcs config draft.checkpoint-interval 1h
```

---

## 🏗️ Virtual Workspaces

Virtual workspaces allow you to work with subsets of large repositories efficiently.

### Workspace Setup

```bash
# Initialize virtual workspace
rune-vcs workspace init

# Add virtual roots (focus areas)
rune-vcs workspace add-root src/frontend
rune-vcs workspace add-root docs/

# List virtual roots
rune-vcs workspace list

# Remove virtual root
rune-vcs workspace remove-root src/frontend
```

### Workspace Management

```bash
# Toggle virtual root active/inactive
rune-vcs workspace toggle src/backend

# View current workspace (what files are included)
rune-vcs workspace view

# Add global include patterns
rune-vcs workspace include "*.rs" "*.toml"

# Add global exclude patterns
rune-vcs workspace exclude "target/" "*.tmp"
```

### Performance Controls

```bash
# Validate workspace against performance limits
rune-vcs workspace validate

# Configure performance limits
rune-vcs workspace limits --max-files 10000
rune-vcs workspace limits --max-size 1GB

# Check workspace performance
rune-vcs workspace limits --show
```

**Note**: Advanced performance guardrails, policy-as-code enforcement, and other enterprise features are planned for future releases.

---

## �🔄 Branch Management

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

## 🩹 Patch Management

### Creating and Applying Patches

```bash
# Create patch from changes
rune-vcs patch create --range HEAD~2..HEAD

# Apply patch file
rune-vcs patch apply changes.patch

# Interactive staging (patch mode)
rune-vcs add --patch  # Choose hunks interactively
```

---

## � Advanced Ignore Management

### Ignore Operations

```bash
# Check if a file would be ignored
rune-vcs ignore check --debug file.txt

# Add patterns to ignore file
rune-vcs ignore add "*.log" "temp/"
rune-vcs ignore add --global "*.DS_Store"

# List current ignore rules
rune-vcs ignore list
rune-vcs ignore list --templates

# Apply project templates
rune-vcs ignore templates  # Show available templates
rune-vcs ignore apply rust  # Apply Rust project template
rune-vcs ignore apply node  # Apply Node.js template

# Initialize smart ignore configuration
rune-vcs ignore init
rune-vcs ignore init --force

# Optimize ignore rules
rune-vcs ignore optimize
rune-vcs ignore optimize --dry-run
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
rune-vcs docs view <topic>
rune-vcs docs search <query>
rune-vcs docs serve --open  # Start local docs server
rune-vcs docs list

# Examples
rune-vcs examples list
rune-vcs examples category workflow
rune-vcs examples search "branch"
rune-vcs examples show "feature-branch"

# Tutorial
rune-vcs tutorial basics
rune-vcs tutorial branching
rune-vcs tutorial collaboration
rune-vcs tutorial advanced
rune-vcs tutorial list
```

---

## �📦 Large File Support (LFS)

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

## 📈 Release Information

### Current Release: v0.3.0-alpha.4

**Alpha Release Notice**: This is an alpha version for testing and feedback. While functional, it's not recommended for production use.

#### What's New in 0.3.0-alpha.4
- ✅ Complete Homebrew integration with `johan-ott/rune-vcs` tap
- ✅ Consistent binary naming (`rune-vcs` command)
- ✅ Pre-built macOS binaries (ARM64 + x86_64)
- ✅ Modernized CI/CD pipeline with GitHub Actions
- ✅ Docker containerization support
- ✅ Local build and test automation
- ✅ Comprehensive documentation updates

#### Download Options
- **Homebrew**: `brew install rune-vcs` (recommended)
- **Direct Download**: GitHub releases with SHA256 verification
- **From Source**: Cargo build with Rust toolchain

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

| Git Command               | Rune VCS Equivalent           | Notes                          |
| ------------------------- | ----------------------------- | ------------------------------ |
| `git init`                | `rune-vcs init`               |                                |
| `git add .`               | `rune-vcs add .`              |                                |
| `git add -p`              | `rune-vcs add --patch`        | Interactive staging            |
| `git commit -m "msg"`     | `rune-vcs commit -m "msg"`    |                                |
| `git commit --amend`      | `rune-vcs commit --amend`     | Amend last commit              |
| `git status`              | `rune-vcs status`             |                                |
| `git log`                 | `rune-vcs log`                |                                |
| `git show`                | `rune-vcs show`               | Show commit details            |
| `git blame`               | `rune-vcs blame`              | Line-by-line file history      |
| `git branch`              | `rune-vcs branch`             |                                |
| `git checkout`            | `rune-vcs checkout`           |                                |
| `git merge`               | `rune-vcs merge`              |                                |
| `git revert`              | `rune-vcs revert`             | With --mainline support       |
| `git reset`               | `rune-vcs reset`              | Soft/hard reset modes          |
| `git rm`                  | `rune-vcs remove`             | Remove files                   |
| `git mv`                  | `rune-vcs move`               | Move/rename files              |
| `git stash`               | `rune-vcs stash`              |                                |
| `git cherry-pick`         | `rune-vcs cherry-pick`        |                                |
| `git rebase`              | `rune-vcs rebase`             |                                |
| `git push`                | `rune-vcs push`               |                                |
| `git pull`                | `rune-vcs pull`               |                                |
| `git clone`               | `rune-vcs clone`              |                                |
| `git fetch`               | `rune-vcs fetch`              |                                |
| `git diff`                | `rune-vcs diff`               |                                |
| `git submodule`           | `rune-vcs submodule`          |                                |
| `git config`              | `rune-vcs config`             |                                |
| _N/A_                     | `rune-vcs draft`              | 🆕 Draft commits & checkpoints |
| _N/A_                     | `rune-vcs workspace`          | 🆕 Virtual workspaces          |
| _N/A_                     | `rune-vcs intelligence`       | 🆕 AI-powered analysis         |
| _N/A_                     | `rune-vcs doctor`             | 🆕 System diagnostics          |

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

**Note**: These commands are planned for future releases. Current version includes basic performance optimizations.

### Performance Statistics (Planned)

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

### Optimization (Planned)

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
- **Homebrew Tap**: https://github.com/Johan-Ott/homebrew-rune-vcs

### Community

- **Discussions**: GitHub Discussions
- **Stack Overflow**: Tag `rune-vcs`
- **Discord**: [Community Server]

---

## 🧭 Next Steps & Roadmap (Planned / Not Yet Implemented)

The following capabilities are referenced in vision, docs, or design notes but are NOT yet implemented in the current release (v0.3.0-alpha.4). They are candidates for upcoming alpha milestones.

### 1. Performance Guardrails & Commit Constraints
- Automatic enforcement of workspace performance limits (current: only manual workspace limits commands)
- Commit size / changed-file thresholds with warnings or blocks
- Time-to-index budget enforcement per operation
- Predictive prefetch + adaptive caching heuristics toggles

### 2. Policy-as-Code
- `rune-vcs policy` command group for listing, testing, and enforcing policies
- Configurable repository rules (branch protections, required labels, file path restrictions)
- Extensible policy engine (Rego / WASM based) for custom org rules
- Built-in conventional commit validation (currently only achievable via hooks)

### 3. Structured Changelog Generation
- `rune-vcs changelog generate` for Conventional Commit parsing
- Release note templating & grouping (feat/fix/perf/chore/breaking)
- Automatic linking to issues / PRs
- Ability to diff changelog between two refs

### 4. Impact-Based Test Selection
- `rune-vcs test` wrapper integrating dependency graph + file ↔ test mapping
- Change impact analyzer suggesting minimal test set
- Historical flake tracking & quarantine suggestions
- Coverage delta reporting per draft / commit

### 5. Intelligent Change Graph Visualization
- `rune-vcs graph` to render commit / draft / workspace dependency graph
- Hotspot & risk surface overlays (AI-assisted)
- Focus mode: collapse unrelated subgraphs
- Export to JSON / SVG

### 6. Supply Chain & Dependency Diff Scanning
- `rune-vcs deps scan` for third-party manifest inventory
- Version drift detection & security advisory cross-referencing
- SBOM generation (CycloneDX / SPDX)
- Upgrade impact simulation report

### 7. Pluggable Storage Backend Abstraction
- `rune-vcs storage` command for listing / switching backends
- Local FS, object store (S3/GCS), and embedded KV modes
- Content tiering (hot vs archival)
- Encryption at rest per backend

### 8. Encrypted Path Subsets
- `rune-vcs encrypt path/` to selectively encrypt directories
- Secure key management (local KMS plugin architecture)
- Policy requiring encryption for classified paths
- Auditable access attempts & decryption events

### 9. Additional Enterprise Features (Exploratory)
- SLA-aware replication scheduling
- Multi-tenant namespace isolation
- AI risk scoring for incoming patches
- Merge queue with automated validation pipeline

### Status Legend Proposal
(Will appear once features begin rolling out.)
- ⏳ Planned
- 🔨 In Progress
- 🧪 Experimental (flagged)
- ✅ Implemented

> Contributions or feedback on prioritization are welcome via Issues / Discussions.

---

## 📄 License

MIT License - see LICENSE file for details.

---

_Last updated: 2025-08-26_  
_Rune VCS version: 0.3.0-alpha.4_
