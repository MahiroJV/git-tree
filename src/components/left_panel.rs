use crate::git::parser::CommitNode;
use crate::git::url::commit_web_url;
use dioxus::document::eval;
use dioxus::prelude::*;

const PANEL_SHARED_CSS: &str = include_str!("../../assets/css/panel_shared.css");
const LEFT_PANEL_CSS: &str = include_str!("../../assets/css/left_panel.css");

#[component]
pub fn LeftPanel(
    commit: Option<CommitNode>,
    open: bool,
    on_toggle: EventHandler<()>,
    remote_url: Option<String>,
) -> Element {
    // ── Collapsed state — thin strip ──────────────────────────────────────
    if !open {
        return rsx! {
            style { "{PANEL_SHARED_CSS}" }
            style { "{LEFT_PANEL_CSS}" }
            div {
                class: "panel left-panel left-panel--collapsed",
                button {
                    class: "panel-collapse-btn",
                    title: "Expand commit info",
                    onclick: move |_| on_toggle.call(()),
                    "▶"
                }
                span { class: "panel-collapsed-label", "COMMIT INFO" }
            }
        };
    }

    // ── Expanded state ────────────────────────────────────────────────────
    let Some(commit) = commit else {
        return rsx! {
            style { "{PANEL_SHARED_CSS}" }
            style { "{LEFT_PANEL_CSS}" }
            div {
                class: "panel left-panel",
                div {
                    class: "panel-header",
                    span { "// COMMIT INFO" }
                    button {
                        class: "panel-collapse-btn panel-collapse-btn--header",
                        title: "Collapse",
                        onclick: move |_| on_toggle.call(()),
                        "◀"
                    }
                }
                div {
                    class: "panel-empty",
                    "// select a commit"
                }
            }
        };
    };

    let tags = commit.tags.clone();
    let hash = commit.short_hash.clone();

    rsx! {
        style { "{PANEL_SHARED_CSS}" }
        style { "{LEFT_PANEL_CSS}" }
        div {
            class: "panel left-panel",

            div {
                class: "panel-header",
                span { "// COMMIT INFO" }
                button {
                    class: "panel-collapse-btn panel-collapse-btn--header",
                    title: "Collapse",
                    onclick: move |_| on_toggle.call(()),
                    "◀"
                }
            }

            div { class: "commit-field-label", "AUTHOR" }
            div { class: "commit-field-value", "{commit.author_name}" }

            div { class: "commit-field-label", "EMAIL" }
            div { class: "commit-field-value commit-field-value--accent", "{commit.author_email}" }

            div { class: "commit-field-label", "DATE" }
            div { class: "commit-field-value", "{commit.timestamp}" }

            div { class: "commit-field-label", "HASH" }
            div {
                class: "commit-hash-row",
                span { class: "commit-field-value commit-field-value--accent", "{hash}" }
                button {
                    class: "copy-hash-btn",
                    title: "Copy full hash",
                    onclick: move |_| {
                        // Copy to clipboard — best-effort in desktop webview
                        let full_hash = commit.hash.clone();
                        eval(&format!("navigator.clipboard.writeText('{full_hash}')"));
                    },
                    "⊞"
                }
            }

            if let Some(web_url) = remote_url.as_ref()
                .and_then(|r| commit_web_url(r, &commit.hash))
            {
                button {
                    class: "copy-hash-btn",
                    title: "Open in browser",
                    onclick: move |_|  {
                        let _ = webbrowser::open(&web_url);
                    },
                    "Check in Github.com ↗"
                }
            }

            if !tags.is_empty() {
                div { class: "commit-field-label", "TAGS" }
                div {
                    class: "commit-tags-row",
                    for tag in tags.iter() {
                        span { class: "commit-tag", "+ {tag}" }
                    }
                }
            }

            div { class: "commit-field-label", "MESSAGE" }
            div { class: "commit-message-box", "{commit.full_message}" }
        }
    }
}
