// app.rs — Root component + global state
use dioxus::prelude::*;
use crate::git::parser::RepoTree;
use crate::git::parser::CommitNode;
use crate::theme::theme_by_name;
use crate::components::{
    home_screen::HomeScreen,
    toolbar::Toolbar,
    tree_canvas::TreeCanvas,
    left_panel::LeftPanel,
    right_panel::RightPanel,
    settings::Settings,
};

const BASE_CSS: &str = include_str!("assets/style.css");

#[derive(Clone, PartialEq)]
pub enum Screen {
    Home,
    Loading(String),
    Tree,
    Settings,
}

#[component]
pub fn App() -> Element {
    let mut screen = use_signal(|| Screen::Home);
    let mut repo_tree: Signal<Option<RepoTree>> = use_signal(|| None);
    let mut selected_commit: Signal<Option<CommitNode>> = use_signal(|| None);
    let mut theme_name = use_signal(|| "Terminal".to_string());

    let theme_css = use_memo(move || {
        let t = theme_by_name(&theme_name.read());
        format!(
            ":root {{ --bg:{}; --bg-secondary:{}; --text:{}; --text-muted:{}; --accent:{}; --border:{}; --success:{}; --danger:{}; }}",
            t.bg, t.bg_secondary, t.text, t.text_muted, t.accent, t.border, t.success, t.danger
        )
    });

    rsx! {
        // Inject base CSS
        style { "{BASE_CSS}" }
        // Inject theme CSS variables on top
        style { "{theme_css}" }

        div {
            class: "app-root",

            match screen.read().clone() {
                Screen::Tree => rsx! {
                    Toolbar {
                        repo_name: repo_tree.read().as_ref().map(|r| r.repo_name.clone()).unwrap_or_default(),
                        on_home: move |_| screen.set(Screen::Home),
                        on_settings: move |_| screen.set(Screen::Settings),
                        on_refresh: move |_| {},
                    }
                },
                _ => rsx! {}
            }

            match screen.read().clone() {
                Screen::Home => rsx! {
                    HomeScreen {
                        on_load: move |tree: RepoTree| {
                            repo_tree.set(Some(tree));
                            selected_commit.set(None);
                            screen.set(Screen::Tree);
                        },
                        on_loading: move |msg: String| {
                            screen.set(Screen::Loading(msg));
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
                    div {
                        class: "tree-layout",
                        LeftPanel { commit: selected_commit.read().clone() }
                        TreeCanvas {
                            tree: repo_tree.read().clone(),
                            selected_hash: selected_commit.read().as_ref().map(|c| c.hash.clone()),
                            on_select: move |commit: CommitNode| selected_commit.set(Some(commit)),
                        }
                        RightPanel { commit: selected_commit.read().clone() }
                    }
                },

                Screen::Settings => rsx! {
                    Settings {
                        current_theme: theme_name.read().clone(),
                        on_theme_change: move |name: String| theme_name.set(name),
                        on_back: move |_| screen.set(Screen::Tree),
                    }
                },
            }
        }
    }
}