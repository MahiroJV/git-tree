// components/toolbar.rs
use dioxus::prelude::*;

#[component]
pub fn Toolbar(
    repo_name: String,
    on_home: EventHandler<()>,
    on_settings: EventHandler<()>,
    on_refresh: EventHandler<()>,
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
                if !repo_name.is_empty() {
                    span { class: "toolbar-repo", "/ {repo_name}" }
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
                    class: "toolbar-btn toolbar-btn-accent",
                    onclick: move |_| on_settings.call(()),
                    "[ SETTINGS ]"
                }
            }
        }
    }
}
