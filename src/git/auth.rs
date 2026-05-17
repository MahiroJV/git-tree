// git/auth.rs — GitHub Device Flow OAuth
//
// Flow:
//   1. request_device_code() → get user_code + device_code from GitHub
//   2. Show user_code to user, open github.com/login/device in browser
//   3. poll_for_token()      → poll until user completes auth
//   4. Token saved to ~/.config/git-tree/token.json

use anyhow::{Context, Result};
#[allow(unused_imports)]
use openssl::pkey::Public;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CLIENT_ID: &str = "Ov23liJhiBAeSXGr1SMv";

const SCOPE: &str = "repo read:user";

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AuthToken {
    pub access_token: String,
    pub scope: String,
    pub username: Option<String>,
}

pub struct DeviceFlowStart {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub interval: u64,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct GithubRepo {
    pub id: u64,
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub language: Option<String>,
    pub clone_url: String,
    pub html_url: String,
    pub updated: Option<String>,
    pub default_branch: String,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub struct GithubProfile {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub public_repos: u64,
    pub followers: u64,
    pub following: u64,
    pub avatar_url: String,
}

// ── GitHub API response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct RawDeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
}

#[derive(Deserialize)]
struct RawToken {
    access_token: Option<String>,
    scope: Option<String>,
    error: Option<String>,
}

// ── Token persistence ─────────────────────────────────────────────────────────

fn token_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("git-tree").join("token.json"))
}

pub fn load_token() -> Option<AuthToken> {
    let path = token_path()?;
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_token(token: &AuthToken) {
    let Some(path) = token_path() else { return };
    if let Some(p) = path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if let Ok(data) = serde_json::to_string_pretty(token) {
        let _ = std::fs::write(path, data);
    }
}

pub fn clear_token() {
    if let Some(path) = token_path() {
        let _ = std::fs::remove_file(path);
    }
}

// ── HTTP client helper ────────────────────────────────────────────────────────

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("git-tree/0.4 (https://github.com/MahiroJV/git-tree)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .context("Failed to build HTTP client")
}

// ── Device flow — Step 1 ──────────────────────────────────────────────────────

/// Request a device code from GitHub.
/// Returns the user_code to show to the user + device_code to poll with.
pub async fn request_device_code() -> Result<DeviceFlowStart> {
    let client = http_client()?;

    let resp = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPE)])
        .send()
        .await
        .context("Device code request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("Github returned HTTP {}", resp.status());
    }

    let raw: RawDeviceCode = resp
        .json()
        .await
        .context("Failed to parse device code response")?;

    Ok(DeviceFlowStart {
        user_code: raw.user_code,
        verification_uri: raw.verification_uri,
        device_code: raw.device_code,
        interval: raw.interval,
    })
}

// ── Device flow — Step 2 ──────────────────────────────────────────────────────

/// Poll GitHub until the user completes auth in the browser.
/// Respects the `interval` returned by GitHub (usually 5 s).
pub async fn poll_for_token(device_code: String, interval: u64) -> Result<AuthToken> {
    let client = http_client()?;
    let mut wait_secs = interval.max(5);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

        let resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .context("Token poll request failed")?;

        let body = resp.text().await.context("Failed to read response body")?;

        // GitHub returns HTML if device code expired — catch it early
        if body.trim_start().starts_with('<') {
            anyhow::bail!("Device code expired — please sign in again");
        }

        let raw: RawToken =
            serde_json::from_str(&body).with_context(|| format!("Failed to parse: {body}"))?;

        match raw.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                wait_secs += 5;
                continue;
            }
            Some("expired_token") => anyhow::bail!("Code expired — please sign in again"),
            Some(e) => anyhow::bail!("OAuth error: {}", e),
            None => {}
        }

        if let Some(token) = raw.access_token {
            let username = fetch_username(&token).await.ok();
            let auth = AuthToken {
                access_token: token,
                scope: raw.scope.unwrap_or_default(),
                username,
            };
            save_token(&auth);
            return Ok(auth);
        }
    }
}

// ── Fetch user profile ────────────────────────────────────────────────────────

pub async fn fetch_profile(_token: &str) -> Result<GithubProfile> {
    let client = http_client()?;
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {_token}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Failed to fetch GitHub profile")?;

    resp.json().await.context("Failed to parse GitHub profile")
}

// ── Fetch user repos (public + private) ──────────────────────────────────────

pub async fn fetch_user_repos(token: &str) -> Result<Vec<GithubRepo>> {
    let client = http_client()?;
    let mut all_repos: Vec<GithubRepo> = Vec::new();
    let mut page = 1u32;

    loop {
        let url = format!(
            "https://api.github.com/user/repos\
            ?sort=updated&per_page=100&page={page}&affiliation=owner"
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .context("Failed to fetch user repos")?;

        if !resp.status().is_success() {
            anyhow::bail!("Github API Error {}", resp.status());
        }

        let repos: Vec<GithubRepo> = resp
            .json()
            .await
            .context("Failed to parse user repos response")?;

        let done = repos.len() < 100;
        all_repos.extend(repos);

        if done {
            break;
        }
        page += 1
    }

    Ok(all_repos)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn fetch_username(token: &str) -> Result<String> {
    let client = http_client()?;
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Failed to fetch GitHub user")?;

    let profile: GithubProfile = resp
        .json()
        .await
        .context("Failed to parse GitHub user response")?;
    Ok(profile.login)
}
