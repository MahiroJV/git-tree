use crate::components::tree_canvas::TreeDirection;
use crate::theme::THEMES;
use dioxus::prelude::*;

#[component]
pub fn Settings(
    current_theme: String,
    font_size: u32,
    node_spacing: f64,
    show_merges: bool,
    crt_overlay: bool,
    on_theme_change: EventHandler<String>,
    on_font_size_change: EventHandler<u32>,
    on_node_spacing_change: EventHandler<f64>,
    on_show_merges_change: EventHandler<bool>,
    on_crt_overlay_change: EventHandler<bool>,
    on_back: EventHandler<()>,
    tree_direction: TreeDirection,
    on_tree_direction_change: EventHandler<TreeDirection>,
) -> Element {
    rsx! {
        div {
            class: "settings-screen",

            // ── Header ────────────────────────────────────────────────────
            div { class: "settings-header",
                button {
                    class: "back-btn",
                    onclick: move |_| on_back.call(()),
                    "← BACK"
                }
                h2 { class: "settings-title", "// SETTINGS" }
            }

            div { class: "settings-body",

                // ── Theme ─────────────────────────────────────────────────
                div { class: "settings-section",
                    h3 { class: "section-title", "THEME" }
                    div { class: "theme-grid",
                        for theme in THEMES.iter() {
                            {
                                let name      = theme.name;
                                let is_active = current_theme == name;
                                let bg        = theme.bg;
                                let border    = theme.border;
                                let accent    = theme.accent;
                                let success   = theme.success;
                                rsx! {
                                    div {
                                        class: if is_active { "theme-card theme-card-active" } else { "theme-card" },
                                        onclick: move |_| on_theme_change.call(name.to_string()),
                                        div {
                                            class: "theme-preview",
                                            style: "background: {bg}; border: 1px solid {border};",
                                            svg {
                                                width: "80", height: "40",
                                                line { x1:"5",  y1:"20", x2:"75", y2:"20", stroke:"{accent}",  stroke_width:"1.5" }
                                                path { d:"M 30 20 C 38 20, 40 8, 50 8", stroke:"{success}", stroke_width:"1.5", fill:"none" }
                                                circle { cx:"15", cy:"20", r:"3", fill:"none", stroke:"{accent}",  stroke_width:"1.5" }
                                                circle { cx:"40", cy:"20", r:"3", fill:"none", stroke:"{accent}",  stroke_width:"1.5" }
                                                circle { cx:"65", cy:"20", r:"3", fill:"none", stroke:"{accent}",  stroke_width:"1.5" }
                                                circle { cx:"50", cy:"8",  r:"3", fill:"none", stroke:"{success}", stroke_width:"1.5" }
                                            }
                                        }
                                        div { class: "theme-name", style: "color: {accent};", "{name}" }
                                        if is_active {
                                            div { class: "theme-active-badge", "● ACTIVE" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Display ───────────────────────────────────────────────
                div { class: "settings-section",
                    h3 { class: "section-title", "DISPLAY" }

                    // Font size
                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "FONT SIZE" }
                            span { class: "setting-label-value", "{font_size}px" }
                        }
                        div { class: "setting-options",
                            for size in [11_u32, 12, 13, 14, 16] {
                                button {
                                    class: if font_size == size { "option-btn option-btn--active" } else { "option-btn" },
                                    onclick: move |_| on_font_size_change.call(size),
                                    "{size}px"
                                }
                            }
                        }
                    }

                    // Node spacing
                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "NODE SPACING" }
                            span {
                                class: "setting-label-value",
                                {
                                    if node_spacing < 100.0 { "Compact" }
                                    else if node_spacing > 140.0 { "Wide" }
                                    else { "Normal" }
                                }
                            }
                        }
                        div { class: "setting-options",
                            for (label, val) in [("Compact", 80.0_f64), ("Normal", 120.0), ("Wide", 180.0)] {
                                button {
                                    class: if (node_spacing - val).abs() < 1.0 { "option-btn option-btn--active" } else { "option-btn" },
                                    onclick: move |_| on_node_spacing_change.call(val),
                                    "{label}"
                                }
                            }
                        }
                    }

                    // CRT overlay
                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "CRT SCANLINES" }
                            span { class: "setting-label-value", if crt_overlay { "On" } else { "Off" } }
                        }
                        div { class: "setting-options",
                            button {
                                class: if crt_overlay { "option-btn option-btn--active" } else { "option-btn" },
                                onclick: move |_| on_crt_overlay_change.call(true),
                                "[ ON ]"
                            }
                            button {
                                class: if !crt_overlay { "option-btn option-btn--active" } else { "option-btn" },
                                onclick: move |_| on_crt_overlay_change.call(false),
                                "[ OFF ]"
                            }
                        }
                    }
                }

                // ── Tree ──────────────────────────────────────────────────
                div { class: "settings-section",
                    h3 { class: "section-title", "TREE" }

                    // Show merge commits
                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "MERGE COMMITS" }
                            span { class: "setting-label-value", if show_merges { "Visible" } else { "Hidden" } }
                        }
                        div { class: "setting-options",
                            button {
                                class: if show_merges { "option-btn option-btn--active" } else { "option-btn" },
                                onclick: move |_| on_show_merges_change.call(true),
                                "[ SHOW ]"
                            }
                            button {
                                class: if !show_merges { "option-btn option-btn--active" } else { "option-btn" },
                                onclick: move |_| on_show_merges_change.call(false),
                                "[ HIDE ]"
                            }
                        }
                    }

                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "TREE DIRECTION" }
                            span { class: "setting-label-value",
                                match tree_direction {
                                    TreeDirection::Horizontal => "Horizontal",
                                    TreeDirection::Vertical => "Vertical",
                                }
                            }
                        }
                        div { class: "setting-options",
                             button {
                                class: if matches!(tree_direction, TreeDirection::Horizontal) {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| on_tree_direction_change.call(TreeDirection::Horizontal),
                                "Horizontal"
                            }
                            button {
                                class: if matches!(tree_direction, TreeDirection::Vertical) {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| on_tree_direction_change.call(TreeDirection::Vertical),
                                "Vertical"
                            }
                        }
                    }
                }
            }
        }
    }
}
