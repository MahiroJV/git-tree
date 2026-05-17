use crate::components::user_repos::UserRepos;
use crate::git::auth::{clear_token, load_token, poll_for_token, request_device_code, AuthToken};
use crate::git::loader;
use crate::git::parser::RepoTree;
use crate::git::search::{fmt_stars, search_github, SearchResult};
use crate::recent;
use dioxus::document::eval;
use dioxus::prelude::*;
use rfd::AsyncFileDialog;

// ── OAuth UI state machine ────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum OAuthState {
    Idle,
    RequestingCode,
    WaitingForUser {
        user_code: String,
        verification_uri: String,
        device_code: String,
        interval: u64,
    },
    Error(String),
}

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

    let mut search_input = use_signal(String::new);

    #[allow(clippy::redundant_closure)]
    let mut search_results = use_signal(|| Vec::<SearchResult>::new());

    let mut is_searching = use_signal(|| false);
    let mut search_error = use_signal(|| Option::<String>::None);
    let mut cloning_url = use_signal(|| Option::<String>::None);

    #[allow(clippy::redundant_closure)]
    let mut auth_token: Signal<Option<AuthToken>> = use_signal(|| load_token());
    let mut oauth_state = use_signal(|| OAuthState::Idle);

    #[allow(clippy::redundant_closure)]
    let mut recent_repos = use_signal(|| recent::load_recent());

    let displayed_error = local_error.read().clone().or(props.initial_error.clone());

    // ── Open local ────────────────────────────────────────────────────────
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

    // ── Filtered recents ──────────────────────────────────────────────────
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

    // ── Clone a URL ───────────────────────────────────────────────────────
    let mut clone_url_fn = move |url: String, display_name: String| {
        local_error.set(None);
        search_error.set(None);
        cloning_url.set(Some(url.clone()));
        is_cloning.set(true);

        let token = auth_token.read().clone();
        let clone_url = if let Some(ref t) = token {
            inject_token_into_url(&url, &t.access_token)
        } else {
            url.clone()
        };

        spawn(async move {
            let (tx, rx) = tokio::sync::oneshot::channel();
            std::thread::spawn(move || {
                let result = loader::load_remote(&clone_url);
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

            // ── Tab bar ───────────────────────────────────────────────────
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
                // MY REPOS tab — only visible when logged in
                if auth_token.read().is_some() {
                    button {
                        class: if *tab.read() == "myrepos" { "tab tab-active" } else { "tab" },
                        style: "color: var(--success);",
                        onclick: move |_| tab.set("myrepos"),
                        "[ MY REPOS ]"
                    }
                }
            }

            div { class: "home-form",

                // ── LOCAL ─────────────────────────────────────────────────
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
                                        AsyncFileDialog::new().pick_folder().await
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

                // ── REMOTE ────────────────────────────────────────────────
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

                // ── SEARCH ONLINE ─────────────────────────────────────────
                if *tab.read() == "search" {
                    {
                        let token_read = auth_token.read().clone();
                        let state_read = oauth_state.read().clone();

                        rsx! {
                            div {
                                style: "width: 100%; max-width: 580px; \
                                        display: flex; flex-direction: column; gap: 8px;",

                                // Logged in banner
                                if let Some(ref token) = token_read {
                                    div {
                                        style: "display: flex; align-items: center; \
                                                gap: 12px; padding: 8px 14px; \
                                                border: 1px solid var(--success); \
                                                background: rgba(0,255,133,0.05); \
                                                font-size: 10px; letter-spacing: 0.1em;",
                                        span {
                                            style: "color: var(--success); flex: 1;",
                                            "● GITHUB"
                                            if let Some(ref u) = token.username {
                                                span { style: "color: var(--text);", " @{u}" }
                                            }
                                            span {
                                                style: "color: var(--text-muted); margin-left: 8px;",
                                                "5 000 req/h"
                                            }
                                        }
                                        button {
                                            class: "btn-primary",
                                            style: "font-size: 9px; padding: 4px 10px; \
                                                    border-color: var(--danger); \
                                                    color: var(--danger);",
                                            onclick: move |_| {
                                                clear_token();
                                                auth_token.set(None);
                                                oauth_state.set(OAuthState::Idle);
                                                // If on myrepos tab, go back to search
                                                if *tab.read() == "myrepos" {
                                                    tab.set("search");
                                                }
                                            },
                                            "[ SIGN OUT ]"
                                        }
                                    }
                                }

                                // Not logged in idle
                                if token_read.is_none() && state_read == OAuthState::Idle {
                                    div {
                                        style: "display: flex; align-items: center; \
                                                gap: 12px; padding: 8px 14px; \
                                                border: 1px solid var(--border); \
                                                font-size: 10px; letter-spacing: 0.08em;",
                                        span {
                                            style: "color: var(--text-muted); flex: 1;",
                                            "Sign in for private repos + higher rate limits"
                                        }
                                        button {
                                            class: "btn-primary",
                                            style: "font-size: 9px; padding: 4px 10px;",
                                            onclick: move |_| {
                                                oauth_state.set(OAuthState::RequestingCode);
                                                spawn(async move {
                                                    match request_device_code().await {
                                                        Ok(start) => {
                                                            let _ = webbrowser::open(
                                                                &start.verification_uri
                                                            );
                                                            let device_code = start.device_code.clone();
                                                            let interval = start.interval;
                                                            oauth_state.set(OAuthState::WaitingForUser {
                                                                user_code: start.user_code,
                                                                verification_uri: start.verification_uri,
                                                                device_code: start.device_code,
                                                                interval: start.interval,
                                                            });
                                                            spawn(async move {
                                                                match poll_for_token(device_code, interval).await {
                                                                    Ok(token) => {
                                                                        auth_token.set(Some(token));
                                                                        oauth_state.set(OAuthState::Idle);
                                                                    }
                                                                    Err(e) => {
                                                                        oauth_state.set(OAuthState::Error(e.to_string()));
                                                                    }
                                                                }
                                                            });
                                                        }
                                                        Err(e) => {
                                                            oauth_state.set(OAuthState::Error(e.to_string()));
                                                        }
                                                    }
                                                });
                                            },
                                            "[ SIGN IN WITH GITHUB ]"
                                        }
                                    }
                                }

                                // Requesting code spinner
                                if token_read.is_none() && state_read == OAuthState::RequestingCode {
                                    div {
                                        style: "padding: 8px 14px; border: 1px solid var(--border); \
                                                font-size: 10px; color: var(--text-muted); \
                                                letter-spacing: 0.1em;",
                                        "> contacting GitHub "
                                        span { class: "loading-blink", style: "font-size: 13px;", "█" }
                                    }
                                }

                                // Waiting for user — show code
                                if let OAuthState::WaitingForUser {
                                    ref user_code,
                                    ref verification_uri,
                                    ..
                                } = state_read {
                                    if token_read.is_none() {
                                        {
                                            let code  = user_code.clone();
                                            let uri   = verification_uri.clone();
                                            let code_c = code.clone();
                                            let uri_c  = uri.clone();
                                            rsx! {
                                                div {
                                                    style: "display: flex; flex-direction: column; \
                                                            gap: 10px; padding: 16px 14px; \
                                                            border: 1px solid var(--accent); \
                                                            background: rgba(155,93,229,0.06);",
                                                    div {
                                                        style: "font-size: 10px; color: var(--text-muted); \
                                                                letter-spacing: 0.1em;",
                                                        "1. Browser opened → go to "
                                                        span { style: "color: var(--accent);", "{uri}" }
                                                    }
                                                    div {
                                                        style: "display: flex; align-items: center; gap: 12px;",
                                                        div {
                                                            style: "font-size: 24px; font-weight: 700; \
                                                                    letter-spacing: 0.3em; color: var(--accent); \
                                                                    border: 2px solid var(--accent); \
                                                                    padding: 10px 20px;",
                                                            "{code}"
                                                        }
                                                        button {
                                                            class: "btn-primary",
                                                            style: "font-size: 9px; padding: 4px 10px;",
                                                            onclick: move |_| {
                                                                eval(&format!(
                                                                    "navigator.clipboard.writeText('{}')",
                                                                    code_c
                                                                ));
                                                            },
                                                            "⊞ COPY"
                                                        }
                                                    }
                                                    div {
                                                        style: "font-size: 10px; color: var(--text-muted); \
                                                                letter-spacing: 0.1em;",
                                                        "2. Enter the code above → click "
                                                        span { style: "color: var(--success);", "Authorize" }
                                                        " → come back here"
                                                    }
                                                    div {
                                                        style: "font-size: 10px; color: var(--text-muted); \
                                                                display: flex; align-items: center; gap: 6px;",
                                                        span { class: "loading-blink", style: "font-size: 12px;", "█" }
                                                        "waiting for authorization..."
                                                    }
                                                    div {
                                                        style: "display: flex; gap: 8px;",
                                                        button {
                                                            class: "btn-primary",
                                                            style: "font-size: 9px; padding: 4px 10px;",
                                                            onclick: move |_| { let _ = webbrowser::open(&uri_c); },
                                                            "↗ REOPEN BROWSER"
                                                        }
                                                        button {
                                                            class: "btn-primary",
                                                            style: "font-size: 9px; padding: 4px 10px; \
                                                                    border-color: var(--danger); \
                                                                    color: var(--danger);",
                                                            onclick: move |_| oauth_state.set(OAuthState::Idle),
                                                            "[ CANCEL ]"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // OAuth error
                                if let OAuthState::Error(ref msg) = state_read {
                                    if token_read.is_none() {
                                        {
                                            let msg = msg.clone();
                                            rsx! {
                                                div {
                                                    style: "display: flex; align-items: center; \
                                                            gap: 12px; padding: 8px 14px; \
                                                            border: 1px solid var(--danger); \
                                                            font-size: 10px;",
                                                    span { style: "color: var(--danger); flex: 1;", "✗ {msg}" }
                                                    button {
                                                        class: "btn-primary",
                                                        style: "font-size: 9px; padding: 4px 10px;",
                                                        onclick: move |_| oauth_state.set(OAuthState::Idle),
                                                        "[ RETRY ]"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Search input
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
                                            Ok(results) => { search_results.set(results); is_searching.set(false); }
                                            Err(e) => { search_error.set(Some(e.to_string())); is_searching.set(false); }
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
                                        Ok(results) => { search_results.set(results); is_searching.set(false); }
                                        Err(e) => { search_error.set(Some(e.to_string())); is_searching.set(false); }
                                    }
                                });
                            },
                            if *is_searching.read() { "SEARCHING..." } else { "SEARCH →" }
                        }
                    }

                    if *is_searching.read() {
                        div {
                            class: "loading-cursor",
                            style: "font-size: 11px; letter-spacing: 0.1em; margin-top: 4px;",
                            "> searching GitHub "
                            span { class: "loading-blink", style: "font-size: 13px;", "█" }
                        }
                    }

                    if let Some(err) = search_error.read().clone() {
                        p { class: "error-msg", "! {err}" }
                    }

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

                    if !search_results.read().is_empty() {
                        div { class: "search-results-header",
                            span {
                                style: "font-size: 10px; color: var(--text-muted); letter-spacing: 0.15em;",
                                "— {search_results.read().len()} RESULTS —"
                            }
                        }
                        div { class: "search-results-list",
                            for result in search_results.read().clone() {
                                {
                                    let clone_url   = result.clone_url.clone();
                                    let repo_name   = result.full_name.clone();
                                    let is_this_one = cloning_url.read().as_deref() == Some(&clone_url);
                                    let disabled    = *is_cloning.read();
                                    let stars_fmt   = fmt_stars(result.stargazers_count);
                                    let forks_fmt   = fmt_stars(result.forks_count);
                                    let lang = result.language.clone().unwrap_or_else(|| "—".to_string());
                                    let desc = result.description.clone().unwrap_or_default();
                                    let license = result.license.as_ref()
                                        .and_then(|l| l.spdx_id.clone()).unwrap_or_default();
                                    let topics = result.topics.clone().unwrap_or_default();
                                    let cu2 = clone_url.clone();
                                    let rn2 = repo_name.clone();

                                    rsx! {
                                        div {
                                            class: "search-result-card",
                                            key: "{clone_url}",
                                            div { class: "search-result-top",
                                                span { class: "search-result-name", "{repo_name}" }
                                                button {
                                                    class: if is_this_one {
                                                        "btn-primary search-clone-btn search-clone-btn--active"
                                                    } else { "btn-primary search-clone-btn" },
                                                    disabled: disabled,
                                                    onclick: move |_| clone_url_fn(cu2.clone(), rn2.clone()),
                                                    if is_this_one { "CLONING..." } else { "CLONE →" }
                                                }
                                            }
                                            if !desc.is_empty() {
                                                p { class: "search-result-desc", "{desc}" }
                                            }
                                            if !topics.is_empty() {
                                                div { class: "search-result-topics",
                                                    for topic in topics.iter().take(5) {
                                                        span { class: "search-topic-badge", key: "{topic}", "{topic}" }
                                                    }
                                                }
                                            }
                                            div { class: "search-result-meta",
                                                if lang != "—" {
                                                    span { class: "search-meta-item",
                                                        span { class: "search-meta-lang-dot", "●" }
                                                        " {lang}"
                                                    }
                                                }
                                                span { class: "search-meta-item", "★ {stars_fmt}" }
                                                span { class: "search-meta-item", "⑂ {forks_fmt}" }
                                                if !license.is_empty() && license != "NOASSERTION" {
                                                    span { class: "search-meta-item search-meta-license", "{license}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── MY REPOS ──────────────────────────────────────────────
                if *tab.read() == "myrepos" {
                    if let Some(ref token) = auth_token.read().clone() {
                        UserRepos {
                            token: token.access_token.clone(),
                            on_load: props.on_load,
                            on_loading: props.on_loading,
                        }
                    }
                }

                // Shared error for local/remote tabs
                if *tab.read() != "search" && *tab.read() != "myrepos" {
                    if let Some(err) = &displayed_error {
                        p { class: "error-msg", "! {err}" }
                    }
                }
            }

            // Recent repos — only on local/remote tabs
            if *tab.read() == "local" || *tab.read() == "remote" {
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
                                            span { class: "recent-item-path text-muted", "{repo.path}" }
                                            span { class: "recent-item-time text-muted", "{repo.opened_at}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "home-footer",
                span { class: "text-muted", "git-tree v0.4 · built with Rust + Dioxus" }
            }
        }
    }
}

fn inject_token_into_url(url: &str, token: &str) -> String {
    if url.starts_with("https://") {
        url.replacen("https://", &format!("https://{}@", token), 1)
    } else {
        url.to_string()
    }
}
