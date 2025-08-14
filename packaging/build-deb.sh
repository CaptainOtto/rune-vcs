#!/bin/bash
# Debian package build script for Rune VCS

set -euo pipefail

VERSION=${1:-"0.0.2"}
ARCH=${2:-"amd64"}
PKG_NAME="rune_${VERSION}_${ARCH}"

echo "Building Debian package: ${PKG_NAME}"

# Create package directory structure
mkdir -p "target/debian/${PKG_NAME}/DEBIAN"
mkdir -p "target/debian/${PKG_NAME}/usr/bin"
mkdir -p "target/debian/${PKG_NAME}/usr/share/doc/rune"
mkdir -p "target/debian/${PKG_NAME}/usr/share/man/man1"

# Copy control file
cp packaging/debian/control "target/debian/${PKG_NAME}/DEBIAN/"
sed -i "s/Version: .*/Version: ${VERSION}/" "target/debian/${PKG_NAME}/DEBIAN/control"
sed -i "s/Architecture: .*/Architecture: ${ARCH}/" "target/debian/${PKG_NAME}/DEBIAN/control"

# Copy binary
if [ -f "target/release/rune" ]; then
    cp target/release/rune "target/debian/${PKG_NAME}/usr/bin/"
else
    echo "Error: Binary not found. Run 'cargo build --release' first."
    exit 1
fi

# Copy documentation
cp README.md "target/debian/${PKG_NAME}/usr/share/doc/rune/"
cp docs/*.md "target/debian/${PKG_NAME}/usr/share/doc/rune/" 2>/dev/null || true

# Create man page (basic)
cat > "target/debian/${PKG_NAME}/usr/share/man/man1/rune.1" << 'EOF'
.TH RUNE 1 "August 2025" "rune 0.0.2" "User Commands"
.SH NAME
rune \- modern distributed version control system
.SH SYNOPSIS
.B rune
[\fIOPTIONS\fR] \fICOMMAND\fR [\fIARGS\fR]
.SH DESCRIPTION
Rune is a modern, intelligent distributed version control system designed for performance and ease of use.
.SH OPTIONS
.TP
\fB\-v\fR, \fB\-\-verbose\fR
Enable verbose output
.TP
\fB\-q\fR, \fB\-\-quiet\fR
Suppress non-essential output
.TP
\fB\-y\fR, \fB\-\-yes\fR
Assume yes for confirmation prompts
.SH COMMANDS
.TP
\fBinit\fR
Initialize a new repository
.TP
\fBadd\fR \fIfiles\fR
Add files to staging area
.TP
\fBcommit\fR \fB\-m\fR \fImessage\fR
Commit staged changes
.TP
\fBstatus\fR
Show working directory status
.TP
\fBlog\fR
Show commit history
.SH AUTHORS
Rune Maintainers <dev@example.invalid>
.SH SEE ALSO
git(1)
EOF

# Compress man page
gzip -9 "target/debian/${PKG_NAME}/usr/share/man/man1/rune.1"

# Set permissions
chmod 755 "target/debian/${PKG_NAME}/usr/bin/rune"
chmod 644 "target/debian/${PKG_NAME}/DEBIAN/control"

# Build package
dpkg-deb --build "target/debian/${PKG_NAME}"

echo "✅ Debian package created: target/debian/${PKG_NAME}.deb"
echo "📦 Install with: sudo dpkg -i target/debian/${PKG_NAME}.deb"
