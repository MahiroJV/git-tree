use crate::theme::contributor_color;
use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::{Patch, Repository};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// The full parsed repository tree ready for rendering
#[derive(Debug, PartialEq, Clone)]
pub struct RepoTree {
    pub commits: Vec<CommitNode>,
    pub repo_name: String,
    pub remote_url: Option<String>,
    pub total_contributors: usize,
    pub repo_path: Option<PathBuf>,
    pub truncated: bool,
}

// ─── Limits ──────────────────────────────────────────────────────────────────

pub(crate) const MAX_COMMITS: usize = 500;
const MAX_LINES_PER_FILE: usize = 80;
const MAX_LINES_PER_COMMIT: usize = 300;

// ─── Parser ──────────────────────────────────────────────────────────────────

pub fn parse_repo(repo: &Repository) -> Result<RepoTree> {
    let mut walk = repo.revwalk()?;
    walk.push_head()?;

    #[allow(clippy::manual_flatten)]
    for branch in repo.branches(Some(git2::BranchType::Local))? {
        if let Ok((branch, _)) = branch {
            if let Some(oid) = branch.get().target() {
                let _ = walk.push(oid);
            }
        }
    }
    // Newest-first (out from HEAD). Previously this included `REVERSE`, which
    // walked oldest-first — on a repo with more than MAX_COMMITS history, the
    // cap below then kept the *oldest* commits and silently dropped HEAD and
    // everything recent. Collecting newest-first and reversing afterwards
    // keeps the same oldest→newest display order while actually showing
    // recent work on large repos.
    walk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    let mut commits: Vec<CommitNode> = Vec::new();
    let head_id = repo.head().ok().and_then(|h| h.target());

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

    let mut truncated = false;

    for oid in walk {
        // ── Commit cap ── stop as soon as we have enough; no need to keep
        // draining the rest of a huge history just to count it.
        if commits.len() >= MAX_COMMITS {
            truncated = true;
            break;
        }

        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let color = contributor_color(&author_email).to_string();

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
            files_changed: Vec::new(),   // ← empty — loaded lazily on click
            stats: DiffStats::default(), // ← empty
            branch_name: None,
            hash,
        });
    }

    // Walk yielded newest-first; reverse so the rest of the app (which lays
    // out index 0 as the leftmost/topmost node) sees oldest → newest, same as
    // before this fix.
    commits.reverse();

    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let repo_path = repo.workdir().map(|p| p.to_path_buf());
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
        repo_name,
        remote_url,
        total_contributors,
        repo_path,
        truncated,
    })
}

/// Load diff for a single commit — called lazily when user clicks a node
pub fn load_diff_for_commit(repo: &Repository, hash: &str) -> Result<(Vec<FileChange>, DiffStats)> {
    let oid = git2::Oid::from_str(hash)?;
    let commit = repo.find_commit(oid)?;

    let commit_tree = commit.tree()?;
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.ignore_whitespace_eol(true).context_lines(3);

    let diff = if let Some(parent) = commit.parents().next() {
        let parent_tree = parent.tree()?;
        repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut diff_opts))?
    } else {
        repo.diff_tree_to_tree(None, Some(&commit_tree), Some(&mut diff_opts))?
    };

    let mut files_changed: Vec<FileChange> = Vec::new();
    let mut total_lines = 0usize;

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

        let mut diff_lines: Vec<DiffLine> = Vec::new();
        let mut file_adds = 0usize;
        let mut file_dels = 0usize;

        if total_lines < MAX_LINES_PER_COMMIT {
            if let Ok(Some(patch)) = Patch::from_diff(&diff, delta_idx) {
                // O(1) from libgit2 — avoids walking every line of a huge
                // file just to tally +/- counts (the loop below only needs
                // to run far enough to fill the display cap).
                if let Ok((_, adds, dels)) = patch.line_stats() {
                    file_adds = adds;
                    file_dels = dels;
                }

                'outer: for hunk_idx in 0..patch.num_hunks() {
                    for line_idx in 0..patch.num_lines_in_hunk(hunk_idx).unwrap_or(0) {
                        if diff_lines.len() >= MAX_LINES_PER_FILE
                            || total_lines >= MAX_LINES_PER_COMMIT
                        {
                            break 'outer;
                        }
                        if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
                            let origin = line.origin();
                            if matches!(origin, '+' | '-' | ' ') {
                                let content = std::str::from_utf8(line.content())
                                    .unwrap_or("")
                                    .trim_end_matches('\n')
                                    .trim_end_matches('\r')
                                    .to_string();
                                diff_lines.push(DiffLine { origin, content });
                                total_lines += 1;
                            }
                        }
                    }
                }
            }
        }

        files_changed.push(FileChange {
            path,
            additions: file_adds,
            deletions: file_dels,
            status,
            lines: diff_lines,
        });
    }

    let raw_stats = diff.stats()?;
    let stats = DiffStats {
        files_changed: raw_stats.files_changed(),
        insertions: raw_stats.insertions(),
        deletions: raw_stats.deletions(),
    };

    Ok((files_changed, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for the `Sort::REVERSE` bug: on a repo bigger than
    /// `MAX_COMMITS`, `parse_repo` must keep the commits nearest HEAD, not
    /// the oldest ones in the repo's history.
    #[test]
    fn parse_repo_keeps_newest_commits_within_cap() {
        let dir = std::env::temp_dir().join(format!(
            "git-tree-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let repo = Repository::init(&dir).unwrap();

        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let total_commits = MAX_COMMITS * 2;
        let mut parent_oid: Option<git2::Oid> = None;
        for i in 0..total_commits {
            let parents: Vec<git2::Commit> = parent_oid
                .map(|oid| vec![repo.find_commit(oid).unwrap()])
                .unwrap_or_default();
            let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
            let message = format!("commit {i}");
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_refs)
                .unwrap();
            parent_oid = Some(oid);
        }

        let result = parse_repo(&repo).unwrap();
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(result.commits.len(), MAX_COMMITS);
        assert!(result.truncated);

        // Oldest → newest display order: the last entry should be HEAD, i.e.
        // the very last commit created ("commit {total_commits - 1}"), not
        // an old one from the start of history.
        let newest = result.commits.last().unwrap();
        assert!(newest.is_head);
        assert_eq!(newest.message, format!("commit {}", total_commits - 1));

        // And the window should be contiguous, ending at HEAD: the oldest
        // commit kept is exactly MAX_COMMITS back from HEAD.
        let oldest = result.commits.first().unwrap();
        assert_eq!(
            oldest.message,
            format!("commit {}", total_commits - MAX_COMMITS)
        );
    }
}
