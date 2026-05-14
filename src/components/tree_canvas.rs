// components/tree_canvas.rs
use crate::git::parser::CommitNode;
use crate::git::parser::RepoTree;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
// ── Layout constants ──────────────────────────────────────────────────────────

const NODE_RADIUS: f64 = 10.0;

// Horizontal mode
const H_CANVAS_HEIGHT: f64 = 500.0;
const H_V_MAIN: f64 = 250.0;
const H_V_BRANCH_UP: f64 = 130.0; // 120 px above main
const H_V_BRANCH_DOWN: f64 = 370.0; // 120 px below main

// Vertical mode
const V_CANVAS_WIDTH: f64 = 600.0;
const V_H_MAIN: f64 = 300.0;
const V_H_BRANCH_LEFT: f64 = 175.0;
const V_H_BRANCH_RIGHT: f64 = 425.0;

// Minimap
const MINIMAP_W: f64 = 180.0;
const MINIMAP_H: f64 = 110.0;
const APPROX_VW: f64 = 850.0;
const APPROX_VH: f64 = 480.0;

/// Half-width (px) of the flat-top segment in Geometric mode.
/// The merge node is centred at the midpoint of this segment.
const FLAT_HALF: f64 = 28.0;

// ── Public enums ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TreeDirection {
    Horizontal,
    Vertical,
}

/// Controls how branch connector lines are drawn.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum BranchStyle {
    Curved,
    Geometric,
}

// ── Search helper ─────────────────────────────────────────────────────────────

fn commit_matches(commit: &CommitNode, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    commit.author_name.to_lowercase().contains(&q)
        || commit.author_email.to_lowercase().contains(&q)
        || commit.message.to_lowercase().contains(&q)
        || commit.full_message.to_lowercase().contains(&q)
        || commit.short_hash.to_lowercase().contains(&q)
        || commit.hash.to_lowercase().contains(&q)
}

// ── Path builders — Curved ────────────────────────────────────────────────────

fn bezier_h(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let cx = x1 + (x2 - x1) * 0.5;
    format!("M {x1} {y1} C {cx} {y1}, {cx} {y2}, {x2} {y2}")
}

fn bezier_v(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    let cy = y1 + (y2 - y1) * 0.5;
    format!("M {x1} {y1} C {x1} {cy}, {x2} {cy}, {x2} {y2}")
}

/// H approach: stay at y1, then angle up/down to (x2, y2)
fn geo_h_approach(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (y2 - y1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }
    let ramp = (y2 - y1).abs().min((x2 - x1).abs() * 0.80);
    let xc = if x2 >= x1 { x2 - ramp } else { x2 + ramp };
    format!("M {x1} {y1} L {xc} {y1} L {x2} {y2}")
}

/// H leave: angle from (x1, y1) back to main y2, then stay flat
fn geo_h_leave(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (y2 - y1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }
    let ramp = (y2 - y1).abs().min((x2 - x1).abs() * 0.80);
    let xc = if x2 >= x1 { x1 + ramp } else { x1 - ramp };
    format!("M {x1} {y1} L {xc} {y2} L {x2} {y2}")
}

/// V approach: stay at x1, then angle out to (x2, y2)
fn geo_v_approach(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (x2 - x1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }
    let ramp = (x2 - x1).abs().min((y2 - y1).abs() * 0.80);
    let yc = if y2 >= y1 { y2 - ramp } else { y2 + ramp };
    format!("M {x1} {y1} L {x1} {yc} L {x2} {y2}")
}

