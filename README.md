# THAT PROJECT NOT-FULLY PREPARED
# git-tree 🌿

> Terminal-style git branch visualizer — built with Rust + Dioxus

## Features (v0.1)
- Open local git repos or clone remote URLs
- Horizontal branch tree — branches go **up and down**
- Each contributor gets a unique persistent color
- Click any commit node → left panel (author, date, message, hash) + right panel (diff stats)
- 9 built-in themes with live preview in settings
- Default theme: **Terminal** (black + purple, Space Mono font)

## Run

```bash
# Install Dioxus CLI
cargo install dioxus-cli

# Run desktop app
dx serve --platform desktop

# Build release
dx build --platform desktop --release
```

## Requirements
- Rust 1.75+
- `libgit2` system library
    - Ubuntu/Debian: `sudo apt install libgit2-dev`
    - Arch: `sudo pacman -S libgit2`
    - macOS: `brew install libgit2`

## Project Structure
```
src/
├── main.rs              # Entry point, window config
├── app.rs               # Root component + global state
├── theme.rs             # 9 themes + contributor color engine
├── components/
│   ├── home_screen.rs   # Repo open/clone screen
│   ├── toolbar.rs       # Top navigation
│   ├── tree_canvas.rs   # SVG tree visualization
│   ├── left_panel.rs    # Commit details
│   ├── right_panel.rs   # Diff stats
│   └── settings.rs      # Theme selector
└── git/
    ├── loader.rs        # Open local / clone remote
    └── parser.rs        # Git history → tree data structures

assets/
└── style.css            # Terminal theme styling
```

## Roadmap
| Version | Features |
|---|---|
| v0.1 | Tree render, click panels, 9 themes, local + remote |
| v0.2 | Zoom/pan, search, diff viewer, minimap, keyboard nav, per-file changes |
| v0.3 | Stats screen, export SVG/PNG, animations, CRT overlay, copy hash |
| v1.0 | Polish + Android port via Dioxus mobile |

## Author
MahiroJV
