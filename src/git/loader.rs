// git/loader.rs — Open a local repo or clone a remote one
use anyhow::{Context, Result};
use git2::{build::RepoBuilder, FetchOptions, Repository};
use std::path::Path;

use crate::git::parser::{parse_repo, RepoTree};

/// Load from a local folder path
pub fn load_local(path: &Path) -> Result<RepoTree> {
    let repo =
        Repository::open(path).with_context(|| format!("Failed to open repo at {:?}", path))?;
    parse_repo(&repo)
}

pub fn load_remote(url: &str) -> Result<RepoTree> {
    let temp_dir = std::env::temp_dir().join("git-tree-clones");
    std::fs::create_dir_all(&temp_dir)?;

    let folder_name = url
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("repo")
        .trim_end_matches(".git");

    let clone_path = temp_dir.join(folder_name);

    let repo = if clone_path.exists() {
        Repository::open(&clone_path)
            .with_context(|| format!("Failed to open cached clone at {:?}", clone_path))?
    } else {
        let fetch_opts = FetchOptions::new();
        RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(url, &clone_path)
            .with_context(|| format!("Failed to Clone {}", url))?
    };
    parse_repo(&repo)
}

/// Fetch + pull latest changes for an already-loaded repo
#[warn(dead_code)]
pub fn refresh_local(path: &Path) -> Result<RepoTree> {
    let repo = Repository::open(path)?;

    // Try to fetch from origin if available
    if let Ok(mut remote) = repo.find_remote("origin") {
        let mut fetch_opts = FetchOptions::new();
        let _ = remote.fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_opts),
            None,
        );
    }

    parse_repo(&repo)
}
