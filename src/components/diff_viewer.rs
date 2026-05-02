use crate::git::parser::{ChangeStatus, CommitNode};
use dioxus::prelude::*;
use std::collections::HashSet;

const DIFF_VIEWER_CSS: &str = include_str!("../../assets/css/diff_viewer.css");

#[component]
pub fn DiffViewer(commit: CommitNode, on_back: EventHandler<()>) -> Element {
    let mut collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);

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
                span { class: "diffview-stat",              "{file_count} files" }
                span { class: "diffview-stat diffview-stat--add", "+{total_adds}" }
                span { class: "diffview-stat diffview-stat--del", "-{total_dels}" }

                div { style: "margin-left: auto; display: flex; gap: 8px;",
                    button {
                        class: "diffview-fold-btn",
                        onclick: move |_| {
                            let paths: HashSet<String> = commit
                                .files_changed
                                .iter()
                                .map(|f| f.path.clone())
                                .collect();
                            collapsed.set(paths);
                        },
                        "[ FOLD ALL ]"
                    }
                    button {
                        class: "diffview-fold-btn",
                        onclick: move |_| collapsed.set(HashSet::new()),
                        "[ EXPAND ALL ]"
                    }
                }
            }

            // ── Scrollable file list ───────────────────────────────────────
            div {
                class: "diffview-body",

                if commit.files_changed.is_empty() {
                    div { class: "diffview-empty", "// no file changes recorded" }
                }

                for file in commit.files_changed.iter() {
                    {
                        let path        = file.path.clone();
                        let path_toggle = path.clone();
                        let badge_class = format!(
                            "diffview-file-badge diffview-file-badge--{}",
                            status_class(&file.status)
                        );
                        let status_text = status_label(&file.status);
                        let adds        = file.additions;
                        let dels        = file.deletions;
                        let has_lines   = !file.lines.is_empty();
                        let is_collapsed = collapsed.read().contains(&path);
                        let arrow = if is_collapsed { "▶" } else { "▼" };

                        rsx! {
                            div {
                                class: "diffview-file",
                                key: "{path}",

                                // ── File header ───────────────────────────
                                div {
                                    class: "diffview-file-header",
                                    button {
                                        class: "diffview-collapse-btn",
                                        title: if is_collapsed { "Expand" } else { "Collapse" },
                                        onclick: move |_| {
                                            let mut set = collapsed.write();
                                            if set.contains(&path_toggle) {
                                                set.remove(&path_toggle);
                                            } else {
                                                set.insert(path_toggle.clone());
                                            }
                                        },
                                        "{arrow}"
                                    }
                                    span { class: "{badge_class}", "{status_text}" }
                                    span { class: "diffview-file-path", "{path}" }
                                    span {
                                        class: "diffview-file-counts",
                                        span { class: "dv-plus",  "+{adds}" }
                                        " / "
                                        span { class: "dv-minus", "-{dels}" }
                                    }
                                }

                                // ── Diff lines — hidden when collapsed ────
                                if !is_collapsed {
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
                                                    let line_class = format!(
                                                        "dv-line dv-line--{}",
                                                        origin_class(line.origin)
                                                    );
                                                    let num     = i + 1;
                                                    let glyph   = line.origin.to_string();
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
