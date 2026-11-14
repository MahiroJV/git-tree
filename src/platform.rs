// platform.rs — Small shims over things that differ between desktop and Android.
//
// Desktop : native dialogs via `rfd`, normal XDG / AppData / Library dirs.
// Android : no file dialogs; the app may only write inside its own sandbox
//           (/data/data/<package>/…), so HOME, XDG_* and TMPDIR are pointed
//           there on startup. That keeps `dirs::config_dir()` (settings,
//           recent repos, OAuth token) and `std::env::temp_dir()` (clones)
//           working without touching the rest of the code.

use std::path::PathBuf;

/// Whether a native "pick folder" dialog exists on this platform.
pub const CAN_PICK_FOLDER: bool = cfg!(feature = "desktop");

/// Call once at the very start of `main`, before any threads are spawned.
pub fn init() {
    #[cfg(target_os = "android")]
    android::init_env();
}

/// Open a folder picker. Returns `None` if cancelled or unsupported.
pub async fn pick_folder() -> Option<String> {
    #[cfg(feature = "desktop")]
    {
        rfd::AsyncFileDialog::new()
            .pick_folder()
            .await
            .map(|f| f.path().to_string_lossy().to_string())
    }
    #[cfg(not(feature = "desktop"))]
    {
        None
    }
}

/// Save `bytes` as a file the user can find later.
///
/// Desktop: shows a "Save as" dialog. Android: writes into the shared
/// Download folder (falls back to app storage). Returns the saved path.
pub async fn save_file(
    title: &str,
    filter_name: &str,
    extension: &str,
    default_name: &str,
    bytes: Vec<u8>,
) -> Option<PathBuf> {
    #[cfg(feature = "desktop")]
    {
        let handle = rfd::AsyncFileDialog::new()
            .set_title(title)
            .add_filter(filter_name, &[extension])
            .set_file_name(default_name)
            .save_file()
            .await?;
        handle.write(&bytes).await.ok()?;
        Some(handle.path().to_path_buf())
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = (title, filter_name, extension);
        save_to_downloads(default_name, &bytes)
    }
}

#[cfg(not(feature = "desktop"))]
fn save_to_downloads(name: &str, bytes: &[u8]) -> Option<PathBuf> {
    // Android 10+ lets an app create its own files in Download/ without any
    // storage permission. If that fails, keep the file in app storage.
    let candidates = [
        Some(PathBuf::from("/storage/emulated/0/Download")),
        dirs::data_dir().map(|d| d.join("git-tree").join("exports")),
    ];
    for dir in candidates.into_iter().flatten() {
        if std::fs::create_dir_all(&dir).is_err() {
            continue;
        }
        let path = unique_path(&dir, name);
        if std::fs::write(&path, bytes).is_ok() {
            return Some(path);
        }
    }
    None
}

#[cfg(not(feature = "desktop"))]
fn unique_path(dir: &std::path::Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s, format!(".{e}")),
        None => (name, String::new()),
    };
    (1..)
        .map(|n| dir.join(format!("{stem}-{n}{ext}")))
        .find(|p| !p.exists())
        .expect("infinite iterator")
}

#[cfg(target_os = "android")]
mod android {
    use std::path::PathBuf;

    /// An Android app process's cmdline is its package name, e.g.
    /// `io.github.mahirojv.gittree`. Cheaper than a JNI call for getFilesDir().
    fn package_name() -> Option<String> {
        let raw = std::fs::read("/proc/self/cmdline").ok()?;
        let name = raw.split(|b| *b == 0).next()?;
        let name = String::from_utf8_lossy(name).trim().to_string();
        // Strip a ":process" suffix if the app ever runs in a named process.
        let name = name.split(':').next()?.to_string();
        (!name.is_empty() && !name.starts_with('/')).then_some(name)
    }

    pub fn init_env() {
        let Some(pkg) = package_name() else { return };
        let base = PathBuf::from("/data/data").join(pkg);
        let files = base.join("files");
        let cache = base.join("cache");
        let config = files.join(".config");
        let data = files.join(".local/share");
        for d in [&files, &cache, &config, &data] {
            let _ = std::fs::create_dir_all(d);
        }
        // Only fill in what Android left unset.
        let set = |k: &str, v: &PathBuf| {
            if std::env::var_os(k).is_none() {
                std::env::set_var(k, v);
            }
        };
        set("HOME", &files);
        set("XDG_CONFIG_HOME", &config);
        set("XDG_DATA_HOME", &data);
        set("XDG_CACHE_HOME", &cache);
        // Android's default temp_dir() (/data/local/tmp) isn't app-writable.
        std::env::set_var("TMPDIR", &cache);

        // libgit2's own HTTPS can't work here: openssl-src builds OpenSSL for
        // Android with `no-stdio`, so it can't load any CA certificates and
        // every clone fails with "the SSL certificate is invalid". Send git's
        // HTTP traffic through reqwest instead, whose TLS uses Android's store.
        crate::git::http_transport::register();
    }
}
