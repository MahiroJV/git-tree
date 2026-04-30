use crate::git::parser::{ChangeStatus, CommitNode};
use dioxus::prelude::*;

const PANEL_SHARED_CSS: &str = include_str!("../../assets/css/panel_shared.css");
const RIGHT_PANEL_CSS: &str = include_str!("../../assets/css/right_panel.css");

#[component]
pub fn RightPanel(
    commit: Option<CommitNode>,
    open: bool,
    on_toggle: EventHandler<()>,
    on_view_diff: EventHandler<CommitNode>,
) -> Element {
    // ── Collapsed state — thin strip ──────────────────────────────────────
    if !open {
        return rsx! {
            style { "{PANEL_SHARED_CSS}" }
            style { "{RIGHT_PANEL_CSS}" }
            div {
                class: "right-panel--collapsed",
                button {
                    class: "panel-collapse-btn",
                    title: "Expand diff stats",
                    onclick: move |_| on_toggle.call(()),
                    "◀"
                }
                span { class: "panel-collapsed-label", "DIFF STATS" }
            }
        };
    }

    // ── Expanded — empty state ─────────────────────────────────────────────
    let Some(commit) = commit else {
        return rsx! {
            style { "{PANEL_SHARED_CSS}" }
            style { "{RIGHT_PANEL_CSS}" }
            div {
                class: "panel right-panel",
                div {
                    class: "panel-header",
                    span { "// DIFF STATS" }
                    button {
                        class: "panel-collapse-btn panel-collapse-btn--header",
                        title: "Collapse",
                        onclick: move |_| on_toggle.call(()),
                        "▶"
                    }
                }
                div { class: "panel-empty", "// select a commit" }
            }
        };
    };

    let total_adds: usize = commit.files_changed.iter().map(|f| f.additions).sum();
    let total_dels: usize = commit.files_changed.iter().map(|f| f.deletions).sum();
    let file_count = commit.files_changed.len();
    let ratio_pct = if total_adds + total_dels > 0 {
        (total_adds as f64 / (total_adds + total_dels) as f64 * 100.0) as u32
    } else {
        50
    };
    let commit_for_btn = commit.clone();

    rsx! {
        style { "{PANEL_SHARED_CSS}" }
        style { "{RIGHT_PANEL_CSS}" }
        div {
            class: "panel right-panel",

            div {
                class: "panel-header",
                span { "// DIFF STATS" }
                button {
                    class: "panel-collapse-btn panel-collapse-btn--header",
                    title: "Collapse",
                    onclick: move |_| on_toggle.call(()),
                    "▶"
                }
            }

            div {
                class: "diff-summary",
                div {
                    class: "diff-summary-row",
                    span { class: "diff-label", "FILES" }
                    span { class: "diff-value", "{file_count}" }
                }
                div {
                    class: "diff-summary-row",
                    span { class: "diff-label diff-plus", "+" }
                    span { class: "diff-value diff-plus", "{total_adds} added" }
                }
                div {
                    class: "diff-summary-row",
                    span { class: "diff-label diff-minus", "-" }
                    span { class: "diff-value diff-minus", "{total_dels} removed" }
                }
            }

            div {
                class: "diff-ratio-wrap",
                span { class: "diff-ratio-label", "CHANGE RATIO" }
                div {
                    class: "diff-ratio-bar",
                    div {
                        class: "diff-ratio-fill",
                        style: "width: {ratio_pct}%",
                    }
                }
            }

            div { class: "diff-files-header", "FILES CHANGED" }

            div {
                class: "diff-files-list",
                for file in commit.files_changed.iter() {
                    div {
                        class: "diff-file-row",
                        key: "{file.path}",
                        span {
                            class: "diff-file-status diff-file-status--{status_class(&file.status)}",
                            "{status_label(&file.status)}"
                        }
                        span {
                            class: "diff-file-path",
                            title: "{file.path}",
                            "{file.path}"
                        }
                    }
                }
            }

            div {
                class: "diff-btn-wrap",
                button {
                    class: "diff-view-btn",
                    onclick: move |_| on_view_diff.call(commit_for_btn.clone()),
                    "[ VIEW DIFF ]"
                }
            }
        }
    }
}

fn status_label(s: &ChangeStatus) -> &'static str {
    match s {
        ChangeStatus::Added => "Added",
        ChangeStatus::Modified => "Modified",
        ChangeStatus::Deleted => "Deleted",
        ChangeStatus::Renamed => "Renamed",
    }
}

fn status_class(s: &ChangeStatus) -> &'static str {
    match s {
        ChangeStatus::Added => "added",
        ChangeStatus::Modified => "modified",
        ChangeStatus::Deleted => "deleted",
        ChangeStatus::Renamed => "renamed",
    }
}
