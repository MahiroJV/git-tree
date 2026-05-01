use crate::git::loader;
use crate::git::parser::RepoTree;
use crate::recent;
use dioxus::prelude::*;
#[allow(unused_imports)]
use rfd::FileDialog;

#[derive(Props, Clone, PartialEq)]
pub struct HomeScreenProps {
    pub initial_error: Option<String>,
    pub on_load: EventHandler<RepoTree>,
    pub on_loading: EventHandler<String>,
    pub on_error: EventHandler<String>,
}

#[component]
pub fn HomeScreen(props: HomeScreenProps) -> Element {
    let mut local_path = use_signal(String::new);
    let mut remote_url = use_signal(String::new);
    let mut local_error = use_signal(|| Option::<String>::None);
    let mut tab = use_signal(|| "local");
    let mut recent_search = use_signal(String::new);
    let mut is_cloning = use_signal(|| false);
    #[allow(clippy::redundant_closure)]
    let mut recent_repos = use_signal(|| recent::load_recent());

    // Show either a local error (bad path) or one passed down from app (failed clone)
    let displayed_error = local_error.read().clone().or(props.initial_error.clone());

    let mut open_local = move |path_str: String| {
        if path_str.is_empty() {
            local_error.set(Some("Please enter a path.".into()));
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
            Err(e) => local_error.set(Some(format!("Error: {e}"))),
        }
    };

    let filtered_recents = use_memo(move || {
        let q = recent_search.read().to_lowercase();
        recent_repos
            .read()
            .iter()
            .filter(|r| {
                q.is_empty()
                    || r.name.to_lowercase().contains(&q)
                    || r.path.to_lowercase().contains(&q)
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            class: "home-screen",

            h1 { class: "ascii-header", "GIT-TREE" }
            p { class: "home-tagline", "VISUALIZE YOUR GIT HISTORY — TERMINAL STYLE" }

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
                                    local_error.set(Some("Please enter a path.".into()));
                                    return;
                                }
                                local_error.set(None);
                                props.on_loading.call("reading commits...".into());
                                let path = std::path::PathBuf::from(&path_str);
                                match loader::load_local(&path) {
                                    Ok(tree) => props.on_load.call(tree),
                                    Err(e) => local_error.set(Some(format!("Error: {}", e))),
                                }
                            },
                            "OPEN →"
                        }
                    }
                } else {
                    // ── Remote clone ──────────────────────────────────────────────
                    // NOTE: We intentionally do NOT switch to Screen::Loading here.
                    // Switching screens unmounts this component, which invalidates the
                    // EventHandler closures captured by the spawn task — the callbacks
                    // would silently fire into the void. Instead we show inline loading
                    // state and keep this component mounted for the full duration of the
                    // clone, then call on_load / set local error when done.
                    div { class: "input-group",
                        span { class: "prompt", "> URL:" }
                        input {
                            class: "terminal-input",
                            r#type: "text",
                            placeholder: "https://github.com/user/repo",
                            value: "{remote_url}",
                            disabled: *is_cloning.read(),
                            oninput: move |e| remote_url.set(e.value()),
                        }
                        button {
                            class: "btn-primary",
                            disabled: *is_cloning.read(),
                            onclick: move |_| {
                                let url = remote_url.read().clone();
                                if url.is_empty() {
                                    local_error.set(Some("Please enter a URL.".into()));
                                    return;
                                }
                                local_error.set(None);
                                is_cloning.set(true);

                                // Spawn the clone on a real OS thread (libgit2 is blocking
                                // and can hang tokio's thread-pool on some systems). We pair
                                // it with a oneshot channel so the Dioxus async runtime can
                                // await completion without blocking the UI thread.
                                spawn(async move {
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    std::thread::spawn(move || {
                                        let result = loader::load_remote(&url);
                                        let _ = tx.send(result);
                                    });

                                    match rx.await {
                                        Ok(Ok(tree)) => {
                                            is_cloning.set(false);
                                            props.on_load.call(tree);
                                        }
                                        Ok(Err(e)) => {
                                            is_cloning.set(false);
                                            local_error.set(Some(format!("Clone failed: {e}")));
                                        }
                                        Err(_) => {
                                            is_cloning.set(false);
                                            local_error.set(Some("Clone task panicked".into()));
                                        }
                                    }
                                });
                            },
                            if *is_cloning.read() { "CLONING..." } else { "CLONE →" }
                        }
                    }

                    // Inline loading indicator — visible while clone is in progress
                    if *is_cloning.read() {
                        div {
                            class: "loading-cursor",
                            style: "font-size: 11px; letter-spacing: 0.1em; margin-top: 4px;",
                            "> cloning repository "
                            span { class: "loading-blink", style: "font-size: 13px;", "█" }
                        }
                    }
                }

                if let Some(err) = &displayed_error {
                    p { class: "error-msg", "! {err}" }
                }
            }

            // ── Recent repos ──────────────────────────────────────────────
            div { class: "recent-section",
                p { class: "recent-title", "— RECENT REPOS —" }

                if !recent_repos.read().is_empty() {
                    div { class: "input-group recent-search-wrap",
                        span { class: "prompt", ">" }
                        input {
                            class: "terminal-input",
                            r#type: "text",
                            placeholder: "filter recent...",
                            value: "{recent_search}",
                            oninput: move |e| recent_search.set(e.value()),
                        }
                        if !recent_search.read().is_empty() {
                            button {
                                class: "btn-primary",
                                onclick: move |_| recent_search.set(String::new()),
                                "✕"
                            }
                        }
                    }
                }

                if recent_repos.read().is_empty() {
                    p { class: "text-muted recent-empty", "no recent repos yet" }
                } else if filtered_recents.read().is_empty() {
                    p { class: "text-muted recent-empty", "no match" }
                } else {
                    div { class: "recent-list",
                        for repo in filtered_recents.read().clone() {
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
                span { class: "text-muted", "git-tree v0.2 · built with Rust + Dioxus" }
            }
        }
    }
}
