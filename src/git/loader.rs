// git/loader.rs — Open a local repo or clone a remote one
use crate::git::parser::{load_diff_for_commit, parse_repo, RepoTree, MAX_COMMITS};
use anyhow::{Context, Result};
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Returned by [`shallow_clone`]/[`load_remote`] when `cancel` was flipped
/// mid-clone, so callers can show "cancelled" instead of a scary error.
pub const CLONE_CANCELLED: &str = "clone cancelled by user";

/// Waits on `rx`, but gives up the moment `cancel` is set — polling instead
/// of a plain `rx.await` so the caller's UI never blocks forever on a
/// background git2 call that might itself be stuck with no way to interrupt
/// it (see the comment in `shallow_clone`: on a large enough repo, libgit2
/// can go a long time — confirmed well past a minute against the real
/// torvalds/linux repo — inside a single blocking read with no callback
/// firing in between, so the `cancel` flag alone can't always reach it).
/// `None` means cancelled: the sender side is simply abandoned, and its
/// eventual `send` (if any) silently no-ops once `rx` is dropped.
///
/// Used identically by HomeScreen's and UserRepos' clone flows — this is
/// the one place that guarantee is actually implemented and tested.
pub async fn await_cancellable<T>(
    mut rx: tokio::sync::oneshot::Receiver<T>,
    cancel: &Arc<AtomicBool>,
    poll_interval: std::time::Duration,
) -> Option<std::result::Result<T, tokio::sync::oneshot::error::RecvError>> {
    loop {
        match tokio::time::timeout(poll_interval, &mut rx).await {
            Ok(received) => return Some(received),
            Err(_elapsed) => {
                if cancel.load(Ordering::Relaxed) {
                    return None;
                }
            }
        }
    }
}

/// Clone `url` into `clone_path`, shallow — we only ever display the newest
/// `MAX_COMMITS` commits (see parser.rs), so there's no reason to download a
/// repo's entire history up front. For something the size of the Linux
/// kernel that's the difference between a multi-GB transfer (slow, and
/// prone to failing/timing out partway — leaving a corrupt clone behind)
/// and a fetch of a few hundred commits.
///
/// `cancel` is checked on every transfer-progress tick; flipping it to
/// `true` (e.g. from a Cancel button) aborts the clone.
fn shallow_clone(url: &str, clone_path: &Path, cancel: &Arc<AtomicBool>) -> Result<Repository> {
    let mut callbacks = RemoteCallbacks::new();
    // Checked in both callbacks: `sideband_progress` fires during the early
    // "Counting objects..." negotiation (before any pack bytes transfer),
    // `transfer_progress` during the transfer itself. A slow clone can be
    // stuck in either phase, so Cancel needs to interrupt both to actually
    // be responsive rather than only working once a transfer has started.
    let cancel_sideband = Arc::clone(cancel);
    callbacks.sideband_progress(move |_msg| !cancel_sideband.load(Ordering::Relaxed));
    let cancel_transfer = Arc::clone(cancel);
    callbacks.transfer_progress(move |_stats| !cancel_transfer.load(Ordering::Relaxed));

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.depth(MAX_COMMITS as i32);
    fetch_opts.remote_callbacks(callbacks);

    let result = RepoBuilder::new()
        .fetch_options(fetch_opts)
        .clone(url, clone_path)
        .with_context(|| format!("Failed to clone {url}"));

    if result.is_err() {
        // Whether this was the cancellation or a genuine network error, a
        // partial clone directory isn't reusable (see `load_remote`'s
        // leftover-clone check) — don't leave debris behind in the temp dir.
        std::fs::remove_dir_all(clone_path).ok();
    }

    if result.is_err() && cancel.load(Ordering::Relaxed) {
        return Err(anyhow::anyhow!(CLONE_CANCELLED));
    }
    result
}

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

