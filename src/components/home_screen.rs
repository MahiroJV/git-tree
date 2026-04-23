// components/home_screen.rs — Landing screen
use crate::git::loader;
use crate::git::parser::RepoTree;
use crate::recent;
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
    #[allow(clippy::redundant_closure)]
    let mut recent_repos = use_signal(|| recent::load_recent());

    //Helpers
    let mut open_local = move |path_str: String| {
        if path_str.is_empty() {
            error.set(Some("Please enter a path.".into()));
            return;
        }
        props.on_loading.call("reading commits...".into());
        let path = std::path::PathBuf::from(&path_str);
        match loader::load_local(&path) {
            Ok(tree) => {
                let _ = recent::save_recent(&path_str, &tree.repo_name);
                recent_repos.set(recent::load_recent());
                props.on_load.call(tree);
            }
            Err(e) => error.set(Some(format!("Error: {e}"))),
        }
    };

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
                                spawn(async move {
                                    if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                        local_path.set(folder.path().to_string_lossy().to_string());
                                    }
                                });
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

             // ── Recent repos ─────────────────────────────────────
            div { class: "recent-section",
                p { class: "recent-title", "— RECENT REPOS —" }

                if recent_repos.read().is_empty() {
                    p { class: "text-muted recent-empty", "no recent repos yet" }
                } else {
                    div { class: "recent-list",
                        for repo in recent_repos.read().clone() {
                            {
                                let path_str = repo.path.clone();
                                rsx! {
                                    div {
                                        class: "recent-item",
                                        onclick: move |_| open_local(path_str.clone()),
                                        span { class: "recent-item-name", "{repo.name}" }
                                        span { class: "recent-item-path text-muted", "{repo.path}" }
                                        span { class: "recent-item-time text-muted", "{repo.opened_at}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }


            div { class: "home-footer",
                span { class: "text-muted", "git-tree v0.1 · built with Rust + Dioxus" }
            }
        }
    }
}
