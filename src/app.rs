use crate::components::{
    diff_viewer::DiffViewer,
    home_screen::HomeScreen,
    left_panel::LeftPanel,
    onboarding::OnboardingScreen,
    right_panel::RightPanel,
    settings::Settings,
    shortcuts::ShortcutsHelp,
    stats::StatsScreen,
    toolbar::Toolbar,
    tree_canvas::{BranchStyle, TreeCanvas, TreeDirection},
};
use crate::git::export::generate_svg;
use crate::git::loader::load_commit_diff;
use crate::git::parser::CommitNode;
use crate::git::parser::RepoTree;
use crate::settings_store::{load_settings, save_settings, AppSettings, AppStyle};
use crate::theme::theme_by_name;
use crate::updater::{check_for_updates, UpdateInfo};
use dioxus::prelude::*;

const BASE_CSS: &str = include_str!("../assets/css/style.css");
const MODERN_CSS: &str = include_str!("../assets/css/modern.css");

// ── Screen enum ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum Screen {
    Home,
    Loading(String),
    Tree,
    Settings,
    Stats,
    Diff(Box<CommitNode>),
}

// ── Update state ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum UpdateState {
    /// No check has run yet this session — the default, since we no longer
    /// auto-check on startup. User triggers a check from Settings.
    Idle,
    Checking,
    Available(UpdateInfo),
    UpToDate,
    Error(String),
    /// User closed the result notification — stay quiet until they check again.
    Dismissed,
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn App() -> Element {
    let saved = load_settings();

    // ── Onboarding gate ───────────────────────────────────────────────────
    // Separate signal so we can dismiss the overlay without a full remount.
    let mut onboarding_done = use_signal(|| saved.onboarding_done);

    // ── App signals seeded from disk ──────────────────────────────────────
    let mut screen = use_signal(|| Screen::Home);
    let mut repo_tree: Signal<Option<RepoTree>> = use_signal(|| None);
    let mut selected_commit: Signal<Option<CommitNode>> = use_signal(|| None);
    let mut clone_error = use_signal(|| Option::<String>::None);
    let mut diff_loading = use_signal(|| false);

    let mut theme_name = use_signal(|| saved.theme_name.clone());
    let mut font_size = use_signal(|| saved.font_size);
    let mut node_spacing = use_signal(|| saved.node_spacing);
    let mut show_merges = use_signal(|| saved.show_merges);
    let mut crt_overlay = use_signal(|| saved.crt_overlay);
    let mut tree_direction = use_signal(|| saved.tree_direction.clone());
    let mut branch_style = use_signal(|| saved.branch_style.clone());
    let mut app_style = use_signal(|| saved.app_style.clone());

    let current_settings = move || AppSettings {
        theme_name: theme_name.read().clone(),
        font_size: *font_size.read(),
        node_spacing: *node_spacing.read(),
        show_merges: *show_merges.read(),
        crt_overlay: *crt_overlay.read(),
        tree_direction: tree_direction.read().clone(),
        branch_style: branch_style.read().clone(),
        app_style: app_style.read().clone(),
        onboarding_done: *onboarding_done.read(),
    };

    let mut search_query = use_signal(String::new);
    let mut left_open = use_signal(|| true);
    let mut right_open = use_signal(|| true);

    // ── Update state ───────────────────────────────────────────────────────
    // No auto-check on startup anymore — purely opt-in via the "Check for
    // Updates" button in Settings (see on_check_updates below).
    let mut update_state: Signal<UpdateState> = use_signal(|| UpdateState::Idle);

    let on_check_updates = move |_| {
        update_state.set(UpdateState::Checking);
        spawn(async move {
            match check_for_updates().await {
                Ok(Some(info)) => update_state.set(UpdateState::Available(info)),
                Ok(None) => update_state.set(UpdateState::UpToDate),
                Err(e) => update_state.set(UpdateState::Error(e.to_string())),
            }
        });
    };

    let zoom = use_memo(move || *font_size.read() as f64 / 13.0);

    let mut diff_cache: Signal<
        std::collections::HashMap<
            String,
            (
                Vec<crate::git::parser::FileChange>,
                crate::git::parser::DiffStats,
            ),
        >,
    > = use_signal(std::collections::HashMap::new);

    // ── Theme CSS (derived from signals) ───────────────────────────────────
    let theme_css = use_memo(move || {
        let t = theme_by_name(&theme_name.read());
        let fs = *font_size.read();
        format!(
            ":root {{ \
                --bg:{bg}; --bg-secondary:{bgs}; --text:{text}; \
                --text-muted:{tm}; --accent:{ac}; --border:{bo}; \
                --success:{su}; --danger:{da}; \
                --font-size:{fs}px; --zoom:{zoom:.4}; \
            }}",
            zoom = *zoom.read(),
            bg = t.bg,
            bgs = t.bg_secondary,
            text = t.text,
            tm = t.text_muted,
            ac = t.accent,
            bo = t.border,
            su = t.success,
            da = t.danger,
            fs = fs,
        )
    });

    // ── SVG export ─────────────────────────────────────────────────────────
    let do_export = move || {
        let maybe_tree = repo_tree.read().clone();
        let spacing_s = *node_spacing.read();
        let merges = *show_merges.read();
        let direction = tree_direction.read().clone();
        let style = branch_style.read().clone();

        if let Some(tree) = maybe_tree {
            let svg_content = generate_svg(&tree, spacing_s, merges, &direction, &style);
            spawn(async move {
                let file = rfd::AsyncFileDialog::new()
                    .set_title("Export tree as SVG")
                    .add_filter("SVG image", &["svg"])
                    .set_file_name("git-tree-export.svg")
                    .save_file()
                    .await;
                if let Some(handle) = file {
                    let _ = handle.write(svg_content.as_bytes()).await;
                }
            });
        }
    };
    let handle_export = move |_| do_export();

    // ── Keyboard shortcuts ───────────────────────────────────────────────
    let mut show_shortcuts = use_signal(|| false);

    let global_keydown = move |e: KeyboardEvent| {
        // The only text input outside Screen::Home stops propagation itself
        // (toolbar search), so anything reaching here is a real shortcut.
        if matches!(*screen.read(), Screen::Home | Screen::Loading(_)) {
            return;
        }

        let is_char = |c: &str| matches!(&e.key(), Key::Character(s) if s.eq_ignore_ascii_case(c));
        let plain = !e.modifiers().ctrl() && !e.modifiers().meta() && !e.modifiers().alt();

        if plain && is_char("?") {
            show_shortcuts.set(!show_shortcuts());
            return;
        }

        if e.key() == Key::Escape {
            if *show_shortcuts.read() {
                show_shortcuts.set(false);
            } else if matches!(*screen.read(), Screen::Settings | Screen::Stats | Screen::Diff(_)) {
                screen.set(Screen::Tree);
            }
            return;
        }

        if *show_shortcuts.read() || !plain || !matches!(*screen.read(), Screen::Tree) {
            return;
        }

        if is_char("/") {
            let _ = dioxus::document::eval(
                "document.getElementById('toolbar-search-input')?.focus()",
            );
        } else if is_char("s") {
            screen.set(Screen::Stats);
        } else if is_char(",") {
            screen.set(Screen::Settings);
        } else if is_char("e") {
            do_export();
        } else if is_char("d") {
            if let Some(c) = selected_commit.read().clone() {
                screen.set(Screen::Diff(Box::new(c)));
            }
        } else if is_char("[") {
            left_open.set(!left_open());
        } else if is_char("]") {
            right_open.set(!right_open());
        }
    };

    // ── Root CSS class — drives Terminal vs Modern overrides ───────────────
    let root_style_class = use_memo(move || app_style.read().css_class());
    //let root_style_class = move || app_style.read().css_class();

    rsx! {
        style { "{BASE_CSS}" }
        style { "{MODERN_CSS}" }
        style { "{theme_css}" }

        div {
            class: format!("app-root {}", root_style_class.read()),
            tabindex: "0",
            autofocus: true,
            style: "
                width: calc(100vw / var(--zoom));
                height: calc(100vh / var(--zoom));
                transform: scale(var(--zoom));
                transform-origin: top left;
                position: relative;
                outline: none;
            ",
            onkeydown: global_keydown,

            if *crt_overlay.read() {
                div { class: "crt-overlay", aria_hidden: "true" }
            }

            // ── Update banner — shown on all screens ───────────────────────
            UpdateBanner {
                state: update_state.read().clone(),
                on_update: move |_| {
                    let state = update_state.read().clone();
                    if let UpdateState::Available(info) = state {
                        let _ = webbrowser::open(&info.release_url);
                    }
                },
                on_dismiss: move |_| update_state.set(UpdateState::Dismissed),
            }

            // ── Toolbar (Tree screen only) ─────────────────────────────────
            match screen.read().clone() {
                Screen::Tree => rsx! {
                    Toolbar {
                        repo_name:    repo_tree.read().as_ref()
                                          .map(|r| r.repo_name.clone())
                                          .unwrap_or_default(),
                        search_query: search_query.read().clone(),
                        on_search:    move |q: String| search_query.set(q),
                        on_home:      move |_| {
                            search_query.set(String::new());
                            screen.set(Screen::Home);
                        },
                        on_settings:  move |_| screen.set(Screen::Settings),
                        on_refresh:   move |_| {},
                        on_stats:     move |_| screen.set(Screen::Stats),
                        on_export:    handle_export,
                        on_shortcuts: move |_| show_shortcuts.set(true),
                    }
                },
                _ => rsx! {}
            }

            // ── Main screen router ─────────────────────────────────────────
            match screen.read().clone() {

                Screen::Home => rsx! {
                    HomeScreen {
                        initial_error: clone_error.read().clone(),
                        on_load: move |tree: RepoTree| {
                            clone_error.set(None);
                            repo_tree.set(Some(tree));
                            selected_commit.set(None);
                            search_query.set(String::new());
                            screen.set(Screen::Tree);
                        },
                        on_loading: move |msg: String| {
                            clone_error.set(None);
                            screen.set(Screen::Loading(msg));
                        },
                        on_error: move |err: String| {
                            clone_error.set(Some(err));
                            screen.set(Screen::Home);
                        },
                    }
                },

                Screen::Loading(msg) => rsx! {
                    div {
                        class: "loading-screen",
                        div { class: "loading-cursor", "> {msg}" }
                        div { class: "loading-blink",  "█" }
                    }
                },

                Screen::Tree => rsx! {
                    {
                        let lw = if *left_open.read()  { "var(--panel-width)" } else { "28px" };
                        let rw = if *right_open.read() { "var(--panel-width)" } else { "28px" };
                        rsx! {
                            div {
                                class: "tree-layout",
                                style: "grid-template-columns: {lw} 1fr {rw};",

                                LeftPanel {
                                    commit:     selected_commit.read().clone(),
                                    open:       *left_open.read(),
                                    on_toggle:  move |_| left_open.set(!left_open()),
                                    remote_url: repo_tree.read().as_ref()
                                        .and_then(|r| r.remote_url.clone()),
                                }

                                TreeCanvas {
                                    tree:          repo_tree.read().clone(),
                                    selected_hash: selected_commit.read()
                                                       .as_ref()
                                                       .map(|c| c.hash.clone()),
                                    search_query:  search_query.read().clone(),
                                    node_spacing:  *node_spacing.read(),
                                    show_merges:   *show_merges.read(),
                                    direction:     tree_direction.read().clone(),
                                    branch_style:  branch_style.read().clone(),
                                    on_select: move |commit: CommitNode| {
                                        let hash = commit.hash.clone();
                                        selected_commit.set(Some(commit.clone()));

                                        if diff_cache.read().contains_key(&hash) {
                                            let (files, stats) =
                                                diff_cache.read()[&hash].clone();
                                            let mut c = commit.clone();
                                            c.files_changed = files;
                                            c.stats = stats;
                                            selected_commit.set(Some(c));
                                            return;
                                        }

                                        diff_loading.set(true);
                                        let repo_path = repo_tree.read().as_ref()
                                            .and_then(|r| r.repo_path.clone());

                                        spawn(async move {
                                            if let Some(path) = repo_path {
                                                let (tx, rx) =
                                                    tokio::sync::oneshot::channel();
                                                let h = hash.clone();
                                                std::thread::spawn(move || {
                                                    let _ = tx.send(
                                                        load_commit_diff(&path, &h)
                                                    );
                                                });
                                                if let Ok(Ok((files, stats))) = rx.await {
                                                    diff_cache.write().insert(
                                                        hash.clone(),
                                                        (files.clone(), stats.clone()),
                                                    );
                                                    let still_selected =
                                                        selected_commit.read()
                                                            .as_ref()
                                                            .map(|c| c.hash == hash)
                                                            .unwrap_or(false);
                                                    if still_selected {
                                                        let maybe_c = selected_commit.read().clone();
                                                        if let Some(mut c) = maybe_c
                                                        {
                                                            c.files_changed = files;
                                                            c.stats = stats;
                                                            selected_commit.set(Some(c));
                                                        }
                                                    }
                                                }
                                                diff_loading.set(false);
                                            }
                                        });
                                    },
                                    on_deselect: move |_| selected_commit.set(None),
                                }

                                RightPanel {
                                    commit:       selected_commit.read().clone(),
                                    open:         *right_open.read(),
                                    on_toggle:    move |_| right_open.set(!right_open()),
                                    diff_loading: *diff_loading.read(),
                                    on_view_diff: move |commit: CommitNode| {
                                        screen.set(Screen::Diff(Box::new(commit)))
                                    },
                                }
                            }
                        }
                    }
                },

                Screen::Settings => rsx! {
                    Settings {
                        current_theme:  theme_name.read().clone(),
                        font_size:      *font_size.read(),
                        node_spacing:   *node_spacing.read(),
                        show_merges:    *show_merges.read(),
                        crt_overlay:    *crt_overlay.read(),
                        tree_direction: tree_direction.read().clone(),
                        branch_style:   branch_style.read().clone(),
                        app_style:      app_style.read().clone(),
                        current_version: crate::updater::CURRENT_VERSION,
                        checking_for_updates: matches!(*update_state.read(), UpdateState::Checking),
                        on_check_updates: on_check_updates,
                        on_theme_change: move |name: String| {
                            theme_name.set(name);
                            save_settings(&current_settings());
                        },
                        on_font_size_change: move |fs: u32| {
                            font_size.set(fs);
                            save_settings(&current_settings());
                        },
                        on_node_spacing_change: move |ns: f64| {
                            node_spacing.set(ns);
                            save_settings(&current_settings());
                        },
                        on_show_merges_change: move |v: bool| {
                            show_merges.set(v);
                            save_settings(&current_settings());
                        },
                        on_crt_overlay_change: move |v: bool| {
                            crt_overlay.set(v);
                            save_settings(&current_settings());
                        },
                        on_tree_direction_change: move |d: TreeDirection| {
                            tree_direction.set(d);
                            save_settings(&current_settings());
                        },
                        on_branch_style_change: move |s: BranchStyle| {
                            branch_style.set(s);
                            save_settings(&current_settings());
                        },
                        on_app_style_change: move |ms: AppStyle| {
                            app_style.set(ms);
                            save_settings(&current_settings());
                        },
                        on_back: move |_| screen.set(Screen::Tree),
                    }
                },

                Screen::Diff(commit) => rsx! {
                    DiffViewer {
                        commit:  *commit.clone(),
                        on_back: move |_| screen.set(Screen::Tree),
                    }
                },

                Screen::Stats => rsx! {
                    {
                        let maybe_tree = repo_tree.read().clone();
                        rsx! {
                            if let Some(ref tree) = maybe_tree {
                                StatsScreen {
                                    tree:    tree.clone(),
                                    on_back: move |_| screen.set(Screen::Tree),
                                }
                            }
                        }
                    }
                },
            }

            // ── Onboarding overlay — rendered on top of everything ─────────
            if !*onboarding_done.read() {
                OnboardingScreen {
                    // Live-preview: user picks Modern → class is applied NOW
                    on_style_preview: move |s: AppStyle| {
                        app_style.set(s);
                        // Don't save yet — they haven't confirmed
                    },
                    // Done: persist style + mark onboarding complete
                    on_complete: move |s: AppStyle| {
                        app_style.set(s.clone());
                        onboarding_done.set(true);
                        let mut settings = current_settings();
                        settings.app_style      = s;
                        settings.onboarding_done = true;
                        save_settings(&settings);
                    },
                }
            }

            // ── Keyboard shortcuts overlay ──────────────────────────────────
            if *show_shortcuts.read() {
                ShortcutsHelp {
                    on_close: move |_| show_shortcuts.set(false),
                }
            }
        }
    }
}

