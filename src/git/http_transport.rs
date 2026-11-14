// git/http_transport.rs — git "smart HTTP" transport over reqwest.
//
// Why: on Android, openssl-src builds the vendored OpenSSL with `no-stdio`,
// so libgit2's built-in HTTPS can't load any CA certificates and every clone
// fails with "the SSL certificate is invalid". reqwest's TLS (native-tls)
// loads the Android system store itself, so we let libgit2 speak the git
// protocol and only hand the HTTP requests to reqwest.
//
// Only fetching is supported (clone/fetch = upload-pack); git-tree never pushes.
// Same approach as the `git2-curl` crate, see:
// https://git-scm.com/docs/http-protocol (stateless RPC, protocol v1)

use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
use git2::{Error, ErrorClass, ErrorCode, Remote};
use std::io::{self, Read, Write};
use std::sync::Once;
use std::time::Duration;

/// Route `https://` and `http://` remotes through reqwest. Call once at
/// startup, before any clone. Safe to call more than once.
pub fn register() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for scheme in ["https", "http"] {
            // SAFETY: called once, before any other thread uses libgit2.
            let _ = unsafe { git2::transport::register(scheme, factory) };
        }
    });
}

fn factory(remote: &Remote<'_>) -> Result<Transport, Error> {
    // rpc = true: every request is a separate HTTP call (stateless RPC).
    Transport::smart(remote, true, ReqwestSubtransport)
}

struct ReqwestSubtransport;

impl SmartSubtransport for ReqwestSubtransport {
    fn action(
        &self,
        url: &str,
        action: Service,
    ) -> Result<Box<dyn SmartSubtransportStream>, Error> {
        let base = url.trim_end_matches('/');
        let (method, endpoint, content_type, accept) = match action {
            Service::UploadPackLs => (
                Method::Get,
                format!("{base}/info/refs?service=git-upload-pack"),
                None,
                "application/x-git-upload-pack-advertisement",
            ),
            Service::UploadPack => (
                Method::Post,
                format!("{base}/git-upload-pack"),
                Some("application/x-git-upload-pack-request"),
                "application/x-git-upload-pack-result",
            ),
            Service::ReceivePackLs | Service::ReceivePack => {
                return Err(git_error("pushing is not supported by git-tree"))
            }
        };
        Ok(Box::new(HttpStream {
            method,
            url: endpoint,
            content_type,
            accept,
            body: Vec::new(),
            response: None,
        }))
    }

    fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}

enum Method {
    Get,
    Post,
}

/// libgit2 writes the request body (POST only), then reads the response.
/// The request is sent on the first read, once the whole body is known.
struct HttpStream {
    method: Method,
    url: String,
    content_type: Option<&'static str>,
    accept: &'static str,
    body: Vec<u8>,
    response: Option<reqwest::blocking::Response>,
}

impl HttpStream {
    fn send(&mut self) -> io::Result<reqwest::blocking::Response> {
        // Tokens arrive embedded in the URL (`https://TOKEN@github.com/…`);
        // turn them into a Basic auth header like git does.
        let mut url = reqwest::Url::parse(&self.url).map_err(io_error)?;
        let user = url.username().to_string();
        let pass = url.password().map(str::to_string);
        let _ = url.set_username("");
        let _ = url.set_password(None);

        let mut req = match self.method {
            Method::Get => client()?.get(url),
            Method::Post => client()?.post(url).body(std::mem::take(&mut self.body)),
        }
        .header("Accept", self.accept);
        if let Some(ct) = self.content_type {
            req = req.header("Content-Type", ct);
        }
        if !user.is_empty() {
            let user = urlencoding::decode(&user).map_err(io_error)?.into_owned();
            let pass = pass
                .map(|p| urlencoding::decode(&p).map(|p| p.into_owned()))
                .transpose()
                .map_err(io_error)?;
            req = req.basic_auth(user, pass);
        }

        let resp = req.send().map_err(io_error)?;
        let status = resp.status();
        if !status.is_success() {
            let hint = match status.as_u16() {
                // GitHub answers 401 for missing repos too, to hide private ones.
                401 | 403 | 404 => {
                    " (repository not found, or private — sign in with GitHub for private repos)"
                }
                _ => "",
            };
            return Err(io_error(format!("HTTP {status} from {}{hint}", self.url)));
        }
        // A dumb HTTP server (or a login page) answers with something else.
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !ct.starts_with(self.accept) {
            return Err(io_error(format!(
                "{} is not a git smart-HTTP server (content-type: {ct})",
                self.url
            )));
        }
        Ok(resp)
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.response.is_none() {
            self.response = Some(self.send()?);
        }
        self.response.as_mut().expect("set above").read(buf)
    }
}

impl Write for HttpStream {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.body.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// One shared client: connection reuse across the many requests of a clone.
fn client() -> io::Result<&'static reqwest::blocking::Client> {
    static CLIENT: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
    if let Some(c) = CLIENT.get() {
        return Ok(c);
    }
    let c = reqwest::blocking::Client::builder()
        // GitHub expects a git-like user agent for smart HTTP.
        .user_agent(concat!("git/2.0 git-tree/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(20))
        // No overall timeout: big packfiles can take minutes on mobile data.
        .timeout(None)
        .build()
        .map_err(io_error)?;
    Ok(CLIENT.get_or_init(|| c))
}

fn io_error(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

fn git_error(msg: &str) -> Error {
    Error::new(ErrorCode::GenericError, ErrorClass::Net, msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Clones a small public repo through this transport (network needed):
    ///   cargo test http_transport -- --ignored --nocapture
    #[test]
    #[ignore]
    fn shallow_clone_over_reqwest() {
        register();
        let dir = std::env::temp_dir().join(format!("git-tree-transport-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let mut fo = git2::FetchOptions::new();
        fo.depth(20);
        let repo = git2::build::RepoBuilder::new()
            .fetch_options(fo)
            .clone("https://github.com/MahiroJV/git-tree", &dir)
            .expect("clone through reqwest transport");

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        println!("HEAD = {} {}", head.id(), head.summary().unwrap_or(""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Proves requests go through this transport, not libgit2's built-in
    /// HTTPS: only our code produces the "repository not found" hint.
    #[test]
    #[ignore]
    fn missing_repo_error_comes_from_this_transport() {
        register();
        let dir =
            std::env::temp_dir().join(format!("git-tree-transport-404-{}", std::process::id()));
        let err = git2::Repository::clone(
            "https://github.com/MahiroJV/this-repo-does-not-exist-404",
            &dir,
        )
        .err()
        .expect("clone of a missing repo must fail");
        let _ = std::fs::remove_dir_all(&dir);
        println!("error = {err}");
        assert!(
            err.message().contains("repository not found"),
            "unexpected error: {err}"
        );
    }
}
