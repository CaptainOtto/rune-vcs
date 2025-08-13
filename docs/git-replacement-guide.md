# 🔄 Git to Rune Migration Guide

This guide shows you how to replace Git with Rune VCS for your development workflow, including repository management, collaboration, and large file handling.

## 🚀 Quick Start: Your First Rune Repository

### Basic Repository Operations

```bash
# Initialize a new repository (like 'git init')
mkdir my-project
cd my-project
rune init

# Check repository status (like 'git status')
rune status

# Add files to staging (like 'git add')
echo "Hello, Rune!" > README.md
rune add README.md
# Or add all files
rune add .

# Commit changes (like 'git commit')
rune commit -m "Initial commit"

# View commit history (like 'git log')
rune log
```

### Branch Management

```bash
# List branches (like 'git branch')
rune branch

# Create a new branch (like 'git branch feature-branch')
rune branch feature-branch

# Switch to a branch (like 'git checkout')
rune checkout feature-branch

# Work with stashing (like 'git stash')
rune stash          # Stash current changes
rune stash --apply  # Apply stashed changes
```

## 📁 Working with Large Files (Better than Git LFS)

Rune has built-in large file support that's much simpler than Git LFS:

```bash
# Track large file patterns
rune lfs track "*.psd"      # Photoshop files
rune lfs track "*.blend"    # Blender files
rune lfs track "*.mp4"      # Video files
rune lfs track "*.zip"      # Archive files

# Add and commit large files (just like normal files!)
cp ~/large-design.psd .
rune add large-design.psd
rune commit -m "Add design mockup"

# The large file is automatically chunked and stored efficiently
```

## 🔐 File Locking (Perforce-style)

Unlike Git, Rune supports file locking for binary assets that shouldn't be merged:

```bash
# Lock a file for exclusive editing
rune lfs lock --path design.psd --owner "alice@company.com"

# Work on the file...
# (Other team members can't edit it until unlocked)

# Unlock when done
rune lfs unlock --path design.psd --owner "alice@company.com"

# List all locked files
rune api --addr 127.0.0.1:7421 &
curl http://127.0.0.1:7421/v1/locks
```

## 🌐 Collaboration Setup

### Option 1: Embedded Mode (Easiest)

Start a local server that handles both version control and large files:

```bash
# Start server with both API and Shrine
rune api --with-shrine --addr 127.0.0.1:7421 --shrine-addr 127.0.0.1:7420

# Your repository is now accessible at:
# - API: http://127.0.0.1:7421
# - Shrine (LFS): http://127.0.0.1:7420
```

### Option 2: Separate Services

For production deployments:

```bash
# Terminal 1: Start API server
rune api --addr 0.0.0.0:7421

# Terminal 2: Start Shrine server (for large files)
rune shrine serve --addr 0.0.0.0:7420
```

### Team Configuration

Each team member configures their remote:

```bash
# Set up remote endpoints
export RUNE_API_URL="http://your-server:7421"
export RUNE_SHRINE_URL="http://your-server:7420"

# Push large files to shared storage
rune lfs push my-large-file.psd

# Pull large files from shared storage
rune lfs pull my-large-file.psd
```

## 📋 Common Git → Rune Command Mapping

| Git Command             | Rune Equivalent               | Notes                     |
| ----------------------- | ----------------------------- | ------------------------- |
| `git init`              | `rune init`                   | Same concept              |
| `git status`            | `rune status`                 | Cleaner output            |
| `git add <file>`        | `rune add <file>`             | Same syntax               |
| `git commit -m "msg"`   | `rune commit -m "msg"`        | Same syntax               |
| `git log`               | `rune log`                    | Better formatting options |
| `git branch`            | `rune branch`                 | Same concept              |
| `git checkout <branch>` | `rune checkout <branch>`      | Same syntax               |
| `git stash`             | `rune stash`                  | Simplified interface      |
| `git lfs track "*.psd"` | `rune lfs track "*.psd"`      | Built-in, no setup needed |
| N/A                     | `rune lfs lock --path <file>` | Unique to Rune            |

## 🏢 Real-World Workflow Examples

### Game Development Studio

