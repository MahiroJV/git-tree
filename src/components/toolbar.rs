// components/toolbar.rs
use dioxus::prelude::*;

#[component]
pub fn Toolbar(
    repo_name: String,
    search_query: String,
    on_home: EventHandler<()>,
    on_search: EventHandler<String>,
    on_settings: EventHandler<()>,
    on_refresh: EventHandler<()>,
    on_stats: EventHandler<()>,
    on_export: EventHandler<()>,
) -> Element {
    rsx! {
        nav {
            class: "toolbar",

            // Left — brand only
            div { class: "toolbar-left",
                span {
                    class: "toolbar-brand",
                    onclick: move |_| on_home.call(()),
                    "GIT-TREE"
                }
            }

            // Center — repo name
            div { class: "toolbar-center",
                div { class: "toolbar-search-wrap",
                    span { class: "toolbar-search-prompt", ">" }
                    input {
                        class: "toolbar-search-input",
                        r#type: "text",
                        placeholder: if repo_name.is_empty() {
                            "search commits...".to_string()
                        } else {
                            format!("search in {}...", repo_name)
                        },
                        value: "{search_query}",
                        oninput: move |e| on_search.call(e.value()),
                    }
                    // live match indicator — only visible when typing
                    if !search_query.is_empty() {
                        button {
                            class: "toolbar-search-clear",
                            onclick: move |_| on_search.call(String::new()),
                            "✕"
                        }
                    }
                }
            }


            // Right — buttons always visible
            div { class: "toolbar-right",
                button {
                    class: "toolbar-btn",
                    onclick: move |_| on_refresh.call(()),
                    "[ REFRESH ]"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| on_stats.call(()),
                    "[ STATS ]"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| on_export.call(()),
                    "[ EXPORT ]"
                }
                button {
                    class: "toolbar-btn toolbar-btn-accent",
                    onclick: move |_| on_settings.call(()),
                    "[ SETTINGS ]"
                }
            }
        }
    }
}
