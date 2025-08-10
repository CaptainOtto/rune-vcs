# Large File Support & Locks in Rune

## Large File Support (LFS)
Rune LFS stores large files outside of the main commit objects as *pointer files*.  
This keeps repository size small and makes operations faster.

Tracked patterns are stored in `.rune/lfsconfig`.

### Basic Workflow
```bash
rune lfs track "*.psd"
rune add .rune/lfsconfig
rune commit -m "Track PSD files"
```

When you commit a tracked file:
1. A pointer file is committed to the repo
2. The actual file data is chunked and stored in Shrine

---

## File Locking
Rune supports Perforce-style *exclusive locks* for files.  

**Use case:**  
- Prevent binary files from being edited by multiple users at once  
- Works best for non-mergeable formats (e.g. `.psd`, `.fbx`, `.blend`)

**Lock a file:**
```bash
rune lfs lock --path Assets/character.fbx --owner johan
```

**Unlock a file:**
```bash
rune lfs unlock --path Assets/character.fbx
```

---

## Notes
- Locks are tracked in Shrine
- Unlocking can be forced by admins if necessary
