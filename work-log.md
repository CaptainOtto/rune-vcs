# Rune VCS – Development Instructions

## ✅ Completed for v0.0.1

- **Core CLI**: `init`, `status`, `add`, `commit`, `log`, `branch`, `checkout`, `stash`
- **Repo format**: `.rune/` directory, custom object store
- **Embedded Shrine Mode** – API + Shrine server in one process
- **Large File Support (LFS)** – pointer files, chunked upload, resumable
- **File Locking (P4-style)** – lock/unlock via CLI or API
- **Delta Compression** – efficient storage of binary diffs
- **JSON API** – `/v1/status`, `/v1/commit`, `/v1/log`, `/v1/branch`, `/lfs/*`, `/locks/*`
- **Shell completions** – bash, zsh, fish, powershell
- **Cross-platform** – macOS, Windows, Linux
- **Release Artifacts** – ready for Homebrew tap & Scoop bucket
- **Docs** – `README.md` (Getting Started), `RELEASE_NOTES_v0.0.1.md`, `CHECKLIST.md`
- **.editorconfig** & `.gitignore` added

---

## 🛠 Planned for v0.1.0

- Advanced merge & conflict resolution
- Auth + TLS for Shrine server
- More efficient object model (tree/blob graph like Git)
- Compression tuning for large binary packs
- Optional YAML/TOML API output
- Config profiles for different workflows (solo/team)
- Extended hooks system (pre-commit, post-merge)
- Improved error messages & CLI help UX
- Basic unit test coverage for all crates
- Benchmarks for commit & LFS performance

---

## 🚀 Build & Run

```bash
# Install CLI locally
cargo install --path crates/rune-cli

# Start in embedded mode (API + Shrine)
rune api --addr 127.0.0.1:7421 --with-shrine --shrine-addr 127.0.0.1:7420

# Init repo & test
mkdir demo && cd demo
rune init
curl http://127.0.0.1:7421/v1/status | jq
```
