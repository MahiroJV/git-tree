// components/tree_canvas.rs — SVG tree visualization with zoom + pan + keyboard nav
use crate::git::parser::CommitNode;
use crate::git::parser::RepoTree;
use dioxus::prelude::*;

const NODE_RADIUS: f64 = 10.0;
const H_SPACING: f64 = 120.0;
const CANVAS_HEIGHT: f64 = 500.0;
const V_MAIN: f64 = 250.0;
const V_BRANCH_UP: f64 = 130.0;
const V_BRANCH_DOWN: f64 = 370.0;

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

#[component]
pub fn TreeCanvas(
    tree: Option<RepoTree>,
    selected_hash: Option<String>,
    search_query: String,
    on_select: EventHandler<CommitNode>,
    on_deselect: EventHandler<()>,
) -> Element {
    let mut scale = use_signal(|| 1.0_f64);
    let mut offset_x = use_signal(|| 0.0_f64);
    let mut offset_y = use_signal(|| 0.0_f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_start_x = use_signal(|| 0.0_f64);
    let mut drag_start_y = use_signal(|| 0.0_f64);

    let Some(tree) = tree else {
        return rsx! {
            div { class: "canvas-empty", "> NO REPOSITORY LOADED" }
        };
    };

    let commit_count = tree.commits.len().max(1);
    let canvas_width = (commit_count as f64 * H_SPACING) + 200.0;

    let positioned: Vec<(CommitNode, f64, f64, bool)> = tree
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
            let matches = commit_matches(commit, &search_query);
            (commit.clone(), x, y, matches)

        })
        .collect();
    // Match count for the hint label
    let match_count = if search_query.is_empty() {
        None
    } else {
        Some(positioned.iter().filter(|(_, _, _, m)| *m).count())
    };

    // ── keyboard handler ────────────────────────────────────────────────────
    // Find current index so arrow keys know where to go
    let commits_for_kb = positioned
        .iter()
        .filter(|(_, _, _, m)| *m)
        .map(|(c, _, _, _)| c.clone())
        .collect::<Vec<_>>();

    let handle_key = {
        let commits = commits_for_kb.clone();
        let selected = selected_hash.clone();
        move |e: KeyboardEvent| match e.key() {
            Key::ArrowRight => {
                let next = match &selected {
                    None => commits.first().cloned(),
                    Some(h) => {
                        let idx = commits.iter().position(|c| &c.hash == h);
                        idx.and_then(|i| commits.get(i + 1)).cloned()
                    }
                };
                if let Some(c) = next {
                    on_select.call(c);
                }
            }
            Key::ArrowLeft => {
                let prev = match &selected {
                    None => commits.last().cloned(),
                    Some(h) => {
                        let idx = commits.iter().position(|c| &c.hash == h);
                        idx.and_then(|i| i.checked_sub(1).and_then(|j| commits.get(j)))
                            .cloned()
                    }
                };
                if let Some(c) = prev {
                    on_select.call(c);
                }
            }
            Key::Escape => {
                on_deselect.call(());
            }
            _ => {}
        }
    };

    rsx! {
        div {
            class: "canvas-wrapper",
            // tabIndex so div can receive keyboard events; outline:none hides focus ring
            tabindex: "0",
            style: "overflow: hidden; cursor: grab; position: relative; outline: none;",
            onkeydown: handle_key,

            // ── zoom controls ───────────────────────────────────────────────
            div {
                style: "position: absolute; top: 12px; right: 12px; z-index: 10; display: flex; gap: 6px;",
                button {
                    class: "toolbar-btn",
                     onclick: move |_| {
                        let s = (*scale.read() + 0.1).min(3.0);
                        scale.set(s);
                    },
                    "[ + ]"
                }
                button {
                    class: "toolbar-btn",
                     onclick: move |_| {
                        let s = (*scale.read() - 0.1).max(0.2);
                        scale.set(s);
                    },
                    "[ - ]"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| { scale.set(1.0); offset_x.set(0.0); offset_y.set(0.0); },
                    "[ RESET ]"
                }
            }

            // ── keyboard hint ───────────────────────────────────────────────
            div {
                style: "position: absolute; bottom: 10px; left: 50%; transform: translateX(-50%);
                        z-index: 10; color: var(--text-muted); font-size: 10px;
                        letter-spacing: 0.12em; pointer-events: none; white-space: nowrap;",
                {
                    match match_count {
                        Some(n) => format!("{n} match{}  ·  ← → navigate  ·  ESC deselect", if n == 1 { "" } else { "es" }),
                        None => "← → navigate  ·  ESC deselect  ·  CTRL+scroll zoom  ·  drag pan".to_string(),
                    }
                }
            }


            // ── canvas area ─────────────────────────────────────────────────
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
                    drag_start_x.set(e.client_coordinates().x - *offset_x.read());
                    drag_start_y.set(e.client_coordinates().y - *offset_y.read());
                },
                onmousemove: move |e| {
                    if *is_dragging.read() {
                        offset_x.set(e.client_coordinates().x - *drag_start_x.read());
                        offset_y.set(e.client_coordinates().y - *drag_start_y.read());
                    }
                },
                onmouseup:    move |_| is_dragging.set(false),
                onmouseleave: move |_| is_dragging.set(false),

                svg {
                    width: "{canvas_width}",
                    height: "{CANVAS_HEIGHT}",
                    xmlns: "http://www.w3.org/2000/svg",
                    style: "transform: scale({scale}) translate({offset_x}px, {offset_y}px); transform-origin: 0 0; display: block;",

                    defs {
                        pattern {
                            id: "dotgrid",
                            width: "40", height: "40",
                            pattern_units: "userSpaceOnUse",
                            circle { cx: "1", cy: "1", r: "0.8", fill: "#1a1a1a" }
                        }
                    }
                    rect { width: "100%", height: "100%", fill: "url(#dotgrid)" }

                    // main branch line
                    line {
                        x1: "40",
                        y1: "{V_MAIN}",
                        x2: "{canvas_width - 40.0}",
                        y2: "{V_MAIN}",
                        stroke: "var(--accent)",
                        stroke_width: "2",
                        opacity: if search_query.is_empty() { "0.6" } else { "0.15" }
                    }

                    // branch Bézier curves
                    for (commit, x, y, matches) in &positioned {
                        if commit.is_merge {
                            {
                                let opacity = if search_query.is_empty() || *matches { "1" } else { "0.08" };
                                rsx! {
                                    path {
                                        d: bezier(*x - H_SPACING, V_MAIN, *x, *y),
                                        stroke: "{commit.color}",
                                        stroke_width: "1.5",
                                        fill: "none",
                                        opacity: "{opacity}"
                                    }
                                    path {
                                        d: bezier(*x, *y, *x + H_SPACING, V_MAIN),
                                        stroke: "{commit.color}",
                                        stroke_width: "1.5",
                                        fill: "none",
                                        opacity: "{opacity}"
                                    }
                                }
                            }
                        }
                    }

                    // commit nodes
                    for (commit, x, y, matches) in positioned {
                        CommitDot {
                            commit: commit.clone(),
                            x,
                            y,
                            selected: selected_hash.as_deref() == Some(&commit.hash),
                            dimmed: !search_query.is_empty() && !matches,
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
    dimmed: bool,
    on_click: EventHandler<CommitNode>,
) -> Element {
    let color = commit.color.clone();
    let fill = if selected {
        color.clone()
    } else {
        "#000000".to_string()
    };
    let stroke_w = if selected { "3" } else { "2" };
    let opacity = if dimmed { "0.08" } else { "1" };
    let c = commit.clone();

    rsx! {
        g {
            style: "cursor: pointer; opacity: {opacity};",
            onclick: move |_| {
                if !dimmed { on_click.call(c.clone()); }
            },

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

            circle {
                cx: "{x}", cy: "{y}",
                r: "{NODE_RADIUS}",
                fill: "{fill}",
                stroke: "{color}",
                stroke_width: "{stroke_w}"
            }

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

            text {
                x: "{x}", y: "{y + NODE_RADIUS + 18.0}",
                text_anchor: "middle",
                font_size: "10",
                font_family: "Space Mono, monospace",
                fill: "var(--text-muted)",
                letter_spacing: "0.05em",
                "{commit.short_hash}"
            }

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