// ── Update banner ─────────────────────────────────────────────────────────────

#[component]
fn UpdateBanner(
    state: UpdateState,
    on_update: EventHandler<()>,
    on_dismiss: EventHandler<()>,
) -> Element {
    match state {
        UpdateState::Checking => rsx! {
            div { class: "update-banner update-banner--checking",
                span { class: "update-banner__text", "… checking for updates" }
                button {
                    class: "update-banner__close",
                    "aria-label": "Dismiss",
                    onclick: move |_| on_dismiss.call(()),
                    "✕"
                }
            }
        },
        UpdateState::UpToDate => rsx! {
            div { class: "update-banner update-banner--uptodate",
                span { class: "update-banner__text", "✓ you're on the latest version" }
                button {
                    class: "update-banner__close",
                    "aria-label": "Dismiss",
                    onclick: move |_| on_dismiss.call(()),
                    "✕"
                }
            }
        },
        UpdateState::Error(msg) => rsx! {
            div { class: "update-banner update-banner--error",
                span { class: "update-banner__text", "✗ update check failed: {msg}" }
                button {
                    class: "update-banner__close",
                    "aria-label": "Dismiss",
                    onclick: move |_| on_dismiss.call(()),
                    "✕"
                }
            }
        },
        UpdateState::Available(info) => rsx! {
            div { class: "update-banner update-banner--available",
                span { class: "update-banner__text",
                    "▲ {info.latest_version} available"
                    if !info.release_notes.is_empty() {
                        span { class: "update-banner__notes",
                            " — {info.release_notes}" }
                    }
                }
                button {
                    class: "update-banner__btn",
                    onclick: move |_| on_update.call(()),
                    "[ VIEW RELEASE ]"
                }
                button {
                    class: "update-banner__close",
                    "aria-label": "Dismiss update notice",
                    onclick: move |_| on_dismiss.call(()),
                    "✕"
                }
            }
        },
        _ => rsx! {},
    }
}
