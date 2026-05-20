// components/onboarding.rs
//
// Three-step first-launch onboarding:
//   Step 0 — Welcome splash (1-second auto-advance or click)
//   Step 1 — Pick UI style  (Terminal / Modern) — root class updates live
//   Step 2 — GitHub login or skip
//   Step 3 — "You're all set" finish splash → emits on_complete
#[allow(unused_imports)]
use crate::git::auth::{
    clear_token, load_token, poll_for_token, request_device_code, save_token, AuthToken,
};
use crate::settings_store::AppStyle;
use dioxus::document::eval;
use dioxus::prelude::*;

// ── OAuth state machine ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum OAuthState {
    Idle,
    RequestingCode,
    WaitingForUser {
        user_code: String,
        verification_uri: String,
        device_code: String,
        interval: u64,
    },
    Done(String), // username
    Error(String),
}

// ── Public component ──────────────────────────────────────────────────────────

/// Emits `(chosen_style, had_token)` when the user finishes.
/// `had_token` is true if they signed in — the token is already persisted
/// via `save_token()` inside this component.
#[component]
pub fn OnboardingScreen(
    /// Called with the style the user chose so app.rs can apply it live.
    on_style_preview: EventHandler<AppStyle>,
    /// Called when all steps are done.
    on_complete: EventHandler<AppStyle>,
) -> Element {
    let mut step = use_signal(|| 0u8);
    let mut chosen = use_signal(|| AppStyle::Terminal);
    let mut oauth_state = use_signal(|| {
        // If a token already exists from a previous run that reset onboarding,
        // skip the auth step entirely.
        if load_token().is_some() {
            OAuthState::Done("(already signed in)".into())
        } else {
            OAuthState::Idle
        }
    });

    rsx! {
        // Full-screen overlay
        div {
            class: "ob-root",
            style: "
                position: fixed; inset: 0;
                display: flex; flex-direction: column;
                align-items: center; justify-content: center;
                background: var(--bg);
                z-index: 9000;
                padding: 40px;
            ",

            // Step indicator dots (only steps 1 and 2 are user-facing)
            if *step.read() >= 1 && *step.read() <= 2 {
                div {
                    style: "
                        position: absolute; top: 28px;
                        display: flex; gap: 8px;
                    ",
                    for i in 1u8..=2 {
                        div {
                            style: format!(
                                "width: 6px; height: 6px; border-radius: 50%; \
                                 background: {}; transition: background 0.3s;",
                                if *step.read() == i { "var(--accent)" }
                                else { "rgba(255,255,255,0.15)" }
                            ),
                        }
                    }
                }
            }

            // ── Step router ────────────────────────────────────────────────
            match *step.read() {
                0 => rsx! {
                    WelcomeSplash {
                        on_next: move |_| step.set(1),
                    }
                },
                1 => rsx! {
                    StyleStep {
                        chosen: chosen.read().clone(),
                        on_choose: move |s: AppStyle| {
                            chosen.set(s.clone());
                            // Live-preview: tell app.rs to apply the class now
                            on_style_preview.call(s);
                        },
                        on_next: move |_| step.set(2),
                        on_back: move |_| step.set(0),
                    }
                },
                2 => rsx! {
                    LoginStep {
                        oauth_state: oauth_state.read().clone(),
                        on_start_oauth: move |_| {
                            oauth_state.set(OAuthState::RequestingCode);
                            spawn(async move {
                                match request_device_code().await {
                                    Ok(start) => {
                                        let _ = webbrowser::open(&start.verification_uri);
                                        let dc = start.device_code.clone();
                                        let iv = start.interval;
                                        oauth_state.set(OAuthState::WaitingForUser {
                                            user_code:        start.user_code,
                                            verification_uri: start.verification_uri,
                                            device_code:      start.device_code,
                                            interval:         start.interval,
                                        });
                                        spawn(async move {
                                            match poll_for_token(dc, iv).await {
                                                Ok(token) => {
                                                    let name = token.username
                                                        .clone()
                                                        .unwrap_or_else(|| "GitHub".into());
                                                    save_token(&token);
                                                    oauth_state.set(OAuthState::Done(name));
                                                }
                                                Err(e) => {
                                                    oauth_state.set(OAuthState::Error(
                                                        e.to_string()
                                                    ));
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        oauth_state.set(OAuthState::Error(e.to_string()));
                                    }
                                }
                            });
                        },
                        on_cancel: move |_| oauth_state.set(OAuthState::Idle),
                        on_skip:   move |_| {
                            on_complete.call(chosen.read().clone());
                        },
                        on_finish: move |_| {
                            on_complete.call(chosen.read().clone());
                        },
                        on_back:   move |_| step.set(1),
                    }
                },
                _ => rsx! {},
            }
        }

        // Shared keyframe animation — injected once
        style {
            "
            @keyframes ob-up {{
                from {{ opacity:0; transform:translateY(20px); }}
                to   {{ opacity:1; transform:translateY(0); }}
            }}
            @keyframes ob-fade {{
                from {{ opacity:0; }}
                to   {{ opacity:1; }}
            }}
            .ob-card {{
                animation: ob-up 0.45s cubic-bezier(0.22,1,0.36,1) both;
            }}
            "
        }
    }
}

// ── Step 0 — Welcome splash ───────────────────────────────────────────────────

#[component]
fn WelcomeSplash(on_next: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "ob-card",
            style: "
                display: flex; flex-direction: column;
                align-items: center; gap: 28px;
                text-align: center;
            ",

            // Logo / ASCII
            div {
                style: "
                    font-family: 'Space Mono', monospace;
                    font-size: 56px;
                    font-weight: 700;
                    color: transparent;
                    -webkit-text-stroke: 1.5px var(--accent);
                    letter-spacing: 0.28em;
                    line-height: 1;
                ",
                "GIT-TREE"
            }

            // Tagline
            p {
                style: "
                    font-family: 'Space Mono', monospace;
                    font-size: 11px;
                    color: var(--text-muted);
                    letter-spacing: 0.22em;
                    margin: 0;
                    text-transform: uppercase;
                ",
                "Terminal-style git branch visualizer"
            }

            // Animated tree
            svg {
                width: "200", height: "70",
                view_box: "0 0 200 70",
                xmlns: "http://www.w3.org/2000/svg",
                style: "animation: ob-fade 1s ease 0.3s both;",

                line {
                    x1:"100", y1:"5", x2:"100", y2:"65",
                    stroke:"var(--accent)", stroke_width:"1.5", opacity:"0.4"
                }
                path {
                    d:"M 100 28 C 100 28, 60 28, 60 44 C 60 60, 100 60, 100 60",
                    stroke:"var(--success)", stroke_width:"1.2", fill:"none"
                }
                path {
                    d:"M 100 44 C 100 44, 140 44, 140 28",
                    stroke:"var(--accent)", stroke_width:"1.2", fill:"none", opacity:"0.6"
                }
                // Main nodes
                for (cx, cy, acc) in [
                    (100.0_f64, 14.0_f64, true),
                    (100.0, 44.0, false),
                    (100.0, 60.0, false),
                ] {
                    circle {
                        cx: "{cx}", cy: "{cy}", r: "6",
                        fill: if acc { "var(--accent)" } else { "var(--bg)" },
                        stroke: "var(--accent)", stroke_width: "1.5"
                    }
                }
                // Branch nodes
                circle {
                    cx:"60", cy:"44", r:"5",
                    fill:"var(--bg)", stroke:"var(--success)", stroke_width:"1.2"
                }
                text {
                    x:"60", y:"47", text_anchor:"middle",
                    font_size:"5", fill:"var(--success)",
                    font_family:"'Space Mono', monospace", "M"
                }
                circle {
                    cx:"140", cy:"28", r:"5",
                    fill:"var(--bg)", stroke:"var(--accent)", stroke_width:"1.2",
                    opacity:"0.6"
                }
                text {
                    x:"100", y:"5", text_anchor:"middle",
                    font_size:"5", fill:"var(--accent)",
                    font_family:"'Space Mono', monospace",
                    font_weight:"bold", dy:"-4", "HEAD"
                }
            }

            button {
                style: "
                    background: var(--accent);
                    color: var(--bg);
                    border: none;
                    font-family: 'Space Mono', monospace;
                    font-size: 11px;
                    font-weight: 700;
                    letter-spacing: 0.2em;
                    padding: 13px 44px;
                    cursor: pointer;
                    transition: opacity 0.15s;
                    animation: ob-fade 0.6s ease 0.6s both;
                ",
                onclick: move |_| on_next.call(()),
                "GET STARTED →"
            }
        }
    }
}

// ── Step 1 — Style picker ─────────────────────────────────────────────────────

#[component]
fn StyleStep(
    chosen: AppStyle,
    on_choose: EventHandler<AppStyle>,
    on_next: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "ob-card",
            style: "
                display: flex; flex-direction: column;
                align-items: center; gap: 32px; width: 100%;
                max-width: 600px;
            ",

            // Header
            div {
                style: "display:flex; flex-direction:column; align-items:center; gap:8px;",
                h2 {
                    style: "
                        font-family: 'Space Mono', monospace;
                        font-size: 16px; font-weight: 700;
                        color: var(--text); letter-spacing: 0.2em;
                        margin: 0; text-transform: uppercase;
                    ",
                    "CHOOSE YOUR STYLE"
                }
                p {
                    style: "
                        font-size: 11px; color: var(--text-muted);
                        letter-spacing: 0.08em; margin: 0;
                        font-family: 'Space Mono', monospace;
                    ",
                    "You can always change this later in Settings"
                }
            }

            // Two cards side by side
            div {
                style: "display:flex; gap:16px; width:100%;",

                // Terminal
                ObStyleCard {
                    is_selected: chosen == AppStyle::Terminal,
                    on_select: move |_| on_choose.call(AppStyle::Terminal),
                    title: "Terminal",
                    subtitle: "Monospace · Retro · ASCII",
                    preview: rsx! { ObTerminalPreview {} },
                }

                // Modern
                ObStyleCard {
                    is_selected: chosen == AppStyle::Modern,
                    on_select: move |_| on_choose.call(AppStyle::Modern),
                    title: "Modern",
                    subtitle: "System UI · Rounded · Clean",
                    preview: rsx! { ObModernPreview {} },
                }
            }

            // Nav buttons
            div {
                style: "display:flex; gap:12px; align-items:center;",

                button {
                    style: "
                        background: transparent;
                        border: none;
                        color: var(--text-muted);
                        font-family: 'Space Mono', monospace;
                        font-size: 10px;
                        letter-spacing: 0.12em;
                        cursor: pointer;
                        padding: 8px 16px;
                    ",
                    onclick: move |_| on_back.call(()),
                    "← BACK"
                }

                button {
                    style: "
                        background: var(--accent);
                        color: var(--bg);
                        border: none;
                        font-family: 'Space Mono', monospace;
                        font-size: 11px;
                        font-weight: 700;
                        letter-spacing: 0.18em;
                        padding: 12px 36px;
                        cursor: pointer;
                        transition: opacity 0.15s;
                    ",
                    onclick: move |_| on_next.call(()),
                    "CONTINUE →"
                }
            }
        }
    }
}

