// components/right_panel.rs — Diff stats panel
use dioxus::prelude::*;
use crate::git::parser::CommitNode;

#[component]
pub fn RightPanel(commit: Option<CommitNode>) -> Element {
    rsx! {
        aside {
            class: "panel panel-right",

            div { class: "panel-header", "// DIFF STATS" }

            if commit.is_none() {
                div { class: "panel-empty",
                    p { class: "text-muted", ">" }
                    p { class: "text-muted", "> SELECT A NODE" }
                    p { class: "text-muted", "> TO VIEW DIFF" }
                    p { class: "text-muted", ">" }
                }
            }

            if let Some(c) = &commit {
                div { class: "panel-content",

                    div { class: "stat-row",
                        span { class: "stat-label", "FILES" }
                        span { class: "stat-value", "{c.stats.files_changed}" }
                    }
                    div { class: "stat-row",
                        span { class: "stat-label", "+" }
                        span { class: "stat-value success", "{c.stats.insertions} added" }
                    }
                    div { class: "stat-row",
                        span { class: "stat-label", "-" }
                        span { class: "stat-value danger", "{c.stats.deletions} removed" }
                    }

                    div { class: "divider" }

                    div { class: "diff-bar-label", "CHANGE RATIO" }
                    {
                        let total = (c.stats.insertions + c.stats.deletions).max(1);
                        let add_pct = (c.stats.insertions * 100) / total;
                        let del_pct = 100 - add_pct;
                        rsx! {
                            div { class: "diff-bar",
                                div { class: "diff-bar-add", style: "width: {add_pct}%;" }
                                div { class: "diff-bar-del", style: "width: {del_pct}%;" }
                            }
                        }
                    }

                    div { class: "divider" }

                    div { class: "info-label", "FILES CHANGED" }

                    if c.files_changed.is_empty() {
                        p { class: "text-muted", "─ full file list in v0.2 ─" }
                    }

                    for file in &c.files_changed {
                        div { class: "file-row",
                            span { class: "file-status", "{file.status:?}" }
                            span { class: "file-path", "{file.path}" }
                        }
                    }
                }
            }
        }
    }
}