use crate::components::{
    diff_viewer::DiffViewer,
    home_screen::HomeScreen,
    left_panel::LeftPanel,
    right_panel::RightPanel,
    settings::Settings,
    toolbar::Toolbar,
    tree_canvas::{BranchStyle, TreeCanvas, TreeDirection},
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
    let mut clone_error = use_signal(|| Option::<String>::None);
    let mut font_size = use_signal(|| 13_u32);
    let mut node_spacing = use_signal(|| 120.0_f64);
    let mut show_merges = use_signal(|| true);
    let mut crt_overlay = use_signal(|| false);
    let mut tree_direction = use_signal(|| TreeDirection::Horizontal);
    let mut branch_style = use_signal(|| BranchStyle::Curved);

    let theme_css = use_memo(move || {
        let t = theme_by_name(&theme_name.read());
        let fs = *font_size.read();
        let zoom = use_memo(move || format!("{:.3}", *font_size.read() as f64 / 13.0));
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
            zoom = zoom,
        )
    });

    rsx! {
        style { "{BASE_CSS}" }
        style { "{theme_css}" }

        div {
            class: "app-root",
            //style: "zoom: {zoom};",

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
                                        selected_commit.set(Some(commit))
                                    },
                                    on_deselect:   move |_| selected_commit.set(None),
                                }

                                RightPanel {
                                    commit:       selected_commit.read().clone(),
                                    open:         *right_open.read(),
                                    on_toggle:    move |_| right_open.set(!right_open()),
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
            }
        }
    }
}
