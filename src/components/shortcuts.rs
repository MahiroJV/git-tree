// components/shortcuts.rs
use dioxus::prelude::*;

#[component]
pub fn ShortcutsHelp(on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "shortcuts-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "shortcuts-card",
                // swallow clicks so they don't bubble to the overlay backdrop
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "shortcuts-card__header",
                    span { class: "shortcuts-card__title", "// KEYBOARD SHORTCUTS" }
                    button {
                        class: "shortcuts-card__close",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                div { class: "shortcuts-group-title", "GLOBAL" }
                Row { keys: "?", label: "toggle this help" }
                Row { keys: "ESC", label: "go back / close" }

                div { class: "shortcuts-group-title", "TREE VIEW" }
                Row { keys: "/", label: "focus search" }
                Row { keys: "↑ ↓ / j k", label: "navigate commits (vertical layout)" }
                Row { keys: "← → / h l", label: "navigate commits (horizontal layout)" }
                Row { keys: "+ / -", label: "zoom in / out" }
                Row { keys: "0", label: "reset zoom & pan" }
                Row { keys: "[", label: "toggle left panel" }
                Row { keys: "]", label: "toggle right panel" }
                Row { keys: "d", label: "view diff of selected commit" }
                Row { keys: "s", label: "open repo stats" }
                Row { keys: ",", label: "open settings" }
                Row { keys: "e", label: "export tree as SVG" }

                div { class: "shortcuts-group-title", "CANVAS" }
                Row { keys: "CTRL + scroll", label: "zoom" }
                Row { keys: "drag", label: "pan" }
            }
        }
    }
}

#[component]
fn Row(keys: String, label: String) -> Element {
    rsx! {
        div { class: "shortcuts-row",
            span { "{label}" }
            kbd { "{keys}" }
        }
    }
}