pub fn load_remote(url: &str, cancel: Arc<AtomicBool>) -> Result<RepoTree> {
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

            // Reuse an existing clone only if it's actually usable — a prior
            // attempt that failed or was interrupted partway (network drop,
            // huge repo taking too long, app closed mid-clone) can leave a
            // `.git` dir behind that *opens* fine but has no resolvable HEAD.
            // Blindly reusing that gave a confusing "reference ... not
            // found" error on every retry, with no way to recover short of
            // manually deleting the temp dir.
            let existing = if clone_path.exists() {
                Repository::open(&clone_path)
                    .ok()
                    .filter(|r| r.head().is_ok())
            } else {
                None
            };

            let repo = match existing {
                Some(r) => r,
                None => {
                    if clone_path.exists() {
                        std::fs::remove_dir_all(&clone_path).with_context(|| {
                            format!("Failed to remove corrupt clone at {:?}", clone_path)
                        })?;
                    }
                    shallow_clone(&url, &clone_path, &cancel)?
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Fast, deterministic, no network — this is what actually guarantees
    /// the UI never hangs on Cancel, independent of whether git2's own
    /// callback-based cancellation manages to fire in time (it empirically
    /// doesn't always, see the network-gated tests below).
    #[test]
    fn await_cancellable_gives_up_once_cancel_is_set() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async {
            // A sender that's never sent to — stands in for a background
            // thread that's stuck forever.
            let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
            let cancel = Arc::new(AtomicBool::new(false));
            let cancel_setter = Arc::clone(&cancel);

            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                cancel_setter.store(true, Ordering::Relaxed);
            });

            let started = std::time::Instant::now();
            let result = await_cancellable(rx, &cancel, std::time::Duration::from_millis(20)).await;
            let elapsed = started.elapsed();

            assert!(result.is_none(), "should give up, not resolve a value");
            assert!(
                elapsed < std::time::Duration::from_secs(1),
                "should give up shortly after cancel is set, took {elapsed:?}"
            );
        });
    }

    #[test]
    fn await_cancellable_returns_the_value_when_not_cancelled() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let cancel = Arc::new(AtomicBool::new(false));
            tx.send(42).unwrap();

            let result = await_cancellable(rx, &cancel, std::time::Duration::from_millis(20)).await;
            assert_eq!(result.unwrap().unwrap(), 42);
        });
    }

    /// Network-gated — not run in CI (`cargo test` skips `#[ignore]`d tests
    /// by default). Run manually with:
    ///   cargo test --package git-tree-viz git::loader::tests -- --ignored --nocapture
    ///
    /// Reproduces the real-world bug report: cloning a huge repo
    /// (`torvalds/linux`) used to do a full, unbounded clone (multi-GB) and
    /// could fail partway with a confusing "reference 'refs/heads/master'
    /// not found" error — then keep failing the same way on every retry,
    /// because the corrupt clone directory was silently reused. This proves
    /// the shallow clone completes, is fast, and that a corrupted leftover
    /// clone self-heals on retry instead of wedging permanently.
    #[test]
    #[ignore]
    fn load_remote_shallow_clones_linux_and_self_heals_a_corrupt_clone() {
        let dir_name = format!("git-tree-test-loader-{}", std::process::id());
        let clone_path = std::env::temp_dir().join("git-tree-clones").join(&dir_name);
        let _ = std::fs::remove_dir_all(&clone_path);

        // Point load_remote at a throwaway folder name by cloning directly
        // via shallow_clone (what load_remote does internally) so we don't
        // depend on / collide with a real cached "linux" clone.
        let started = std::time::Instant::now();
        let no_cancel = Arc::new(AtomicBool::new(false));
        let repo = shallow_clone("https://github.com/torvalds/linux", &clone_path, &no_cancel)
            .expect("shallow clone of torvalds/linux should succeed");
        let elapsed = started.elapsed();
        // Observed, not asserted: a shallow clone is dramatically less data
        // than a full one, but GitHub's server-side pack negotiation for a
        // repo the size of Linux can still take a few minutes depending on
        // network conditions. Correctness (right commit count, HEAD
        // present) is what this test guards, not a timing budget.
        println!("shallow clone took {elapsed:?}");

        let tree = parse_repo(&repo).expect("parse_repo should succeed on the shallow clone");
        assert_eq!(tree.commits.len(), MAX_COMMITS);
        assert!(tree.commits.iter().any(|c| c.is_head));

        drop(repo);

        // Simulate a corrupted leftover clone (e.g. interrupted transfer):
        // delete the ref that HEAD points at, so `Repository::open` still
        // succeeds but `repo.head()` can't resolve.
        std::fs::remove_file(clone_path.join(".git/refs/heads/master"))
            .or_else(|_| std::fs::remove_file(clone_path.join(".git/HEAD")))
            .expect("should be able to corrupt the test clone");
        assert!(
            Repository::open(&clone_path)
                .map(|r| r.head().is_err())
                .unwrap_or(true),
            "test setup didn't actually corrupt the clone"
        );

        // The self-healing path in load_remote's closure: open, find it
        // broken, wipe, re-clone. Exercise that same logic directly here.
        let existing = Repository::open(&clone_path)
            .ok()
            .filter(|r| r.head().is_ok());
        assert!(existing.is_none(), "corrupt clone should not be reused");
        std::fs::remove_dir_all(&clone_path).unwrap();
        let healed = shallow_clone("https://github.com/torvalds/linux", &clone_path, &no_cancel)
            .expect("should self-heal by re-cloning");
        assert!(healed.head().is_ok());

        std::fs::remove_dir_all(&clone_path).ok();
    }

    /// Network-gated, same as above.
    ///
    /// Important nuance this test deliberately does NOT claim any timing
    /// guarantee for: `cancel` is only checked the next time libgit2
    /// invokes one of our callbacks, and on a very large repo libgit2 can
    /// be blocked for a long time — confirmed empirically, both mid-flight
    /// *and* with `cancel` pre-set before the clone even starts — inside a
    /// single blocking call (DNS/TLS/the initial smart-HTTP discovery
    /// request can all happen before any callback we hook ever fires) with
    /// no way to interrupt it from here. So this test only checks
    /// *correctness* of what happens once `shallow_clone` does return: the
    /// distinct `CLONE_CANCELLED` error and no leftover partial directory —
    /// not how fast that happens, which this network can't make reliable.
    ///
    /// The actual responsiveness guarantee — the UI never hangs even if
    /// this call itself does — is `await_cancellable` above, which is fast,
    /// deterministic, and tested without any network dependency.
    #[test]
    #[ignore]
    fn shallow_clone_aborts_when_already_cancelled() {
        let dir_name = format!("git-tree-test-loader-cancel-{}", std::process::id());
        let clone_path = std::env::temp_dir().join("git-tree-clones").join(&dir_name);
        let _ = std::fs::remove_dir_all(&clone_path);

        let cancel = Arc::new(AtomicBool::new(true));

        let started = std::time::Instant::now();
        let result = shallow_clone("https://github.com/torvalds/linux", &clone_path, &cancel);
        let elapsed = started.elapsed();
        println!("already-cancelled clone aborted after {elapsed:?}");

        match result {
            Ok(_) => panic!("a cancelled clone should return an error"),
            Err(err) => assert_eq!(err.to_string(), CLONE_CANCELLED),
        }
        assert!(
            !clone_path.exists(),
            "cancelled clone should not leave a partial directory behind"
        );
        assert!(
            elapsed.as_secs() < 60,
            "cancellation took suspiciously long: {elapsed:?}"
        );
    }
}
