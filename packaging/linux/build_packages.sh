#!/usr/bin/env bash
# Build Debian (.deb), Fedora (.rpm), and Arch (.pkg.tar.zst) packages for Krebs
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"

echo "=== Building Krebs v${VERSION} release binary ==="
cargo build --release

# 1. Debian (.deb)
if command -v cargo-deb >/dev/null 2>&1; then
    echo "=== Creating Debian package (.deb) ==="
    cargo deb --no-build
    cp target/debian/*.deb "$DIST_DIR/"
else
    echo "cargo-deb not found. Skipping .deb. (Install with: cargo install cargo-deb)"
fi

# 2. RPM / DNF (.rpm)
if command -v cargo-generate-rpm >/dev/null 2>&1; then
    echo "=== Creating RPM package (.rpm for DNF) ==="
    cargo generate-rpm
    cp target/generate-rpm/*.rpm "$DIST_DIR/"
else
    echo "cargo-generate-rpm not found. Skipping .rpm. (Install with: cargo install cargo-generate-rpm)"
fi

# 3. Arch Linux (.pkg.tar.zst for Pacman)
if command -v zstd >/dev/null 2>&1 && command -v tar >/dev/null 2>&1; then
    echo "=== Creating Arch Linux package (.pkg.tar.zst for Pacman) ==="
    PKG_DIR=$(mktemp -d)
    trap 'rm -rf "$PKG_DIR"' EXIT

    mkdir -p "$PKG_DIR/usr/bin"
    mkdir -p "$PKG_DIR/usr/share/applications"
    mkdir -p "$PKG_DIR/usr/share/icons/hicolor/scalable/apps"
    mkdir -p "$PKG_DIR/usr/share/icons/hicolor/512x512/apps"
    mkdir -p "$PKG_DIR/usr/share/licenses/krebs"
    mkdir -p "$PKG_DIR/usr/share/doc/krebs"

    cp target/release/krebs "$PKG_DIR/usr/bin/krebs"
    chmod 755 "$PKG_DIR/usr/bin/krebs"
    cp packaging/linux/krabbe "$PKG_DIR/usr/bin/krabbe"
    chmod 755 "$PKG_DIR/usr/bin/krabbe"
    cp packaging/linux/krebs.desktop "$PKG_DIR/usr/share/applications/krebs.desktop"
    cp assets/logo.svg "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/krebs.svg"
    cp assets/logo.png "$PKG_DIR/usr/share/icons/hicolor/512x512/apps/krebs.png"
    cp LICENSE "$PKG_DIR/usr/share/licenses/krebs/LICENSE"
    cp README.md "$PKG_DIR/usr/share/doc/krebs/README.md"

    INST_SIZE=$(du -sb "$PKG_DIR" | cut -f1)
    BUILD_DATE=$(date +%s)

    cat << EOF > "$PKG_DIR/.PKGINFO"
pkgname = krebs
pkgbase = krebs
pkgver = ${VERSION}-1
pkgdesc = Minimalist Writing & Infinite Whiteboard Studio in Pure Rust
url = https://github.com/ybreakless/Krebs
builddate = ${BUILD_DATE}
packager = ybreakless <https://github.com/ybreakless>
size = ${INST_SIZE}
arch = x86_64
license = MIT
depend = fontconfig
depend = libx11
depend = libxcursor
depend = libxrandr
depend = libxi
depend = libxinerama
depend = libxkbcommon
depend = mesa
depend = wayland
EOF

    sed -i 's/^[ \t]*//' "$PKG_DIR/.PKGINFO"
    tar -C "$PKG_DIR" -c .PKGINFO usr | zstd -19 -T0 > "$DIST_DIR/krebs-${VERSION}-1-x86_64.pkg.tar.zst"
    echo "Created: $DIST_DIR/krebs-${VERSION}-1-x86_64.pkg.tar.zst"
fi

echo "=== Linux packaging complete! Packages in $DIST_DIR ==="
