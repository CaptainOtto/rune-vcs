/// Embedded documentation content
/// This content is compiled into the binary for offline access

pub const GETTING_STARTED: &str = r#"
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "content/"]
pub struct ContentAssets;

/// Get embedded documentation content by name
pub fn get_content(name: &str) -> Option<String> {
    let filename = format!("{}.md", name);
    ContentAssets::get(&filename)
        .and_then(|file| String::from_utf8(file.data.into_owned()).ok())
}

/// List all available content files
pub fn list_content() -> Vec<String> {
    ContentAssets::iter()
        .map(|file| file.strip_suffix(".md").unwrap_or(&file).to_string())
        .collect()
}
"#;

pub const COMMANDS: &str = r#"
# Command Reference

Complete reference for all Rune VCS commands.

## Core Commands

### rune init
Initialize a new Rune repository in the current directory.

**Usage:**
```bash
rune init [options]
```

**Options:**
- `--bare` - Create a bare repository (no working directory)
- `--template <template>` - Use a specific project template

**Examples:**
```bash
rune init                    # Initialize in current directory
rune init --template rust    # Initialize with Rust template
```

### rune add
Add files to the staging area for the next commit.

**Usage:**
```bash
rune add <files>...
```

**Options:**
- `-A, --all` - Add all files (including deleted)
- `-u, --update` - Add only modified tracked files
- `-p, --patch` - Interactively choose hunks to add

**Examples:**
```bash
rune add .                   # Add all files in current directory
rune add src/ tests/         # Add specific directories
rune add --all               # Add all changes including deletions
```

### rune commit
Create a new commit with staged changes.

**Usage:**
```bash
rune commit [options]
```

**Options:**
- `-m, --message <msg>` - Commit message
- `-a, --all` - Automatically stage all modified files
- `--amend` - Amend the previous commit

**Examples:**
```bash
rune commit -m "Fix bug in parser"
rune commit -a -m "Update documentation"
rune commit --amend         # Fix the last commit
```

### rune status
Show the working tree status.

**Usage:**
```bash
rune status [options]
```

**Options:**
- `-s, --short` - Give output in short format
- `-b, --branch` - Show branch information

### rune log
Show commit history.

**Usage:**
```bash
rune log [options]
```

**Options:**
- `--oneline` - Show each commit on a single line
- `-n <number>` - Limit number of commits to show
- `--graph` - Show ASCII art commit graph

## Branching Commands

### rune branch
List, create, or delete branches.

**Usage:**
```bash
rune branch [options] [<branch-name>]
```

**Options:**
- `-d, --delete <branch>` - Delete a branch
- `-m, --move <old> <new>` - Rename a branch
- `-r, --remote` - List remote branches

### rune checkout
Switch branches or restore files.

**Usage:**
```bash
rune checkout <branch>
rune checkout -b <new-branch>
```

**Options:**
- `-b, --branch` - Create and switch to new branch
- `-f, --force` - Force checkout (discard local changes)

### rune merge
Merge branches.

**Usage:**
```bash
rune merge <branch>
```

**Options:**
- `--no-ff` - Always create a merge commit
- `--squash` - Squash all commits into one

## File Operations

### rune diff
Show differences between commits, commit and working tree, etc.

**Usage:**
```bash
rune diff [options] [<files>...]
```

**Options:**
- `--cached` - Show differences between index and HEAD
- `--stat` - Show only statistics

### rune reset
Reset current HEAD to the specified state.

**Usage:**
```bash
rune reset [options] [<files>...]
```

**Options:**
- `--soft` - Reset only HEAD
- `--mixed` - Reset HEAD and index (default)
- `--hard` - Reset HEAD, index, and working tree

### rune show
Show various types of objects.

**Usage:**
```bash
rune show [<commit>]
```

## Remote Operations

### rune clone
Clone a repository.

**Usage:**
```bash
rune clone <url> [<directory>]
```

### rune pull
Fetch and merge changes from remote.

**Usage:**
```bash
rune pull [<remote>] [<branch>]
```

### rune push
Push changes to remote repository.

**Usage:**
```bash
rune push [<remote>] [<branch>]
```

### rune fetch
Download objects and refs from remote.

**Usage:**
```bash
rune fetch [<remote>]
```

## Ignore System

