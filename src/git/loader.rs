// git/loader.rs — Open a local repo or clone a remote one
use crate::git::parser::{load_diff_for_commit, parse_repo, RepoTree};
use anyhow::{Context, Result};
use git2::{build::RepoBuilder, FetchOptions, Repository};
use std::path::Path;

/// Load from a local folder path
pub fn load_local(path: &Path) -> Result<RepoTree> {
    let path = path.to_owned();
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let repo = Repository::open(&path)
                .with_context(|| format!("Failed to open repo at {:?}", path))?;
            parse_repo(&repo)
        })
        .context("Failed to spawn git thread")?
        .join()
        .map_err(|_| {
            anyhow::anyhow!(
                "Repository processing crashed (libgit2 stack overflow).\n\
             Fix: run  git config --global core.autocrlf false  then retry."
            )
        })?
}

pub fn load_remote(url: &str) -> Result<RepoTree> {
    let temp_dir = std::env::temp_dir().join("git-tree-clones");
    std::fs::create_dir_all(&temp_dir)?;

    let folder_name = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap_or("repo")
        .trim_end_matches(".git")
        .to_string();
    let url = url.to_owned();

    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let clone_path = temp_dir.join(&folder_name);
            let repo = if clone_path.exists() {
                Repository::open(&clone_path)
                    .with_context(|| format!("Failed to open cached clone at {:?}", clone_path))?
            } else {
                RepoBuilder::new()
                    .fetch_options(FetchOptions::new())
                    .clone(&url, &clone_path)
                    .with_context(|| format!("Failed to clone {}", url))?
            };
            parse_repo(&repo)
        })
        .context("Failed to spawn git thread")?
        .join()
        .map_err(|_| {
            anyhow::anyhow!(
                "Repository processing crashed (libgit2 stack overflow).\n\
             Fix: run  git config --global core.autocrlf false  then retry."
            )
        })?
}

pub fn load_commit_diff(
    path: &Path,
    hash: &str,
) -> Result<(
    Vec<crate::git::parser::FileChange>,
    crate::git::parser::DiffStats,
)> {
    let path = path.to_owned();
    let hash = hash.to_owned();

    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let repo = Repository::open(&path)
                .with_context(|| format!("Failed to open repo at {:?}", path))?;
            load_diff_for_commit(&repo, &hash)
        })
        .context("Failed to spawn diff thread")?
        .join()
        .map_err(|_| anyhow::anyhow!("Diff loading crashed"))?
}

/// Fetch + pull latest changes for an already-loaded repo
#[allow(dead_code)]
pub fn refresh_local(path: &Path) -> Result<RepoTree> {
    let repo = Repository::open(path)?;
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
