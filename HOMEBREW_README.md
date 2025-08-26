# Rune VCS Homebrew Installation

This repository contains the Homebrew formula for installing Rune VCS.

## Installation

```bash
# Add the tap
brew tap johan-ott/rune-vcs

# Install Rune
brew install rune

# Verify installation
rune version
```

## Usage

After installation, you can use Rune VCS in any directory:

```bash
# Initialize a new repository
rune init

# Add files
rune add .

# Commit changes
rune commit -m "Initial commit"

# See all available commands
rune help
```

## About Rune VCS

Rune VCS is a modern, scalable version control system designed to be faster and more intuitive than traditional VCS tools.

- **GitHub Repository**: [CaptainOtto/rune-vcs](https://github.com/CaptainOtto/rune-vcs)
- **License**: MIT
- **Version**: 0.2.0+

## Support

For issues, feature requests, or questions, please visit the main repository: https://github.com/CaptainOtto/rune-vcs