// ── Style card ────────────────────────────────────────────────────────────────

#[component]
fn ObStyleCard(
    is_selected: bool,
    on_select: EventHandler<()>,
    title: &'static str,
    subtitle: &'static str,
    preview: Element,
) -> Element {
    rsx! {
        div {
            style: format!("
                flex: 1; min-width: 0;
                display: flex; flex-direction: column; gap: 12px;
                padding: 16px;
                border: {};
                background: {};
                cursor: pointer;
                transition: border-color 0.18s, background 0.18s;
            ",
                if is_selected {
                    "2px solid var(--accent)".to_string()
                } else {
                    "1px solid rgba(255,255,255,0.1)".to_string()
                },
                if is_selected {
                    "rgba(155,93,229,0.06)".to_string()
                } else {
                    "rgba(255,255,255,0.02)".to_string()
                }
            ),
            onclick: move |_| on_select.call(()),

            // Preview box
            div {
                style: "
                    height: 120px; overflow: hidden;
                    border: 1px solid rgba(255,255,255,0.07);
                    background: #000;
                ",
                {preview}
            }

            // Label
            div {
                style: "display:flex; align-items:center; justify-content:space-between; gap:8px;",
                div {
                    span {
                        style: "
                            font-family: 'Space Mono', monospace;
                            font-size: 12px; font-weight: 700;
                            color: var(--text); letter-spacing: 0.1em;
                            display: block;
                        ",
                        "{title}"
                    }
                    span {
                        style: "
                            font-family: 'Space Mono', monospace;
                            font-size: 9px; color: var(--text-muted);
                            letter-spacing: 0.06em; margin-top: 3px;
                            display: block;
                        ",
                        "{subtitle}"
                    }
                }
                if is_selected {
                    span {
                        style: "
                            font-size: 8px; color: var(--accent);
                            border: 1px solid var(--accent);
                            padding: 2px 6px; letter-spacing: 0.1em;
                            flex-shrink: 0;
                            font-family: 'Space Mono', monospace;
                        ",
                        "✓ SELECTED"
                    }
                }
            }
        }
    }
}

