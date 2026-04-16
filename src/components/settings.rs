// components/settings.rs — Theme selector
use dioxus::prelude::*;
use crate::theme::THEMES;

#[component]
pub fn Settings(
    current_theme: String,
    on_theme_change: EventHandler<String>,
    on_back: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "settings-screen",

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
                    h3 { class: "section-title", "THEME" }

                    div { class: "theme-grid",
                        for theme in THEMES.iter() {
                            {
                                let name = theme.name;
                                let is_active = current_theme == name;
                                let bg = theme.bg;
                                let border = theme.border;
                                let accent = theme.accent;
                                let success = theme.success;
                                rsx! {
                                    div {
                                        class: if is_active { "theme-card theme-card-active" } else { "theme-card" },
                                        onclick: move |_| on_theme_change.call(name.to_string()),

                                        div {
                                            class: "theme-preview",
                                            style: "background: {bg}; border: 1px solid {border};",
                                            svg {
                                                width: "80", height: "40",
                                                line { x1: "5", y1: "20", x2: "75", y2: "20", stroke: "{accent}", stroke_width: "1.5" }
                                                path { d: "M 30 20 C 38 20, 40 8, 50 8", stroke: "{success}", stroke_width: "1.5", fill: "none" }
                                                circle { cx: "15", cy: "20", r: "3", fill: "none", stroke: "{accent}", stroke_width: "1.5" }
                                                circle { cx: "40", cy: "20", r: "3", fill: "none", stroke: "{accent}", stroke_width: "1.5" }
                                                circle { cx: "65", cy: "20", r: "3", fill: "none", stroke: "{accent}", stroke_width: "1.5" }
                                                circle { cx: "50", cy: "8",  r: "3", fill: "none", stroke: "{success}", stroke_width: "1.5" }
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

                div { class: "settings-section",
                    h3 { class: "section-title", "DISPLAY" }
                    p { class: "text-muted setting-coming", "─ font size, animations, CRT overlay — coming in v0.2 ─" }
                }
            }
        }
    }
}