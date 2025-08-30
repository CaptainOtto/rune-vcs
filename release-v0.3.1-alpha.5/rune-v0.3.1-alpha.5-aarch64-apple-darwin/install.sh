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
