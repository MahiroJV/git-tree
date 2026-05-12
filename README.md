<div align="center">
  <img src="assets/icon/icon.svg" height="256" alt="git-tree logo" />
  <h1>git-tree</h1>

  <p><i>Terminal-style git branch visualizer — built with Rust + Dioxus</i></p>

  <p>
   <img src="https://img.shields.io/github/actions/workflow/status/MahiroJV/git-tree/test.yml?label=CI%2FCD&logo=githubactions" />
   <img src="https://img.shields.io/badge/Rust-1.75+-orange" />
   <img src="https://img.shields.io/badge/Dioxus-UI_Framework-purple?style=flat&logo=rust" />
   <img src="https://img.shields.io/badge/Built%20with-Claude-CC785C?style=flat&logo=anthropic&logoColor=white" alt="Built with Claude">
   <br> 
   <img src="https://img.shields.io/badge/Linux-E11837?style=flat&logo=linux&logoColor=white" alt="Linux">
   <img src="https://img.shields.io/badge/Linux-supported-orange" />
   <img src="https://img.shields.io/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white" alt="Windows">   
   <img src="https://img.shields.io/badge/Windows-supported-blue" />
   <br>
   <img src="https://img.shields.io/crates/v/git-tree-viz?style=flat&color=orange" alt="crates.io"> 
   <img src="https://img.shields.io/badge/License-MIT-green" />
   <img src="https://img.shields.io/github/repo-size/MahiroJV/git-tree?style=flat&color=blue" alt="size">
  </p>
</div>

---

## ✨ Overview

**git-tree** is a desktop Git visualizer that turns commit history into an interactive tree.

Explore branches, inspect commits, view diffs, and analyze repository activity in a clean terminal-style UI.

---

## ⚡ Features

- 🌳 Interactive commit tree (horizontal & vertical)
- 🔍 Commit search (author, message, hash)
- 📄 Full diff viewer with collapsible files
- 🧭 Minimap navigation
- 📊 Repository statistics (contributors + heatmap)
- 🎨 15 built-in themes
- 🖱️ Mouse + keyboard navigation
- 🔗 Open commits in GitHub/GitLab
- 📁 Clone or open local repositories
- ⚡ Fast native Rust performance

---

# Installation

### Linux (recommended)

**1. Install system dependencies**

## ![Ubuntu](https://img.shields.io/badge/Ubuntu-E95420?style=for-the-badge&logo=ubuntu&logoColor=white)
```bash
sudo apt update
sudo apt install -y \
  libgit2-dev \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libglib2.0-dev \
  libcairo2-dev \
  libpango1.0-dev \
  libxdo-dev \
  pkg-config
```

## ![Arch Linux](https://img.shields.io/badge/Arch%20Linux-1793D1?style=for-the-badge&logo=arch-linux&logoColor=white)
```bash
sudo pacman -S libgit2 webkit2gtk-4.1 gtk3 base-devel
```
### 🚀 Recommended Installation for Arch 

For the best experience, use the official installer:

```bash id="lnx03"
curl -fsSL https://raw.githubusercontent.com/<repo>/install.sh | bash
```

## ![Fedora](https://img.shields.io/badge/Fedora-51A2DA?style=for-the-badge&logo=fedora&logoColor=white)
```bash
sudo dnf install libgit2-devel webkit2gtk4.1-devel gtk3-devel
```
---

**2. Download the binary**

