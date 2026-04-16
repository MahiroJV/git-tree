// components/tree_canvas.rs — Main SVG tree visualization
use dioxus::prelude::*;
use crate::git::{parser::RepoTree, parser::CommitNode};

#[derive(Props, Clone, PartialEq)]
pub struct TreeCanvasProps {
    pub tree: Option<RepoTree>,
    pub selected_hash: Option<String>,
    pub on_select: EventHandler<CommitNode>,
}

// Layout constants
const NODE_RADIUS: f64 = 10.0;
const H_SPACING: f64 = 120.0;   // horizontal space between commits
const V_MAIN: f64 = 200.0;      // Y position of main branch
const V_BRANCH_UP: f64 = 90.0;  // Y for branches going up
const V_BRANCH_DOWN: f64 = 310.0; // Y for branches going down

#[component]
pub fn TreeCanvas(props: TreeCanvasProps) -> Element {
    let tree = match &props.tree {
        Some(t) => t,
        None => return rsx! { div { class: "canvas-empty", "No repository loaded." } },
    };

    // Calculate canvas width based on commit count
    let commit_count = tree.commits.len().max(1);
    let canvas_width = (commit_count as f64 * H_SPACING) + 200.0;
    let canvas_height = 400.0;

    // Build a list of (commit, x, y) for rendering
    let positioned: Vec<(&CommitNode, f64, f64)> = tree
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let x = 100.0 + (i as f64 * H_SPACING);
            // Main branch commits stay at V_MAIN
            // Merge commits alternate up/down (simplified v0.1)
            let y = if commit.is_merge {
                if i % 2 == 0 { V_BRANCH_UP } else { V_BRANCH_DOWN }
            } else {
                V_MAIN
            };
            (commit, x, y)
        })
        .collect();

    rsx! {
        div {
            class: "canvas-wrapper",

            // Horizontal scrollable SVG canvas
            div {
                class: "canvas-scroll",
                style: "overflow-x: auto; overflow-y: hidden;",

                svg {
                    width: "{canvas_width}",
                    height: "{canvas_height}",
                    xmlns: "http://www.w3.org/2000/svg",

                    // ── Grid lines (subtle terminal feel) ──
                    defs {
                        pattern {
                            id: "grid",
                            width: "40",
                            height: "40",
                            pattern_units: "userSpaceOnUse",
                            line {
                                x1: "0", y1: "0", x2: "0", y2: "40",
                                stroke: "#111111", stroke_width: "0.5"
                            }
                            line {
                                x1: "0", y1: "0", x2: "40", y2: "0",
                                stroke: "#111111", stroke_width: "0.5"
                            }
                        }
                    }
                    rect {
                        width: "100%", height: "100%",
                        fill: "url(#grid)"
                    }

                    // ── Main branch line (horizontal) ──
                    line {
                        x1: "60",
                        y1: "{V_MAIN}",
                        x2: "{canvas_width - 60.0}",
                        y2: "{V_MAIN}",
                        stroke: "var(--accent)",
                        stroke_width: "2",
                        stroke_dasharray: "none",
                    }

                    // ── Branch Bézier curves ──
                    for (commit, x, y) in &positioned {
                        if commit.is_merge {
                            // Draw bezier from main to branch point and back
                            path {
                                d: bezier_path(
                                    *x - H_SPACING, V_MAIN,
                                    *x, *y
                                ),
                                stroke: "{commit.color}",
                                stroke_width: "2",
                                fill: "none",
                            }
                            path {
                                d: bezier_path(
                                    *x, *y,
                                    *x + H_SPACING, V_MAIN
                                ),
                                stroke: "{commit.color}",
                                stroke_width: "2",
                                fill: "none",
                            }
                        }
                    }

                    // ── Commit nodes ──
                    for (commit, x, y) in &positioned {
                        CommitNodeSvg {
                            commit: (*commit).clone(),
                            x: *x,
                            y: *y,
                            selected: props.selected_hash.as_deref() == Some(&commit.hash),
                            on_click: props.on_select.clone(),
                        }
                    }
                }
            }
        }
    }
}

// ── SVG Commit Node Component ─────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct CommitNodeSvgProps {
    commit: CommitNode,
    x: f64,
    y: f64,
    selected: bool,
    on_click: EventHandler<CommitNode>,
}

#[component]
fn CommitNodeSvg(props: CommitNodeSvgProps) -> Element {
    let commit = props.commit.clone();
    let stroke_width = if props.selected { 3.0 } else { 2.0 };
    let fill = if props.selected { props.commit.color.clone() } else { "var(--bg)".to_string() };

    rsx! {
        g {
            class: "commit-node",
            style: "cursor: pointer;",
            onclick: move |_| props.on_click.call(commit.clone()),

            // Outer ring (selected state)
            if props.selected {
                circle {
                    cx: "{props.x}",
                    cy: "{props.y}",
                    r: "{NODE_RADIUS + 5.0}",
                    fill: "none",
                    stroke: "{props.commit.color}",
                    stroke_width: "1",
                    opacity: "0.3",
                }
            }

            // Main circle
            circle {
                cx: "{props.x}",
                cy: "{props.y}",
                r: "{NODE_RADIUS}",
                fill: "{fill}",
                stroke: "{props.commit.color}",
                stroke_width: "{stroke_width}",
            }

            // HEAD indicator
            if props.commit.is_head {
                text {
                    x: "{props.x}",
                    y: "{props.y - NODE_RADIUS - 16.0}",
                    text_anchor: "middle",
                    font_size: "10",
                    font_family: "Space Mono, monospace",
                    fill: "var(--accent)",
                    "HEAD"
                }
            }

            // Tag badges
            for (i, tag) in props.commit.tags.iter().enumerate() {
                text {
                    x: "{props.x}",
                    y: "{props.y - NODE_RADIUS - 28.0 - (i as f64 * 14.0)}",
                    text_anchor: "middle",
                    font_size: "9",
                    font_family: "Space Mono, monospace",
                    fill: "var(--success)",
                    "◆ {tag}"
                }
            }

            // Short hash below the node
            text {
                x: "{props.x}",
                y: "{props.y + NODE_RADIUS + 16.0}",
                text_anchor: "middle",
                font_size: "10",
                font_family: "Space Mono, monospace",
                fill: "var(--text-muted)",
                "{props.commit.short_hash}"
            }

            // Merge indicator
            if props.commit.is_merge {
                text {
                    x: "{props.x}",
                    y: "{props.y + 4.0}",
                    text_anchor: "middle",
                    font_size: "9",
                    font_family: "Space Mono, monospace",
                    fill: "var(--bg)",
                    "M"
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build an SVG cubic bezier path string between two points
fn bezier_path(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let cx1 = x1 + (x2 - x1) * 0.5;
    let cy1 = y1;
    let cx2 = x1 + (x2 - x1) * 0.5;
    let cy2 = y2;
    format!(
        "M {x1} {y1} C {cx1} {cy1}, {cx2} {cy2}, {x2} {y2}"
    )
}