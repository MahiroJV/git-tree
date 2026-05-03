use crate::git::loader;
use crate::git::parser::RepoTree;
use crate::git::search::{fmt_stars, search_github, SearchResult};
use crate::recent;
use dioxus::prelude::*;
#[allow(unused_imports)]
use rfd::FileDialog;

#[derive(Props, Clone, PartialEq)]
pub struct HomeScreenProps {
    pub initial_error: Option<String>,
    pub on_load:       EventHandler<RepoTree>,
    pub on_loading:    EventHandler<String>,
    pub on_error:      EventHandler<String>,
}

#[component]
pub fn HomeScreen(props: HomeScreenProps) -> Element {
    let mut local_path    = use_signal(String::new);
    let mut remote_url    = use_signal(String::new);
    let mut local_error   = use_signal(|| Option::<String>::None);
    let mut tab           = use_signal(|| "local");
    let mut recent_search = use_signal(String::new);
    let mut is_cloning    = use_signal(|| false);

    // ── Search state ──────────────────────────────────────────────────────────
    let mut search_input   = use_signal(String::new);
    let mut search_results = use_signal(|| Vec::<SearchResult>::new());
    let mut is_searching   = use_signal(|| false);
    let mut search_error   = use_signal(|| Option::<String>::None);
    let mut cloning_url    = use_signal(|| Option::<String>::None); // which result is cloning

    #[allow(clippy::redundant_closure)]
    let mut recent_repos = use_signal(|| recent::load_recent());

    let displayed_error = local_error.read().clone().or(props.initial_error.clone());

    // ── Open local ────────────────────────────────────────────────────────────
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

    // ── Filtered recents ──────────────────────────────────────────────────────
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

    // ── Clone a URL (used for remote tab AND search results) ──────────────────
    let mut clone_url_fn = move |url: String, display_name: String| {
        local_error.set(None);
        search_error.set(None);
        cloning_url.set(Some(url.clone()));
        is_cloning.set(true);

        spawn(async move {
            let (tx, rx) = tokio::sync::oneshot::channel();
            std::thread::spawn(move || {
                let result = loader::load_remote(&url);
                let _ = tx.send(result);
            });

            match rx.await {
                Ok(Ok(tree)) => {
                    is_cloning.set(false);
                    cloning_url.set(None);
                    props.on_load.call(tree);
                }
                Ok(Err(e)) => {
                    is_cloning.set(false);
                    cloning_url.set(None);
                    let msg = format!("Clone failed for {display_name}: {e}");
                    if *tab.read() == "search" {
                        search_error.set(Some(msg));
                    } else {
                        local_error.set(Some(msg));
                    }
                }
                Err(_) => {
                    is_cloning.set(false);
                    cloning_url.set(None);
                    search_error.set(Some("Clone task panicked".into()));
                }
            }
        });
    };

    rsx! {
        div {
            class: "home-screen",

            h1 { class: "ascii-header", "GIT-TREE" }
            p  { class: "home-tagline", "VISUALIZE YOUR GIT HISTORY — TERMINAL STYLE" }

            // ── Tab bar ───────────────────────────────────────────────────────
            div {
                class: "tab-bar",
                button {
                    class: if *tab.read() == "local"  { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set("local"),
                    "[ LOCAL FOLDER ]"
                }
                button {
                    class: if *tab.read() == "remote" { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set("remote"),
                    "[ REMOTE URL ]"
                }
                button {
                    class: if *tab.read() == "search" { "tab tab-active" } else { "tab" },
                    onclick: move |_| tab.set("search"),
                    "[ SEARCH ONLINE ]"
                }
            }

            // ── Tab content ───────────────────────────────────────────────────
            div { class: "home-form",

                // ── LOCAL ─────────────────────────────────────────────────────
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
                                    if let Some(folder) =
                                        rfd::AsyncFileDialog::new().pick_folder().await
                                    {
                                        local_path.set(
                                            folder.path().to_string_lossy().to_string()
                                        );
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
                                    Ok(tree)  => props.on_load.call(tree),
                                    Err(e)    => local_error.set(
                                        Some(format!("Error: {}", e))
                                    ),
                                }
                            },
                            "OPEN →"
                        }
                    }
                }

                // ── REMOTE ────────────────────────────────────────────────────
                if *tab.read() == "remote" {
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
                                let name = url.clone();
                                clone_url_fn(url, name);
                            },
                            if *is_cloning.read() { "CLONING..." } else { "CLONE →" }
                        }
                    }

                    if *is_cloning.read() {
                        div {
                            class: "loading-cursor",
                            style: "font-size: 11px; letter-spacing: 0.1em; margin-top: 4px;",
                            "> cloning repository "
                            span { class: "loading-blink", style: "font-size: 13px;", "█" }
                        }
                    }
                }

                // ── SEARCH ONLINE ─────────────────────────────────────────────
                if *tab.read() == "search" {
                    // Search input row
                    div { class: "input-group",
                        span { class: "prompt", "> QUERY:" }
                        input {
                            class: "terminal-input",
                            r#type: "text",
                            placeholder: "e.g. rust async runtime, neovim plugin...",
                            value: "{search_input}",
                            disabled: *is_searching.read() || *is_cloning.read(),
                            oninput: move |e| search_input.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    let q = search_input.read().clone();
                                    if q.trim().is_empty() { return; }
                                    search_error.set(None);
                                    search_results.set(vec![]);
                                    is_searching.set(true);
                                    spawn(async move {
                                        match search_github(&q, 15).await {
                                            Ok(results) => {
                                                search_results.set(results);
                                                is_searching.set(false);
                                            }
                                            Err(e) => {
                                                search_error.set(Some(e.to_string()));
                                                is_searching.set(false);
                                            }
                                        }
                                    });
                                }
                            },
                        }
                        button {
                            class: "btn-primary",
                            disabled: *is_searching.read() || *is_cloning.read(),
                            onclick: move |_| {
                                let q = search_input.read().clone();
                                if q.trim().is_empty() { return; }
                                search_error.set(None);
                                search_results.set(vec![]);
                                is_searching.set(true);
                                spawn(async move {
                                    match search_github(&q, 15).await {
                                        Ok(results) => {
                                            search_results.set(results);
                                            is_searching.set(false);
                                        }
                                        Err(e) => {
                                            search_error.set(Some(e.to_string()));
                                            is_searching.set(false);
                                        }
                                    }
                                });
                            },
                            if *is_searching.read() { "SEARCHING..." } else { "SEARCH →" }
                        }
                    }

                    // Searching spinner
                    if *is_searching.read() {
                        div {
                            class: "loading-cursor",
                            style: "font-size: 11px; letter-spacing: 0.1em; margin-top: 4px;",
                            "> searching GitHub "
                            span { class: "loading-blink", style: "font-size: 13px;", "█" }
                        }
                    }

                    // Search error
                    if let Some(err) = search_error.read().clone() {
                        p { class: "error-msg", "! {err}" }
                    }

                    // Cloning indicator (search tab)
                    if *is_cloning.read() {
                        if let Some(url) = cloning_url.read().clone() {
                            div {
                                class: "loading-cursor",
                                style: "font-size: 11px; letter-spacing: 0.1em;",
                                "> cloning {url} "
                                span { class: "loading-blink", style: "font-size: 13px;", "█" }
                            }
                        }
                    }

                    // Results list
                    if !search_results.read().is_empty() {
                        div { class: "search-results-header",
                            span {
                                style: "font-size: 10px; color: var(--text-muted); \
                                        letter-spacing: 0.15em;",
                                "— {search_results.read().len()} RESULTS —"
                            }
                        }

                        div { class: "search-results-list",
                            for result in search_results.read().clone() {
                                {
                                    let clone_url   = result.clone_url.clone();
                                    let repo_name   = result.full_name.clone();
                                    let is_this_one = cloning_url.read()
                                        .as_deref() == Some(&clone_url);
                                    let disabled    = *is_cloning.read();
                                    let stars_fmt   = fmt_stars(result.stargazers_count);
                                    let forks_fmt   = fmt_stars(result.forks_count);
                                    let lang        = result.language.clone()
                                        .unwrap_or_else(|| "—".to_string());
                                    let desc        = result.description.clone()
                                        .unwrap_or_default();
                                    let license     = result.license.as_ref()
                                        .and_then(|l| l.spdx_id.clone())
                                        .unwrap_or_default();
                                    let topics      = result.topics.clone()
                                        .unwrap_or_default();
                                    let cu2         = clone_url.clone();
                                    let rn2         = repo_name.clone();

                                    rsx! {
                                        div {
                                            class: "search-result-card",
                                            key: "{clone_url}",

                                            // ── Top row: name + action ────
                                            div { class: "search-result-top",
                                                span { class: "search-result-name",
                                                    "{repo_name}"
                                                }
                                                button {
                                                    class: if is_this_one {
                                                        "btn-primary search-clone-btn \
                                                         search-clone-btn--active"
                                                    } else {
                                                        "btn-primary search-clone-btn"
                                                    },
                                                    disabled: disabled,
                                                    onclick: move |_| {
                                                        clone_url_fn(cu2.clone(), rn2.clone());
                                                    },
                                                    if is_this_one { "CLONING..." }
                                                    else           { "CLONE →" }
                                                }
                                            }

                                            // ── Description ───────────────
                                            if !desc.is_empty() {
                                                p { class: "search-result-desc", "{desc}" }
                                            }

                                            // ── Topics ────────────────────
                                            if !topics.is_empty() {
                                                div { class: "search-result-topics",
                                                    for topic in topics.iter().take(5) {
                                                        span {
                                                            class: "search-topic-badge",
                                                            key: "{topic}",
                                                            "{topic}"
                                                        }
                                                    }
                                                }
                                            }

                                            // ── Meta row: lang/stars/forks/license ──
                                            div { class: "search-result-meta",
                                                if lang != "—" {
                                                    span { class: "search-meta-item",
                                                        span {
                                                            class: "search-meta-lang-dot",
                                                            "●"
                                                        }
                                                        " {lang}"
                                                    }
                                                }
                                                span { class: "search-meta-item",
                                                    "★ {stars_fmt}"
                                                }
                                                span { class: "search-meta-item",
                                                    "⑂ {forks_fmt}"
                                                }
                                                if !license.is_empty() && license != "NOASSERTION" {
                                                    span { class: "search-meta-item search-meta-license",
                                                        "{license}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Shared error (local / remote tabs) ────────────────────────
                if *tab.read() != "search" {
                    if let Some(err) = &displayed_error {
                        p { class: "error-msg", "! {err}" }
                    }
                }
            }

            // ── Recent repos — only on local/remote tabs ──────────────────────
            if *tab.read() != "search" {
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
                                            span { class: "recent-item-name",  "{repo.name}" }
                                            span {
                                                class: "recent-item-path text-muted",
                                                "{repo.path}"
                                            }
                                            span {
                                                class: "recent-item-time text-muted",
                                                "{repo.opened_at}"
                                            }
                                        }
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