### rune ignore
Manage ignore patterns with advanced features.

**Usage:**
```bash
rune ignore <subcommand>
```

**Subcommands:**
- `check <files>...` - Check if files would be ignored
- `add <pattern>` - Add ignore pattern
- `list` - List all ignore rules
- `templates` - Show available templates
- `init` - Initialize ignore configuration

## Utility Commands

### rune help
Show help information.

**Usage:**
```bash
rune help [<command>]
```

### rune version
Show version information.

**Usage:**
```bash
rune version
```

### rune docs
Open documentation.

**Usage:**
```bash
rune docs [options]
```

**Options:**
- `--serve` - Start local documentation server
- `--port <port>` - Specify port for server

### rune examples
Show example workflows.

**Usage:**
```bash
rune examples [<category>]
```

### rune tutorial
Start interactive tutorial.

**Usage:**
```bash
rune tutorial [<section>]
```
"#;

pub const MIGRATION_FROM_GIT: &str = r#"
# Migration from Git

Moving from Git to Rune is straightforward. This guide covers the differences and helps you transition smoothly.

## Philosophy Differences

### Git Philosophy
- Complex but powerful
- Many ways to do the same thing
- Steep learning curve
- Command syntax can be inconsistent

### Rune Philosophy
- Intuitive and powerful
- One clear way to do each task
- Gentle learning curve
- Consistent command patterns

## Command Mapping

### Basic Operations

| Git | Rune | Notes |
|-----|------|-------|
| `git init` | `rune init` | Identical functionality |
| `git clone <url>` | `rune clone <url>` | Same syntax, better progress |
| `git add <files>` | `rune add <files>` | Same functionality |
| `git commit -m "msg"` | `rune commit -m "msg"` | Enhanced commit experience |
| `git status` | `rune status` | More colorful and informative |
| `git log` | `rune log` | Better default formatting |
| `git diff` | `rune diff` | Improved diff display |

### Branching

| Git | Rune | Notes |
|-----|------|-------|
| `git branch` | `rune branch` | Same functionality |
| `git checkout <branch>` | `rune checkout <branch>` | Safer with confirmations |
| `git checkout -b <branch>` | `rune checkout -b <branch>` | Identical |
| `git merge <branch>` | `rune merge <branch>` | Better conflict resolution |

### Remote Operations

| Git | Rune | Notes |
|-----|------|-------|
| `git pull` | `rune pull` | Simplified workflow |
| `git push` | `rune push` | Enhanced push validation |
| `git fetch` | `rune fetch` | Same functionality |

## Ignore Files

### Git (.gitignore)
```
# Simple patterns only
*.log
build/
node_modules/
```

### Rune (.runeignore.yml)
```yaml
version: "1.0"
global:
  - pattern: "**/*.log"
    rule_type: Ignore
    priority: 80
    description: "Log files"
project:
  - pattern: "build/"
    rule_type: Ignore
    priority: 50
    description: "Build output"
templates:
  - rust
  - node
```

**Rune Advantages:**
- **Priority system**: Resolve conflicts clearly
- **Auto-detection**: Templates applied automatically
- **Debug mode**: Understand why files are ignored
- **Better patterns**: More powerful pattern matching

## Configuration

### Git Config
```bash
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
git config --global init.defaultBranch main
```

### Rune Config
Rune auto-detects most settings, but you can configure:
```bash
rune config set user.name "Your Name"
rune config set user.email "you@example.com"
rune config set default.branch main
```

## Workflow Differences

### Git Workflow
```bash
# Git requires more explicit commands
git status                    # Check status
git add .                     # Stage files
git status                    # Check staged files
git commit -m "message"       # Commit
git push origin main          # Push
```

### Rune Workflow
```bash
# Rune provides better guidance
rune status                   # Status with helpful suggestions
rune add .                    # Stage with smart defaults
rune commit -m "message"      # Commit with validation
rune push                     # Push with smart defaults
```

## Enhanced Features

### Ignore System
```bash
# Git
echo "*.log" >> .gitignore

# Rune
rune ignore add "**/*.log" --description "Log files" --priority 80
rune ignore check suspicious_file.log  # Check if ignored
rune ignore list                        # See all rules
```

### Help System
```bash
# Git
git help commit               # Opens man page or browser

# Rune
rune help commit              # Inline help with examples
rune examples branching       # See workflow examples
rune tutorial                 # Interactive learning
```

