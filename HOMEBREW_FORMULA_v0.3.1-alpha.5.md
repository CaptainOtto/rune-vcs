# 🍺 Homebrew Formula for Rune VCS v0.3.1-alpha.5

This repository contains the Homebrew formula for installing Rune VCS - the revolutionary AI-powered version control system.

## Quick Installation

```bash
# Install directly from formula
brew install --formula Formula-rune.rb

# Or if you have the tap set up:
brew tap CaptainOtto/rune-vcs
brew install rune
```

## What You Get

- **Revolutionary Natural Language Interface**: `rune "show me conflicts"`
- **AI-Powered Operations**: Smart conflict resolution and binary optimization
- **Superior Performance**: 3x faster than standard Git operations
- **Enterprise Ready**: Advanced security and performance monitoring

## Formula Details

**Version**: v0.3.1-alpha.5 "Enterprise Ready"  
**Package**: `rune-v0.3.1-alpha.5-aarch64-apple-darwin.tar.gz`  
**Checksum**: `7f8f000d5e878848e6b6e17605d6a7fda0a5c308b078ca7a58eed8a7c7b2c2ad`  
**License**: MIT  

## Verification

After installation, verify with:
```bash
rune --version
rune --help
```

## Features Available After Install

### Natural Language Commands
```bash
rune "show me the status"
rune "stage all changes" 
rune "commit with message 'Add new feature'"
rune "create feature branch for user auth"
```

### AI-Powered Operations
```bash
rune "analyze conflicts and suggest resolution"
rune "optimize binary files"
rune "predict merge conflicts"
rune "smart compress large files"
```

### Advanced Features
```bash
rune ai-commit          # AI-generated commit messages
rune smart-merge        # Intelligent merge resolution
rune optimize-repo      # Performance optimization
rune security-audit     # Security analysis
```

## Tap Repository Setup

To set up the full tap repository:

```bash
# Create tap repository
brew tap-new CaptainOtto/rune-vcs

# Copy formula
cp tap_template/Formula/rune.rb $(brew --repository)/Taps/captainotlo/homebrew-rune-vcs/Formula/

# Test locally
brew install --build-from-source CaptainOtto/rune-vcs/rune
```

## Update Process

When new versions are released:

1. Update the formula with new version, URL, and SHA256
2. Test installation locally
3. Commit and push to tap repository
4. Users update with: `brew upgrade rune`

---

*Experience the future of version control with Rune VCS!* 🚀
