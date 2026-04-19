# THAT PROJECT NOT-FULLY PREPARED
# git-tree 🌿

> Terminal-style git branch visualizer — built with Rust + Dioxus

![version](https://img.shields.io/badge/version-0.1.0-9B5DE5?style=flat-square)
![rust](https://img.shields.io/badge/rust-1.75+-orange?style=flat-square)
![platform](https://img.shields.io/badge/platform-linux-blue?style=flat-square)
![license](https://img.shields.io/badge/license-MIT-green?style=flat-square)

---

## What is it?

git-tree lets you visualize your git history in a clean terminal-style desktop app. open any local repo or clone a remote one, click any commit node to see author, date, message, and diff stats.

```
 ──●──●──●──●──●──●──●──
        ╲       ╱
         ●─────●
```

---

## Features (v0.1)

- open local git repos or clone remote URLs
- horizontal branch tree — branches go up and down
- each contributor gets a unique persistent color
- click any commit node → author, date, message, hash, diff stats
- 9 built-in themes with live preview in settings
- default theme: **Terminal** (black + purple, Oxanium + Space Mono fonts)

---

## Install

### Linux (recommended)

**1. install system dependencies**

Ubuntu / Debian:
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

Arch Linux:
```bash
sudo pacman -S libgit2 webkit2gtk-4.1 gtk3 base-devel
```

Fedora:
```bash
sudo dnf install libgit2-devel webkit2gtk4.1-devel gtk3-devel
```

**2. download the binary**

grab the latest release from the [Releases page](https://github.com/MahiroJV/git-tree/releases/latest):

```bash
wget https://github.com/MahiroJV/git-tree/releases/latest/download/git-tree-linux
chmod +x git-tree-linux
./git-tree-linux
```

or move it to your PATH for system-wide access:
```bash
sudo mv git-tree-linux /usr/local/bin/git-tree
git-tree
```

---

## Build from source

**requirements:**
- Rust 1.75+
- Dioxus CLI
- system dependencies (see above)

```bash
# clone the repo
git clone https://github.com/MahiroJV/git-tree
cd git-tree

# install dioxus cli
cargo install dioxus-cli

# run in dev mode
dx serve --platform desktop

# build release binary
dx build --platform desktop --release
# binary will be at: dist/git-tree
```

---

## Usage

**open a local repo:**
1. launch git-tree
2. make sure `[ LOCAL FOLDER ]` tab is selected
3. type the full path to your repo (e.g. `/home/user/my-project`)
4. click `OPEN →`

**clone a remote repo:**
1. click `[ REMOTE URL ]` tab
2. paste a GitHub/GitLab URL (e.g. `https://github.com/user/repo`)
3. click `CLONE →`
4. git-tree clones it to a temp folder and opens it

**navigating the tree:**
- click any commit node → left panel shows commit info, right panel shows diff stats
- use the toolbar to go back home, refresh, or open settings
- settings → pick from 9 themes, preview updates live

---

## Themes

| Name | Description |
|---|---|
| **Terminal** | black + purple, default |
| **Matrix** | hacker green |
| **Amber** | old phosphor monitor |
| **Synthwave** | 80s retrowave |
| **Nord** | cold Nordic blues |
| **Dracula** | popular dark dev theme |
| **Gruvbox** | warm retro |
| **Blood Moon** | dark dramatic red |
| **Ice Terminal** | cold blue cyberpunk |

---

## Roadmap

| Version | Features |
|---|---|
| **v0.1** | ✅ tree render, click panels, 9 themes, contributor colors, local + remote |
| **v0.2** | zoom + pan, keyboard navigation, search by author/message/hash, diff viewer, minimap |
| **v0.3** | repo stats screen, export SVG/PNG, animations, CRT overlay, copy hash, open in browser |
| **v1.0** | full polish, recent repos list, AppImage packaging, Android port via Dioxus mobile |

---

## Project Structure

```
src/
├── main.rs                 # entry point, window config
├── app.rs                  # root component + global state
├── theme.rs                # 9 themes + contributor color engine
├── components/
│   ├── home_screen.rs      # repo open/clone screen
│   ├── toolbar.rs          # top navigation
│   ├── tree_canvas.rs      # SVG tree visualization
│   ├── left_panel.rs       # commit details
│   ├── right_panel.rs      # diff stats
│   └── settings.rs         # theme selector
└── git/
    ├── loader.rs           # open local / clone remote
    └── parser.rs           # git history → tree data structures

assets/
└── style.css               # terminal theme styling
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

built with Rust + Dioxus 🦀

---

## License

MIT
