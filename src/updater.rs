// updater.rs — Check GitHub releases and self-update the binary.
//
// Flow:
//   1. check_for_updates()  → async, called once on app start
//   2. download_and_apply() → async, called when user confirms
//
// Linux  : atomic rename() — the old inode stays alive for the running process,
//          new launches pick up the new binary. User just restarts.
// Windows: can't overwrite a running .exe, so we write a .bat that swaps the
//          file and relaunches after the current process exits.

use anyhow::{Context, Result};
use serde::Deserialize;

// ── Constants ─────────────────────────────────────────────────────────────────

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_OWNER: &str = "MahiroJV";
const GITHUB_REPO: &str = "git-tree";

// ── GitHub API types ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
}

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

// ── Check ─────────────────────────────────────────────────────────────────────

pub async fn check_for_updates() -> Result<Option<UpdateInfo>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent(format!("git-tree/{} updater", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Update check request failed")?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let release: GithubRelease = resp
        .json()
        .await
        .context("Failed to parse GitHub release JSON")?;

    let latest_raw = release.tag_name.trim_start_matches('v');

    if !is_newer(latest_raw, CURRENT_VERSION) {
        return Ok(None);
    }

    let notes = release
        .body
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" · ");

    Ok(Some(UpdateInfo {
        latest_version: release.tag_name,
        release_url: release.html_url,
        release_notes: notes,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_newer(candidate: &str, current: &str) -> bool {
    fn parse(s: &str) -> (u32, u32, u32) {
        #[allow(clippy::manual_pattern_char_comparison)]
        let mut it = s
            .split(|c: char| c == '-' || c == '+')
            .next()
            .unwrap_or(s)
            .split('.')
            .filter_map(|p| p.parse::<u32>().ok());
        (
            it.next().unwrap_or(0),
            it.next().unwrap_or(0),
            it.next().unwrap_or(0),
        )
    }
    parse(candidate) > parse(current)
}

#[cfg(test)]
mod tests {
    use super::is_newer;
    #[test]
    fn newer_patch() {
        assert!(is_newer("0.3.1", "0.3.0"));
    }
    #[test]
    fn newer_minor() {
        assert!(is_newer("0.4.0", "0.3.9"));
    }
    #[test]
    fn same_version() {
        assert!(!is_newer("0.3.0", "0.3.0"));
    }
    #[test]
    fn older() {
        assert!(!is_newer("0.2.9", "0.3.0"));
    }
    #[test]
    fn prerelease() {
        assert!(is_newer("0.4.0-beta.1", "0.3.0"));
    }
}
