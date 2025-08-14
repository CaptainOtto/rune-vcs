#!/bin/bash
# RPM package build script for Rune VCS

set -euo pipefail

VERSION=${1:-"0.0.2"}
RELEASE=${2:-"1"}

echo "Building RPM package: rune-${VERSION}-${RELEASE}"

# Ensure binary exists
if [ ! -f "target/release/rune" ]; then
    echo "Error: Binary not found. Run 'cargo build --release' first."
    exit 1
fi

# Create RPM build directory structure
mkdir -p target/rpm/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

# Copy spec file and update version
cp packaging/rpm/rune.spec target/rpm/SPECS/
sed -i "s/Version:.*/Version: ${VERSION}/" target/rpm/SPECS/rune.spec
sed -i "s/Release:.*/Release: ${RELEASE}%{?dist}/" target/rpm/SPECS/rune.spec

# Create source archive
mkdir -p "target/rpm/rune-${VERSION}"
cp target/release/rune "target/rpm/rune-${VERSION}/"
cp README.md "target/rpm/rune-${VERSION}/"
cp -r docs "target/rpm/rune-${VERSION}/" 2>/dev/null || mkdir -p "target/rpm/rune-${VERSION}/docs"

cd target/rpm
tar -czf "SOURCES/rune-${VERSION}.tar.gz" "rune-${VERSION}/"
rm -rf "rune-${VERSION}/"

# Build RPM
rpmbuild --define "_topdir $(pwd)" -ba SPECS/rune.spec

echo "✅ RPM package created: target/rpm/RPMS/*/rune-${VERSION}-${RELEASE}.*.rpm"
echo "📦 Install with: sudo rpm -i target/rpm/RPMS/*/rune-${VERSION}-${RELEASE}.*.rpm"
