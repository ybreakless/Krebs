#!/usr/bin/env bash
# ==============================================================================
# Krebs Studio Universal One-Liner Installer for Linux & macOS
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ybreakless/Krebs/main/install.sh | bash
# ==============================================================================
set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${CYAN}${BOLD}"
echo "  🦀 Krebs Studio — Minimalist Writing & Whiteboard"
echo "  Terminal Command: krabbe"
echo "  https://github.com/ybreakless/Krebs"
echo -e "${NC}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_TYPE="linux"
        ;;
    Darwin)
        OS_TYPE="macos"
        ;;
    *)
        echo -e "${RED}Unsupported operating system: $OS${NC}"
        echo "For Windows, run in PowerShell:"
        echo "  irm https://raw.githubusercontent.com/ybreakless/Krebs/main/install.ps1 | iex"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_TYPE="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        echo -e "${YELLOW}Warning: Architecture '$ARCH' may require building from source.${NC}"
        ARCH_TYPE="$ARCH"
        ;;
esac

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

APP_DIR="$HOME/.local/share/krebs"
mkdir -p "$APP_DIR"

REPO_URL="https://github.com/ybreakless/Krebs"
TAG="v0.2.0"

echo -e "${CYAN}>>> Detecting environment: ${BOLD}$OS_TYPE ($ARCH_TYPE)${NC}"

# 1. Download or locate binary
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

INSTALLED=0

# Try downloading pre-built release binary from GitHub Releases
if [ "$OS_TYPE" = "linux" ]; then
    TAR_NAME="krebs-${TAG}-${ARCH_TYPE}-unknown-linux-gnu.tar.gz"
    DOWNLOAD_URL="${REPO_URL}/releases/download/${TAG}/${TAR_NAME}"
    echo -e "${CYAN}>>> Fetching release package from GitHub...${NC}"
    
    if curl -sLf "$DOWNLOAD_URL" -o "$TMP_DIR/$TAR_NAME" 2>/dev/null; then
        tar -xzf "$TMP_DIR/$TAR_NAME" -C "$TMP_DIR"
        find "$TMP_DIR" -name "krebs" -type f -exec cp -f {} "$BIN_DIR/krebs" \;
        chmod +x "$BIN_DIR/krebs"
        INSTALLED=1
        echo -e "${GREEN}[OK] Downloaded and installed pre-built binary.${NC}"
    fi
elif [ "$OS_TYPE" = "macos" ]; then
    ZIP_NAME="krebs-${TAG}-${ARCH_TYPE}-apple-darwin.tar.gz"
    DOWNLOAD_URL="${REPO_URL}/releases/download/${TAG}/${ZIP_NAME}"
    echo -e "${CYAN}>>> Fetching macOS package from GitHub...${NC}"

    if curl -sLf "$DOWNLOAD_URL" -o "$TMP_DIR/$ZIP_NAME" 2>/dev/null; then
        tar -xzf "$TMP_DIR/$ZIP_NAME" -C "$TMP_DIR"
        find "$TMP_DIR" -name "krebs" -type f -exec cp -f {} "$BIN_DIR/krebs" \;
        chmod +x "$BIN_DIR/krebs"
        INSTALLED=1
        echo -e "${GREEN}[OK] Downloaded and installed macOS binary.${NC}"
    fi
fi

