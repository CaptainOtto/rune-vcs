
# Rune VCS (v0.0.1)

Modern, minimal DVCS with big-file support and locks — **Git+P4 hybrid, but simpler**.

## TL;DR Getting Started
```bash
# Build & install (places 'rune' in PATH)
cargo install --path crates/rune-cli

# New repo
mkdir demo && cd demo
rune init

# Start everything in one go (API + Shrine)
rune api --addr 127.0.0.1:7421 --with-shrine --shrine-addr 127.0.0.1:7420

# Another terminal: test
curl http://127.0.0.1:7421/v1/status | jq

# LFS quick test
rune lfs config --remote http://127.0.0.1:7420
rune lfs track "*.psd"
echo "bigdata" > Art/highres.psd
rune lfs clean Art/highres.psd
rune lfs push Art/highres.psd
```

---

# Rune VCS — v0.0.1

Now with:
- **Large files (LFS-like)** — pointers + chunked storage.
- **P4-style locking** — via lightweight shrine server.
- **Delta compression** — binary diff (copy/insert ops) + resumable LFS uploads (server checks existing chunks).
- **Basic pack (zstd)** groundwork.

No GUI yet — CLI only.

## Quick demo
```bash
cargo build --release -p rune-cli
./target/release/rune guide
```


## Install (once you publish a release)

### Homebrew (macOS + Linux)
```bash
brew tap CaptainOtto/tap
brew install rune
```

### Scoop (Windows)
```powershell
scoop bucket add rune https://github.com/CaptainOtto/scoop-bucket
scoop install rune
```

### Cargo (from source)
```bash
cargo install --path crates/rune-cli
```


### Shell completions
- Bash: `eval "$(rune completions bash)"` (tillfälligt), Brew installerar permanent.
- Zsh: `eval "$(rune completions zsh)"` (tillfälligt), Brew lägger i `_rune`.
- Fish: `rune completions fish | source`, Brew lägger i rätt mapp.
- PowerShell: Scoop-manifestet försöker dot-sourca `rune.ps1` i `$PROFILE`.


### JSON API (lokal)
```bash
rune api --addr 127.0.0.1:7421
# i ett annat fönster
curl http://127.0.0.1:7421/v1/status | jq
curl http://127.0.0.1:7421/v1/log | jq
curl -X POST http://127.0.0.1:7421/v1/commit -H 'content-type: application/json' -d '{"message":"via api"}'
```
`--format json|yaml|table` finns för `status`, `log`, `branch` i CLI.
Konfig: `.rune/config.toml`.


### API endpoints
- Core: `GET /v1/status`, `GET /v1/log`, `POST /v1/commit`, `POST /v1/stage`, `GET /v1/branches`, `POST /v1/branch`, `POST /v1/checkout`
- LFS: `POST /v1/lfs/track`, `POST /v1/lfs/clean`, `POST /v1/lfs/smudge`, `POST /v1/lfs/push`, `POST /v1/lfs/pull`
- Locks: `GET /v1/locks`, `POST /v1/lock`, `POST /v1/unlock`

Alla endpoints är JSON.


### Embedded Shrine-läge (enklast)
Kör API och Shrine i **en** process:
```bash
rune api --addr 127.0.0.1:7421 --with-shrine --shrine-addr 127.0.0.1:7420

# Sen kan du använda LFS/locks via API:t eller CLI:t:
rune lfs config --remote http://127.0.0.1:7420
rune lfs push path/to/large.psd
```
Om du vill köra Shrine separat i team: `rune shrine serve --addr 0.0.0.0:7420`