### Error Messages
**Git:**
```
error: pathspec 'nonexistent' did not match any file(s) known to git
```

**Rune:**
```
❌ Error: File 'nonexistent' not found
💡 Suggestion: Use 'rune status' to see available files
💡 Or use 'rune add .' to add all files
```

## Migration Steps

### 1. Install Rune
```bash
# Using cargo
cargo install rune-cli

# Using package manager
brew install rune            # macOS
scoop install rune          # Windows
```

### 2. Migrate Existing Repository
```bash
# Option 1: Convert in place
cd my-git-repo
rune init                    # Initialize Rune in Git repo
rune status                  # See what needs to be staged

# Option 2: Fresh start
rune clone /path/to/git/repo my-rune-repo
cd my-rune-repo
rune ignore init             # Set up smart ignore patterns
```

### 3. Update Ignore Patterns
```bash
# Convert .gitignore to smart ignore
rune ignore init             # Auto-detect and create templates
rune ignore list             # Review generated rules

# Add custom patterns
rune ignore add "**/*.bak" --description "Backup files"
```

### 4. Update Workflows
Update your scripts and documentation to use `rune` instead of `git` commands.

## Common Gotchas

### 1. Different Default Behavior
- **Git**: Often requires explicit flags
- **Rune**: Smart defaults with confirmations

### 2. Enhanced Safety
- **Git**: Easy to lose work with wrong commands
- **Rune**: Confirmation prompts for destructive operations

### 3. Better Performance
- **Rune**: Operations are generally faster due to optimizations
- Some operations may have different output formats

## Getting Help

If you're stuck during migration:

```bash
rune help                    # Overview of all commands
rune help <command>          # Detailed help for specific command
rune examples migration      # Migration-specific examples
rune tutorial               # Interactive tutorial
rune docs                   # Full documentation
```

## Why Migrate?

### Performance
- Faster status checks
- Optimized storage
- Better handling of large repositories

### Usability
- Clearer error messages
- Better default behavior
- Helpful suggestions

### Features
- Smart ignore system
- Built-in documentation
- Interactive tutorials
- Enhanced conflict resolution

### Consistency
- Uniform command patterns
- Predictable behavior
- Less cognitive load
"#;

pub const BEST_PRACTICES: &str = r#"
# Best Practices

Guidelines for using Rune VCS effectively and maintaining high-quality repositories.

## Repository Organization

### Directory Structure
```
project/
├── .rune/                  # Rune metadata (don't touch)
├── .runeignore.yml         # Ignore configuration
├── README.md               # Project documentation
├── src/                    # Source code
├── tests/                  # Test files
├── docs/                   # Documentation
└── scripts/                # Build/utility scripts
```

### File Naming
- Use clear, descriptive names
- Follow your language's conventions
- Avoid spaces in filenames
- Use lowercase with hyphens for scripts

## Commit Best Practices

### Commit Messages
Follow the conventional commit format:

```
type(scope): short description

Longer explanation if needed.

- Bullet points for details
- Reference issues: Fixes #123
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples:**
```bash
rune commit -m "feat(auth): add OAuth2 integration"
rune commit -m "fix(parser): handle empty input correctly"
rune commit -m "docs: update installation guide"
```

### Commit Frequency
- Commit early and often
- Each commit should be a logical unit
- Don't commit broken code to main branch
- Use feature branches for experimental work

### Atomic Commits
Each commit should:
- Have a single purpose
- Be the smallest meaningful change
- Pass all tests
- Be reversible

## Branching Strategy

### Branch Naming
```
main                        # Stable, deployable code
develop                     # Integration branch
feature/user-authentication # New features
fix/login-bug              # Bug fixes
hotfix/security-patch      # Critical fixes
release/v1.2.0             # Release preparation
```

### Workflow
1. **Create feature branches from main**
   ```bash
   rune checkout main
   rune pull
   rune checkout -b feature/new-feature
   ```

2. **Work on your feature**
   ```bash
   # Make changes
   rune add .
   rune commit -m "feat: implement new feature"
   ```

3. **Keep branch updated**
   ```bash
   rune checkout main
   rune pull
   rune checkout feature/new-feature
   rune merge main
   ```

4. **Merge when complete**
   ```bash
   rune checkout main
   rune merge feature/new-feature
   rune branch -d feature/new-feature
   ```

## Ignore Patterns

### Use Smart Ignore System
```bash
# Initialize with project templates
rune ignore init

