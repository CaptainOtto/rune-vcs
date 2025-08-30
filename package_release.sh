#!/bin/bash

# 🚀 Rune VCS v0.3.1-alpha.5 Release Package Script
# Creates release binaries and packages for distribution

set -e

VERSION="v0.3.1-alpha.5"
RELEASE_DIR="release-$VERSION"

echo "🚀 Creating Rune VCS $VERSION Release Package"
echo "================================================"

# Create release directory
mkdir -p "$RELEASE_DIR"

# Wait for build to complete if still running
echo "⏳ Waiting for release build to complete..."
while pgrep -f "cargo build --release" > /dev/null; do
    sleep 2
    echo "   Still building..."
done

echo "✅ Build completed!"

# Check if binary exists
if [ ! -f "target/release/rune" ]; then
    echo "❌ Release binary not found at target/release/rune"
    exit 1
fi

echo "📦 Packaging release artifacts..."

# Get system info
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

# Normalize architecture names
case "$ARCH" in
    "arm64"|"aarch64") ARCH="aarch64" ;;
    "x86_64"|"amd64") ARCH="x86_64" ;;
esac

# Normalize OS names
case "$OS" in
    "darwin") TARGET="$ARCH-apple-darwin" ;;
    "linux") TARGET="$ARCH-unknown-linux-gnu" ;;
    *) TARGET="$ARCH-$OS" ;;
esac

PACKAGE_NAME="rune-$VERSION-$TARGET"
PACKAGE_DIR="$RELEASE_DIR/$PACKAGE_NAME"

# Create package directory
mkdir -p "$PACKAGE_DIR"

# Copy binary
cp target/release/rune "$PACKAGE_DIR/"

# Copy documentation
cp README.md "$PACKAGE_DIR/"
cp LICENSE "$PACKAGE_DIR/"
cp "RELEASE_NOTES_$VERSION.md" "$PACKAGE_DIR/"

# Create installation script
cat > "$PACKAGE_DIR/install.sh" << 'EOF'
#!/bin/bash
echo "🚀 Installing Rune VCS..."

# Check if binary exists
if [ ! -f "rune" ]; then
    echo "❌ rune binary not found in current directory"
    exit 1
fi

# Make executable
chmod +x rune

# Try to install to system path
if [ -w "/usr/local/bin" ]; then
    cp rune /usr/local/bin/
    echo "✅ Installed rune to /usr/local/bin/"
elif [ -d "$HOME/.local/bin" ]; then
    cp rune "$HOME/.local/bin/"
    echo "✅ Installed rune to $HOME/.local/bin/"
    echo "💡 Make sure $HOME/.local/bin is in your PATH"
else
    echo "💡 Manual installation required:"
    echo "   sudo cp rune /usr/local/bin/"
    echo "   or add current directory to PATH"
fi

echo ""
echo "🎉 Installation complete!"
echo "   Test with: rune --version"
EOF

chmod +x "$PACKAGE_DIR/install.sh"

# Create archive
cd "$RELEASE_DIR"
tar -czf "$PACKAGE_NAME.tar.gz" "$PACKAGE_NAME"

# Generate checksums
shasum -a 256 "$PACKAGE_NAME.tar.gz" > checksums.txt

cd ..

# Display results
echo ""
echo "✅ Release package created successfully!"
echo "📁 Location: $RELEASE_DIR/$PACKAGE_NAME.tar.gz"
echo "🔐 Checksum: $(cat $RELEASE_DIR/checksums.txt)"
echo ""
echo "📊 Package contents:"
ls -la "$PACKAGE_DIR"
echo ""
echo "🎯 Ready for distribution!"
