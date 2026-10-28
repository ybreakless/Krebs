#!/usr/bin/env bash
# Build Flatpak bundle for Krebs Studio
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

APP_ID="org.ybreakless.krebs"
DIST_DIR="$PROJECT_ROOT/dist"
BUILD_DIR="$PROJECT_ROOT/build-flatpak"
REPO_DIR="$PROJECT_ROOT/repo-flatpak"

mkdir -p "$DIST_DIR"

if ! command -v flatpak-builder >/dev/null 2>&1; then
    echo "Error: flatpak-builder is required to build the Flatpak package."
    echo "Install via: sudo apt install flatpak-builder  OR  sudo dnf install flatpak-builder"
    exit 1
fi

echo "=== Building Flatpak for $APP_ID ==="
flatpak-builder --force-clean --repo="$REPO_DIR" "$BUILD_DIR" packaging/linux/flatpak/org.ybreakless.krebs.yml

echo "=== Generating Flatpak Bundle: $DIST_DIR/$APP_ID.flatpak ==="
flatpak build-bundle "$REPO_DIR" "$DIST_DIR/$APP_ID.flatpak" "$APP_ID"

echo "=== Flatpak bundle generated successfully at $DIST_DIR/$APP_ID.flatpak ==="
echo "Install with: flatpak install --user $DIST_DIR/$APP_ID.flatpak"