# Fallback: if pre-built binary wasn't found or architecture differs, build from source.
# The source snapshot is fetched as a zip so neither git nor GitHub cloning is required.
if [ "$INSTALLED" -eq 0 ]; then
    if command -v cargo >/dev/null 2>&1; then
        echo -e "${YELLOW}>>> Pre-built binary not reachable. Building from source via cargo...${NC}"
        SOURCE_URL="${REPO_URL}/archive/refs/heads/main.zip"
        BUILD_DIR="${TMP_DIR}/krebs-build"
        mkdir -p "$BUILD_DIR"

        if command -v unzip >/dev/null 2>&1; then
            echo -e "${CYAN}>>> Downloading source snapshot...${NC}"
            curl -sLf "$SOURCE_URL" -o "$BUILD_DIR/krebs-main.zip"
            unzip -qo "$BUILD_DIR/krebs-main.zip" -d "$BUILD_DIR"
        elif command -v git >/dev/null 2>&1; then
            echo -e "${CYAN}>>> unzip unavailable, falling back to shallow git clone...${NC}"
            git clone --depth 1 "$REPO_URL.git" "$BUILD_DIR/krebs-main"
        else
            echo -e "${RED}Need either 'unzip' or 'git' to unpack the source snapshot.${NC}"
            echo "Install one of them (e.g. 'sudo apt install unzip') and re-run."
            exit 1
        fi

        SRC_DIR="$(dirname "$(find "$BUILD_DIR" -maxdepth 3 -name Cargo.toml | head -n 1 || true)")"
        if [ -z "$SRC_DIR" ]; then
            echo -e "${RED}Downloaded source snapshot contains no Cargo.toml.${NC}"
            exit 1
        fi

        (cd "$SRC_DIR" && cargo build --release)
        cp -f "$SRC_DIR/target/release/krebs" "$BIN_DIR/krebs"
        chmod +x "$BIN_DIR/krebs"
        INSTALLED=1
        echo -e "${GREEN}[OK] Compiled and installed from source.${NC}"
    else
        echo -e "${RED}Could not download release binary and Rust/Cargo is not installed.${NC}"
        echo "Please install Rust (https://rustup.rs) or download the package directly from:"
        echo "  $REPO_URL/releases"
        exit 1
    fi
fi

# 2. Terminal Pathway: Create 'krabbe' command
ln -sf "$BIN_DIR/krebs" "$BIN_DIR/krabbe"
echo -e "${GREEN}[OK] Created terminal command: ${BOLD}krabbe${NC}"

# If sudo / /usr/local/bin is available and writable, link there too
if [ -w "/usr/local/bin" ]; then
    ln -sf "$BIN_DIR/krebs" "/usr/local/bin/krebs" 2>/dev/null || true
    ln -sf "$BIN_DIR/krebs" "/usr/local/bin/krabbe" 2>/dev/null || true
fi

# 3. Desktop Entry and Icon on Linux
if [ "$OS_TYPE" = "linux" ]; then
    DESKTOP_DIR="$HOME/.local/share/applications"
    ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
    mkdir -p "$DESKTOP_DIR" "$ICON_DIR"

    # Download or copy icon
    curl -sLf "https://raw.githubusercontent.com/ybreakless/Krebs/main/assets/logo.svg" -o "$ICON_DIR/krebs.svg" 2>/dev/null || true

    cat << EOF > "$DESKTOP_DIR/krebs.desktop"
[Desktop Entry]
Name=Krebs
Comment=Minimalist Writing & Infinite Whiteboard Studio in Pure Rust
GenericName=Text Editor & Whiteboard
Exec=$BIN_DIR/krebs %F
Icon=krebs
Terminal=false
Type=Application
Categories=Office;WordProcessor;Utility;TextEditor;Graphics;
MimeType=text/plain;text/markdown;
StartupWMClass=krebs
Keywords=Markdown;Editor;Whiteboard;Writing;Notes;Bidi;German;Krabbe;
EOF
    chmod +x "$DESKTOP_DIR/krebs.desktop"

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
    echo -e "${GREEN}[OK] Registered application menu shortcut (krebs.desktop)${NC}"
fi

# 4. Ensure $BIN_DIR is in PATH
PATH_CONFIGURED=0
if [[ ":$PATH:" == *":$BIN_DIR:"* ]]; then
    PATH_CONFIGURED=1
fi

if [ "$PATH_CONFIGURED" -eq 0 ]; then
    SHELL_PROFILE=""
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [ -f "$HOME/.profile" ]; then
        SHELL_PROFILE="$HOME/.profile"
    fi

    if [ -n "$SHELL_PROFILE" ]; then
        if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$SHELL_PROFILE" 2>/dev/null; then
            echo '' >> "$SHELL_PROFILE"
            echo '# Added by Krebs installer' >> "$SHELL_PROFILE"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_PROFILE"
            echo -e "${GREEN}[OK] Added $BIN_DIR to PATH in $SHELL_PROFILE${NC}"
        fi
    fi
fi

echo -e "\n${GREEN}${BOLD}>>> Krebs Studio installed successfully!${NC}"
echo -e "You can now launch the app from any terminal with:"
echo -e "    ${CYAN}${BOLD}krabbe${NC}"
echo -e "Or open a file directly:"
echo -e "    ${CYAN}${BOLD}krabbe my_notes.md${NC}\n"
