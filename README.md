# 🪄 Rune VCS

> A modern, minimal, and powerful distributed version control system for 2025 and beyond.  
> Hybrid Git + Perforce approach, built in Rust, designed for speed, simplicity, and large file handling.

![Rune Logo](docs/assets/rune-banner.png)

---

## ✨ Features

- **Hybrid Git + P4 design** – the flexibility of Git with Perforce-style file locking
- **Built-in Large File Support (LFS)** – chunked storage for huge assets
- **Cross-platform** – macOS, Windows, Linux
- **Embedded Mode** – one-command local server with LFS + locks
- **JSON API** – simple integration with custom UIs
- **Fast, minimal CLI** – no unnecessary complexity

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
🚀 Quick Start
bash
Copy
Edit
# Init new repo
rune init

# Track large files
rune lfs track "*.psd"

# Commit
rune add .
rune commit -m "Initial commit"

# Start embedded mode
rune api --with-shrine
📚 Documentation
Overview

CLI Commands

API Reference

LFS & Locks

Embedded Mode

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
