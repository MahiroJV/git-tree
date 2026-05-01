use crate::components::{
    diff_viewer::DiffViewer, home_screen::HomeScreen, left_panel::LeftPanel,
    right_panel::RightPanel, settings::Settings, toolbar::Toolbar, tree_canvas::TreeCanvas,
};
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
    // Error lives in app — survives screen transitions so clone errors can show
    let mut clone_error = use_signal(|| Option::<String>::None);

    let theme_css = use_memo(move || {
        let t = theme_by_name(&theme_name.read());
        format!(
            ":root {{ --bg:{}; --bg-secondary:{}; --text:{}; --text-muted:{}; --accent:{}; --border:{}; --success:{}; --danger:{}; }}",
            t.bg, t.bg_secondary, t.text, t.text_muted, t.accent, t.border, t.success, t.danger
        )
    });

    rsx! {
        style { "{BASE_CSS}" }
        style { "{theme_css}" }

        div {
            class: "app-root",

            match screen.read().clone() {
                Screen::Tree => rsx! {
                    Toolbar {
                        repo_name: repo_tree.read().as_ref().map(|r| r.repo_name.clone()).unwrap_or_default(),
                        search_query: search_query.read().clone(),
                        on_search: move |q: String| search_query.set(q),
                        on_home: move |_| {
                            search_query.set(String::new());
                            screen.set(Screen::Home);
                        },
                        on_settings: move |_| screen.set(Screen::Settings),
                        on_refresh: move |_| {},
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
                        div { class: "loading-blink", "█" }
                    }
                },

                Screen::Tree => rsx! {
                    {
                        // Columns shrink to 28 px when a panel is collapsed so the
                        // canvas takes the reclaimed space instead of leaving a gap.
                        let lw = if *left_open.read() { "var(--panel-width)" } else { "28px" };
                        let rw = if *right_open.read() { "var(--panel-width)" } else { "28px" };
                        rsx! {
                            div {
                                class: "tree-layout",
                                style: "grid-template-columns: {lw} 1fr {rw};",
                                LeftPanel {
                                    commit: selected_commit.read().clone(),
                                    open: *left_open.read(),
                                    on_toggle: move |_| left_open.set(!left_open()),
                                }
                                TreeCanvas {
                                    tree: repo_tree.read().clone(),
                                    selected_hash: selected_commit.read().as_ref().map(|c| c.hash.clone()),
                                    search_query: search_query.read().clone(),
                                    on_select: move |commit: CommitNode| selected_commit.set(Some(commit)),
                                    on_deselect: move |_| selected_commit.set(None),
                                }
                                RightPanel {
                                    commit: selected_commit.read().clone(),
                                    open: *right_open.read(),
                                    on_toggle: move |_| right_open.set(!right_open()),
                                    on_view_diff: move |commit: CommitNode| screen.set(Screen::Diff(Box::new(commit))),
                                }
                            }
                        }
                    }
                },

                Screen::Settings => rsx! {
                    Settings {
                        current_theme: theme_name.read().clone(),
                        on_theme_change: move |name: String| theme_name.set(name),
                        on_back: move |_| screen.set(Screen::Tree),
                    }
                },

                Screen::Diff(commit) => rsx! {
                    DiffViewer {
                        commit: *commit.clone(),
                        on_back: move |_| screen.set(Screen::Tree),
                    }
                },
            }
        }
    }
}
