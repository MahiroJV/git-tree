#!/usr/bin/env bash

set -e

APP_NAME="git-tree"
BIN_NAME="git-tree"
GITHUB_REPO="MahiroJV/git-tree"

INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "== git-tree installer =="

# ─────────────────────────────────────────────
# Detect OS
# ─────────────────────────────────────────────

if [ -f /etc/arch-release ]; then
    DISTRO="arch"
elif [ -f /etc/debian_version ]; then
    DISTRO="debian"
elif [ -f /etc/fedora-release ]; then
    DISTRO="fedora"
else
    DISTRO="unknown"
fi

echo "Detected OS: $DISTRO"

# ─────────────────────────────────────────────
# Install dependencies
# ─────────────────────────────────────────────

install_deps() {
    echo "Installing dependencies..."

    case "$DISTRO" in
        arch)
            # --noconfirm: when run as `curl ... | bash`, pacman's [Y/n] prompt
            # would read the rest of this script from stdin instead of the keyboard
            # -S, not -Sy: syncing the database without upgrading (-Sy pkg) is a
            # partial upgrade, which Arch doesn't support and can break the system.
            # webkit2gtk-4.1: git-tree links libwebkit2gtk-4.1; the old
            # "webkit2gtk" (4.0) package no longer exists in the Arch repos.
            sudo pacman -S --needed --noconfirm \
                webkit2gtk-4.1 \
                libgit2 \
                xdotool \
                gtk3 \
                zlib

            # fix libxdo issue if needed
            if [ ! -e /usr/lib/libxdo.so.3 ] && [ -e /usr/lib/libxdo.so.4 ]; then
                sudo ln -sf /usr/lib/libxdo.so.4 /usr/lib/libxdo.so.3
            fi
            ;;
        debian)
            sudo apt update
            sudo apt install -y \
                libwebkit2gtk-4.1-0 \
                libgtk-3-0 \
                libxdo3 \
                libgit2-dev \
                zlib1g
            ;;
        fedora)
            sudo dnf install -y \
                webkit2gtk4.1 \
                gtk3 \
                xdotool \
                libgit2 \
                zlib
            ;;
        *)
            echo "Unsupported distro"
            exit 1
            ;;
    esac
}

install_deps

# ─────────────────────────────────────────────
# Get latest release
# ─────────────────────────────────────────────

echo "Fetching latest release..."

LATEST=$(curl -s https://api.github.com/repos/$GITHUB_REPO/releases/latest | grep tag_name | cut -d '"' -f4)

if [ -z "$LATEST" ]; then
    echo "Failed to get latest version"
    exit 1
fi

echo "Latest version: $LATEST"

DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$LATEST/git-tree-linux-x86_64"

# ─────────────────────────────────────────────
# Install binary
# ─────────────────────────────────────────────

echo "Downloading binary..."

mkdir -p "$INSTALL_DIR"

curl -L "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BIN_NAME"

chmod +x "$INSTALL_DIR/$BIN_NAME"

# ─────────────────────────────────────────────
# Install icon
# ─────────────────────────────────────────────

mkdir -p "$ICON_DIR"

# Use the local copy when run from a clone, otherwise download it
# (with `curl ... | bash` there are no repo files on disk).
if [ -f "assets/icon/png/icon.png" ]; then
    cp assets/icon/png/icon.png "$ICON_DIR/git-tree.png"
else
    curl -fsSL "https://raw.githubusercontent.com/$GITHUB_REPO/master/assets/icon/png/icon.png" \
        -o "$ICON_DIR/git-tree.png" || echo "Warning: could not download icon"
fi

# ─────────────────────────────────────────────
# Desktop entry
# ─────────────────────────────────────────────

mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_DIR/git-tree.desktop" <<EOF
[Desktop Entry]
Name=git-tree
Exec=$INSTALL_DIR/$BIN_NAME
Icon=git-tree
Type=Application
Categories=Development;
Terminal=false
EOF

chmod +x "$DESKTOP_DIR/git-tree.desktop"

update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true

echo ""
echo "✅ Installation completed!"
echo "Run: git-tree"
