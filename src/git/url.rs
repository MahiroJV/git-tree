/// Convert any remote URL format to a browser-friendly HTTPS URL
fn to_https(remote: &str) -> Option<String> {
    let remote = remote.trim_end_matches('/').trim_end_matches(".git");

    if remote.starts_with("git@") {
        // git@github.com:user/repo → https://github.com/user/repo
        let without_prefix = remote.strip_prefix("git@")?;
        let (host, path) = without_prefix.split_once(':')?;
        Some(format!("https://{}/{}", host, path))
    } else if remote.starts_with("https://") || remote.starts_with("http://") {
        Some(remote.to_string())
    } else {
        None
    }
}

/// Build a commit URL for GitHub, GitLab, or Bitbucket
pub fn commit_web_url(remote_url: &str, hash: &str) -> Option<String> {
    let base = to_https(remote_url)?;
    Some(format!("{}/commit/{}", base, hash))
}
