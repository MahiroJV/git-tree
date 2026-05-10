#!/usr/bin/env bash

set -e

echo "==== git-tree installer ===="

# Detect distro
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

install_arch() {
    echo "Installing dependencies for Arch Linux..."

    sudo pacman -Sy --needed \
        webkit2gtk \
        libgit2 \
        xdotool \
        gtk3 \
        gcc-libs \
        zlib

    # libxdo.so.3 compatibility fix
    if [ ! -e /usr/lib/libxdo.so.3 ] && [ -e /usr/lib/libxdo.so.4 ]; then
        echo "Creating libxdo.so.3 compatibility symlink..."
        sudo ln -s /usr/lib/libxdo.so.4 /usr/lib/libxdo.so.3
    fi
}

install_debian() {
    echo "Installing dependencies for Debian/Ubuntu..."

    sudo apt update

    sudo apt install -y \
        libwebkit2gtk-4.1-dev \
        libgtk-3-0 \
        libxdo3 \
        libgit2-dev \
        zlib1g
}

install_fedora() {
    echo "Installing dependencies for Fedora..."

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
        echo "Install dependencies manually."
        exit 1
        ;;
esac

echo ""
echo "Dependencies installed!"
echo ""

echo "NOTE:"
echo "AppImage may still require some WebKitGTK system libraries."
echo "If AppImage fails, try the raw binary instead."