/// V leave: angle from (x1, y1) back to main x2, then stay straight
fn geo_v_leave(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
    if (x2 - x1).abs() < 1.0 {
        return format!("M {x1} {y1} L {x2} {y2}");
    }
    let ramp = (x2 - x1).abs().min((y2 - y1).abs() * 0.80);
    let yc = if y2 >= y1 { y1 + ramp } else { y1 - ramp };
    format!("M {x1} {y1} L {x2} {yc} L {x2} {y2}")
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TreeCanvas(
    tree: Option<RepoTree>,
    selected_hash: Option<String>,
    search_query: String,
    node_spacing: f64,
    show_merges: bool,
    direction: TreeDirection,
    branch_style: BranchStyle,
    on_select: EventHandler<CommitNode>,
    on_deselect: EventHandler<()>,
) -> Element {
    let mut scale = use_signal(|| 1.0_f64);
    let mut offset_x = use_signal(|| 0.0_f64);
    let mut offset_y = use_signal(|| 0.0_f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_sx = use_signal(|| 0.0_f64);
    let mut drag_sy = use_signal(|| 0.0_f64);
    let mut show_minimap = use_signal(|| true);

    let Some(tree) = tree else {
        return rsx! { div { class: "canvas-empty", "> NO REPOSITORY LOADED" } };
    };

    let is_vertical = direction == TreeDirection::Vertical;
    let is_geometric = branch_style == BranchStyle::Geometric;

    let visible: Vec<CommitNode> = tree
        .commits
        .iter()
        .filter(|c| show_merges || !c.is_merge)
        .cloned()
        .collect();

    let count = visible.len().max(1);

    let (canvas_width, canvas_height) = if is_vertical {
        (V_CANVAS_WIDTH, (count as f64 * node_spacing) + 200.0)
    } else {
        ((count as f64 * node_spacing) + 200.0, H_CANVAS_HEIGHT)
    };

    let positioned: Vec<(CommitNode, f64, f64, bool)> = visible
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let (x, y) = if is_vertical {
                let y = 100.0 + i as f64 * node_spacing;
                let x = if commit.is_merge {
                    let base = if i % 2 == 0 {
                        V_H_BRANCH_LEFT
                    } else {
                        V_H_BRANCH_RIGHT
                    };
                    if is_geometric {
                        if i % 2 == 0 {
                            base - 15.0
                        } else {
                            base + 15.0
                        }
                    } else {
                        base
                    }
                } else {
                    V_H_MAIN
                };
                (x, y)
            } else {
                let x = 100.0 + i as f64 * node_spacing;
                let y = if commit.is_merge {
                    let base = if i % 2 == 0 {
                        H_V_BRANCH_UP
                    } else {
                        H_V_BRANCH_DOWN
                    };
                    if is_geometric {
                        if i % 2 == 0 {
                            base - 15.0
                        } else {
                            base + 15.0
                        }
                    } else {
                        base
                    }
                } else {
                    H_V_MAIN
                };
                (x, y)
            };
            (commit.clone(), x, y, commit_matches(commit, &search_query))
        })
        .collect();

    let mm_positioned = positioned.clone();
    let match_count = if search_query.is_empty() {
        None
    } else {
        Some(positioned.iter().filter(|(_, _, _, m)| *m).count())
    };

    // ── Keyboard nav ──────────────────────────────────────────────────────────
    let commits_for_kb = positioned
        .iter()
        .filter(|(_, _, _, m)| *m)
        .map(|(c, _, _, _)| c.clone())
        .collect::<Vec<_>>();

    let handle_key = {
        let commits = commits_for_kb.clone();
        let selected = selected_hash.clone();
        let vert = is_vertical;
        move |e: KeyboardEvent| {
            let (fwd, back) = if vert {
                (Key::ArrowDown, Key::ArrowUp)
            } else {
                (Key::ArrowRight, Key::ArrowLeft)
            };
            if e.key() == fwd {
                let next = match &selected {
                    None => commits.first().cloned(),
                    Some(h) => commits
                        .iter()
                        .position(|c| &c.hash == h)
                        .and_then(|i| commits.get(i + 1))
                        .cloned(),
                };
                if let Some(c) = next {
                    on_select.call(c);
                }
            } else if e.key() == back {
                let prev = match &selected {
                    None => commits.last().cloned(),
                    Some(h) => commits
                        .iter()
                        .position(|c| &c.hash == h)
                        .and_then(|i| i.checked_sub(1).and_then(|j| commits.get(j)))
                        .cloned(),
                };
                if let Some(c) = prev {
                    on_select.call(c);
                }
            } else if e.key() == Key::Escape {
                on_deselect.call(());
            }
        }
    };

    let hint = match match_count {
        Some(n) => format!(
            "{n} match{}  ·  {} navigate  ·  ESC deselect",
            if n == 1 { "" } else { "es" },
            if is_vertical { "↑ ↓" } else { "← →" }
        ),
        None => format!(
            "{} navigate  ·  ESC deselect  ·  CTRL+scroll zoom  ·  drag pan",
            if is_vertical { "↑ ↓" } else { "← →" }
        ),
    };

    // ── Minimap geometry (computed once, reused in rsx!) ──────────────────────
    let mm_s = (MINIMAP_W / canvas_width).min(MINIMAP_H / canvas_height);
    let s_now = *scale.read();
    let ox = *offset_x.read();
    let oy = *offset_y.read();
    #[allow(clippy::manual_clamp)]
    let vp_x = (-ox * mm_s).max(0.0_f64).min(MINIMAP_W);
    #[allow(clippy::manual_clamp)]
    let vp_y = (-oy * mm_s).max(0.0_f64).min(MINIMAP_H);
    let vp_w = ((APPROX_VW / s_now) * mm_s).min(MINIMAP_W);
    let vp_h = ((APPROX_VH / s_now) * mm_s).min(MINIMAP_H);

    let mm_s_click = mm_s;
    let s_click = s_now;

    rsx! {
        div {
            class: "canvas-wrapper",
            tabindex: "0",
            style: "overflow: hidden; cursor: grab; position: relative; outline: none;",
            onkeydown: handle_key,

            // ── Zoom controls ─────────────────────────────────────────────
            div {
                style: "position: absolute; top: 12px; right: 12px; \
                        z-index: 10; display: flex; gap: 6px;",
                button {
                    class: "toolbar-btn",
                    onclick: move |_| { let s = (*scale.read() + 0.1).min(3.0); scale.set(s); },
                    "[ + ]"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| { let s = (*scale.read() - 0.1).max(0.2); scale.set(s); },
                    "[ - ]"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| {
                        scale.set(1.0);
                        offset_x.set(0.0);
                        offset_y.set(0.0);
                    },
                    "[ RESET ]"
                }
                button {
                    class: "toolbar-btn",
                    title: "Toggle Minimap",
                    onclick: move |_| show_minimap.set(!show_minimap()),
                    if *show_minimap.read() { "[ MAP ✕ ]" } else { "[ MAP ]" }
                }
            }

            // ── Hint bar ──────────────────────────────────────────────────
            div {
                style: "position: absolute; bottom: 10px; left: 50%; \
                        transform: translateX(-50%); z-index: 10; \
                        color: var(--text-muted); font-size: 10px; \
                        letter-spacing: 0.12em; pointer-events: none; \
                        white-space: nowrap;",
                "{hint}"
            }

            if *show_minimap.read() {
                div {
                    style: "position: absolute; \
                    bottom: 36px; left: 12px; \
                    z-index: 10; \
                    background: rgba(0,0,0,0.88); \
                    border: 1px solid var(--border); \
                    padding: 4px; \
                    user-select: none;",

                    svg {
                        width: "{MINIMAP_W}",
                        height: "{MINIMAP_H}",
                        xmlns: "http://www.w3.org/2000/svg",
                        style: "display: block;",

                        // Subtle
                        defs {
                            pattern {
                                id: "mm-dots", width: "10", height: "10",
                                pattern_units: "userSpaceOnUse",
                                circle { cx: "1", cy: "1", r: "0.4", fill: "#1a1a1a" }
                            }
                        }
                        rect {
                            width: "{MINIMAP_W}",
                            height: "{MINIMAP_H}",
                            fill: "url(#mm-dots)",
                        }
                        //Clickable transparent overlay — navigate on click
                        rect {
                            width: "{MINIMAP_W}",
                            height: "{MINIMAP_H}",
                            fill: "transparent",
                            style: "cursor: crosshair;",
                            onclick: move |e| {
                                let coords = e.element_coordinates();
                                let mx = coords.x;
                                let my = coords.y;
                                // Canvas point the user clicked
                                let cx = mx / mm_s_click;
                                let cy = my / mm_s_click;
                                // Center the viewport on that canvas point.
                                offset_x.set(APPROX_VW / (2.0 * s_click) - cx);
                                offset_y.set(APPROX_VW / (2.0 * s_click) - cy);
                            },
                        }

                        //Commit dots
                        for (commit, cx, cy, matches) in mm_positioned.iter() {
                            {
                                let dot_x = cx * mm_s;
                                let dot_y = cy * mm_s;
                                let color = commit.color.clone();
                                let dot_r = if commit.is_merge { 2.5_f64 } else { 2.0_f64 };
                                let sel   = selected_hash.as_deref() == Some(&commit.hash);
                                let opacity = if search_query.is_empty() || *matches { "1" } else { "0.15" };
                                rsx! {
                                    circle {
                                        key: "mm-{commit.short_hash}",
                                        cx: "{dot_x}",
                                        cy: "{dot_y}",
                                        r: "{dot_r}",
                                        fill: "{color}",
                                        opacity: "{opacity}",
                                        stroke: if sel { "white" } else { "" },
                                        stroke_width: if sel { "0.8" } else { "0" },
                                    }
                                }
                            }
                        }
                        // Viewport indicator rectangle
                        rect {
                            x: "{vp_x}",
                            y: "{vp_y}",
                            width: "{vp_w}",
                            height: "{vp_h}",
                            fill: "rgba(155, 93, 229, 0.08)",
                            stroke: "var(--accent)",
                            stroke_width: "0.75",
                        }

                        text {
                            x: "4",
                            y: "{MINIMAP_H - 4.0}",
                            fill: "var(--text-muted)",
                            font_size: "7",
                            font_family: "Space Mono, monospace",
                            letter_spacing: "0.12em",
                            "MINIMAP"
                        }
                    }
                }
            }

            // ── Main canvas ────────────────────────────────────────────────
            div {
                style: "width: 100%; height: 100%; overflow: hidden;",

                onwheel: move |e| {
                    let delta_y = e.delta().strip_units().y;
                    let delta_x = e.delta().strip_units().x;
                    if e.modifiers().ctrl() {
                        let s = if delta_y < 0.0 {
                            (*scale.read() + 0.08).min(3.0)
                        } else {
                            (*scale.read() - 0.08).max(0.2)
                        };
                        scale.set(s);
                    } else {
                        let new_x = *offset_x.read() - delta_x;
                        let new_y = *offset_y.read() - delta_y;
                        offset_x.set(new_x);
                        offset_y.set(new_y);
                    }
                },
                onmousedown: move |e| {
                    is_dragging.set(true);
                    drag_sx.set(e.client_coordinates().x - *offset_x.read());
                    drag_sy.set(e.client_coordinates().y - *offset_y.read());
                },
                onmousemove: move |e| {
                    if *is_dragging.read() {
                        offset_x.set(e.client_coordinates().x - *drag_sx.read());
                        offset_y.set(e.client_coordinates().y - *drag_sy.read());
                    }
                },
                onmouseup:    move |_| is_dragging.set(false),
                onmouseleave: move |_| is_dragging.set(false),

                svg {
                    width:  "{canvas_width}",
                    height: "{canvas_height}",
                    xmlns:  "http://www.w3.org/2000/svg",
                    style:  "transform: scale({scale}) \
                             translate({offset_x}px, {offset_y}px); \
                             transform-origin: 0 0; display: block;",

                    defs {
                        pattern {
                            id: "dotgrid", width: "40", height: "40",
                            pattern_units: "userSpaceOnUse",
                            circle { cx:"1", cy:"1", r:"0.8", fill:"#1a1a1a" }
                        }
                    }
                    rect { width:"100%", height:"100%", fill:"url(#dotgrid)" }

                    // ── Main spine ────────────────────────────────────────
                    if is_vertical {
                        line {
                            class: "spine-animated",
                            x1: "{V_H_MAIN}", y1: "40",
                            x2: "{V_H_MAIN}", y2: "{canvas_height - 40.0}",
                            stroke: "var(--accent)", stroke_width: "2",
                            opacity: if search_query.is_empty() { "0.6" } else { "0.15" }
                        }
                    } else {
                        line {
                            class: "spine-animated",
                            x1: "40",
                            y1: "{H_V_MAIN}",
                            x2: "{canvas_width - 40.0}",
                            y2: "{H_V_MAIN}",
                            stroke: "var(--accent)", stroke_width: "2",
                            opacity: if search_query.is_empty() { "0.6" } else { "0.15" }
                        }
                    }

                    // ── Branch connectors ─────────────────────────────────
                    for (idx,(commit, x, y, matches)) in positioned.iter().enumerate() {
                        if commit.is_merge {
                            {
                                let delay = format!("animation-delay: {}ms", idx * 35);
                                let opacity = if search_query.is_empty() || *matches {
                                    "1"
                                } else {
                                    "0.08"
                                };
                                let color = commit.color.clone();

                                if is_geometric {
                                    let (d_approach, d_flat, d_leave) = if is_vertical {
                                        // flat-top is vertical: a short vertical segment at x,
                                        // centred on y
                                        let y_top    = *y - FLAT_HALF;
                                        let y_bottom = *y + FLAT_HALF;
                                        (
                                            geo_v_approach(V_H_MAIN, *y - node_spacing, *x, y_top),
                                            format!("M {x} {y_top} L {x} {y_bottom}"),
                                            geo_v_leave(*x, y_bottom, V_H_MAIN, *y + node_spacing),
                                        )
                                    } else {
                                        // flat-top is horizontal: a short horizontal segment at y,
                                        // centred on x
                                        let x_left  = *x - FLAT_HALF;
                                        let x_right = *x + FLAT_HALF;
                                        (
                                            geo_h_approach(*x - node_spacing, H_V_MAIN, x_left, *y),
                                            format!("M {x_left} {y} L {x_right} {y}"),
                                            geo_h_leave(x_right, *y, *x + node_spacing, H_V_MAIN),
                                        )
                                    };

                                    rsx! {
                                        path {
                                            class: "branch-line-animated",
                                            style: "{delay}",
                                            d: "{d_approach}",
                                            stroke: "{color}", stroke_width: "1.5",
                                            fill: "none", opacity: "{opacity}",
                                        }
                                        path {
                                            class: "branch-line-animated",
                                            style: "{delay}",
                                            d: "{d_flat}",
                                            stroke: "{color}", stroke_width: "1.5",
                                            fill: "none", opacity: "{opacity}",
                                        }
                                        path {
                                            class: "branch-line-animated",
                                            style: "{delay}",
                                            d: "{d_leave}",
                                            stroke: "{color}", stroke_width: "1.5",
                                            fill: "none", opacity: "{opacity}",
                                        }
                                    }
                                } else {
                                    // ─────────────────────────────────────────
                                    // CURVED — smooth bezier S-curves
                                    // ─────────────────────────────────────────
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
                                    rsx! {
                                        path {
                                            class: "branch-line-animated",
                                            style: "{delay}",
                                            d: "{d_in}",
                                            stroke: "{color}", stroke_width: "1.5",
                                            fill: "none", opacity: "{opacity}",
                                        }
                                        path {
                                            class: "branch-line-animated",
                                            style: "{delay}",
                                            d: "{d_out}",
                                            stroke: "{color}", stroke_width: "1.5",
                                            fill: "none", opacity: "{opacity}",
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ── Commit nodes ──────────────────────────────────────
                    for (commit, x, y, matches) in positioned {
                        CommitDot {
                            commit:   commit.clone(),
                            x, y,
                            is_vertical,
                            selected: selected_hash.as_deref() == Some(&commit.hash),
                            dimmed:   !search_query.is_empty() && !matches,
                            on_click: on_select,
                        }
                    }
                }
            }
        }
    }
}

// ── Commit dot ────────────────────────────────────────────────────────────────

#[component]
fn CommitDot(
    commit: CommitNode,
    x: f64,
    y: f64,
    is_vertical: bool,
    selected: bool,
    dimmed: bool,
    on_click: EventHandler<CommitNode>,
) -> Element {
    let mut click_id = use_signal(|| 0_u32);
    let color = commit.color.clone();
    let fill = if selected {
        color.clone()
    } else {
        "#000000".to_string()
    };
    let stroke_w = if selected { "3" } else { "2" };
    let opacity = if dimmed { "0.08" } else { "1" };
    let c = commit.clone();

    let (label_x, label_y, label_anchor) = if is_vertical {
        (x + NODE_RADIUS + 10.0, y + 4.0, "start")
    } else {
        (x, y + NODE_RADIUS + 18.0, "middle")
    };

    rsx! {
        g {
            style: "cursor: pointer; opacity: {opacity};",
            onclick: move |_| {
                if !dimmed {
                    on_click.call(c.clone());
                    click_id += 1
                }
            },

            if selected {
                circle {
                    key: "pulse-{click_id}",
                    class: "node-pulse-ring",
                    cx: "{x}", cy: "{y}", r: "{NODE_RADIUS}",
                    fill: "none",
                    stroke: "{color}",
                    stroke_width: "1",
                }
            }

            if selected {
                circle {
                    cx: "{x}", cy: "{y}", r: "{NODE_RADIUS + 6.0}",
                    fill: "none",
                    stroke: "{color}",
                    stroke_width: "1",
                    opacity: "0.25"
                }
            }

            circle {
                cx: "{x}", cy: "{y}", r: "{NODE_RADIUS}",
                fill: "{fill}", stroke: "{color}", stroke_width: "{stroke_w}"
            }

            if commit.is_head {
                text {
                    x: if is_vertical { "{x + NODE_RADIUS + 10.0}" } else { "{x}" },
                    y: if is_vertical { "{y - NODE_RADIUS - 2.0}" }
                       else           { "{y - NODE_RADIUS - 14.0}" },
                    text_anchor: if is_vertical { "start" } else { "middle" },
                    font_size: "10", font_family: "Space Mono, monospace",
                    fill: "var(--accent)", font_weight: "bold",
                    letter_spacing: "0.1em",
                    "HEAD"
                }
            }

            for (i, tag) in commit.tags.iter().enumerate() {
                text {
                    x: if is_vertical { "{x + NODE_RADIUS + 10.0}" } else { "{x}" },
                    y: if is_vertical {
                           "{y - NODE_RADIUS - 16.0 - (i as f64 * 14.0)}"
                       } else {
                           "{y - NODE_RADIUS - 28.0 - (i as f64 * 15.0)}"
                       },
                    text_anchor: if is_vertical { "start" } else { "middle" },
                    font_size: "9", font_family: "Space Mono, monospace",
                    fill: "var(--success)",
                    "◆ {tag}"
                }
            }

            text {
                x: "{label_x}", y: "{label_y}",
                text_anchor: "{label_anchor}",
                font_size: "10", font_family: "Space Mono, monospace",
                fill: "var(--text-muted)", letter_spacing: "0.05em",
                "{commit.short_hash}"
            }

            if commit.is_merge {
                text {
                    x: "{x}", y: "{y + 4.0}", text_anchor: "middle",
                    font_size: "8", font_family: "Space Mono, monospace",
                    fill: "{color}",
                    "M"
                }
            }
        }
    }
}
