pub mod auth;
pub(crate) mod export;
// Only registered on Android (see platform.rs); desktop uses it in tests.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub mod http_transport;
pub mod loader;
pub mod parser;
pub(crate) mod search;
pub mod url;
