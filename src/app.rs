use crate::components::{
    diff_viewer::DiffViewer,
    home_screen::HomeScreen,
    left_panel::LeftPanel,
    right_panel::RightPanel,
    settings::Settings,
    stats::StatsScreen,
    toolbar::Toolbar,
    tree_canvas::{BranchStyle, TreeCanvas, TreeDirection},
};
use crate::git::export::generate_svg;
use crate::git::loader::load_commit_diff;
use crate::git::parser::CommitNode;
use crate::git::parser::RepoTree;
use crate::theme::theme_by_name;
use dioxus::prelude::*;

const BASE_CSS: &str = include_str!("../assets/css/style.css");

#[derive(Clone, PartialEq)]
pub enum Screen {
    Home,
    Loading(String),
    Tree,
    Settings,
    Stats,
    Diff(Box<CommitNode>),
}

#[component]
pub fn App() -> Element {
    let mut screen = use_signal(|| Screen::Home);
    let mut repo_tree: Signal<Option<RepoTree>> = use_signal(|| None);
    let mut selected_commit: Signal<Option<CommitNode>> = use_signal(|| None);
    let mut theme_name = use_signal(|| "Terminal".to_string());
    let mut search_query = use_signal(String::new);
    let mut left_open = use_signal(|| true);
    let mut right_open = use_signal(|| true);
    let mut clone_error = use_signal(|| Option::<String>::None);
    let mut font_size = use_signal(|| 13_u32);
    let mut node_spacing = use_signal(|| 120.0_f64);
    let mut show_merges = use_signal(|| true);
    let mut crt_overlay = use_signal(|| false);
    let mut tree_direction = use_signal(|| TreeDirection::Horizontal);
    let mut branch_style = use_signal(|| BranchStyle::Curved);
    let zoom = use_memo(move || *font_size.read() as f64 / 13.0);
    let mut diff_loading = use_signal(|| false);

    let mut diff_cache: Signal<
        std::collections::HashMap<
            String,
            (
                Vec<crate::git::parser::FileChange>,
                crate::git::parser::DiffStats,
            ),
        >,
    > = use_signal(std::collections::HashMap::new);

    let theme_css = use_memo(move || {
        let t = theme_by_name(&theme_name.read());
        let fs = *font_size.read();
        let _z = *zoom.read();
        format!(
            ":root {{ \
                --bg:{bg}; --bg-secondary:{bgs}; --text:{text}; \
                --text-muted:{tm}; --accent:{ac}; --border:{bo}; \
                --success:{su}; --danger:{da}; \
                --font-size:{fs}px; \
            }} \
            html {{ zoom: {zoom:.3}; }}",
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

    //SVG
    let handle_export = move |_| {
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
    rsx! {
        style { "{BASE_CSS}" }
        style { "{theme_css}" }

        div {
            class: "app-root",
            style: "transform: scale({zoom}); transform-origin: 0 0; \
            width: calc(100vw / {zoom}); height: calc(100vh / {zoom});",

            if *crt_overlay.read() {
                div { class: "crt-overlay", aria_hidden: "true" }
            }

            match screen.read().clone() {
                Screen::Tree => rsx! {
                    Toolbar {
                        repo_name:   repo_tree.read().as_ref()
                                         .map(|r| r.repo_name.clone())
                                         .unwrap_or_default(),
                        search_query: search_query.read().clone(),
                        on_search:   move |q: String| search_query.set(q),
                        on_home:     move |_| {
                            search_query.set(String::new());
                            screen.set(Screen::Home);
                        },
                        on_settings: move |_| screen.set(Screen::Settings),
                        on_refresh:  move |_| {},
                        on_stats: move |_| screen.set(Screen::Stats),
                        on_export: handle_export,
                    }
                },
                _ => rsx! {}
            }

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
                                    commit:    selected_commit.read().clone(),
                                    open:      *left_open.read(),
                                    on_toggle: move |_| left_open.set(!left_open()),
                                    remote_url: repo_tree.read().as_ref()
                                        .and_then(|r| r.remote_url.clone()),
                                }

                                TreeCanvas {
                                    tree:          repo_tree.read().clone(),
                                    selected_hash: selected_commit.read()
                                                       .as_ref().map(|c| c.hash.clone()),
                                    search_query:  search_query.read().clone(),
                                    node_spacing:  *node_spacing.read(),
                                    show_merges:   *show_merges.read(),
                                    direction:     tree_direction.read().clone(),
                                    branch_style:  branch_style.read().clone(),
                                    on_select:     move |commit: CommitNode| {
                                        let hash = commit.hash.clone();
                                        // Show commit metadata immediately
                                        selected_commit.set(Some(commit.clone()));

                                        // Return early if diff already cached
                                        if diff_cache.read().contains_key(&hash) {
                                            let (files, stats) = diff_cache.read()[&hash].clone();
                                            let mut c = commit.clone();
                                            c.files_changed = files;
                                            c.stats = stats;
                                            selected_commit.set(Some(c));
                                            return;
                                        }

                                        // Lazy load diff in background
                                        diff_loading.set(true);
                                        let repo_path = repo_tree.read().as_ref()
                                            .and_then(|r| r.repo_path.clone());

                                        spawn(async move {
                                            if let Some(path) = repo_path {
                                                let (tx, rx) = tokio::sync::oneshot::channel();
                                                let h = hash.clone();
                                                std::thread::spawn(move || {
                                                    let _ = tx.send(load_commit_diff(&path, &h));
                                                });

                                                if let Ok(Ok((files, stats))) = rx.await {
                                                    diff_cache.write().insert(
                                                        hash.clone(),
                                                        (files.clone(), stats.clone())
                                                    );
                                                    let still_selected = selected_commit.read()
                                                        .as_ref()
                                                        .map(|c| c.hash == hash).unwrap_or(false);

                                                    if still_selected {
                                                        let maybe_c = selected_commit.read().clone();
                                                        if let Some(mut c) = maybe_c {
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
                                    on_deselect:   move |_| selected_commit.set(None),
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
                        current_theme:            theme_name.read().clone(),
                        font_size:                *font_size.read(),
                        node_spacing:             *node_spacing.read(),
                        show_merges:              *show_merges.read(),
                        crt_overlay:              *crt_overlay.read(),
                        tree_direction:           tree_direction.read().clone(),
                        branch_style:             branch_style.read().clone(),
                        on_theme_change:          move |name: String| theme_name.set(name),
                        on_font_size_change:      move |fs: u32|      font_size.set(fs),
                        on_node_spacing_change:   move |ns: f64|      node_spacing.set(ns),
                        on_show_merges_change:    move |v: bool|      show_merges.set(v),
                        on_crt_overlay_change:    move |v: bool|      crt_overlay.set(v),
                        on_tree_direction_change: move |d: TreeDirection| tree_direction.set(d),
                        on_branch_style_change:   move |s: BranchStyle|  branch_style.set(s),
                        on_back:                  move |_| screen.set(Screen::Tree),
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
                                    tree: tree.clone(),
                                    on_back: move |_| screen.set(Screen::Tree),
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}
