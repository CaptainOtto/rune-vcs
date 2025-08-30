# 🪄 Rune VCS

> A modern, minimal, and powerful distributed version control system for 2025 and beyond.  
> Hybrid Git + Perforce approach, built in Rust, designed for speed, simplicity, and large file handling.

![Rune Logo](docs/assets/rune-banner.png)

---

## ✨ Features

- **🎨 Professional Interface** – Beautiful, colored CLI output with helpful guidance
- **🔄 Hybrid Git + P4 Design** – Distributed workflow with Perforce-style file locking
- **📦 Built-in Large File Support** – No setup needed, chunked storage for huge assets
- **🌐 Cross-platform** – macOS, Windows, Linux with single binary deployment
- **🔧 Embedded Mode** – One command starts full team server with LFS + locks
- **🔌 JSON API** – Clean REST interface for custom tools and integrations
- **⚡ Fast & Minimal** – Essential commands only, no unnecessary complexity
- **🔒 File Locking** – Prevent binary asset conflicts like Perforce
- **📝 Git-Compatible Workflow** – Familiar commands: init, add, commit, branch, etc.

---

## 📦 Installation

```bash
# Clone the repo
git clone https://github.com/CaptainOtto/rune-vcs.git
cd rune-vcs

# Build (requires Rust)
cargo build --release

# Optional: Install globally
cargo install --path .
```

---

## 🚀 Quick Start

```bash
# Init new repo
rune init

# Track large files (built-in LFS)
rune lfs track "*.psd"
rune lfs track "*.mp4"

# Add and commit (just like Git!)
rune add .
rune commit -m "Initial commit"

# Start embedded server for team collaboration
rune api --with-shrine
```

### 🔄 Migrating from Git?

**[→ See our complete Git replacement guide](docs/git-replacement-guide.md)**

Common commands work the same:

- `git init` → `rune init`
- `git add .` → `rune add .`
- `git commit -m "msg"` → `rune commit -m "msg"`
- `git status` → `rune status`
- `git log` → `rune log`

**Plus built-in advantages:**

- ✅ **Large files work out-of-the-box** (no Git LFS setup)
- ✅ **File locking for binary assets** (like Perforce)
- ✅ **Professional colored output** with helpful guidance
- ✅ **One command starts team server** (embedded mode)
- ✅ **Clean JSON API** for integrations
- ✅ **Advanced file operations** (move, reset, show, diff)
- ✅ **Modern Rust architecture** for speed and reliability

---

## 📚 Documentation

- **[Git Replacement Guide](docs/git-replacement-guide.md)** - Complete migration walkthrough
- [Overview](docs/overview.md) - Core concepts and philosophy
- [CLI Commands](docs/cli-commands.md) - Full command reference
- [API Reference](docs/api-reference.md) - HTTP API documentation
- [LFS & Locks](docs/lfs-and-locks.md) - Large file handling
- [Embedded Mode](docs/embedded-mode.md) - Local server setup

🛠 Roadmap
GUI client (Rune Desktop) – minimal GitHub Desktop + P4 hybrid

Remote hosting service

Plugin system

Visual diff for large files

🤝 Contributing
We welcome issues, feature requests, and PRs!
See INSTRUCTIONS.md for current status and open tasks.

📜 License
MIT

```

```
