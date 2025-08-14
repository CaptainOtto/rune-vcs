#!/bin/bash
# Rune VCS Release Automation Script

set -euo pipefail

# Configuration
CURRENT_VERSION=${1:-}
TARGET=${2:-"patch"} # patch, minor, major
DRY_RUN=${3:-"false"}

if [ -z "$CURRENT_VERSION" ]; then
    echo "Usage: $0 <current_version> [patch|minor|major] [dry_run]"
    echo "Example: $0 0.0.2 patch false"
    exit 1
fi

# Parse current version
IFS='.' read -r -a version_parts <<< "${CURRENT_VERSION}"
major=${version_parts[0]}
minor=${version_parts[1]}
patch=${version_parts[2]}

# Calculate next version
case "$TARGET" in
    "patch")
        patch=$((patch + 1))
        ;;
    "minor")
        minor=$((minor + 1))
        patch=0
        ;;
    "major")
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    *)
        echo "Error: Invalid target. Use patch, minor, or major"
        exit 1
        ;;
esac

NEW_VERSION="${major}.${minor}.${patch}"

echo "🚀 Rune VCS Release Automation"
echo "Current Version: ${CURRENT_VERSION}"
echo "New Version: ${NEW_VERSION}"
echo "Release Type: ${TARGET}"
echo "Dry Run: ${DRY_RUN}"
echo ""

if [ "$DRY_RUN" = "true" ]; then
    echo "🔍 DRY RUN MODE - No changes will be made"
    echo ""
fi

# Function to run commands based on dry run mode
run_command() {
    echo "➤ $1"
    if [ "$DRY_RUN" = "false" ]; then
        eval "$1"
    else
        echo "  [DRY RUN] Would execute: $1"
    fi
}

echo "📋 Release Checklist:"
echo ""

echo "1. 🧪 Running Tests..."
run_command "cargo test --all"

echo ""
echo "2. 🔨 Building Release Binary..."
run_command "cargo build --release"

echo ""
echo "3. 📝 Updating Version Numbers..."
run_command "sed -i '' 's/version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/g' Cargo.toml"
run_command "sed -i '' 's/version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/g' crates/*/Cargo.toml"
run_command "sed -i '' 's/version = \"v${CURRENT_VERSION}\"/version = \"v${NEW_VERSION}\"/g' scoop_template/bucket/rune.json"
run_command "sed -i '' 's/version \"v${CURRENT_VERSION}\"/version \"v${NEW_VERSION}\"/g' tap_template/Formula/rune.rb"
run_command "sed -i '' 's/version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/g' crates/rune-cli/src/main.rs"

echo ""
echo "4. 📦 Building Packages..."
if command -v dpkg-deb >/dev/null 2>&1; then
    run_command "./packaging/build-deb.sh ${NEW_VERSION}"
else
    echo "  ⚠️  dpkg-deb not available, skipping Debian package"
fi

if command -v rpmbuild >/dev/null 2>&1; then
    run_command "./packaging/build-rpm.sh ${NEW_VERSION}"
else
    echo "  ⚠️  rpmbuild not available, skipping RPM package"
fi

echo ""
echo "5. 🏷️  Creating Git Tag..."
run_command "git add ."
run_command "git commit -m \"Release v${NEW_VERSION}\""
run_command "git tag -a \"v${NEW_VERSION}\" -m \"Release v${NEW_VERSION}\""

echo ""
echo "6. 📤 Pushing to Repository..."
run_command "git push origin main"
run_command "git push origin \"v${NEW_VERSION}\""

echo ""
if [ "$DRY_RUN" = "false" ]; then
    echo "✅ Release v${NEW_VERSION} completed successfully!"
    echo ""
    echo "📋 Next Steps:"
    echo "  • Create GitHub Release at: https://github.com/CaptainOtto/rune-vcs/releases/new?tag=v${NEW_VERSION}"
    echo "  • Upload release binaries and packages"
    echo "  • Update package repositories (Homebrew tap, Scoop bucket)"
    echo "  • Announce release on social media and community channels"
else
    echo "🔍 Dry run completed. Review the planned changes above."
    echo "Run with 'false' as third argument to execute the release."
fi
