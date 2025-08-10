
# Rune VCS v0.0.1 — First public MVP

## Highlights
- Own repo format (`.rune/`), no Git dependency
- Core CLI: init, status, add, commit, log, branch, checkout, stash
- Large Files (LFS-like): pointer files, chunked storage, resumable uploads
- Shrine: file-locking (P4-style) + LFS endpoints
- **Embedded Shrine mode**: run API + Shrine in one process for easy local testing
- JSON API for UI integration (`/v1/status`, `/v1/log`, `/v1/commit`, `/v1/stage`, branches, LFS, locks)
- Delta compression crate (copy/insert ops) + CLI
- Pack groundwork (zstd)
- Shell completions (bash/zsh/fish/powershell)
- Multi-OS CI + Homebrew/Scoop templates

## Known limitations
- Simple object model (no full tree/blob graph yet)
- Basic merge; advanced merge/resolve not included
- No auth/TLS on Shrine (dev-only default)

## Upgrade notes
- Config is TOML at `.rune/config.toml`
- API is JSON; CLI supports `--format json|yaml|table`
