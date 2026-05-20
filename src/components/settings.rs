use crate::components::tree_canvas::{BranchStyle, TreeDirection};
use crate::settings_store::AppStyle;
use crate::theme::THEMES;
use dioxus::prelude::*;

#[component]
pub fn Settings(
    current_theme: String,
    font_size: u32,
    node_spacing: f64,
    show_merges: bool,
    crt_overlay: bool,
    app_style: AppStyle,
    on_theme_change: EventHandler<String>,
    on_font_size_change: EventHandler<u32>,
    on_node_spacing_change: EventHandler<f64>,
    on_show_merges_change: EventHandler<bool>,
    on_crt_overlay_change: EventHandler<bool>,
    on_back: EventHandler<()>,
    tree_direction: TreeDirection,
    on_tree_direction_change: EventHandler<TreeDirection>,
    branch_style: BranchStyle,
    on_branch_style_change: EventHandler<BranchStyle>,
    on_app_style_change: EventHandler<AppStyle>,
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
                div { class: "settings-section",
                    h3 { class: "section-title", "UI STYLE" }

                    div { class: "style-picker",

                        // ── Terminal card ─────────────────────────────────
                        {
                            let is_active = app_style == AppStyle::Terminal;
                            rsx! {
                                div {
                                    class: if is_active {
                                        "style-picker-card style-picker-card--active"
                                    } else {
                                        "style-picker-card"
                                    },
                                    onclick: move |_| on_app_style_change.call(AppStyle::Terminal),

                                    div { class: "style-picker-preview",
                                        svg {
                                            width: "100%", height: "100%",
                                            view_box: "0 0 260 100",
                                            xmlns: "http://www.w3.org/2000/svg",

                                            rect { width:"260", height:"100", fill:"#000000" }

                                            // Toolbar
                                            rect { x:"0", y:"0", width:"260", height:"18", fill:"#000000" }
                                            rect { x:"0", y:"17", width:"260", height:"1", fill:"#9B5DE5", opacity:"0.7" }
                                            text { x:"8", y:"13", font_size:"7", fill:"#9B5DE5",
                                                font_family:"'Space Mono', monospace",
                                                letter_spacing:"0.22em", "GIT-TREE" }
                                            text { x:"190", y:"13", font_size:"5.5", fill:"#555",
                                                font_family:"'Space Mono', monospace", "[ SETTINGS ]" }

                                            // Left panel
                                            rect { x:"0", y:"18", width:"58", height:"82", fill:"#0a0a0a" }
                                            rect { x:"57", y:"18", width:"1", height:"82", fill:"#9B5DE5", opacity:"0.35" }
                                            text { x:"5", y:"30", font_size:"5", fill:"#9B5DE5",
                                                font_family:"'Space Mono', monospace",
                                                letter_spacing:"0.15em", "// COMMIT" }
                                            text { x:"5", y:"42", font_size:"4.5", fill:"#555",
                                                font_family:"'Space Mono', monospace", "AUTHOR" }
                                            text { x:"5", y:"52", font_size:"5", fill:"#fff",
                                                font_family:"'Space Mono', monospace", "MahiroJV" }
                                            text { x:"5", y:"64", font_size:"4.5", fill:"#9B5DE5",
                                                font_family:"'Space Mono', monospace", "a3f9b12" }

                                            // Spine + branch
                                            line { x1:"158", y1:"22", x2:"158", y2:"98",
                                                stroke:"#9B5DE5", stroke_width:"1.5", opacity:"0.5" }
                                            path {
                                                d:"M 158 58 C 158 58, 120 58, 120 74 C 120 90, 158 90, 158 90",
                                                stroke:"#00FF85", stroke_width:"1", fill:"none"
                                            }
                                            circle { cx:"158", cy:"34", r:"5",
                                                fill:"#000", stroke:"#9B5DE5", stroke_width:"1.5" }
                                            circle { cx:"158", cy:"58", r:"5",
                                                fill:"#9B5DE5", stroke:"#9B5DE5", stroke_width:"1.5" }
                                            circle { cx:"158", cy:"90", r:"5",
                                                fill:"#000", stroke:"#9B5DE5", stroke_width:"1.5" }
                                            circle { cx:"120", cy:"74", r:"4",
                                                fill:"#000", stroke:"#00FF85", stroke_width:"1" }
                                            text { x:"120", y:"77", text_anchor:"middle",
                                                font_size:"4", fill:"#00FF85",
                                                font_family:"'Space Mono', monospace", "M" }
                                            text { x:"158", y:"25", text_anchor:"middle",
                                                font_size:"4", fill:"#9B5DE5",
                                                font_family:"'Space Mono', monospace",
                                                font_weight:"bold", "HEAD" }
                                        }
                                    }

                                    div {
                                        div { class: "style-picker-label",
                                            span { class: "style-picker-name", "Terminal" }
                                            if is_active {
                                                span { class: "style-picker-badge", "ACTIVE" }
                                            }
                                        }
                                        p { class: "style-picker-sub",
                                            "Monospace · ASCII · Retro" }
                                    }
                                }
                            }
                        }

                        // ── Modern card ───────────────────────────────────
                        {
                            let is_active = app_style == AppStyle::Modern;
                            rsx! {
                                div {
                                    class: if is_active {
                                        "style-picker-card style-picker-card--active"
                                    } else {
                                        "style-picker-card"
                                    },
                                    onclick: move |_| on_app_style_change.call(AppStyle::Modern),

                                    div { class: "style-picker-preview",
                                        svg {
                                            width: "100%", height: "100%",
                                            view_box: "0 0 260 100",
                                            xmlns: "http://www.w3.org/2000/svg",

                                            rect { width:"260", height:"100", fill:"#0F0F13" }

                                            defs {
                                                pattern {
                                                    id: "sp-dots", width:"10", height:"10",
                                                    pattern_units:"userSpaceOnUse",
                                                    circle { cx:"1", cy:"1", r:"0.4", fill:"#1C1C24" }
                                                }
                                            }
                                            rect { width:"260", height:"100", fill:"url(#sp-dots)" }

                                            // Toolbar
                                            rect { x:"0", y:"0", width:"260", height:"18", fill:"#0F0F13" }
                                            rect { x:"0", y:"17", width:"260", height:"0.5",
                                                fill:"rgba(255,255,255,0.06)" }
                                            text { x:"10", y:"13", font_size:"7.5", fill:"#5E6AD2",
                                                font_family:"system-ui, sans-serif",
                                                font_weight:"700", letter_spacing:"0.04em", "git-tree" }
                                            rect { x:"60", y:"5", width:"90", height:"10", rx:"3",
                                                fill:"#16161E",
                                                stroke:"rgba(255,255,255,0.07)", stroke_width:"0.5" }
                                            text { x:"66", y:"13", font_size:"5", fill:"#3A3A5A",
                                                font_family:"system-ui, sans-serif", "search commits..." }
                                            rect { x:"200", y:"5", width:"46", height:"10", rx:"3",
                                                fill:"none",
                                                stroke:"rgba(94,106,210,0.4)", stroke_width:"0.5" }
                                            text { x:"223", y:"13", text_anchor:"middle",
                                                font_size:"5", fill:"#5E6AD2",
                                                font_family:"system-ui, sans-serif", "Settings" }

                                            // Left panel
                                            rect { x:"0", y:"18", width:"58", height:"82", fill:"#0C0C12" }
                                            rect { x:"57", y:"18", width:"0.5", height:"82",
                                                fill:"rgba(255,255,255,0.055)" }
                                            text { x:"7", y:"30", font_size:"4.5",
                                                fill:"rgba(255,255,255,0.25)",
                                                font_family:"system-ui, sans-serif",
                                                font_weight:"600", letter_spacing:"0.1em", "COMMIT INFO" }
                                            text { x:"7", y:"44", font_size:"4.5",
                                                fill:"rgba(255,255,255,0.28)",
                                                font_family:"system-ui, sans-serif",
                                                font_weight:"600", "AUTHOR" }
                                            text { x:"7", y:"54", font_size:"5.5", fill:"#C7C7D9",
                                                font_family:"system-ui, sans-serif", "MahiroJV" }
                                            text { x:"7", y:"66", font_size:"4.5",
                                                fill:"rgba(255,255,255,0.28)",
                                                font_family:"system-ui, sans-serif",
                                                font_weight:"600", "HASH" }
                                            text { x:"7", y:"76", font_size:"5", fill:"#5E6AD2",
                                                font_family:"'Space Mono', monospace", "a3f9b12" }

                                            // Spine + branch
                                            line { x1:"158", y1:"22", x2:"158", y2:"98",
                                                stroke:"#5E6AD2", stroke_width:"1.5", opacity:"0.3" }
                                            path {
                                                d:"M 158 58 C 158 58, 120 58, 120 74 C 120 90, 158 90, 158 90",
                                                stroke:"#8B8FC8", stroke_width:"1",
                                                fill:"none", opacity:"0.8"
                                            }
                                            circle { cx:"158", cy:"34", r:"5",
                                                fill:"#0F0F13", stroke:"#5E6AD2", stroke_width:"2" }
                                            circle { cx:"158", cy:"58", r:"5",
                                                fill:"#5E6AD2", stroke:"#5E6AD2", stroke_width:"2" }
                                            circle { cx:"158", cy:"90", r:"5",
                                                fill:"#0F0F13", stroke:"#5E6AD2", stroke_width:"2" }
                                            circle { cx:"120", cy:"74", r:"4",
                                                fill:"#0F0F13", stroke:"#8B8FC8", stroke_width:"1.5" }
                                            text { x:"120", y:"77", text_anchor:"middle",
                                                font_size:"4", fill:"#8B8FC8",
                                                font_family:"system-ui, sans-serif", "M" }
                                            text { x:"158", y:"25", text_anchor:"middle",
                                                font_size:"4.5", fill:"#5E6AD2",
                                                font_family:"system-ui, sans-serif",
                                                font_weight:"700", "HEAD" }
                                        }
                                    }

                                    div {
                                        div { class: "style-picker-label",
                                            span { class: "style-picker-name", "Modern" }
                                            if is_active {
                                                span { class: "style-picker-badge", "ACTIVE" }
                                            }
                                        }
                                        p { class: "style-picker-sub",
                                            "System UI · Rounded · Linear-style" }
                                    }
                                }
                            }
                        }
                    }
                }

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
                                        class: if is_active {
                                            "theme-card theme-card-active"
                                        } else {
                                            "theme-card"
                                        },
                                        onclick: move |_| on_theme_change.call(name.to_string()),
                                        div {
                                            class: "theme-preview",
                                            style: "background: {bg}; border: 1px solid {border};",
                                            svg {
                                                width: "80", height: "40",
                                                line {
                                                    x1:"5",  y1:"20", x2:"75", y2:"20",
                                                    stroke:"{accent}", stroke_width:"1.5"
                                                }
                                                path {
                                                    d:"M 30 20 C 38 20, 40 8, 50 8",
                                                    stroke:"{success}", stroke_width:"1.5",
                                                    fill:"none"
                                                }
                                                circle { cx:"15", cy:"20", r:"3",
                                                    fill:"none", stroke:"{accent}", stroke_width:"1.5" }
                                                circle { cx:"40", cy:"20", r:"3",
                                                    fill:"none", stroke:"{accent}", stroke_width:"1.5" }
                                                circle { cx:"65", cy:"20", r:"3",
                                                    fill:"none", stroke:"{accent}", stroke_width:"1.5" }
                                                circle { cx:"50", cy:"8",  r:"3",
                                                    fill:"none", stroke:"{success}", stroke_width:"1.5" }
                                            }
                                        }
                                        div {
                                            class: "theme-name",
                                            style: "color: {accent};",
                                            "{name}"
                                        }
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

                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "FONT SIZE" }
                            span { class: "setting-label-value", "{font_size}px" }
                        }
                        div { class: "setting-options",
                            for size in [11_u32, 12, 13, 14, 16] {
                                button {
                                    class: if font_size == size {
                                        "option-btn option-btn--active"
                                    } else { "option-btn" },
                                    onclick: move |_| on_font_size_change.call(size),
                                    "{size}px"
                                }
                            }
                        }
                    }

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
                            for (label, val) in [
                                ("Compact", 80.0_f64),
                                ("Normal",  120.0),
                                ("Wide",    180.0),
                            ] {
                                button {
                                    class: if (node_spacing - val).abs() < 1.0 {
                                        "option-btn option-btn--active"
                                    } else { "option-btn" },
                                    onclick: move |_| on_node_spacing_change.call(val),
                                    "{label}"
                                }
                            }
                        }
                    }

                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "CRT SCANLINES" }
                            span {
                                class: "setting-label-value",
                                if crt_overlay { "On" } else { "Off" }
                            }
                        }
                        div { class: "setting-options",
                            button {
                                class: if crt_overlay {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| on_crt_overlay_change.call(true),
                                "[ ON ]"
                            }
                            button {
                                class: if !crt_overlay {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| on_crt_overlay_change.call(false),
                                "[ OFF ]"
                            }
                        }
                    }
                }

                // ── Tree ──────────────────────────────────────────────────
                div { class: "settings-section",
                    h3 { class: "section-title", "TREE" }

                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "MERGE COMMITS" }
                            span {
                                class: "setting-label-value",
                                if show_merges { "Visible" } else { "Hidden" }
                            }
                        }
                        div { class: "setting-options",
                            button {
                                class: if show_merges {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| on_show_merges_change.call(true),
                                "[ SHOW ]"
                            }
                            button {
                                class: if !show_merges {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| on_show_merges_change.call(false),
                                "[ HIDE ]"
                            }
                        }
                    }

                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "TREE DIRECTION" }
                            span {
                                class: "setting-label-value",
                                match tree_direction {
                                    TreeDirection::Horizontal => "Horizontal",
                                    TreeDirection::Vertical   => "Vertical",
                                }
                            }
                        }
                        div { class: "setting-options",
                            button {
                                class: if matches!(tree_direction, TreeDirection::Horizontal) {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| {
                                    on_tree_direction_change.call(TreeDirection::Horizontal)
                                },
                                "Horizontal"
                            }
                            button {
                                class: if matches!(tree_direction, TreeDirection::Vertical) {
                                    "option-btn option-btn--active"
                                } else { "option-btn" },
                                onclick: move |_| {
                                    on_tree_direction_change.call(TreeDirection::Vertical)
                                },
                                "Vertical"
                            }
                        }
                    }

                    div { class: "setting-row",
                        div { class: "setting-label",
                            span { class: "setting-label-title", "BRANCH STYLE" }
                            span {
                                class: "setting-label-value",
                                match branch_style {
                                    BranchStyle::Curved    => "Curved",
                                    BranchStyle::Geometric => "Geometric",
                                }
                            }
                        }
                        div { class: "setting-options",
                            button {
                                class: if matches!(branch_style, BranchStyle::Curved) {
                                    "option-btn option-btn--active branch-style-btn"
                                } else {
                                    "option-btn branch-style-btn"
                                },
                                title: "Smooth bezier curves",
                                onclick: move |_| {
                                    on_branch_style_change.call(BranchStyle::Curved)
                                },
                                svg {
                                    width: "56", height: "28", view_box: "0 0 56 28",
                                    line { x1:"0", y1:"18", x2:"56", y2:"18",
                                        stroke:"currentColor", stroke_width:"1.5", opacity:"0.4" }
                                    path {
                                        d:"M 8 18 C 22 18, 20 8, 28 8 C 36 8, 34 18, 48 18",
                                        stroke:"currentColor", stroke_width:"1.5", fill:"none"
                                    }
                                    circle { cx:"28", cy:"8", r:"3.5", fill:"currentColor" }
                                }
                            }
                            button {
                                class: if matches!(branch_style, BranchStyle::Geometric) {
                                    "option-btn option-btn--active branch-style-btn"
                                } else {
                                    "option-btn branch-style-btn"
                                },
                                title: "Flat-top geometric steps",
                                onclick: move |_| {
                                    on_branch_style_change.call(BranchStyle::Geometric)
                                },
                                svg {
                                    width: "56", height: "28", view_box: "0 0 56 28",
                                    line { x1:"0", y1:"18", x2:"56", y2:"18",
                                        stroke:"currentColor", stroke_width:"1.5", opacity:"0.4" }
                                    path {
                                        d:"M 6 18 L 16 8 L 40 8 L 50 18",
                                        stroke:"currentColor", stroke_width:"1.5", fill:"none"
                                    }
                                    circle { cx:"28", cy:"8", r:"3.5", fill:"currentColor" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
