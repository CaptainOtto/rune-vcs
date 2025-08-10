# Embedded Shrine Mode

Embedded Shrine mode runs the **API server** and the **Shrine server** in one process.  
This makes it easy to test Rune locally without setting up multiple services.

---

## Start Embedded Mode
```bash
rune api --addr 127.0.0.1:7421 --with-shrine --shrine-addr 127.0.0.1:7420
```

---

## Benefits
- Zero extra setup for local repos
- Supports LFS and locks instantly
- Same commands work with remote Shrine later

---

## Example Local Workflow
```bash
# Create test repo
mkdir demo && cd demo
rune init

# Start API + Shrine
rune api --with-shrine

# Track and push large file
rune lfs track "*.psd"
echo "bigdata" > test.psd
rune add test.psd
rune commit -m "PSD test"
rune lfs push test.psd
```