# Add custom patterns
rune ignore add "**/*.log" --description "Log files"
rune ignore add "tmp/" --description "Temporary directory"

# Check patterns
rune ignore check suspicious_file.log
rune ignore list
```

### Common Patterns
```yaml
# In .runeignore.yml
global:
  - pattern: "**/.DS_Store"      # macOS files
  - pattern: "**/Thumbs.db"      # Windows files
  - pattern: "**/*.tmp"          # Temporary files
  - pattern: "**/*.bak"          # Backup files

project:
  - pattern: "build/"            # Build output
  - pattern: "dist/"             # Distribution files
  - pattern: "node_modules/"     # Dependencies
  - pattern: "*.log"             # Log files
```

## Code Review

### Before Requesting Review
```bash
# Check status
rune status

# Review your changes
rune diff

# Ensure tests pass
rune status                     # Check for conflicts
```

### Review Checklist
- [ ] Clear commit messages
- [ ] No unnecessary files committed
- [ ] Tests are included
- [ ] Documentation is updated
- [ ] No merge conflicts

## Performance Tips

### Repository Maintenance
```bash
# Check repository health
rune status --verbose

# Optimize repository (planned feature)
rune optimize

# Clean up old branches
rune branch -d merged-feature-branch
```

### Large Files
- Use Rune's LFS (Large File Storage) for binary files
- Keep repositories focused (separate data from code)
- Consider submodules for shared components

## Security

### Sensitive Data
```bash
# Check for secrets before committing
rune ignore add "**/*.env" --description "Environment files"
rune ignore add "**/secrets.yml" --description "Secret configuration"

# Use environment variables for secrets
echo "API_KEY=your-key-here" > .env
echo ".env" >> .runeignore.yml
```

### Signed Commits (planned)
```bash
# Configure GPG signing (future feature)
rune config set commit.gpgsign true
rune config set user.signingkey YOUR_GPG_KEY
```

## Collaboration

### Team Guidelines
1. **Agree on branching strategy**
2. **Set up ignore patterns for your stack**
3. **Document commit message conventions**
4. **Regular code reviews**
5. **Keep main branch stable**

### Communication
```bash
# Use descriptive commit messages
rune commit -m "fix(api): handle timeout errors gracefully

- Add retry logic with exponential backoff
- Log timeout events for monitoring
- Fixes #123"
```

## Automation

### Pre-commit Hooks (planned)
```bash
# Set up pre-commit validation
rune hooks install

# Run tests before commit
rune config set hooks.pre-commit "cargo test"
```

### CI/CD Integration
```yaml
# Example GitHub Actions
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Rune
        run: cargo install rune-cli
      - name: Run tests
        run: rune status && cargo test
```

## Troubleshooting

### Common Issues
```bash
# Stuck in merge conflict
rune status                     # See conflicted files
rune diff                       # Review conflicts
# Edit files to resolve conflicts
rune add .
rune commit -m "resolve merge conflicts"

# Accidentally committed wrong files
rune reset --soft HEAD~1        # Undo last commit, keep changes
rune reset --hard HEAD~1        # Undo last commit, discard changes

# Need to change last commit message
rune commit --amend -m "new message"
```

### Getting Help
```bash
rune help                       # General help
rune help <command>             # Command-specific help
rune examples troubleshooting   # Common solutions
rune tutorial                   # Interactive learning
rune docs                       # Full documentation
```

## Quality Metrics

### Repository Health
- Commit frequency (regular small commits)
- Branch lifecycle (short-lived feature branches)
- Test coverage (maintain high coverage)
- Documentation coverage (document public APIs)

### Team Metrics
- Code review participation
- Commit message quality
- Branch naming consistency
- Ignore pattern effectiveness

## Continuous Improvement

### Regular Reviews
- Weekly: Review branch cleanup
- Monthly: Audit ignore patterns
- Quarterly: Review branching strategy
- Annually: Evaluate workflow effectiveness

### Stay Updated
```bash
# Check for Rune updates
rune version
rune update                     # Auto-update (planned)

