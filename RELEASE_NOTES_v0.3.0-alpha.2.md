# Release Notes - v0.3.0-alpha.2

**Release Date:** December 2024  
**Focus:** Draft Commits & Checkpoint System

This alpha release introduces Rune's Draft Commit system, providing Perforce-style changelist functionality with improved safety and workflow management compared to traditional Git stash.

## 🚀 New Features

### Draft Commits & Checkpoints
- **Perforce-style Shelving** – Store work-in-progress changes as named drafts
- **Checkpoint System** – Automatic or manual snapshots with timestamps
- **Draft Management** – Create, apply, shelve, update, and delete drafts
- **Tag Organization** – Organize drafts with custom tags and filtering
- **Automatic Cleanup** – Configurable cleanup policies for old drafts

### Draft Management Commands
- `rune draft create <name>` – Create a new draft from working directory
- `rune draft list [--tags] [--active]` – List drafts with filtering options
- `rune draft apply <draft>` – Apply a draft to working directory
- `rune draft shelve <draft>` – Remove active draft from working directory
- `rune draft update <draft>` – Update draft with current changes
- `rune draft delete <draft>` – Permanently delete a draft
- `rune draft show <draft>` – Show detailed draft information
- `rune draft checkpoint [name]` – Create automatic checkpoint
- `rune draft cleanup [--keep-days N]` – Clean up old drafts
- `rune draft tag/untag <draft> <tags>` – Manage draft tags

### Advanced Features
- **JSON/Table Output** – Structured output formats for automation
- **Draft Resolution** – Resolve drafts by name or partial ID
- **Configuration Management** – Persistent settings for cleanup and defaults
- **Author Detection** – Automatic author information from environment
- **Branch Tracking** – Track which branch drafts were created from

## 🔧 Implementation Details

### New Crate: rune-draft
- Complete draft commit management system
- Persistent storage in `.rune/drafts/` directory
- JSON-based configuration and draft storage
- Integration with existing rune-store infrastructure

### CLI Integration
- New `draft` subcommand with comprehensive argument support
- Table and JSON output formats
- Interactive confirmation prompts with `--force` override
- Consistent styling with existing CLI commands

## 📚 Documentation Updates

### New Features Documented
- Draft workflow patterns and best practices
- Checkpoint strategies for different development styles
- Tag organization for team coordination
- Integration with existing VCS workflows

## 🧪 Testing

- **5 new draft tests** covering core functionality
- **Full test suite** – All 71 tests passing (66 existing + 5 new)
- **CLI command validation** – All draft commands working correctly
- **Configuration persistence** – Draft settings properly saved/loaded

## ⚠️ Breaking Changes

None. This is an additive release that maintains full backward compatibility.

## 🔄 Migration

No migration required. Existing repositories continue to work normally. To use draft commits:

```bash
# In existing repository
rune draft create my-feature-work --description "Work in progress on new feature"
rune draft list
```

## 🎯 Roadmap

This alpha release completes **Tier 1 - Draft/Checkpoint Commits** functionality from the v0.3.0 roadmap:

- ✅ Changelist-style shelved snapshots
- ✅ Named draft management with tagging
- ✅ Automatic checkpoint creation
- ✅ Configurable cleanup policies
- ✅ CLI integration with all management commands

### Next: v0.3.0-alpha.3
- Policy-as-code framework
- Repository rules as configuration
- Commit validation hooks

### Future: v0.4.0
- Semantic merge system
- AI-powered conflict resolution
- Intelligent change graph

## 🐛 Known Issues

- Draft file collection currently simplified (to be enhanced in future versions)
- Some unused code warnings (planned for future features)

## 👥 Contributors

- Implementation of complete draft commit system
- CLI command integration and testing
- Configuration management and persistence
- Full test suite validation

---

**Installation:**
```bash
# From source (recommended for alpha)
git clone <repository>
cd rune-vcs
cargo build --release
```

**Usage Examples:**
```bash
# Create a draft
rune draft create feature-work --description "Working on user authentication"

# List drafts
rune draft list --format json

# Apply a draft
rune draft apply feature-work

# Create checkpoint
rune draft checkpoint

# Clean up old drafts
rune draft cleanup --keep-days 7
```

**Next Steps:**
- Try draft commits in your development workflow
- Report issues and feedback
- Follow development for alpha.3 features
