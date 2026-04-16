use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use git2::Repository;
use crate::theme::contributor_color;

// ─── Core Data Structures ────────────────────────────────────────────────────

/// A single commit node on the tree
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct CommitNode {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub full_message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: DateTime<Utc>,
    pub branch_name: Option<String>,
    pub color: String,
    pub parent_hashes: Vec<String>,
    pub is_merge: bool,
    pub is_head: bool,
    pub tags: Vec<String>,
    pub files_changed: Vec<FileChange>,
    pub stats: DiffStats,
}

/// A single file changed in a commit
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Summary diff stats for a commit
#[derive(Debug, Clone, Serialize,PartialEq, Deserialize, Default)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

/// A branch line in the visual tree
#[derive(Debug, Clone, PartialEq)]
pub struct BranchLine {
    pub name: String,
    pub commits: Vec<String>,
    pub color: String,
    pub direction: BranchDirection,
    pub parent_hash: String,
    pub merge_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchDirection {
    Up,
    Down,
}

/// The full parsed repository tree ready for rendering
#[derive(Debug, PartialEq, Clone)]
pub struct RepoTree {
    pub commits: Vec<CommitNode>,
    pub branches: Vec<BranchLine>,
    pub main_branch: Vec<String>,
    pub repo_name: String,
    pub remote_url: Option<String>,
    pub total_contributors: usize,
}

// ─── Parser ──────────────────────────────────────────────────────────────────

pub fn parse_repo(repo: &Repository) -> Result<RepoTree> {
    let mut walk = repo.revwalk()?;

    //Starts from all branch heads
    walk.push_glob("refs/heads/*")?;
    walk.push_glob("refs/remotes/*")?;

    // Topological sort so parent/child order is correct
    walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    let mut commits: Vec<CommitNode> = Vec::new();
    let head_id = repo.head().ok().and_then(|h| h.target());

    // Collect all tags for quick lookup
    let mut tag_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    repo.tag_foreach(|oid, name| {
        let tag_name = std::str::from_utf8(name)
            .unwrap_or("")
            .trim_start_matches("refs/tags/")
            .to_string();
        tag_map.entry(oid.to_string()).or_default().push(tag_name);
        true
    })?;

    for oid in walk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let color = contributor_color(&author_email).to_string();

        // Get diff stats vs first parent
        let stats = if let Some(parent) = commit.parents().next() {
            let parent_tree = parent.tree()?;
            let commit_tree = commit.tree()?;
            let diff = repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&commit_tree),
                None,
            )?;
            let s = diff.stats()?;
            DiffStats {
                files_changed: s.files_changed(),
                insertions: s.insertions(),
                deletions: s.deletions(),
            }
        } else{
            DiffStats::default()
        };

        let hash = oid.to_string();
        let short_hash = hash[..7].to_string();
        let message_full = commit.message().unwrap_or("").to_string();
        let message = message_full.lines().next().unwrap_or("").to_string();

        let timestamp = DateTime::from_timestamp(commit.time().seconds(), 0).unwrap_or_default();

        let parent_hashes: Vec<String> = commit
            .parents()
            .map(|p| p.id().to_string())
            .collect();

        let is_merge = parent_hashes.len() > 1;
        let is_head = head_id.map(|h| h == oid ).unwrap_or(false);
        let tags = tag_map.get(&hash).cloned().unwrap_or_default();

        commits.push(CommitNode {
            short_hash,
            full_message: message_full,
            message,
            author_name,
            author_email,
            color,
            timestamp,
            parent_hashes,
            is_merge,
            is_head,
            tags,
            files_changed: Vec::new(),
            stats,
            branch_name: None,
            hash,
        });
    }

    let branches = build_branch_lines(&commits);

    // Main branch = longest chain from HEAD
    let main_branch = branches
        .first()
        .map(|b| b.commits.clone())
        .unwrap_or_default();

    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()));

    let total_contributors = commits
        .iter()
        .map(|c| c.author_email.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    Ok(RepoTree {
        commits,
        branches,
        main_branch,
        repo_name,
        remote_url,
        total_contributors,
    })
}

/// Build visual branch lines, alternating up/down
fn build_branch_lines(commits: &[CommitNode]) -> Vec<BranchLine> {
    let mut lines: Vec<BranchLine> = Vec::new();
    let mut direction_toggle = true; // true = up, false = down

    if commits.is_empty() {
        return lines;
    }

    let main_commits: Vec<String> = commits
        .iter()
        .filter(|c| c.parent_hashes.len() <= 1)
        .map(|c| c.hash.clone())
        .collect();

    if !main_commits.is_empty() {
        lines.push(BranchLine {
            name: "main".to_string(),
            commits: main_commits,
            color: "#9B5DE5".to_string(), // accent color for main
            direction: BranchDirection::Up, // main is horizontal
            parent_hash: String::new(),
            merge_hash: None,
        });
    }

    // Feature branches = merge commits and their parents
    for commit in commits.iter().filter(|c| c.is_merge) {
        let dir = if direction_toggle {
            BranchDirection::Up
        } else {
            BranchDirection::Down
        };
        direction_toggle = !direction_toggle;

        lines.push(BranchLine {
            name: format!("branch-{}", &commit.short_hash),
            commits: vec![commit.hash.clone()],
            color: commit.color.clone(),
            direction: dir,
            parent_hash: commit
                .parent_hashes
                .first()
                .cloned()
                .unwrap_or_default(),
            merge_hash: Some(commit.hash.clone()),
        });
    }

    lines
}