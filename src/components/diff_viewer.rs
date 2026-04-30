use crate::git::parser::{ChangeStatus, CommitNode};
use dioxus::prelude::*;

const DIFF_VIEWER_CSS: &str = include_str!("../../assets/css/diff_viewer.css");

#[component]
pub fn DiffViewer(commit: CommitNode, on_back: EventHandler<()>) -> Element {
    // Precompute everything before rsx! — Dioxus macro can't handle method calls
    // that return non-Display types (e.g. chrono::DelayedFormat)
    let total_adds: usize = commit.files_changed.iter().map(|f| f.additions).sum();
    let total_dels: usize = commit.files_changed.iter().map(|f| f.deletions).sum();
    let file_count = commit.files_changed.len();
    let date_str = commit.timestamp.format("%Y-%m-%d %H:%M").to_string();
    let short_hash = commit.short_hash.clone();
    let message = commit.message.clone();
    let author = commit.author_name.clone();

    rsx! {
        style { "{DIFF_VIEWER_CSS}" }
        div {
            class: "diffview-root",

            // ── Top bar ───────────────────────────────────────────────────
            div {
                class: "diffview-topbar",
                button {
                    class: "diffview-back-btn",
                    onclick: move |_| on_back.call(()),
                    "[ ← BACK ]"
                }
                div {
                    class: "diffview-title",
                    span { class: "diffview-hash", "{short_hash}" }
                    span { class: "diffview-msg",  "{message}" }
                }
                div {
                    class: "diffview-meta",
                    span { class: "diffview-author", "{author}" }
                    span { class: "diffview-date",   "{date_str}" }
                }
            }

            // ── Stats bar ─────────────────────────────────────────────────
            div {
                class: "diffview-statsbar",
                span { class: "diffview-stat",          "{file_count} files" }
                span { class: "diffview-stat diffview-stat--add", "+{total_adds}" }
                span { class: "diffview-stat diffview-stat--del", "-{total_dels}" }
            }

            // ── File diffs ────────────────────────────────────────────────
            div {
                class: "diffview-body",

                if commit.files_changed.is_empty() {
                    div {
                        class: "diffview-empty",
                        "// no file changes recorded"
                    }
                }

                for file in commit.files_changed.iter() {
                    {
                        let badge_class = format!("diffview-file-badge diffview-file-badge--{}", status_class(&file.status));
                        let status_text = status_label(&file.status);
                        let path = file.path.clone();
                        let adds = file.additions;
                        let dels = file.deletions;
                        let has_lines = !file.lines.is_empty();

                        rsx! {
                            div {
                                class: "diffview-file",
                                key: "{path}",

                                div {
                                    class: "diffview-file-header",
                                    span { class: "{badge_class}", "{status_text}" }
                                    span { class: "diffview-file-path", "{path}" }
                                    span { class: "diffview-file-counts",
                                        span { class: "dv-plus", "+{adds}" }
                                        " / "
                                        span { class: "dv-minus", "-{dels}" }
                                    }
                                }

                                if !has_lines {
                                    div {
                                        class: "diffview-no-lines",
                                        "// binary or empty diff"
                                    }
                                } else {
                                    div {
                                        class: "diffview-hunk",
                                        for (i, line) in file.lines.iter().enumerate() {
                                            {
                                                let line_class = format!("dv-line dv-line--{}", origin_class(line.origin));
                                                let num = i + 1;
                                                let glyph = line.origin.to_string();
                                                let content = line.content.clone();
                                                rsx! {
                                                    div {
                                                        class: "{line_class}",
                                                        key: "{i}",
                                                        span { class: "dv-line-num",    "{num}" }
                                                        span { class: "dv-line-origin", "{glyph}" }
                                                        span { class: "dv-line-text",   "{content}" }
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
        }
    }
}

fn status_label(s: &ChangeStatus) -> &'static str {
    match s {
        ChangeStatus::Added => "ADDED",
        ChangeStatus::Modified => "MODIFIED",
        ChangeStatus::Deleted => "DELETED",
        ChangeStatus::Renamed => "RENAMED",
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

fn origin_class(c: char) -> &'static str {
    match c {
        '+' => "add",
        '-' => "del",
        _ => "ctx",
    }
}