// ── Mini SVG previews for the style cards ─────────────────────────────────────

#[component]
fn ObTerminalPreview() -> Element {
    rsx! {
        svg {
            width: "100%", height: "100%",
            view_box: "0 0 260 120",
            xmlns: "http://www.w3.org/2000/svg",
            preserve_aspect_ratio: "xMidYMid slice",

            rect { width:"260", height:"120", fill:"#000000" }

            // Toolbar
            rect { x:"0", y:"0", width:"260", height:"20", fill:"#000000" }
            rect { x:"0", y:"19", width:"260", height:"1", fill:"#9B5DE5", opacity:"0.8" }
            text { x:"8", y:"14", font_size:"8", fill:"#9B5DE5",
                font_family:"'Space Mono', monospace", letter_spacing:"0.22em", "GIT-TREE" }
            text { x:"185", y:"14", font_size:"6", fill:"#555",
                font_family:"'Space Mono', monospace", "[ SETTINGS ]" }

            // Left panel
            rect { x:"0", y:"20", width:"60", height:"100", fill:"#0a0a0a" }
            rect { x:"59", y:"20", width:"1", height:"100", fill:"#9B5DE5", opacity:"0.35" }
            text { x:"6", y:"34", font_size:"5", fill:"#9B5DE5",
                font_family:"'Space Mono', monospace", letter_spacing:"0.15em", "// COMMIT" }
            text { x:"6", y:"48", font_size:"4.5", fill:"#555",
                font_family:"'Space Mono', monospace", "AUTHOR" }
            text { x:"6", y:"59", font_size:"5", fill:"#fff",
                font_family:"'Space Mono', monospace", "MahiroJV" }
            text { x:"6", y:"73", font_size:"4.5", fill:"#9B5DE5",
                font_family:"'Space Mono', monospace", "a3f9b12" }

            // Main spine
            line { x1:"160", y1:"24", x2:"160", y2:"118",
                stroke:"#9B5DE5", stroke_width:"1.5", opacity:"0.5" }
            path {
                d:"M 160 60 C 160 60, 118 60, 118 80 C 118 100, 160 100, 160 100",
                stroke:"#00FF85", stroke_width:"1.2", fill:"none"
            }
            circle { cx:"160", cy:"36", r:"6", fill:"#000", stroke:"#9B5DE5", stroke_width:"1.8" }
            circle { cx:"160", cy:"60", r:"6", fill:"#9B5DE5", stroke:"#9B5DE5", stroke_width:"1.8" }
            circle { cx:"160", cy:"100", r:"6", fill:"#000", stroke:"#9B5DE5", stroke_width:"1.8" }
            circle { cx:"118", cy:"80", r:"5", fill:"#000", stroke:"#00FF85", stroke_width:"1.2" }
            text { x:"118", y:"83", text_anchor:"middle", font_size:"4",
                fill:"#00FF85", font_family:"'Space Mono', monospace", "M" }
            text { x:"160", y:"25", text_anchor:"middle", font_size:"5",
                fill:"#9B5DE5", font_family:"'Space Mono', monospace",
                font_weight:"bold", "HEAD" }
            text { x:"172", y:"39", font_size:"5", fill:"#555",
                font_family:"'Space Mono', monospace", "a3f9b12" }
            text { x:"172", y:"63", font_size:"5", fill:"#555",
                font_family:"'Space Mono', monospace", "d1e08f4" }
        }
    }
}

