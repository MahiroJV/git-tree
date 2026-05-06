use crate::theme::contributor_color;
use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::{Patch, Repository};
use serde::{Deserialize, Serialize};

// ─── Core Data Structures ────────────────────────────────────────────────────

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

/// A single line in a diff hunk
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct DiffLine {
    /// '+' added, '-' removed, ' ' context
    pub origin: char,
    pub content: String,
}

/// A single file changed in a commit
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub status: ChangeStatus,
    /// Actual diff lines — capped per file to avoid memory blowout
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Summary diff stats for a commit
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Default)]
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

// ─── Limits ──────────────────────────────────────────────────────────────────

/// Max diff lines stored per file (avoids memory blowout on large files)
const MAX_LINES_PER_FILE: usize = 80;
/// Max total diff lines stored per commit across all files
const MAX_LINES_PER_COMMIT: usize = 300;

// ─── Parser ──────────────────────────────────────────────────────────────────

pub fn parse_repo(repo: &Repository) -> Result<RepoTree> {
    let mut walk = repo.revwalk()?;

    walk.push_head()?;

    // Also walk all local branches
    #[allow(clippy::manual_flatten)]
    for branch in repo.branches(Some(git2::BranchType::Local))? {
        if let Ok((branch, _)) = branch {
            if let Some(oid) = branch.get().target() {
                let _ = walk.push(oid);
            }
        }
    }

    // Oldest → newest so the tree reads left-to-right chronologically
    walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME | git2::Sort::REVERSE)?;

    let mut commits: Vec<CommitNode> = Vec::new();
    let head_id = repo.head().ok().and_then(|h| h.target());

    // Collect all tags for quick lookup
    let mut tag_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
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

        // ── Diff ────────────────────────────────────────────────────────────
        let commit_tree = commit.tree()?;
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts
            .ignore_whitespace_eol(true)
            .context_lines(3)
            .include_untracked(false);

        let diff = if let Some(parent) = commit.parents().next() {
            let parent_tree = parent.tree()?;
            repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut diff_opts))?
        } else {
            repo.diff_tree_to_tree(None, Some(&commit_tree), Some(&mut diff_opts))?
        };

        let mut files_changed: Vec<FileChange> = Vec::new();
        let mut total_lines_collected: usize = 0;

        for (delta_idx, delta) in diff.deltas().enumerate() {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("unknown")
                .to_string();

            let status = match delta.status() {
                git2::Delta::Added => ChangeStatus::Added,
                git2::Delta::Deleted => ChangeStatus::Deleted,
                git2::Delta::Renamed => ChangeStatus::Renamed,
                _ => ChangeStatus::Modified,
            };

            // Collect actual diff lines via Patch
            let mut diff_lines: Vec<DiffLine> = Vec::new();
            let mut file_additions: usize = 0;
            let mut file_deletions: usize = 0;

            if total_lines_collected < MAX_LINES_PER_COMMIT {
                if let Ok(Some(patch)) = Patch::from_diff(&diff, delta_idx) {
                    let num_hunks = patch.num_hunks();
                    'outer: for hunk_idx in 0..num_hunks {
                        let num_lines = patch.num_lines_in_hunk(hunk_idx).unwrap_or(0);
                        for line_idx in 0..num_lines {
                            if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
                                let origin = line.origin();
                                match origin {
                                    '+' => file_additions += 1,
                                    '-' => file_deletions += 1,
                                    _ => {}
                                }
                                // Only store +/- and context lines (skip hunk headers etc.)
                                if matches!(origin, '+' | '-' | ' ') {
                                    if diff_lines.len() < MAX_LINES_PER_FILE
                                        && total_lines_collected < MAX_LINES_PER_COMMIT
                                    {
                                        let content = std::str::from_utf8(line.content())
                                            .unwrap_or("")
                                            .trim_end_matches('\n')
                                            .trim_end_matches('\r')
                                            .to_string();
                                        diff_lines.push(DiffLine { origin, content });
                                        total_lines_collected += 1;
                                    } else {
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            files_changed.push(FileChange {
                path,
                additions: file_additions,
                deletions: file_deletions,
                status,
                lines: diff_lines,
            });
        }

        // Overall stats from git2's built-in counter
        let diff_stats = diff.stats()?;
        let stats = DiffStats {
            files_changed: diff_stats.files_changed(),
            insertions: diff_stats.insertions(),
            deletions: diff_stats.deletions(),
        };
        // ── End diff ────────────────────────────────────────────────────────

        let hash = oid.to_string();
        let short_hash = hash[..7].to_string();
        let message_full = commit.message().unwrap_or("").to_string();
        let message = message_full.lines().next().unwrap_or("").to_string();

        let timestamp = DateTime::from_timestamp(commit.time().seconds(), 0).unwrap_or_default();
        let parent_hashes: Vec<String> = commit.parents().map(|p| p.id().to_string()).collect();
        let is_merge = parent_hashes.len() > 1;
        let is_head = head_id.map(|h| h == oid).unwrap_or(false);
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
            files_changed,
            stats,
            branch_name: None,
            hash,
        });
    }

    let branches = build_branch_lines(&commits);
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

fn build_branch_lines(commits: &[CommitNode]) -> Vec<BranchLine> {
    let mut lines: Vec<BranchLine> = Vec::new();
    let mut direction_toggle = true;

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
            color: "#9B5DE5".to_string(),
            direction: BranchDirection::Up,
            parent_hash: String::new(),
            merge_hash: None,
        });
    }

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
            parent_hash: commit.parent_hashes.first().cloned().unwrap_or_default(),
            merge_hash: Some(commit.hash.clone()),
        });
    }

    lines
}
