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

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_OWNER: &str = "MahiroJV";
const GITHUB_REPO: &str = "git-tree";

// ── GitHub API types ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
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

    let asset_name = platform_asset_name();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Release {} has no asset named '{}'",
                release.tag_name,
                asset_name
            )
        })?;

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
        download_url: asset.browser_download_url.clone(),
        release_notes: notes,
    }))
}

// ── Download & apply ──────────────────────────────────────────────────────────

pub async fn download_and_apply(
    download_url: &str,
    progress: Option<tokio::sync::watch::Sender<u8>>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(format!("git-tree/{} updater", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get(download_url)
        .send()
        .await
        .context("Download request failed")?;

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut bytes = Vec::with_capacity(total as usize);

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading download chunk")?;
        downloaded += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);

        #[allow(clippy::manual_checked_ops)]
        if total > 0 {
            let pct = ((downloaded * 100) / total) as u8;
            if let Some(tx) = &progress {
                let _ = tx.send(pct);
            }
        }
    }

    let current_exe = std::env::current_exe().context("Cannot locate current executable")?;
    let tmp_path = current_exe.with_extension("update.tmp");

    std::fs::write(&tmp_path, &bytes).context("Failed to write downloaded binary")?;

    apply_update(&current_exe, &tmp_path)?;
    Ok(())
}

// ── Platform apply ────────────────────────────────────────────────────────────

#[cfg(unix)]
fn apply_update(current_exe: &std::path::Path, tmp_path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(tmp_path, std::fs::Permissions::from_mode(0o755))
        .context("Failed to chmod downloaded binary")?;
    std::fs::rename(tmp_path, current_exe).context("Failed to replace current binary")?;
    Ok(())
}

#[cfg(windows)]
fn apply_update(current_exe: &std::path::Path, tmp_path: &std::path::Path) -> Result<()> {
    let bat_path = current_exe.with_extension("update.bat");
    let bat = format!(
        "@echo off\r\n\
         :wait\r\n\
         timeout /t 1 /nobreak > nul\r\n\
         move /y \"{tmp}\" \"{exe}\" > nul 2>&1\r\n\
         if errorlevel 1 goto wait\r\n\
         start \"\" \"{exe}\"\r\n\
         del \"%~f0\"",
        tmp = tmp_path.display(),
        exe = current_exe.display(),
    );
    std::fs::write(&bat_path, bat).context("Failed to write update helper script")?;
    std::process::Command::new("cmd")
        .args(["/c", "start", "/min", "", bat_path.to_str().unwrap_or("")])
        .spawn()
        .context("Failed to launch update helper script")?;
    std::process::exit(0);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn platform_asset_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "git-tree-windows-x86_64.exe";
    #[cfg(target_os = "linux")]
    return "git-tree-linux-x86_64";
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    return "git-tree-linux-x86_64";
}

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
