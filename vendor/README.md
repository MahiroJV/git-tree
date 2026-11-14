# Vendored crates

## ashpd 0.8.1

Pulled in by `dioxus-desktop 0.6.3 → rfd 0.14 (xdg-portal)`; we can't turn
that feature off. Upstream ashpd 0.8.1 doesn't compile on Rust ≥ 1.100
(never-type fallback change, rust-lang/rust#148922), and there will be no
0.8.x fix — newer ashpd versions have a different API rfd 0.14 can't use.

Changes from the crates.io source:
- `src/desktop/clipboard.rs` (`set_selection`): `call(...)` → `call::<()>(...)`
  — the actual fix.
- `src/lib.rs`: `#![allow(warnings, clippy::all)]` — path dependencies don't
  get cargo's lint cap, and CI runs clippy with `-D warnings`. Wired in with
`[patch.crates-io]` in the root `Cargo.toml`.

Remove this directory and the patch entry once the project moves to
Dioxus 0.7+ (which uses a newer rfd/ashpd).
