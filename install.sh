#!/bin/bash
# Rune VCS Universal Installation Script
# Usage: curl -sSf https://raw.githubusercontent.com/CaptainOtto/rune-vcs/main/install.sh | sh

set -e

# Colors and styling
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Icons
CHECKMARK="✓"
INFO="ℹ"
WARNING="⚠"
ERROR="✗"

print_header() {
    echo -e "${BLUE}${BOLD}"
    echo "🔮 Rune VCS Installation Script"
    echo "================================"
    echo -e "${NC}"
}

log_info() {
    echo -e "${BLUE}${INFO}${NC} $1"
}

log_success() {
    echo -e "${GREEN}${CHECKMARK}${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}${WARNING}${NC} $1"
}

log_error() {
    echo -e "${RED}${ERROR}${NC} $1"
}

detect_platform() {
    local platform
    local arch
    
    case "$(uname -s)" in
        Linux*)
            platform="linux"
            ;;
        Darwin*)
            platform="darwin"
            ;;
        CYGWIN*|MINGW32*|MSYS*|MINGW*)
            platform="windows"
            ;;
        *)
            log_error "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        arm64|aarch64)
            arch="aarch64"
            ;;
        armv7l)
            arch="armv7"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    case "$platform" in
        linux)
            TARGET="${arch}-unknown-linux-gnu"
            BINARY_NAME="rune"
            ;;
        darwin)
            TARGET="${arch}-apple-darwin"
            BINARY_NAME="rune"
            ;;
        windows)
            TARGET="${arch}-pc-windows-msvc"
            BINARY_NAME="rune.exe"
            ;;
    esac
    
    echo "$TARGET"
}

get_latest_version() {
    # For now, we'll use the current version from the project
    # In a real scenario, this would query GitHub API
    echo "v0.0.1"
}

download_and_install() {
    local version="$1"
    local target="$2"
    local binary_name="$3"
    
    local download_url="https://github.com/CaptainOtto/rune-vcs/releases/download/${version}/rune-${version}-${target}.tar.gz"
    local temp_dir=$(mktemp -d)
    local archive_path="${temp_dir}/rune.tar.gz"
    
    log_info "Downloading Rune ${version} for ${target}..."
    
    # Check if curl or wget is available
    if command -v curl >/dev/null 2>&1; then
        if ! curl -L -o "$archive_path" "$download_url"; then
            log_error "Failed to download Rune from $download_url"
            log_info "Note: This is a demo script. Actual releases may not be available yet."
            log_info "You can build from source instead:"
            log_info "  git clone https://github.com/CaptainOtto/rune-vcs.git"
            log_info "  cd rune-vcs && cargo build --release"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -O "$archive_path" "$download_url"; then
            log_error "Failed to download Rune from $download_url"
            exit 1
        fi
    else
        log_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    log_info "Extracting archive..."
    tar -xzf "$archive_path" -C "$temp_dir"
    
    # Determine install directory
    local install_dir
    if [[ -d "$HOME/.local/bin" ]]; then
        install_dir="$HOME/.local/bin"
    elif [[ -d "/usr/local/bin" ]] && [[ -w "/usr/local/bin" ]]; then
        install_dir="/usr/local/bin"
    else
        install_dir="$HOME/bin"
        mkdir -p "$install_dir"
    fi
    
    log_info "Installing to $install_dir..."
    cp "${temp_dir}/${binary_name}" "${install_dir}/rune"
    chmod +x "${install_dir}/rune"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_success "Rune installed to ${install_dir}/rune"
    
    # Check if install directory is in PATH
    if ! echo "$PATH" | grep -q "$install_dir"; then
        log_warning "Warning: $install_dir is not in your PATH"
        log_info "Add this to your shell profile:"
        log_info "  export PATH=\"$install_dir:\$PATH\""
    fi
}

