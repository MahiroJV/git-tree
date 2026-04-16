// components/tree_canvas.rs — SVG tree visualization
use crate::git::parser::CommitNode;
use crate::git::parser::RepoTree;
use dioxus::prelude::*;

const NODE_RADIUS: f64 = 10.0;
const H_SPACING: f64 = 120.0;
const CANVAS_HEIGHT: f64 = 500.0;
const V_MAIN: f64 = 250.0; // dead center vertically
const V_BRANCH_UP: f64 = 130.0;
const V_BRANCH_DOWN: f64 = 370.0;

#[component]
pub fn TreeCanvas(
    tree: Option<RepoTree>,
    selected_hash: Option<String>,
    on_select: EventHandler<CommitNode>,
) -> Element {
    let Some(tree) = tree else {
        return rsx! {
            div { class: "canvas-empty", "> NO REPOSITORY LOADED" }
        };
    };

    let commit_count = tree.commits.len().max(1);
    let canvas_width = (commit_count as f64 * H_SPACING) + 200.0;

    let positioned: Vec<(CommitNode, f64, f64)> = tree
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let x = 100.0 + (i as f64 * H_SPACING);
            let y = if commit.is_merge {
                if i % 2 == 0 {
                    V_BRANCH_UP
                } else {
                    V_BRANCH_DOWN
                }
            } else {
                V_MAIN
            };
            (commit.clone(), x, y)
        })
        .collect();

    rsx! {
        div {
            class: "canvas-wrapper",
            div {
                class: "canvas-scroll",

                svg {
                    width: "{canvas_width}",
                    height: "{CANVAS_HEIGHT}",
                    xmlns: "http://www.w3.org/2000/svg",

                    // Subtle dot grid background
                    defs {
                        pattern {
                            id: "dotgrid",
                            width: "40", height: "40",
                            pattern_units: "userSpaceOnUse",
                            circle { cx: "1", cy: "1", r: "0.8", fill: "#1a1a1a" }
                        }
                    }
                    rect { width: "100%", height: "100%", fill: "url(#dotgrid)" }

                    // Main branch line — full width, dead center
                    line {
                        x1: "40",
                        y1: "{V_MAIN}",
                        x2: "{canvas_width - 40.0}",
                        y2: "{V_MAIN}",
                        stroke: "var(--accent)",
                        stroke_width: "2",
                        opacity: "0.6"
                    }

                    // Branch bezier curves for merge commits
                    for (commit, x, y) in &positioned {
                        if commit.is_merge {
                            // curve from main → branch node
                            path {
                                d: bezier(*x - H_SPACING, V_MAIN, *x, *y),
                                stroke: "{commit.color}",
                                stroke_width: "1.5",
                                fill: "none"
                            }
                            // curve from branch node → back to main
                            path {
                                d: bezier(*x, *y, *x + H_SPACING, V_MAIN),
                                stroke: "{commit.color}",
                                stroke_width: "1.5",
                                fill: "none"
                            }
                        }
                    }

                    // Commit nodes on top
                    for (commit, x, y) in positioned {
                        CommitDot {
                            commit: commit.clone(),
                            x,
                            y,
                            selected: selected_hash.as_deref() == Some(&commit.hash),
                            on_click: on_select,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommitDot(
    commit: CommitNode,
    x: f64,
    y: f64,
    selected: bool,
    on_click: EventHandler<CommitNode>,
) -> Element {
    let color = commit.color.clone();
    let fill = if selected {
        color.clone()
    } else {
        "#000000".to_string()
    };
    let stroke_w = if selected { "3" } else { "2" };
    let c = commit.clone();

    rsx! {
        g {
            style: "cursor: pointer;",
            onclick: move |_| on_click.call(c.clone()),

            // Glow ring when selected
            if selected {
                circle {
                    cx: "{x}", cy: "{y}",
                    r: "{NODE_RADIUS + 6.0}",
                    fill: "none",
                    stroke: "{color}",
                    stroke_width: "1",
                    opacity: "0.25"
                }
            }

            // Main circle
            circle {
                cx: "{x}", cy: "{y}",
                r: "{NODE_RADIUS}",
                fill: "{fill}",
                stroke: "{color}",
                stroke_width: "{stroke_w}"
            }

            // HEAD label above node
            if commit.is_head {
                text {
                    x: "{x}", y: "{y - NODE_RADIUS - 14.0}",
                    text_anchor: "middle",
                    font_size: "10",
                    font_family: "Space Mono, monospace",
                    fill: "var(--accent)",
                    font_weight: "bold",
                    letter_spacing: "0.1em",
                    "HEAD"
                }
            }

            // Tag badges
            for (i, tag) in commit.tags.iter().enumerate() {
                text {
                    x: "{x}", y: "{y - NODE_RADIUS - 28.0 - (i as f64 * 15.0)}",
                    text_anchor: "middle",
                    font_size: "9",
                    font_family: "Space Mono, monospace",
                    fill: "var(--success)",
                    "◆ {tag}"
                }
            }

            // Short hash below node
            text {
                x: "{x}", y: "{y + NODE_RADIUS + 18.0}",
                text_anchor: "middle",
                font_size: "10",
                font_family: "Space Mono, monospace",
                fill: "var(--text-muted)",
                letter_spacing: "0.05em",
                "{commit.short_hash}"
            }

            // Merge indicator dot inside circle
            if commit.is_merge {
                text {
                    x: "{x}", y: "{y + 4.0}",
                    text_anchor: "middle",
                    font_size: "8",
                    font_family: "Space Mono, monospace",
                    fill: "{color}",
                    "M"
                }
            }
        }
    }
}

fn bezier(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let cx = x1 + (x2 - x1) * 0.5;
    format!("M {x1} {y1} C {cx} {y1}, {cx} {y2}, {x2} {y2}")
}
