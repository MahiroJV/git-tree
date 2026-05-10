#!/usr/bin/env bash

set -e

APP_NAME="git-tree"
BINARY_NAME="git-tree-linux-x86_64"

INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"

echo "== git-tree installer =="

# ─────────────────────────────────────────────
# Detect distro
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

install_arch() {
    echo "Installing Arch dependencies..."

    sudo pacman -Sy --needed \
        webkit2gtk \
        libgit2 \
        xdotool \
        gtk3 \
        zlib

    # libxdo compatibility fix
    if [ ! -e /usr/lib/libxdo.so.3 ] && [ -e /usr/lib/libxdo.so.4 ]; then
        echo "Fixing libxdo compatibility..."
        sudo ln -sf /usr/lib/libxdo.so.4 /usr/lib/libxdo.so.3
    fi
}

install_debian() {
    echo "Installing Debian/Ubuntu dependencies..."

    sudo apt update

    sudo apt install -y \
        libwebkit2gtk-4.1-0 \
        libgtk-3-0 \
        libxdo3 \
        libgit2-1.5 \
        zlib1g
}

install_fedora() {
    echo "Installing Fedora dependencies..."

    sudo dnf install -y \
        webkit2gtk4.1 \
        gtk3 \
        xdotool \
        libgit2 \
        zlib
}

case "$DISTRO" in
    arch)
        install_arch
        ;;
    debian)
        install_debian
        ;;
    fedora)
        install_fedora
        ;;
    *)
        echo "Unsupported distro."
        exit 1
        ;;
esac

# ─────────────────────────────────────────────
# Install binary
# ─────────────────────────────────────────────

echo "Installing binary..."

mkdir -p "$INSTALL_DIR"

cp "$BINARY_NAME" "$INSTALL_DIR/git-tree"

chmod +x "$INSTALL_DIR/git-tree"

# ─────────────────────────────────────────────
# Install icon
# ─────────────────────────────────────────────

echo "Installing icon..."

mkdir -p "$ICON_DIR"

cp assets/icon/icon.png \
   "$ICON_DIR/git-tree.png"

# ─────────────────────────────────────────────
# Create desktop entry
# ─────────────────────────────────────────────

echo "Creating desktop entry..."

mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_DIR/git-tree.desktop" <<EOF
[Desktop Entry]
Name=git-tree
Exec=$INSTALL_DIR/git-tree
Icon=git-tree
Type=Application
Categories=Development;
Terminal=false
EOF

chmod +x "$DESKTOP_DIR/git-tree.desktop"

# Refresh desktop db
update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true

echo ""
echo "Installation completed!"
echo ""
echo "Run with:"
echo "  git-tree"