setup_shell_completion() {
    log_info "Setting up shell completions..."
    
    # Try to detect shell
    local shell_name
    if [[ -n "$ZSH_VERSION" ]]; then
        shell_name="zsh"
    elif [[ -n "$BASH_VERSION" ]]; then
        shell_name="bash"
    else
        shell_name=$(basename "$SHELL")
    fi
    
    case "$shell_name" in
        bash)
            if command -v rune >/dev/null 2>&1; then
                rune completions bash > "$HOME/.bash_completion.d/rune" 2>/dev/null || true
                log_success "Bash completions installed"
            fi
            ;;
        zsh)
            if command -v rune >/dev/null 2>&1; then
                local zsh_comp_dir="${HOME}/.zsh/completions"
                mkdir -p "$zsh_comp_dir"
                rune completions zsh > "$zsh_comp_dir/_rune" 2>/dev/null || true
                log_success "Zsh completions installed"
            fi
            ;;
        fish)
            if command -v rune >/dev/null 2>&1; then
                local fish_comp_dir="${HOME}/.config/fish/completions"
                mkdir -p "$fish_comp_dir"
                rune completions fish > "$fish_comp_dir/rune.fish" 2>/dev/null || true
                log_success "Fish completions installed"
            fi
            ;;
        *)
            log_info "Shell completions available for bash, zsh, and fish"
            log_info "Run 'rune completions <shell>' to generate them"
            ;;
    esac
}

verify_installation() {
    log_info "Verifying installation..."
    
    if command -v rune >/dev/null 2>&1; then
        local version_output
        version_output=$(rune version 2>/dev/null || rune --version 2>/dev/null || echo "Unable to get version")
        log_success "Rune is installed and accessible"
        log_info "Version check: $version_output"
        
        # Run doctor command for full verification
        log_info "Running installation verification..."
        if rune doctor >/dev/null 2>&1; then
            log_success "Installation verification passed"
        else
            log_warning "Installation verification had some issues"
            log_info "Run 'rune doctor' for detailed diagnostics"
        fi
    else
        log_error "Rune installation failed - binary not found in PATH"
        exit 1
    fi
}

main() {
    print_header
    
    log_info "Detecting platform..."
    local target
    target=$(detect_platform)
    log_success "Platform detected: $target"
    
    log_info "Getting latest version..."
    local version
    version=$(get_latest_version)
    log_success "Latest version: $version"
    
    # Check if already installed
    if command -v rune >/dev/null 2>&1; then
        local current_version
        current_version=$(rune version 2>/dev/null | grep -o "Version: [0-9.]*" | cut -d' ' -f2 || echo "unknown")
        log_warning "Rune is already installed (version: $current_version)"
        read -p "Do you want to reinstall? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Installation cancelled"
            exit 0
        fi
    fi
    
    # Note: This is a demo installation script
    log_warning "This is a demonstration installation script"
    log_info "For the current version, please build from source:"
    echo ""
    log_info "Building from source:"
    log_info "  1. git clone https://github.com/CaptainOtto/rune-vcs.git"
    log_info "  2. cd rune-vcs"
    log_info "  3. cargo build --release"
    log_info "  4. cp target/release/rune-cli /usr/local/bin/rune"
    echo ""
    log_info "Alternative installation methods:"
    log_info "  • Cargo: cargo install --git https://github.com/CaptainOtto/rune-vcs rune-cli"
    echo ""
    
    # For demo purposes, we'll still proceed with the "installation"
    read -p "Continue with demo installation? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Proceeding with demo installation..."
        # This would normally download and install
        # download_and_install "$version" "$target" "rune"
        log_info "Demo installation completed"
        log_info "Please follow the build-from-source instructions above"
    else
        log_info "Installation cancelled"
    fi
    
    echo ""
    log_success "Thank you for trying Rune VCS!"
    log_info "Visit https://github.com/CaptainOtto/rune-vcs for documentation"
}

# Run the installation
main "$@"
