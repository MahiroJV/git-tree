use anyhow::{Context, Result};
use serde::Deserialize;

// ── API response types ────────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub language: Option<String>,
    pub clone_url: String,
    pub html_url: String,
    pub default_branch: String,
    pub open_issues_count: u64,
    pub license: Option<License>,
    pub topics: Option<Vec<String>>,
    pub pushed_at: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct License {
    pub spdx_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    items: Vec<SearchResult>,
}

// ── Search ────────────────────────────────────────────────────────────────────

pub async fn search_github(query: &str, limit: u8) -> Result<Vec<SearchResult>> {
    if query.is_empty() {
        return Ok(vec![]);
    }

    let url = format!(
        "https://api.github.com/search/repositories\
         ?q={q}&sort=stars&order=desc&per_page={n}",
        q = urlencoding::encode(query),
        n = limit.min(30),
    );

    let client = reqwest::Client::builder()
        .user_agent("git-tree/0.2 (https://github.com/MahiroJV/git-tree)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Network Request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 403 || status.as_u16() == 429 {
            anyhow::bail!("Rate limited by GitHub - Wait a moment and try again");
        }
        anyhow::bail!("Github API error {status}: {body}");
    }

    let data: ApiResponse = resp
        .json()
        .await
        .context("Failed to parse GitHub response")?;
    Ok(data.items)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format a large star count compactly: 1234 → "1.2k", 12345678 → "12.3M"
pub fn fmt_stars(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
