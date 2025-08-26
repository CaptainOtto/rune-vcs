# Rune CLI Commands

## Basic Repository Commands

### `rune init`

Initialize a new repository in the current directory.

```bash
mkdir my-project
cd my-project
rune init
```

### `rune status [--format=<table|json|yaml>]`

Show changed, staged, and untracked files.

```bash
rune status              # Table format (default)
rune status --format=json # JSON output for scripting
```

### `rune add <files...>`

Stage file(s) for commit.

```bash
rune add README.md       # Add single file
rune add src/           # Add directory
rune add .              # Add everything
```

### `rune commit -m "<message>"`

Commit staged changes with a message.

```bash
rune commit -m "Add user authentication"
rune commit -m "Fix: resolve login bug #123"
```

### `rune log [--format=<table|json|yaml>]`

Show commit history.

```bash
rune log                 # Human-readable table
rune log --format=json   # For automation/parsing
```

### `rune branch [<name>] [--format=<table|json|yaml>]`

List branches or create a new one.

```bash
rune branch              # List all branches
rune branch feature-auth # Create new branch
```

### `rune checkout <branch>`

Switch to another branch.

```bash
rune checkout main
rune checkout feature-auth
```

### `rune stash [--apply]`

Stash or restore current changes.

```bash
rune stash          # Save current changes
rune stash --apply  # Restore stashed changes
```

## Large File Support (LFS)

### `rune lfs track "<pattern>"`

Track file patterns for LFS storage.

```bash
rune lfs track "*.psd"      # Photoshop files
rune lfs track "*.blend"    # Blender files
rune lfs track "*.mp4"      # Video files
rune lfs track "**/*.zip"   # Archives in any subdirectory
```

### `rune lfs untrack "<pattern>"`

Remove pattern from LFS tracking.

```bash
rune lfs untrack "*.psd"
```

### `rune lfs push <file>`

Upload large file chunks to remote Shrine.

```bash
rune lfs push design.psd
rune lfs push assets/video.mp4
```

### `rune lfs pull <file>`

Download large file chunks from remote Shrine.

```bash
rune lfs pull design.psd
rune lfs pull assets/video.mp4
```

### `rune lfs lock --path <file> --owner <email>`

Lock a file for exclusive editing.

```bash
rune lfs lock --path design.psd --owner alice@company.com
rune lfs lock --path assets/model.fbx --owner bob@studio.com
```

### `rune lfs unlock --path <file> --owner <email>`

Unlock a file.

```bash
rune lfs unlock --path design.psd --owner alice@company.com
```

## Server/API Commands

### `rune api --addr <host:port>`

Start API server only.

```bash
rune api --addr 127.0.0.1:7421
rune api --addr 0.0.0.0:8080  # Accept external connections
```

### `rune api --with-shrine --addr <api_addr> --shrine-addr <shrine_addr>`

Start API + Shrine in embedded mode.

```bash
# Start both services locally
rune api --with-shrine --addr 127.0.0.1:7421 --shrine-addr 127.0.0.1:7420

# Production setup
rune api --with-shrine --addr 0.0.0.0:7421 --shrine-addr 0.0.0.0:7420
```

### `rune shrine serve --addr <host:port>`

Start Shrine server for large files.

```bash
rune shrine serve --addr 127.0.0.1:7420
rune shrine serve --addr 0.0.0.0:7420
```

## Utility Commands

### `rune completions <shell>`

Generate shell completion scripts.

```bash
rune completions bash > /etc/bash_completion.d/rune
rune completions zsh > ~/.zsh/completions/_rune
rune completions fish > ~/.config/fish/completions/rune.fish
```

### `rune guide`

Show built-in user guide.

```bash
rune guide
```

## Examples by Use Case

### Game Development Team

```bash
# Set up project
rune init
rune lfs track "*.fbx"      # 3D models
rune lfs track "*.texture"  # Textures
rune lfs track "*.wav"      # Audio
rune lfs track "*.mp4"      # Cutscenes

# Daily workflow
rune add src/              # Code changes
rune add assets/           # New assets (auto-LFS)
rune commit -m "Add player movement"

# Designer locks asset
rune lfs lock --path assets/player.fbx --owner designer@studio.com
# ... edit file ...
rune add assets/player.fbx
rune commit -m "Update player model"
rune lfs unlock --path assets/player.fbx --owner designer@studio.com
```

## Virtual Workspace Management

### `rune workspace init [name]`

Initialize a virtual workspace with sparse checkout.

```bash
rune workspace init mobile-app    # Named workspace
rune workspace init               # Use current directory name
```

### `rune workspace add-root <path> <pattern>`

Add a virtual root to focus on specific directories.

```bash
rune workspace add-root frontend "src/web/**"     # Web frontend only
rune workspace add-root backend "src/api/**"      # API backend only
rune workspace add-root mobile "src/mobile/**"    # Mobile app only
```

### `rune workspace toggle <name>`

Toggle a virtual root on/off.

```bash
rune workspace toggle frontend    # Switch on/off
```

### `rune workspace list`

Show configured virtual roots and their status.

```bash
rune workspace list
```

### `rune workspace validate`

Check workspace configuration for issues.

```bash
rune workspace validate
```

### `rune workspace limits [--max-files <n>] [--max-size <size>]`

Configure or view performance limits.

```bash
rune workspace limits                         # Show current limits
rune workspace limits --max-files 5000       # Set file limit
rune workspace limits --max-size 100MB       # Set size limit
```

### Design Agency

```bash
# Set up client project
rune init
rune lfs track "*.psd"     # Photoshop
rune lfs track "*.ai"      # Illustrator
rune lfs track "*.sketch"  # Sketch
rune lfs track "*.fig"     # Figma exports

# Collaborative editing
rune lfs lock --path logo.psd --owner alice@agency.com
# ... edit in Photoshop ...
rune add logo.psd
rune commit -m "Update brand colors"
rune lfs unlock --path logo.psd --owner alice@agency.com
```

### Software Development

```bash
# Regular development
rune add src/
rune commit -m "Implement API endpoints"

# Documentation with screenshots
rune lfs track "*.png"     # Screenshots
rune lfs track "*.gif"     # Demo animations
rune add docs/
rune commit -m "Add API documentation"
```