#[component]
fn ObModernPreview() -> Element {
    rsx! {
        svg {
            width: "100%", height: "100%",
            view_box: "0 0 260 120",
            xmlns: "http://www.w3.org/2000/svg",
            preserve_aspect_ratio: "xMidYMid slice",

            rect { width:"260", height:"120", fill:"#0F0F13" }

            // Dot grid
            defs {
                pattern {
                    id: "ob2-dots", width:"10", height:"10",
                    pattern_units:"userSpaceOnUse",
                    circle { cx:"1", cy:"1", r:"0.4", fill:"#1C1C24" }
                }
            }
            rect { width:"260", height:"120", fill:"url(#ob2-dots)" }

            // Toolbar
            rect { x:"0", y:"0", width:"260", height:"22", fill:"#0F0F13" }
            rect { x:"0", y:"21", width:"260", height:"0.5",
                fill:"rgba(255,255,255,0.06)" }
            text { x:"10", y:"15", font_size:"8.5", fill:"#5E6AD2",
                font_family:"system-ui, sans-serif", font_weight:"700",
                letter_spacing:"0.04em", "git-tree" }
            rect { x:"58", y:"6", width:"90", height:"12", rx:"4",
                fill:"#16161E", stroke:"rgba(255,255,255,0.07)", stroke_width:"0.5" }
            text { x:"64", y:"15", font_size:"5.5", fill:"#3A3A5A",
                font_family:"system-ui, sans-serif", "search commits..." }
            rect { x:"198", y:"6", width:"50", height:"12", rx:"4", fill:"none",
                stroke:"rgba(94,106,210,0.4)", stroke_width:"0.5" }
            text { x:"223", y:"15", text_anchor:"middle", font_size:"5.5",
                fill:"#5E6AD2", font_family:"system-ui, sans-serif", "Settings" }

            // Left panel
            rect { x:"0", y:"22", width:"60", height:"98", fill:"#0C0C12" }
            rect { x:"59", y:"22", width:"0.5", height:"98",
                fill:"rgba(255,255,255,0.055)" }
            text { x:"8", y:"36", font_size:"4.5", fill:"rgba(255,255,255,0.25)",
                font_family:"system-ui, sans-serif", font_weight:"600",
                letter_spacing:"0.1em", "COMMIT INFO" }
            text { x:"8", y:"50", font_size:"4.5", fill:"rgba(255,255,255,0.28)",
                font_family:"system-ui, sans-serif", font_weight:"600", "AUTHOR" }
            text { x:"8", y:"61", font_size:"6", fill:"#C7C7D9",
                font_family:"system-ui, sans-serif", "MahiroJV" }
            text { x:"8", y:"75", font_size:"4.5", fill:"rgba(255,255,255,0.28)",
                font_family:"system-ui, sans-serif", font_weight:"600", "HASH" }
            text { x:"8", y:"86", font_size:"5.5", fill:"#5E6AD2",
                font_family:"'Space Mono', monospace", "a3f9b12" }

            // Main spine
            line { x1:"160", y1:"26", x2:"160", y2:"118",
                stroke:"#5E6AD2", stroke_width:"1.5", opacity:"0.3" }
            path {
                d:"M 160 60 C 160 60, 118 60, 118 80 C 118 100, 160 100, 160 100",
                stroke:"#8B8FC8", stroke_width:"1.2", fill:"none", opacity:"0.8"
            }
            // Nodes — same geometry, cooler palette
            circle { cx:"160", cy:"38", r:"6", fill:"#0F0F13",
                stroke:"#5E6AD2", stroke_width:"2" }
            circle { cx:"160", cy:"60", r:"6", fill:"#5E6AD2",
                stroke:"#5E6AD2", stroke_width:"2" }
            circle { cx:"160", cy:"100", r:"6", fill:"#0F0F13",
                stroke:"#5E6AD2", stroke_width:"2" }
            circle { cx:"118", cy:"80", r:"5", fill:"#0F0F13",
                stroke:"#8B8FC8", stroke_width:"1.5" }
            text { x:"118", y:"83", text_anchor:"middle", font_size:"4",
                fill:"#8B8FC8", font_family:"system-ui, sans-serif", "M" }
            text { x:"160", y:"27", text_anchor:"middle", font_size:"5",
                fill:"#5E6AD2", font_family:"system-ui, sans-serif",
                font_weight:"700", "HEAD" }
            text { x:"172", y:"41", font_size:"5", fill:"#3A3A5A",
                font_family:"'Space Mono', monospace", "a3f9b12" }
            text { x:"172", y:"63", font_size:"5", fill:"#3A3A5A",
                font_family:"'Space Mono', monospace", "d1e08f4" }
        }
    }
}

