// git/export.rs — Generate a self-contained SVG from the current tree state
use crate::components::tree_canvas::{BranchStyle, TreeDirection};
use crate::git::parser::{CommitNode, RepoTree};

// Mirror the same layout constants used in tree_canvas.rs
const H_CANVAS_HEIGHT: f64 = 500.0;
const H_V_MAIN: f64 = 250.0;
const H_V_BRANCH_UP: f64 = 130.0;
const H_V_BRANCH_DOWN: f64 = 370.0;
const V_CANVAS_WIDTH: f64 = 600.0;
const V_H_MAIN: f64 = 300.0;
const V_H_BRANCH_LEFT: f64 = 175.0;
const V_H_BRANCH_RIGHT: f64 = 425.0;
const NODE_RADIUS: f64 = 10.0;
const FLAT_HALF: f64 = 28.0;

// ── Path helpers ─────────────────────────────────────────────────────────────

fn bezier_h(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let cx = x1 + (x2 - x1) * 0.5;
    format!("M {x1} {y1} C {cx} {y1}, {cx} {y2}, {x2} {y2}")
}

fn bezier_v(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let cy = y1 + (y2 - y1) * 0.5;
    format!("M {x1} {y1} C {x1} {cy}, {x2} {cy}, {x2} {y2}")
}

fn geo_h_approach(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (y2 - y1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }

    let ramp = (y2 - y1).abs().min((x2 - x1).abs() * 0.80);
    let xc = if x2 >= x1 { x2 - ramp } else { x2 + ramp };

    format!("M {x1} {y1} L {xc} {y1} L {x2} {y2}")
}

fn geo_h_leave(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (y2 - y1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }

    let ramp = (y2 - y1).abs().min((x2 - x1).abs() * 0.80);
    let xc = if x2 >= x1 { x1 + ramp } else { x1 - ramp };

    format!("M {x1} {y1} L {xc} {y2} L {x2} {y2}")
}

fn geo_v_approach(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (x2 - x1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }

    let ramp = (x2 - x1).abs().min((y2 - y1).abs() * 0.80);
    let yc = if y2 >= y1 { y2 - ramp } else { y2 + ramp };

    format!("M {x1} {y1} L {x1} {yc} L {x2} {y2}")
}

fn geo_v_leave(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (x2 - x1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }

    let ramp = (x2 - x1).abs().min((y2 - y1).abs() * 0.80);
    let yc = if y2 >= y1 { y1 + ramp } else { y1 - ramp };

    format!("M {x1} {y1} L {x2} {yc} L {x2} {y2}")
}

// ── Main export function ─────────────────────────────────────────────────────

pub fn generate_svg(
    tree: &RepoTree,
    node_spacing: f64,
    show_merges: bool,
    direction: &TreeDirection,
    branch_style: &BranchStyle,
) -> String {
    let is_vertical = *direction == TreeDirection::Vertical;
    let is_geometric = *branch_style == BranchStyle::Geometric;

    let visible: Vec<&CommitNode> = tree
        .commits
        .iter()
        .filter(|c| show_merges || !c.is_merge)
        .collect();

    let count = visible.len().max(1);

    let (canvas_width, canvas_height) = if is_vertical {
        (V_CANVAS_WIDTH, (count as f64 * node_spacing) + 200.0)
    } else {
        ((count as f64 * node_spacing) + 200.0, H_CANVAS_HEIGHT)
    };

    let positioned: Vec<(&CommitNode, f64, f64)> = visible
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let (x, y) = if is_vertical {
                let y = 100.0 + i as f64 * node_spacing;

                let x = if commit.is_merge {
                    if i % 2 == 0 {
                        V_H_BRANCH_LEFT
                    } else {
                        V_H_BRANCH_RIGHT
                    }
                } else {
                    V_H_MAIN
                };

                (x, y)
            } else {
                let x = 100.0 + i as f64 * node_spacing;

                let y = if commit.is_merge {
                    if i % 2 == 0 {
                        H_V_BRANCH_UP
                    } else {
                        H_V_BRANCH_DOWN
                    }
                } else {
                    H_V_MAIN
                };

                (x, y)
            };

            (*commit, x, y)
        })
        .collect();

    let w = canvas_width as u32;
    let h = canvas_height as u32;

    let mut s = String::with_capacity(64 * 1024);

    // ── SVG header ─────────────────────────────────────────────────────────

    s.push_str(&format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}"
     xmlns="http://www.w3.org/2000/svg"
     font-family="'Space Mono', 'Courier New', monospace">

  <rect width="100%" height="100%" fill="#000000"/>

  <defs>
    <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
      <circle cx="1" cy="1" r="0.8" fill="#111111"/>
    </pattern>
  </defs>

  <rect width="100%" height="100%" fill="url(#grid)"/>