# Review new features
rune docs --changelog           # See what's new
```
"#;

pub const TROUBLESHOOTING: &str = r#"
# Troubleshooting Guide

Common issues and their solutions when using Rune VCS.

## Installation Issues

### Command Not Found
**Problem:** `rune: command not found`

**Solution:**
```bash
# Check if Rune is installed
which rune

# If not installed, install via cargo
cargo install rune-cli

# Or use package manager
brew install rune              # macOS
scoop install rune            # Windows

# Add to PATH if needed
export PATH="$HOME/.cargo/bin:$PATH"
```

### Version Conflicts
**Problem:** Old version of Rune

**Solution:**
```bash
# Check current version
rune version

# Update to latest
cargo install rune-cli --force

# Verify update
rune version
```

## Repository Issues

### Repository Not Found
**Problem:** `Not a rune repository`

**Solution:**
```bash
# Check if you're in the right directory
pwd
ls -la                         # Look for .rune/ directory

# If not a repository, initialize
rune init

# Or navigate to the correct directory
cd /path/to/your/project
```

### Corrupted Repository
**Problem:** Repository appears corrupted

**Solution:**
```bash
# Check repository integrity
rune status --verbose

# Try to recover
rune check                     # Planned feature

# Last resort: re-clone if you have a remote
cd ..
rune clone <remote-url> <new-directory>
```

## Staging and Commit Issues

### Files Not Staging
**Problem:** `rune add` doesn't seem to work

**Solution:**
```bash
# Check current status
rune status

# Verify file exists
ls -la <filename>

# Check if file is ignored
rune ignore check <filename>

# Force add if ignored (not recommended)
rune add <filename> --force    # Planned feature

# Check for typos in filename
rune add <correct-filename>
```

### Empty Commits
**Problem:** "Nothing to commit"

**Solution:**
```bash
# Check what's staged
rune status

# Check if changes were made
rune diff

# Stage changes if needed
rune add .

# Or commit with --allow-empty for milestone commits
rune commit --allow-empty -m "milestone: project setup complete"
```

### Commit Message Issues
**Problem:** Commit rejected due to message format

**Solution:**
```bash
# Use proper commit message format
rune commit -m "type(scope): description"

# Examples of good messages
rune commit -m "feat(auth): add OAuth2 support"
rune commit -m "fix(parser): handle null input"
rune commit -m "docs: update README installation steps"

# For longer messages
rune commit  # Opens editor for multi-line message
```

## Branching Issues

### Branch Not Found
**Problem:** `branch 'feature' not found`

**Solution:**
```bash
# List all branches
rune branch

# List remote branches too
rune branch -r

# Create the branch if it doesn't exist
rune checkout -b feature

# Or checkout from remote
rune checkout -b feature origin/feature
```

### Cannot Switch Branches
**Problem:** "Cannot checkout, uncommitted changes"

**Solution:**
```bash
# Check what's uncommitted
rune status

# Option 1: Commit changes
rune add .
rune commit -m "wip: work in progress"

# Option 2: Stash changes (planned feature)
rune stash
rune checkout other-branch
rune stash pop

# Option 3: Discard changes (DANGEROUS)
rune reset --hard HEAD
```

### Merge Conflicts
**Problem:** Merge conflicts when merging branches

**Solution:**
```bash
# Check conflict status
rune status

# View conflicted files
rune diff

# Edit conflicted files manually
# Look for conflict markers:
# <<<<<<< HEAD
# Your changes
# =======
# Other changes
# >>>>>>> branch-name

# After resolving conflicts
rune add <resolved-files>
rune commit -m "resolve merge conflicts"
```

## Ignore System Issues

### Files Still Tracked Despite Ignore
**Problem:** Files appear in `rune status` even though they should be ignored

**Solution:**
```bash
# Check ignore status
rune ignore check <filename>

# Files already tracked need to be untracked first
rune reset HEAD <filename>
rm <filename>               # Remove from working directory
rune commit -m "remove tracked file"

# Then add to ignore
rune ignore add <pattern>
```

### Ignore Patterns Not Working
**Problem:** Ignore patterns don't seem to match files

**Solution:**
```bash
# Debug ignore patterns
rune ignore check --debug <filename>

# Check pattern syntax
rune ignore list