// ── Step 2 — GitHub login ─────────────────────────────────────────────────────

#[component]
fn LoginStep(
    oauth_state: OAuthState,
    on_start_oauth: EventHandler<()>,
    on_cancel: EventHandler<()>,
    on_skip: EventHandler<()>,
    on_finish: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "ob-card",
            style: "
                display: flex; flex-direction: column;
                align-items: center; gap: 28px;
                max-width: 460px; width: 100%;
            ",

            // Header
            div {
                style: "display:flex; flex-direction:column; align-items:center; gap:8px;
                        text-align:center;",
                h2 {
                    style: "
                        font-family: 'Space Mono', monospace;
                        font-size: 15px; font-weight: 700;
                        color: var(--text); letter-spacing: 0.2em;
                        margin: 0;
                    ",
                    "CONNECT GITHUB"
                }
                p {
                    style: "
                        font-family: 'Space Mono', monospace;
                        font-size: 10px; color: var(--text-muted);
                        letter-spacing: 0.08em; margin: 0; line-height: 1.9;
                    ",
                    "Get private repo access and higher API rate limits."
                    br {}
                    "You can also skip and connect later."
                }
            }

            // ── OAuth state machine ────────────────────────────────────────
            match &oauth_state {

                // Idle — show sign-in button + skip
                OAuthState::Idle => rsx! {
                    div {
                        style: "display:flex; flex-direction:column; gap:10px; width:100%;",

                        // GitHub sign-in button
                        button {
                            style: "
                                width: 100%;
                                background: transparent;
                                border: 1px solid rgba(255,255,255,0.14);
                                color: var(--text);
                                font-family: 'Space Mono', monospace;
                                font-size: 10px;
                                letter-spacing: 0.15em;
                                padding: 14px 24px;
                                cursor: pointer;
                                display: flex; align-items: center;
                                justify-content: center; gap: 10px;
                                transition: border-color 0.15s, background 0.15s;
                            ",
                            onclick: move |_| on_start_oauth.call(()),
                            // GitHub mark SVG
                            svg {
                                width: "18", height: "18",
                                view_box: "0 0 24 24",
                                fill: "currentColor",
                                xmlns: "http://www.w3.org/2000/svg",
                                path { d: "M12 0C5.37 0 0 5.37 0 12c0 5.31 \
                                    3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 \
                                    0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41\
                                    -.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795\
                                    .945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99\
                                    .105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 \
                                    0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 \
                                    1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405\
                                    c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 \
                                    1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375\
                                    .81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69\
                                    .825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" }
                            }
                            "[ SIGN IN WITH GITHUB ]"
                        }

                        // Divider
                        div {
                            style: "
                                display: flex; align-items: center; gap: 10px;
                                color: var(--text-muted);
                                font-family: 'Space Mono', monospace;
                                font-size: 9px; letter-spacing: 0.12em;
                            ",
                            div { style: "flex:1; height:1px; background:rgba(255,255,255,0.07);" }
                            span { "OR" }
                            div { style: "flex:1; height:1px; background:rgba(255,255,255,0.07);" }
                        }

                        button {
                            style: "
                                width: 100%;
                                background: transparent;
                                border: none;
                                color: var(--text-muted);
                                font-family: 'Space Mono', monospace;
                                font-size: 9px; letter-spacing: 0.14em;
                                padding: 10px;
                                cursor: pointer;
                                transition: color 0.15s;
                            ",
                            onclick: move |_| on_skip.call(()),
                            "SKIP FOR NOW →"
                        }
                    }

                    // Back button
                    button {
                        style: "
                            background: transparent; border: none;
                            color: var(--text-muted);
                            font-family: 'Space Mono', monospace;
                            font-size: 9px; letter-spacing: 0.12em;
                            cursor: pointer; padding: 4px 10px;
                        ",
                        onclick: move |_| on_back.call(()),
                        "← BACK"
                    }
                },

                // Requesting code
                OAuthState::RequestingCode => rsx! {
                    div {
                        style: "
                            display: flex; align-items: center; gap: 14px;
                            padding: 20px; border: 1px solid var(--border);
                            width: 100%;
                            font-family: 'Space Mono', monospace;
                        ",
                        span { class: "loading-blink", style:"font-size:16px;", "█" }
                        span { style:"font-size:10px; color:var(--text-muted);
                               letter-spacing:0.1em;",
                            "contacting GitHub..."
                        }
                    }
                },

                // Waiting for user to enter code in browser
                OAuthState::WaitingForUser { user_code, verification_uri, .. } => {
                    let code  = user_code.clone();
                    let uri   = verification_uri.clone();
                    let code2 = code.clone();
                    let uri2  = uri.clone();
                    rsx! {
                        div {
                            style: "
                                display:flex; flex-direction:column; gap:14px;
                                padding:20px;
                                border: 1px solid var(--accent);
                                background: rgba(155,93,229,0.05);
                                width: 100%;
                            ",

                            p {
                                style: "font-size:10px; color:var(--text-muted);
                                        letter-spacing:0.06em; margin:0;
                                        font-family:'Space Mono',monospace;",
                                "1. Browser opened → go to "
                                span { style:"color:var(--accent);", "{uri}" }
                            }

                            // Code box + copy
                            div { style:"display:flex; align-items:center; gap:12px;",
                                div {
                                    style:"
                                        font-family:'Space Mono',monospace;
                                        font-size:26px; font-weight:700;
                                        letter-spacing:0.3em; color:var(--accent);
                                        border:2px solid var(--accent);
                                        padding:10px 18px;
                                    ",
                                    "{code}"
                                }
                                button {
                                    class: "btn-primary",
                                    style: "font-size:9px; padding:5px 10px;",
                                    onclick: move |_| {
                                        eval(&format!(
                                            "navigator.clipboard.writeText('{}')",
                                            code2
                                        ));
                                    },
                                    "⊞ COPY"
                                }
                            }

                            p {
                                style: "font-size:10px; color:var(--text-muted);
                                        letter-spacing:0.06em; margin:0;
                                        font-family:'Space Mono',monospace;",
                                "2. Enter the code → click "
                                span { style:"color:var(--success);", "Authorize" }
                                " → come back here"
                            }

                            // Waiting blink
                            div {
                                style: "display:flex; align-items:center; gap:8px;
                                        font-size:10px; color:var(--text-muted);
                                        font-family:'Space Mono',monospace;",
                                span { class:"loading-blink", style:"font-size:12px;", "█" }
                                "waiting for authorization..."
                            }

                            div { style:"display:flex; gap:8px;",
                                button {
                                    class:"btn-primary",
                                    style:"font-size:9px; padding:5px 12px;",
                                    onclick: move |_| { let _ = webbrowser::open(&uri2); },
                                    "↗ REOPEN BROWSER"
                                }
                                button {
                                    class:"btn-primary",
                                    style:"font-size:9px; padding:5px 12px;
                                           border-color:var(--danger); color:var(--danger);",
                                    onclick: move |_| on_cancel.call(()),
                                    "[ CANCEL ]"
                                }
                            }
                        }
                    }
                },

                // Done — show success splash then let user continue
                OAuthState::Done(username) => {
                    let name = username.clone();
                    rsx! {
                        div {
                            style: "
                                display:flex; flex-direction:column;
                                align-items:center; gap:20px; width:100%;
                            ",

                            // Success box
                            div {
                                style: "
                                    width:100%; padding:16px 20px;
                                    border:1px solid var(--success);
                                    background:rgba(0,255,133,0.05);
                                    display:flex; align-items:center; gap:12px;
                                ",
                                span { style:"font-size:18px; color:var(--success);", "✓" }
                                div {
                                    style: "display:flex; flex-direction:column; gap:3px;",
                                    span {
                                        style: "font-family:'Space Mono',monospace;
                                                font-size:11px; color:var(--success);
                                                letter-spacing:0.1em;",
                                        "GITHUB CONNECTED"
                                    }
                                    if !name.starts_with('(') {
                                        span {
                                            style: "font-family:'Space Mono',monospace;
                                                    font-size:10px; color:var(--text-muted);",
                                            "Signed in as @{name}"
                                        }
                                    }
                                }
                            }

                            button {
                                style: "
                                    background:var(--accent); color:var(--bg);
                                    border:none;
                                    font-family:'Space Mono',monospace;
                                    font-size:11px; font-weight:700;
                                    letter-spacing:0.18em;
                                    padding:12px 40px;
                                    cursor:pointer;
                                ",
                                onclick: move |_| on_finish.call(()),
                                "START →"
                            }
                        }
                    }
                },

                // Error
                OAuthState::Error(msg) => {
                    let msg = msg.clone();
                    rsx! {
                        div {
                            style: "display:flex; flex-direction:column;
                                    align-items:center; gap:14px; width:100%;",

                            div {
                                style: "
                                    width:100%; padding:12px 16px;
                                    border:1px solid var(--danger);
                                    font-family:'Space Mono',monospace;
                                    font-size:10px; color:var(--danger);
                                ",
                                "✗ {msg}"
                            }

                            div { style:"display:flex; gap:10px;",
                                button {
                                    class:"btn-primary",
                                    style:"font-size:10px;",
                                    onclick: move |_| on_cancel.call(()),
                                    "[ RETRY ]"
                                }
                                button {
                                    style: "
                                        background:transparent; border:none;
                                        color:var(--text-muted);
                                        font-family:'Space Mono',monospace;
                                        font-size:10px; letter-spacing:0.1em;
                                        cursor:pointer;
                                    ",
                                    onclick: move |_| on_skip.call(()),
                                    "skip →"
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}