"##,
        w = w,
        h = h
    ));

    // ── Main spine ─────────────────────────────────────────────────────────

    if is_vertical {
        s.push_str(&format!(
            r##"  <line x1="{x}" y1="40" x2="{x}" y2="{y2}"
        stroke="#9B5DE5" stroke-width="2" opacity="0.6"/>
"##,
            x = V_H_MAIN,
            y2 = canvas_height as u32 - 40
        ));
    } else {
        s.push_str(&format!(
            r##"  <line x1="40" y1="{y}" x2="{x2}" y2="{y}"
        stroke="#9B5DE5" stroke-width="2" opacity="0.6"/>
"##,
            y = H_V_MAIN,
            x2 = canvas_width as u32 - 40
        ));
    }

    // ── Branch connectors ──────────────────────────────────────────────────

    for (commit, x, y) in &positioned {
        if !commit.is_merge {
            continue;
        }

        let color = &commit.color;

        if is_geometric {
            let (d_approach, d_flat, d_leave) = if is_vertical {
                let y_top = *y - FLAT_HALF;
                let y_bottom = *y + FLAT_HALF;

                (
                    geo_v_approach(V_H_MAIN, *y - node_spacing, *x, y_top),
                    format!("M {x} {y_top} L {x} {y_bottom}"),
                    geo_v_leave(*x, y_bottom, V_H_MAIN, *y + node_spacing),
                )
            } else {
                let x_left = *x - FLAT_HALF;
                let x_right = *x + FLAT_HALF;

                (
                    geo_h_approach(*x - node_spacing, H_V_MAIN, x_left, *y),
                    format!("M {x_left} {y} L {x_right} {y}"),
                    geo_h_leave(x_right, *y, *x + node_spacing, H_V_MAIN),
                )
            };

            for d in &[d_approach, d_flat, d_leave] {
                s.push_str(&format!(
                    r##"  <path d="{d}" stroke="{color}" stroke-width="1.5" fill="none"/>
"##
                ));
            }
        } else {
            let (d_in, d_out) = if is_vertical {
                (
                    bezier_v(V_H_MAIN, *y - node_spacing, *x, *y),
                    bezier_v(*x, *y, V_H_MAIN, *y + node_spacing),
                )
            } else {
                (
                    bezier_h(*x - node_spacing, H_V_MAIN, *x, *y),
                    bezier_h(*x, *y, *x + node_spacing, H_V_MAIN),
                )
            };

            s.push_str(&format!(
                r##"  <!-- branch: {} -->
  <path d="{}" stroke="{color}" stroke-width="1.5" fill="none"/>
  <path d="{}" stroke="{color}" stroke-width="1.5" fill="none"/>
"##,
                &commit.short_hash, d_in, d_out
            ));
        }
    }

    // ── Commit nodes ───────────────────────────────────────────────────────

    for (commit, x, y) in &positioned {
        let color = &commit.color;
        let fill = "#000000";

        let (label_x, label_y, anchor) = if is_vertical {
            (*x + NODE_RADIUS + 10.0, *y + 4.0, "start")
        } else {
            (*x, *y + NODE_RADIUS + 18.0, "middle")
        };

        if commit.is_head {
            let (hx, hy, ha) = if is_vertical {
                (*x + NODE_RADIUS + 10.0, *y - NODE_RADIUS - 2.0, "start")
            } else {
                (*x, *y - NODE_RADIUS - 14.0, "middle")
            };

            s.push_str(&format!(
                r##"  <text x="{hx}" y="{hy}" text-anchor="{ha}"
        font-size="10" fill="{color}" font-weight="bold"
        letter-spacing="0.1em">HEAD</text>
"##
            ));
        }

        for (ti, tag) in commit.tags.iter().enumerate() {
            let (tx, ty, ta) = if is_vertical {
                (
                    *x + NODE_RADIUS + 10.0,
                    *y - NODE_RADIUS - 16.0 - ti as f64 * 14.0,
                    "start",
                )
            } else {
                (*x, *y - NODE_RADIUS - 28.0 - ti as f64 * 15.0, "middle")
            };

            s.push_str(&format!(
                r##"  <text x="{tx}" y="{ty}" text-anchor="{ta}"
        font-size="9" fill="#00FF85">◆ {tag}</text>
"##
            ));
        }

        s.push_str(&format!(
            r##"  <circle cx="{x}" cy="{y}" r="{r}"
        fill="{fill}" stroke="{color}" stroke-width="2"/>
"##,
            r = NODE_RADIUS
        ));

        if commit.is_merge {
            s.push_str(&format!(
                r##"  <text x="{x}" y="{ly}" text-anchor="middle"
        font-size="8" fill="{color}">M</text>
"##,
                ly = *y + 3.0
            ));
        }

        s.push_str(&format!(
            r##"  <text x="{label_x}" y="{label_y}" text-anchor="{anchor}"
        font-size="10" fill="#555555"
        letter-spacing="0.05em">{hash}</text>
"##,
            hash = commit.short_hash
        ));
    }

    // ── Footer ─────────────────────────────────────────────────────────────

    s.push_str(&format!(
        r##"
  <text x="{}" y="{}" text-anchor="end"
        font-size="9" fill="#222222"
        letter-spacing="0.12em">git-tree v0.3.0</text>
</svg>
"##,
        w - 10,
        h - 8
    ));

    s
}
