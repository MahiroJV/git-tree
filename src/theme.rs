// theme.rs — Theme definitions + contributor color assignment
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ─── Theme Definition ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub bg: &'static str,
    pub bg_secondary: &'static str, // panels, cards
    pub text: &'static str,
    pub text_muted: &'static str,
    pub accent: &'static str,       // primary accent (purple in default)
    pub border: &'static str,
    pub success: &'static str,      // added lines
    pub danger: &'static str,       // removed lines
}

// ─── All Themes ──────────────────────────────────────────────────────────────

pub const THEMES: &[Theme] = &[
    // ⭐ Default — clean serious terminal, purple accent
    Theme {
        name: "Terminal",
        bg: "#000000",
        bg_secondary: "#0a0a0a",
        text: "#FFFFFF",
        text_muted: "#666666",
        accent: "#9B5DE5",
        border: "#9B5DE5",
        success: "#00FF85",
        danger: "#FF2244",
    },
    // Classic hacker green
    Theme {
        name: "Matrix",
        bg: "#000000",
        bg_secondary: "#001200",
        text: "#00FF41",
        text_muted: "#005500",
        accent: "#00FF41",
        border: "#00AA20",
        success: "#00FF41",
        danger: "#FF2244",
    },
    // Old phosphor amber monitor
    Theme {
        name: "Amber",
        bg: "#0D0800",
        bg_secondary: "#1a1000",
        text: "#FFB000",
        text_muted: "#7A5500",
        accent: "#FFB000",
        border: "#7A5500",
        success: "#FFD700",
        danger: "#FF4411",
    },
    // 80s retrowave
    Theme {
        name: "Synthwave",
        bg: "#0D0221",
        bg_secondary: "#140330",
        text: "#FF00FF",
        text_muted: "#660066",
        accent: "#00FFFF",
        border: "#FF00FF",
        success: "#00FFAA",
        danger: "#FF2244",
    },
    // Cold Nordic
    Theme {
        name: "Nord",
        bg: "#2E3440",
        bg_secondary: "#3B4252",
        text: "#ECEFF4",
        text_muted: "#4C566A",
        accent: "#88C0D0",
        border: "#81A1C1",
        success: "#A3BE8C",
        danger: "#BF616A",
    },
    // Popular dark dev
    Theme {
        name: "Dracula",
        bg: "#282A36",
        bg_secondary: "#1E1F29",
        text: "#F8F8F2",
        text_muted: "#6272A4",
        accent: "#FF79C6",
        border: "#BD93F9",
        success: "#50FA7B",
        danger: "#FF5555",
    },
    // Warm retro
    Theme {
        name: "Gruvbox",
        bg: "#282828",
        bg_secondary: "#1D2021",
        text: "#EBDBB2",
        text_muted: "#928374",
        accent: "#B8BB26",
        border: "#FABD2F",
        success: "#B8BB26",
        danger: "#CC241D",
    },
    // Dark dramatic red
    Theme {
        name: "Blood Moon",
        bg: "#10000A",
        bg_secondary: "#1A000F",
        text: "#FFE4E4",
        text_muted: "#550022",
        accent: "#FF2244",
        border: "#880022",
        success: "#FF6680",
        danger: "#FF0022",
    },
    // Cold blue cyberpunk
    Theme {
        name: "Ice Terminal",
        bg: "#050F1A",
        bg_secondary: "#0A1A2A",
        text: "#E0F4FF",
        text_muted: "#1A4060",
        accent: "#00CFFF",
        border: "#005F7A",
        success: "#00FF85",
        danger: "#FF2244",
    },
];
#[warn(dead_code)]
pub fn default_theme() -> &'static Theme {
    &THEMES[0] // Terminal (purple)
}

pub fn theme_by_name(name: &str) -> &'static Theme {
    THEMES.iter().find(|t| t.name == name).unwrap_or(&THEMES[0])
}

// ─── Contributor Colors ───────────────────────────────────────────────────────
// Each contributor gets a unique color derived from their name/email hash
// Colors shift hue to fit the active theme's feel

/// Palette of distinct neon colors for contributors
const CONTRIBUTOR_PALETTE: &[&str] = &[
    "#9B5DE5", // purple (matches default accent)
    "#00F5D4", // teal
    "#FEE440", // yellow
    "#F15BB5", // pink
    "#00BBF9", // sky blue
    "#FF6B35", // orange
    "#06D6A0", // green mint
    "#EF233C", // red
    "#4CC9F0", // light blue
    "#F72585", // hot pink
    "#7209B7", // deep purple
    "#3A86FF", // blue
];

pub fn contributor_color(email: &str) -> &'static str {
    // Hash the email to get a consistent index
    let mut hasher = Sha256::new();
    hasher.update(email.as_bytes());
    let result = hasher.finalize();
    let index = (result[0] as usize) % CONTRIBUTOR_PALETTE.len();
    CONTRIBUTOR_PALETTE[index]
}