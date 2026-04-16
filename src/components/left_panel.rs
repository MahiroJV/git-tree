// components/left_panel.rs — Commit details panel
use dioxus::prelude::*;
use crate::git::parser::CommitNode;

#[component]
pub fn LeftPanel(commit: Option<CommitNode>) -> Element {
    rsx! {
        aside {
            class: "panel panel-left",

            div { class: "panel-header", "// COMMIT INFO" }

            {
                match &commit {
                    None => rsx! {
                        div { class: "panel-empty",
                            p { class: "text-muted", ">" }
                            p { class: "text-muted", "> SELECT A NODE" }
                            p { class: "text-muted", "> TO VIEW DETAILS" }
                            p { class: "text-muted", ">" }
                        }
                    },
                    Some(c) => rsx! {
                        div { class: "panel-content",

                            div { class: "info-row",
                                span { class: "info-label", "AUTHOR" }
                                span { class: "info-value", {c.author_name.clone()} }
                            }

                            div { class: "info-row",
                                span { class: "info-label", "EMAIL" }
                                span {
                                    class: "info-value info-email",
                                    style: format!("color: {};", c.color),
                                    {c.author_email.clone()}
                                }
                            }

                            div { class: "info-row",
                                span { class: "info-label", "DATE" }
                                span { class: "info-value",
                                    {c.timestamp.format("%Y-%m-%d %H:%M UTC").to_string()}
                                }
                            }

                            div { class: "info-row",
                                span { class: "info-label", "HASH" }
                                div { class: "hash-row",
                                    span { class: "info-value hash", {c.short_hash.clone()} }
                                    button { class: "copy-btn", title: "Copy full hash", "⎘" }
                                }
                            }

                            {
                                if !c.tags.is_empty() {
                                    rsx! {
                                        div { class: "info-row",
                                            span { class: "info-label", "TAGS" }
                                            div { class: "tag-list",
                                                for tag in &c.tags {
                                                    span { class: "tag-badge", {format!("◆ {}", tag)} }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }

                            div { class: "divider" }

                            div { class: "info-label", "MESSAGE" }
                            div { class: "commit-message", {c.full_message.clone()} }

                            {
                                if c.is_head {
                                    rsx! { div { class: "badge badge-head", "● HEAD" } }
                                } else {
                                    rsx! {}
                                }
                            }

                            {
                                if c.is_merge {
                                    rsx! { div { class: "badge badge-merge", "⇄ MERGE COMMIT" } }
                                } else {
                                    rsx! {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}