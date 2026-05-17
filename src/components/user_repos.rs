// components/user_repos.rs — Shows logged-in user's GitHub repos
use crate::git::auth::{fetch_profile, fetch_user_repos, GithubProfile, GithubRepo};
use crate::git::loader;
use crate::git::parser::RepoTree;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum LoadState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

#[derive(Clone, PartialEq)]
enum SortBy {
    Updated,
    Stars,
    Name,
}

#[component]
pub fn UserRepos(
    token: String,
    on_load: EventHandler<RepoTree>,
    on_loading: EventHandler<String>,
) -> Element {
    let mut load_state = use_signal(|| LoadState::Idle);
    let mut profile: Signal<Option<GithubProfile>> = use_signal(|| None);

    #[allow(clippy::redundant_closure)]
    let mut repos: Signal<Vec<GithubRepo>> = use_signal(|| vec![]);
    let mut filter = use_signal(String::new);
    let mut sort_by = use_signal(|| SortBy::Updated);
    let mut show_private = use_signal(|| true);
    let mut cloning_url = use_signal(|| Option::<String>::None);
    let mut clone_error = use_signal(|| Option::<String>::None);

    // ── Auto-load on mount ────────────────────────────────────────────────
    let token_for_effect = token.clone();
    use_effect(move || {
        let t = token_for_effect.clone();
        load_state.set(LoadState::Loading);
        spawn(async move {
            let (profile_res, repos_res) = tokio::join!(fetch_profile(&t), fetch_user_repos(&t));

            match (profile_res, repos_res) {
                (Ok(p), Ok(r)) => {
                    profile.set(Some(p));
                    repos.set(r);
                    load_state.set(LoadState::Loaded);
                }
                (Err(e), _) | (_, Err(e)) => {
                    load_state.set(LoadState::Error(e.to_string()));
                }
            }
        });
    });

    // ── Filtered + sorted repos ───────────────────────────────────────────
    let displayed = use_memo(move || {
        let q = filter.read().to_lowercase();
        let show_priv = *show_private.read();
        let sort = sort_by.read().clone();

        let mut list: Vec<GithubRepo> = repos
            .read()
            .iter()
            .filter(|r| {
                (show_priv || !r.private)
                    && (q.is_empty()
                        || r.name.to_lowercase().contains(&q)
                        || r.description
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&q))
            })
            .cloned()
            .collect();

        match sort {
            SortBy::Stars => list.sort_by_key(|b| std::cmp::Reverse(b.stargazers_count)),
            SortBy::Name => list.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Updated => {} // already sorted by updated_at from API
        }

        list
    });

    // ── Clone handler ─────────────────────────────────────────────────────
    // let token_for_clone = token.clone();
    // let mut clone_repo = move |repo: GithubRepo| {
    //     clone_error.set(None);
    //     cloning_url.set(Some(repo.clone_url.clone()));
    //
    //     // Inject token for private repo access
    //     let clone_url = format!(
    //         "https://{}@{}",
    //         token_for_clone,
    //         repo.clone_url.trim_start_matches("https://")
    //     );
    //
    //     on_loading.call(format!("cloning {}...", repo.name));
    //
    //     spawn(async move {
    //         let (tx, rx) = tokio::sync::oneshot::channel();
    //         std::thread::spawn(move || {
    //             let _ = tx.send(loader::load_remote(&clone_url));
    //         });
    //
    //         match rx.await {
    //             Ok(Ok(tree)) => {
    //                 cloning_url.set(None);
    //                 on_load.call(tree);
    //             }
    //             Ok(Err(e)) => {
    //                 cloning_url.set(None);
    //                 clone_error.set(Some(format!("Clone failed: {e}")));
    //             }
    //             Err(_) => {
    //                 cloning_url.set(None);
    //                 clone_error.set(Some("Clone task panicked".into()));
    //             }
    //         }
    //     });
    // };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 14px; \
                    width: 100%; max-width: 680px;",

            // ── Loading state ─────────────────────────────────────────────
            if *load_state.read() == LoadState::Loading {
                div {
                    class: "loading-cursor",
                    style: "font-size: 11px; letter-spacing: 0.1em;",
                    "> fetching your repositories "
                    span { class: "loading-blink", style: "font-size: 13px;", "█" }
                }
            }

            // ── Error state ───────────────────────────────────────────────
            if let LoadState::Error(ref msg) = *load_state.read() {
                p {
                    class: "error-msg",
                    "! {msg}"
                }
            }

            // ── Loaded ────────────────────────────────────────────────────
            if *load_state.read() == LoadState::Loaded {

                // Profile card
                if let Some(ref p) = *profile.read() {
                    div {
                        style: "display: flex; align-items: center; gap: 16px; \
                                padding: 12px 14px; \
                                border: 1px solid var(--border); \
                                background: var(--bg-secondary);",

                        // Avatar initial circle (no image loading needed)
                        div {
                            style: "width: 38px; height: 38px; border-radius: 50%; \
                                    background: var(--accent); \
                                    display: flex; align-items: center; \
                                    justify-content: center; \
                                    font-size: 16px; font-weight: 700; \
                                    color: var(--bg); flex-shrink: 0;",
                            "{p.login.chars().next().unwrap_or('?').to_uppercase()}"
                        }

                        div {
                            style: "display: flex; flex-direction: column; gap: 3px; flex: 1;",
                            div {
                                style: "display: flex; align-items: center; gap: 10px;",
                                span {
                                    style: "font-size: 13px; color: var(--accent); \
                                            font-weight: 700; letter-spacing: 0.08em;",
                                    "@{p.login}"
                                }
                                if let Some(ref name) = p.name {
                                    span {
                                        style: "font-size: 11px; color: var(--text-muted);",
                                        "{name}"
                                    }
                                }
                            }
                            if let Some(ref bio) = p.bio {
                                span {
                                    style: "font-size: 10px; color: var(--text-muted); \
                                            letter-spacing: 0.04em;",
                                    "{bio}"
                                }
                            }
                            div {
                                style: "display: flex; gap: 14px; margin-top: 2px;",
                                span {
                                    style: "font-size: 9px; color: var(--text-muted); \
                                            letter-spacing: 0.1em;",
                                    "REPOS {p.public_repos}"
                                }
                                span {
                                    style: "font-size: 9px; color: var(--text-muted); \
                                            letter-spacing: 0.1em;",
                                    "FOLLOWERS {p.followers}"
                                }
                                span {
                                    style: "font-size: 9px; color: var(--text-muted); \
                                            letter-spacing: 0.1em;",
                                    "FOLLOWING {p.following}"
                                }
                            }
                        }
                    }
                }

                // ── Toolbar: filter + sort + toggle ───────────────────────
                div {
                    style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap;",

                    // Filter input
                    div {
                        class: "input-group",
                        style: "flex: 1; min-width: 180px; padding: 5px 10px;",
                        span { class: "prompt", ">" }
                        input {
                            class: "terminal-input",
                            r#type: "text",
                            placeholder: "filter repos...",
                            value: "{filter}",
                            oninput: move |e| filter.set(e.value()),
                        }
                        if !filter.read().is_empty() {
                            button {
                                class: "btn-primary",
                                style: "font-size: 9px; padding: 2px 6px;",
                                onclick: move |_| filter.set(String::new()),
                                "✕"
                            }
                        }
                    }

                    // Sort buttons
                    div {
                        style: "display: flex; gap: 4px;",
                        for (label, variant) in [
                            ("UPDATED", SortBy::Updated),
                            ("STARS",   SortBy::Stars),
                            ("NAME",    SortBy::Name),
                        ] {
                            {
                                let is_active = *sort_by.read() == variant;
                                let v = variant.clone();
                                rsx! {
                                    button {
                                        class: if is_active {
                                            "option-btn option-btn--active"
                                        } else {
                                            "option-btn"
                                        },
                                        style: "font-size: 9px; padding: 4px 8px;",
                                        onclick: move |_| sort_by.set(v.clone()),
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }

                    // Private toggle
                    button {
                        class: if *show_private.read() {
                            "option-btn option-btn--active"
                        } else {
                            "option-btn"
                        },
                        style: "font-size: 9px; padding: 4px 8px;",
                        onclick: move |_| show_private.set(!show_private()),
                        if *show_private.read() { "🔒 PRIVATE ON" } else { "🔒 PRIVATE OFF" }
                    }
                }

                // Count
                div {
                    style: "font-size: 10px; color: var(--text-muted); \
                            letter-spacing: 0.15em; text-align: center;",
                    "— {displayed.read().len()} REPOSITORIES —"
                }

                // Clone error
                if let Some(ref err) = *clone_error.read() {
                    p { class: "error-msg", "! {err}" }
                }

                // ── Repo list ─────────────────────────────────────────────
                div {
                    style: "display: flex; flex-direction: column; gap: 6px; \
                            max-height: 420px; overflow-y: auto; padding-right: 4px;",

                    for repo in displayed.read().clone() {
                        {
                            let is_cloning = cloning_url.read()
                                .as_deref() == Some(&repo.clone_url);
                            let any_cloning = cloning_url.read().is_some();
                            let r = repo.clone();
                            let lang = repo.language.clone()
                                .unwrap_or_else(|| "—".into());
                            let desc = repo.description.clone()
                                .unwrap_or_default();
                            let updated = repo.updated.as_deref()
                                .and_then(|s| s.get(..10))
                                .unwrap_or("—")
                                .to_string();

                            rsx! {
                                div {
                                    key: "{repo.id}",
                                    style: "display: flex; flex-direction: column; \
                                            gap: 6px; padding: 10px 14px; \
                                            border: 1px solid #1a1a1a; \
                                            background: var(--bg-secondary); \
                                            transition: border-color 0.15s;",

                                    // Top row
                                    div {
                                        style: "display: flex; align-items: center; \
                                                gap: 10px;",

                                        // Private / public badge
                                        span {
                                            style: if repo.private {
                                                "font-size: 8px; letter-spacing: 0.08em; \
                                                 color: var(--accent); \
                                                 border: 1px solid var(--accent); \
                                                 padding: 1px 5px; flex-shrink: 0;"
                                            } else {
                                                "font-size: 8px; letter-spacing: 0.08em; \
                                                 color: var(--text-muted); \
                                                 border: 1px solid #333; \
                                                 padding: 1px 5px; flex-shrink: 0;"
                                            },
                                            if repo.private { "PRIVATE" } else { "PUBLIC" }
                                        }

                                        // Repo name
                                        span {
                                            style: "font-size: 12px; color: var(--accent); \
                                                    letter-spacing: 0.06em; flex: 1; \
                                                    overflow: hidden; text-overflow: ellipsis; \
                                                    white-space: nowrap;",
                                            "{repo.name}"
                                        }

                                        // Clone button
                                        button {
                                            class: "btn-primary",
                                            style: "font-size: 9px; padding: 4px 10px; flex-shrink: 0;",
                                            disabled: any_cloning,
                                            onclick: {
                                                let repo = r.clone();
                                                let token = token.clone();
                                                move |_| {
                                                    clone_error.set(None);
                                                    cloning_url.set(Some(repo.clone_url.clone()));

                                                    let clone_url = format!(
                                                        "https://{}@{}",
                                                        token,
                                                        repo.clone_url.trim_start_matches("https://")
                                                    );

                                                    on_loading.call(format!("cloning {}...", repo.name));

                                                    spawn(async move {
                                                        let (tx, rx) = tokio::sync::oneshot::channel();
                                                        std::thread::spawn(move || {
                                                            let _ = tx.send(loader::load_remote(&clone_url));
                                                        });
                                                        match rx.await {
                                                            Ok(Ok(tree)) => {
                                                                cloning_url.set(None);
                                                                on_load.call(tree);
                                                            }
                                                            Ok(Err(e)) => {
                                                                cloning_url.set(None);
                                                                clone_error.set(Some(format!("Clone failed: {e}")));
                                                            }
                                                            Err(_) => {
                                                                cloning_url.set(None);
                                                                clone_error.set(Some("Clone task panicked".into()));
                                                            }
                                                        }
                                                    });
                                                }
                                            },
                                            if is_cloning { "CLONING..." } else { "CLONE →" }
                                        }
                                    }

                                    // Description
                                    if !desc.is_empty() {
                                        p {
                                            style: "font-size: 10px; color: var(--text-muted); \
                                                    line-height: 1.5; margin: 0; \
                                                    display: -webkit-box; \
                                                    -webkit-line-clamp: 2; \
                                                    -webkit-box-orient: vertical; \
                                                    overflow: hidden;",
                                            "{desc}"
                                        }
                                    }

                                    // Meta row
                                    div {
                                        style: "display: flex; gap: 14px; \
                                                flex-wrap: wrap; align-items: center;",

                                        if lang != "—" {
                                            span {
                                                style: "font-size: 9px; \
                                                        color: var(--success); \
                                                        letter-spacing: 0.06em;",
                                                "● {lang}"
                                            }
                                        }
                                        span {
                                            style: "font-size: 9px; \
                                                    color: var(--text-muted);",
                                            "★ {repo.stargazers_count}"
                                        }
                                        span {
                                            style: "font-size: 9px; \
                                                    color: var(--text-muted);",
                                            "⑂ {repo.forks_count}"
                                        }
                                        span {
                                            style: "font-size: 9px; \
                                                    color: var(--text-muted); \
                                                    margin-left: auto;",
                                            "updated {updated}"
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
}