# Common pattern issues:
# Wrong: "*.log"           (only current directory)
# Right: "**/*.log"        (all directories)

# Update pattern
rune ignore add "**/*.log" --description "All log files"
```

### Template Not Applied
**Problem:** Project template not auto-detected

**Solution:**
```bash
# Check available templates
rune ignore templates

# Manually apply template
rune ignore apply <template>

# Or reinitialize ignore system
rune ignore init --force
```

## Performance Issues

### Slow Operations
**Problem:** Rune commands are slow

**Solution:**
```bash
# Check repository size
du -sh .rune/

# Check for large files
find . -type f -size +10M

# Use LFS for large files (planned feature)
rune lfs track "*.bin"

# Optimize repository (planned feature)
rune optimize
```

### High Memory Usage
**Problem:** Rune uses too much memory

**Solution:**
```bash
# Check repository statistics
rune status --verbose

# Large repositories may need configuration
rune config set performance.maxMemory 1G

# Consider splitting large repositories
# Use submodules for shared components
```

## Network Issues

### Clone Failures
**Problem:** `rune clone` fails

**Solution:**
```bash
# Check network connectivity
ping github.com

# Verify URL
rune clone <correct-url>

# For SSH issues
ssh -T git@github.com

# Use HTTPS instead of SSH
rune clone https://github.com/user/repo.git
```

### Push/Pull Failures
**Problem:** Cannot push or pull

**Solution:**
```bash
# Check remote configuration
rune remote -v               # Planned feature

# Verify credentials
# For HTTPS: ensure token/password is correct
# For SSH: ensure SSH key is configured

# Try with verbose output
rune push --verbose

# Force push if needed (DANGEROUS)
rune push --force           # Only if you're sure
```

## Documentation Issues

### Help Not Working
**Problem:** `rune help` doesn't show information

**Solution:**
```bash
# Try different help formats
rune help
rune help <command>
rune --help

# Access documentation
rune docs

# If docs server fails
rune docs --port 8080       # Try different port
```

### Examples Not Loading
**Problem:** `rune examples` doesn't work

**Solution:**
```bash
# Check specific category
rune examples branching

# List all categories
rune examples

# Access via docs
rune docs
```

## Configuration Issues

### Config Not Found
**Problem:** Configuration settings not persisting

**Solution:**
```bash
# Check config location
rune config --show-location  # Planned feature

# Set configuration explicitly
rune config set user.name "Your Name"
rune config set user.email "you@example.com"

# Verify settings
rune config list            # Planned feature
```

## Getting Additional Help

### Debug Information
```bash
# Get detailed debug output
rune status --verbose
rune --debug <command>      # Planned feature

# Check environment
echo $RUNE_CONFIG_DIR       # If set
echo $PATH                  # Ensure rune is in PATH
```

### Reporting Bugs
If you encounter a bug:

1. **Reproduce the issue** with minimal steps
2. **Gather information:**
   ```bash
   rune version
   uname -a                 # System information
   pwd                      # Current directory
   ```
3. **Check existing issues** on GitHub
4. **Create detailed bug report** with:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - System information
   - Debug output

### Community Support
- **Documentation**: `rune docs`
- **Examples**: `rune examples troubleshooting`
- **Tutorial**: `rune tutorial`
- **GitHub Issues**: Report bugs and feature requests
- **Discussions**: Community support (planned)

### Emergency Recovery

#### Lost Work
```bash
# Check reflog (planned feature)
rune reflog

# Recover deleted branch
rune branch recover <branch-name>

# Recover uncommitted changes (if possible)
rune fsck --lost-found      # Planned feature
```

#### Repository Reset
```bash
# Nuclear option: start fresh (DANGEROUS)
rm -rf .rune/
rune init
# You'll lose all history!
```

## Prevention Tips

### Regular Maintenance
```bash
# Regular health checks
rune status
rune check                  # Planned feature

# Keep ignore patterns updated
rune ignore list
rune ignore optimize        # Planned feature

# Regular backups
rune push                   # Push to remote regularly
```

### Best Practices
- Commit early and often
- Use descriptive commit messages
- Keep repositories focused and small
- Regular code reviews
- Maintain good ignore patterns
- Keep documentation updated

Remember: When in doubt, use `rune help <command>` or `rune docs` for detailed information.
"#;
