#!/usr/bin/env bash
#
# git-tree uninstaller — undoes what install.sh (and the README install steps) did.
#
# Usage:
#   ./uninstall.sh            remove the app, keep your settings
#   ./uninstall.sh --purge    also remove settings, recent repos and GitHub login
#   ./uninstall.sh --yes      don't ask for confirmation
#
# System packages (webkit2gtk, gtk3, xdotool, ...) are NOT removed on purpose:
# other apps usually depend on them.

set -u

PURGE=0
ASSUME_YES=0

for arg in "$@"; do
    case "$arg" in
        --purge) PURGE=1 ;;
        -y|--yes) ASSUME_YES=1 ;;
        -h|--help)
            sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "Unknown option: $arg (try --help)"
            exit 1
            ;;
    esac
done

BIN_NAME="git-tree"
LOCAL_BIN="$HOME/.local/bin/$BIN_NAME"
SYSTEM_BIN="/usr/local/bin/$BIN_NAME"
DESKTOP_DIR="$HOME/.local/share/applications"
DESKTOP_FILE="$DESKTOP_DIR/git-tree.desktop"
ICON_FILE="$HOME/.local/share/icons/hicolor/256x256/apps/git-tree.png"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/git-tree"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/git-tree"
XDO_LINK="/usr/lib/libxdo.so.3"

REMOVED=0

confirm() {
    [ "$ASSUME_YES" -eq 1 ] && return 0
    read -r -p "$1 [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

remove_file() {
    local path="$1" use_sudo="${2:-}"
    if [ -e "$path" ] || [ -L "$path" ]; then
        if [ "$use_sudo" = "sudo" ]; then
            sudo rm -f "$path"
        else
            rm -f "$path"
        fi
        echo "  removed $path"
        REMOVED=$((REMOVED + 1))
    fi
}

echo "== git-tree uninstaller =="
echo
echo "This will remove git-tree from this system."
[ "$PURGE" -eq 1 ] && echo "Settings, recent repos and GitHub login will be deleted too (--purge)."
echo
confirm "Continue?" || { echo "Aborted."; exit 0; }
echo

# ─────────────────────────────────────────────
# 1. Files from install.sh
# ─────────────────────────────────────────────

echo "-> install.sh files"
remove_file "$LOCAL_BIN"
remove_file "$DESKTOP_FILE"
remove_file "$ICON_FILE"
if command -v update-desktop-database >/dev/null 2>&1 && [ -d "$DESKTOP_DIR" ]; then
    update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
fi

# ─────────────────────────────────────────────
# 2. Manually installed binary
# ─────────────────────────────────────────────

if [ -e "$SYSTEM_BIN" ]; then
    echo "-> manual install"
    if confirm "Remove $SYSTEM_BIN (needs sudo)?"; then
        remove_file "$SYSTEM_BIN" sudo
    fi
fi

# ─────────────────────────────────────────────
# 3. Package managers
# ─────────────────────────────────────────────

if command -v pacman >/dev/null 2>&1; then
    for pkg in git-tree-bin git-tree; do
        if pacman -Qq "$pkg" >/dev/null 2>&1; then
            echo "-> AUR package: $pkg"
            if confirm "Remove package $pkg (needs sudo)?"; then
                sudo pacman -Rns "$pkg" && REMOVED=$((REMOVED + 1))
            fi
        fi
    done
fi

if command -v nix >/dev/null 2>&1; then
    if nix profile list 2>/dev/null | grep -q "git-tree"; then
        echo "-> Nix profile"
        if confirm "Remove git-tree from your nix profile?"; then
            nix profile remove git-tree && REMOVED=$((REMOVED + 1))
            echo "  (run 'nix-collect-garbage' later to free space in /nix/store)"
        fi
    fi
    if [ -f /etc/NIXOS ]; then
        echo "  Note: if you added git-tree to configuration.nix or home.nix,"
        echo "        remove it there and rebuild — this script can't edit those files."
    fi
fi

# ─────────────────────────────────────────────
# 4. libxdo.so.3 workaround symlinks
# ─────────────────────────────────────────────

# Arch: install.sh creates /usr/lib/libxdo.so.3 -> libxdo.so.4.
# Only remove it if it's a symlink that no package owns.
if [ -L "$XDO_LINK" ]; then
    owned=0
    if command -v pacman >/dev/null 2>&1 && pacman -Qo "$XDO_LINK" >/dev/null 2>&1; then
        owned=1
    fi
    if [ "$owned" -eq 0 ]; then
        echo "-> libxdo workaround symlink"
        echo "  $XDO_LINK -> $(readlink "$XDO_LINK")"
        echo "  Other programs built against libxdo.so.3 may still need it."
        if confirm "Remove it (needs sudo)?"; then
            remove_file "$XDO_LINK" sudo
        fi
    fi
fi

# NixOS: the manual /tmp workaround from the issue tracker
if [ -d /tmp/libxdo ]; then
    echo "-> /tmp/libxdo workaround"
    rm -rf /tmp/libxdo && echo "  removed /tmp/libxdo"
    REMOVED=$((REMOVED + 1))
    echo "  Open a new terminal so LD_LIBRARY_PATH no longer points there."
fi

# ─────────────────────────────────────────────
# 5. User data (only with --purge)
# ─────────────────────────────────────────────

if [ "$PURGE" -eq 1 ]; then
    echo "-> user data"
    for dir in "$CONFIG_DIR" "$DATA_DIR"; do
        if [ -d "$dir" ]; then
            rm -rf "$dir"
            echo "  removed $dir"
            REMOVED=$((REMOVED + 1))
        fi
    done
elif [ -d "$CONFIG_DIR" ] || [ -d "$DATA_DIR" ]; then
    echo
    echo "Kept your settings in $CONFIG_DIR"
    echo "Run with --purge to delete them too."
fi

echo
if [ "$REMOVED" -eq 0 ]; then
    echo "Nothing was removed — git-tree doesn't seem to be installed here."
else
    echo "✅ git-tree uninstalled ($REMOVED item(s) removed)."
fi