Grab the latest release from the [Releases page](https://github.com/MahiroJV/git-tree/releases/latest):

```bash
wget https://github.com/MahiroJV/git-tree/releases/latest/download/git-tree-linux-x86_64
chmod +x git-tree-linux-x86_64
./git-tree-linux-x86_64
```

Or move it to your PATH for system-wide access:
```bash
sudo mv git-tree-linux-x86_64 /usr/local/bin/git-tree
git-tree
```

**AppImage (no dependencies needed):**
```bash
chmod +x git-tree-*-x86_64.AppImage
./git-tree-*-x86_64.AppImage
```
---
# Documentation
**If you have any problems with installation, please check the [Wiki](https://github.com/MahiroJV/git-tree/wiki).**

---

## Build from source

**Requirements:**
- Rust 1.75+
- Dioxus CLI
- System dependencies (see above)

```bash
# Clone the repo
git clone https://github.com/MahiroJV/git-tree
cd git-tree

# Install Dioxus CLI
cargo install dioxus-cli

# Run in dev mode
dx serve --platform desktop

# Build release binary
dx build --platform desktop --release
# Binary will be at: dist/git-tree
```

---

## Usage

**Open a local repo:**
1. Launch git-tree
2. Make sure `[ LOCAL FOLDER ]` tab is selected
3. Type the full path to your repo or use the 📁 picker
4. Click `OPEN →`

**Clone a remote repo:**
1. Click `[ REMOTE URL ]` tab
2. Paste a GitHub/GitLab URL (e.g. `https://github.com/user/repo`)
3. Click `CLONE →`
4. git-tree clones it to a temp folder and opens it

**Search GitHub:**
1. Click `[ SEARCH ONLINE ]` tab
2. Type any query (e.g. `rust async runtime`)
3. Hit Enter or click `SEARCH →`
4. Click `CLONE →` on any result to open it instantly

**Navigating the tree:**
- Click any commit node → left panel shows commit info, right panel shows diff stats
- `← →` (horizontal) or `↑ ↓` (vertical) to move between commits
- `ESC` to deselect
- `CTRL+scroll` to zoom, drag to pan
- Click anywhere on the **minimap** to jump to that region
- Toolbar → `[ VIEW DIFF ]` to open the full diff viewer
- Toolbar → `[ STATS ]` to open the repo stats screen

**Settings:**
- 15 themes with live preview
- Font size, node spacing, merge commit visibility
- Tree direction (Horizontal / Vertical)
- Branch style (Curved / Geometric)
- CRT scanline overlay

---

## Themes

| Name | Description |
|---|---|
| **Terminal** | Black + purple — default |
| **Matrix** | Hacker green |
| **Amber** | Old phosphor monitor |
| **Synthwave** | 80s retrowave |
| **Nord** | Cold Nordic blues |
| **Dracula** | Popular dark dev theme |
| **Gruvbox** | Warm retro |
| **Blood Moon** | Dark dramatic red |
| **Ice Terminal** | Cold blue cyberpunk |
| **Light** | Paper white, clean |
| **Dark** | Deeper black than Terminal |
| **Tokyo Night** | Deep blue city lights |
| **Cappuccino Mocha** | Catppuccin-inspired warm dark |
| **Rose Pine** | Muted dawn purple |
| **Everforest** | Muted forest green |

---

## 🚀 Roadmap

---

### 🧱 v0.1 — Foundation ✅
- [x] Tree visualization
- [x] Click panels (commit info + diff stats)
- [x] 9 themes
- [x] Contributor colors
- [x] Local + remote clone
- [x] Zoom + pan
- [x] App icon

---

### 🧪 v0.2 — Usability ✅
- [x] Search by author / message / hash
- [x] Diff viewer (actual +/- code lines with collapse)
- [x] Keyboard navigation (arrow keys between commits)
- [x] Fix font loading (Space Mono offline)
- [x] Recent repositories list with filter
- [x] Copy hash button
- [x] Vertical tree layout
- [x] Settings panel (font size, node spacing, merge visibility)
- [x] CRT scanline overlay
- [x] 2 extra themes (Light, Dark)

---

### 🎨 v0.3 — Polish ✅
- [x] Minimap (corner overview of the full tree, click-to-navigate)
- [x] Repo stats (contributor leaderboard + commit heatmap)
- [x] Open commit in browser (GitHub / GitLab)
- [x] Node pulse animation on click
- [x] GitHub repository search + one-click clone
- [x] Curved and geometric branch connector styles
- [x] 4 extra themes (Tokyo Night, Cappuccino Mocha, Rose Pine, Everforest)
- [ ] Export tree as SVG or PNG

---

### 🖥️ v0.4 — Platform
- [ ] GitHub OAuth login (private repo access)
- [ ] Windows installer (MSI via cargo-wix)
- [ ] macOS build
- [ ] Linux Flatpak / Snap packaging

---

### 🏁 v1.0 — Release
- [ ] Full keyboard shortcut system
- [ ] Performance improvements for very large repos
- [ ] Community themes

---

## Project Structure

```
src/
├── main.rs                 # Entry point, window config
├── app.rs                  # Root component + global state
├── theme.rs                # 15 themes + contributor color engine
├── recent.rs               # Recent repos persistence (~/.config/git-tree/)
├── components/
│   ├── home_screen.rs      # Repo open/clone/search screen + recent list
│   ├── toolbar.rs          # Top navigation + search bar
│   ├── tree_canvas.rs      # SVG tree (horizontal + vertical + minimap)
│   ├── left_panel.rs       # Commit details + copy hash
│   ├── right_panel.rs      # Diff stats + file list
│   ├── diff_viewer.rs      # Full diff viewer with per-file collapse
│   ├── stats.rs            # Repo stats (heatmap + contributor leaderboard)
│   └── settings.rs         # Theme selector + display options
└── git/
    ├── loader.rs           # Open local / clone remote
    ├── parser.rs           # Git history → tree data structures
    ├── search.rs           # GitHub API search
    └── url.rs              # Remote URL → browser URL conversion

assets/
├── css/
│   ├── style.css           # Core terminal theme + layout
│   ├── diff_viewer.css     # Diff viewer styles
│   ├── left_panel.css      # Left panel styles
│   ├── right_panel.css     # Right panel styles
│   ├── panel_shared.css    # Shared collapse/expand styles
│   └── stats.css           # Stats screen styles
└── fonts/
    ├── Oxanium.ttf
    └── SpaceMono-Regular.woff2
```

---

## Requirements

| Dependency | Version |
|---|---|
| Rust | 1.75+ |
| Dioxus | 0.6 |
| git2 | 0.19 |
| libgit2 | system |
| webkit2gtk | 4.1 (Linux) |

---

## Author

**MahiroJV** — [github.com/MahiroJV](https://github.com/MahiroJV)

Built with Rust + Dioxus 🦀

---

## License

MIT
