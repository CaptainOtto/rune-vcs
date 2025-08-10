
# v0.0.1 Release Checklist

- [ ] Push repo to GitHub (e.g. CaptainOtto/rune-vcs)
- [ ] Configure GitHub Secrets:
      - TAP_REPO (e.g. CaptainOtto/homebrew-tap)
      - TAP_GITHUB_TOKEN (PAT with push rights)
      - SCOOP_BUCKET_REPO (e.g. CaptainOtto/scoop-bucket)
      - SCOOP_BUCKET_TOKEN (PAT with push rights)
- [ ] Create tag and push:
      ```bash
      git tag v0.0.1
      git push origin v0.0.1
      ```
- [ ] Verify GitHub Actions artifacts for macOS/Linux/Windows
- [ ] Confirm Homebrew tap updated formula and Scoop bucket manifest
- [ ] Install test:
      - macOS/Linux: `brew tap CaptainOtto/tap && brew install rune`
      - Windows: `scoop bucket add rune <bucket-url> && scoop install rune`
- [ ] Smoke test embedded mode:
      ```bash
      rune api --addr 127.0.0.1:7421 --with-shrine --shrine-addr 127.0.0.1:7420
      curl http://127.0.0.1:7421/v1/status
      ```
