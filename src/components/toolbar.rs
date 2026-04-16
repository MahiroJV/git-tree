// components/toolbar.rs — Top navigation bar
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    pub repo_name: String,
    pub on_home: EventHandler<()>,
    pub on_settings: EventHandler<()>,
    pub on_refresh: EventHandler<()>,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    rsx! {
        nav {
            class: "toolbar",

            // Left — app name + repo
            div {
                class: "toolbar",
                span {
                    class: "toolbar-brand",
                    onclick: move |_| props.on_home.call(()),
                    "GIT-TREE"
                }
                if !props.repo_name.is_empty() {
                    span { class: "toolbar-separator", "/" }
                    span { class: "toolbar-repo", "{props.repo_name}"}
                }
            }

            // Right — actions
            div {
                class: "toolbar-right",
                button {
                    class: "toolbar-btn",
                    onclick: move |_| props.on_refresh.call(()),
                    "[ REFRESH ]"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| props.on_settings.call(()),
                    "[ SETTINGS ]"
                }
            }
        }
    }
}