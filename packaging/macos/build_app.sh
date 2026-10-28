#!/usr/bin/env bash
# Build macOS Universal Krebs.app and DMG
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"

echo "=== Building Krebs v${VERSION} for Apple Silicon & Intel ==="
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

echo "=== Creating Universal 2 binary with lipo ==="
lipo -create \
  -output "$PROJECT_ROOT/krebs_universal" \
  target/x86_64-apple-darwin/release/krebs \
  target/aarch64-apple-darwin/release/krebs

echo "=== Generating krebs.icns ==="
ICONSET=$(mktemp -d)/krebs.iconset
mkdir -p "$ICONSET"
sips -z 16 16     assets/logo.png --out "$ICONSET/icon_16x16.png"
sips -z 32 32     assets/logo.png --out "$ICONSET/icon_16x16@2x.png"
sips -z 32 32     assets/logo.png --out "$ICONSET/icon_32x32.png"
sips -z 64 64     assets/logo.png --out "$ICONSET/icon_32x32@2x.png"
sips -z 128 128   assets/logo.png --out "$ICONSET/icon_128x128.png"
sips -z 256 256   assets/logo.png --out "$ICONSET/icon_128x128@2x.png"
sips -z 256 256   assets/logo.png --out "$ICONSET/icon_256x256.png"
sips -z 512 512   assets/logo.png --out "$ICONSET/icon_256x256@2x.png"
sips -z 512 512   assets/logo.png --out "$ICONSET/icon_512x512.png"
iconutil -c icns "$ICONSET" -o "$PROJECT_ROOT/krebs.icns"
rm -rf "$ICONSET"

echo "=== Assembling Krebs.app ==="
APP_NAME="$DIST_DIR/Krebs.app"
rm -rf "$APP_NAME"
mkdir -p "$APP_NAME/Contents/MacOS"
mkdir -p "$APP_NAME/Contents/Resources"

cp packaging/macos/Info.plist "$APP_NAME/Contents/Info.plist"
echo -n "APPL????" > "$APP_NAME/Contents/PkgInfo"
cp "$PROJECT_ROOT/krebs_universal" "$APP_NAME/Contents/MacOS/krebs"
chmod 755 "$APP_NAME/Contents/MacOS/krebs"
ln -sf "krebs" "$APP_NAME/Contents/MacOS/krabbe"
cp "$PROJECT_ROOT/krebs.icns" "$APP_NAME/Contents/Resources/krebs.icns"
rm -f "$PROJECT_ROOT/krebs_universal" "$PROJECT_ROOT/krebs.icns"

echo "=== Creating Universal DMG ==="
DMG_TMP=$(mktemp -d)
cp -R "$APP_NAME" "$DMG_TMP/"
ln -s /Applications "$DMG_TMP/Applications"
hdiutil create -volname "Krebs" -srcfolder "$DMG_TMP" -ov -format UDZO "$DIST_DIR/krebs-v${VERSION}-macos-universal.dmg"
rm -rf "$DMG_TMP"

(cd "$DIST_DIR" && zip -r -y "krebs-v${VERSION}-macos-universal.zip" "Krebs.app")

echo "=== macOS packaging complete! Artifacts in $DIST_DIR ==="
