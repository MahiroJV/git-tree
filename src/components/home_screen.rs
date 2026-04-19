// components/home_screen.rs — Landing screen
use crate::git::loader;
use crate::git::parser::RepoTree;
use dioxus::prelude::*;
#[allow(unused_imports)]
use rfd::FileDialog;

#[derive(Props, Clone, PartialEq)]
pub struct HomeScreenProps {
    pub on_load: EventHandler<RepoTree>,
    pub on_loading: EventHandler<String>,
}

#[component]
pub fn HomeScreen(props: HomeScreenProps) -> Element {
    let mut local_path = use_signal(String::new);
    let mut remote_url = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);
    let mut tab = use_signal(|| "local");

    rsx! {
        div {
            class: "home-screen",

            h1 {
                class: "ascii-header",
                "GIT-TREE"
            }

            p { class: "home-tagline", "VISUALIZE YOUR GIT HISTORY — TERMINAL STYLE" }

            // Tab switcher
            div {
                class: "tab-bar",
                button {
                    class: if *tab.read() == "local" { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set("local"),
                    "[ LOCAL FOLDER ]"
                }
                button {
                    class: if *tab.read() == "remote" { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set("remote"),
                    "[ REMOTE URL ]"
                }
            }

            div {
                class: "home-form",

                if *tab.read() == "local" {
                    div { class: "input-group",
                        span { class: "prompt", "> PATH:" }
                        input {
                            class: "terminal-input",
                            r#type: "text",
                            placeholder: "/home/user/my-project",
                            value: "{local_path}",
                            oninput: move |e| local_path.set(e.value()),
                        }

                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                 if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    local_path.set(path.to_string_lossy().to_string());
                                }
                            },
                            "📁"
                        }

                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                let path_str = local_path.read().clone();
                                if path_str.is_empty() {
                                    error.set(Some("Please enter a path.".into()));
                                    return;
                                }
                                props.on_loading.call("reading commits...".into());
                                let path = std::path::PathBuf::from(&path_str);
                                match loader::load_local(&path) {
                                    Ok(tree) => props.on_load.call(tree),
                                    Err(e) => error.set(Some(format!("Error: {}", e))),
                                }
                            },
                            "OPEN →"
                        }
                    }
                } else {
                    div { class: "input-group",
                        span { class: "prompt", "> URL:" }
                        input {
                            class: "terminal-input",
                            r#type: "text",
                            placeholder: "https://github.com/user/repo",
                            value: "{remote_url}",
                            oninput: move |e| remote_url.set(e.value()),
                        }
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                let url = remote_url.read().clone();
                                if url.is_empty() {
                                    error.set(Some("Please enter a URL.".into()));
                                    return;
                                }
                                props.on_loading.call("cloning repository...".into());
                                match loader::load_remote(&url) {
                                    Ok(tree) => props.on_load.call(tree),
                                    Err(e) => error.set(Some(format!("Error: {}", e))),
                                }
                            },
                            "CLONE →"
                        }
                    }
                }

                if let Some(err) = error.read().as_ref() {
                    p { class: "error-msg", "! {err}" }
                }
            }

            div { class: "recent-section",
                p { class: "text-muted", "— RECENT REPOS (coming in v0.2) —" }
            }

            div { class: "home-footer",
                span { class: "text-muted", "git-tree v0.1 · built with Rust + Dioxus" }
            }
        }
    }
}