```bash
# Set up a game project
mkdir awesome-game
cd awesome-game
rune init

# Track large asset files
rune lfs track "*.fbx"     # 3D models
rune lfs track "*.texture" # Textures
rune lfs track "*.wav"     # Audio files
rune lfs track "*.mp4"     # Cutscenes

# Daily workflow
rune add src/           # Add code changes
rune add assets/        # Add new assets (auto-handled by LFS)
rune commit -m "Add player movement system"

# Designer locks a file for editing
rune lfs lock --path assets/player-model.fbx --owner "designer@studio.com"
# ... edit file ...
rune lfs unlock --path assets/player-model.fbx --owner "designer@studio.com"
```

### Design Agency

```bash
# Set up client project
mkdir client-branding
cd client-branding
rune init

# Track design files
rune lfs track "*.psd"
rune lfs track "*.ai"
rune lfs track "*.sketch"
rune lfs track "*.fig"

# Designer workflow
rune lfs lock --path logo.psd --owner "alice@agency.com"
# Edit logo.psd in Photoshop
rune add logo.psd
rune commit -m "Update logo color scheme"
rune lfs unlock --path logo.psd --owner "alice@agency.com"

# Share with client
rune lfs push logo.psd  # Upload to shared storage
```

### Software Development Team

```bash
# Regular development workflow
rune add src/
rune commit -m "Implement user authentication"

# Working with documentation assets
rune lfs track "*.png"    # Screenshots
rune lfs track "*.gif"    # Demos
rune add docs/
rune commit -m "Add API documentation with examples"

# Feature branch workflow
rune branch feature-payments
rune checkout feature-payments
# ... develop feature ...
rune add .
rune commit -m "Add payment processing"
rune checkout main
# (merge functionality would be added in future versions)
```

## 🔧 Advanced Configuration

### Server Configuration

Create a `rune-config.toml` for production:

```toml
[server]
api_addr = "0.0.0.0:7421"
shrine_addr = "0.0.0.0:7420"
max_file_size = "500MB"

[lfs]
chunk_size = "64KB"
storage_path = "/var/rune/lfs"

[locks]
timeout_hours = 24
```

### Team Aliases

Add to your shell profile:

```bash
# ~/.bashrc or ~/.zshrc
alias rs="rune status"
alias ra="rune add"
alias rc="rune commit"
alias rl="rune log"
alias rb="rune branch"
alias rco="rune checkout"

# Large file shortcuts
alias rlt="rune lfs track"
alias rlp="rune lfs push"
alias rll="rune lfs pull"
alias rlock="rune lfs lock --owner $(whoami)@$(hostname)"
alias runlock="rune lfs unlock --owner $(whoami)@$(hostname)"
```

## 🚀 API Integration

Rune exposes a clean JSON API for custom tools:

```bash
# Start API server
rune api --addr 127.0.0.1:7421

# Get repository status
curl http://127.0.0.1:7421/v1/status

# Get commit history
curl http://127.0.0.1:7421/v1/log

# Check locked files
curl http://127.0.0.1:7421/v1/locks

# Lock a file via API
curl -X POST http://127.0.0.1:7421/v1/lock \
  -H "Content-Type: application/json" \
  -d '{"path": "design.psd", "owner": "alice@company.com"}'
```

## 📊 Performance Benefits

### Compared to Git:

- ✅ **Faster with large files** - chunked storage vs Git LFS complexity
- ✅ **Simpler setup** - no separate LFS configuration needed
- ✅ **Built-in locking** - prevents binary file conflicts
- ✅ **Cleaner commands** - less cognitive overhead

### Compared to Perforce:

- ✅ **Distributed** - works offline
- ✅ **Modern CLI** - familiar Git-like interface
- ✅ **Open source** - no licensing costs
- ✅ **JSON API** - easy integration

## 🔄 Migration from Existing Git Repository

```bash
# Backup your .git directory
cp -r .git .git.backup

# Initialize Rune in the same directory
rune init

# Add all existing files
rune add .
rune commit -m "Migrate from Git to Rune"

# Set up LFS for existing large files
find . -name "*.psd" -o -name "*.mp4" -o -name "*.zip" | head -10
rune lfs track "*.psd"
rune lfs track "*.mp4"
rune lfs track "*.zip"

# Re-add and commit to apply LFS tracking
rune add .
rune commit -m "Apply LFS tracking to large files"
```

## 🎯 Next Steps

1. **Try the examples above** in a test directory
2. **Set up your team server** using embedded mode
3. **Configure file tracking** for your project's large files
4. **Establish locking conventions** for binary assets
5. **Integrate with your CI/CD** using the JSON API

---

_For more advanced usage, see the other documentation files in the `docs/` directory._
