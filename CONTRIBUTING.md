# Contributing to git-tree

Thanks for taking time to contribute. This doc keep it short.

## Before you start

- Check [existing issues](https://github.com/MahiroJV/git-tree/issues) first. Maybe someone already on it.
- Big change (new feature, big refactor)? Open an issue first. Talk before code. Save your time.

## Setup

```bash
git clone https://github.com/MahiroJV/git-tree.git
cd git-tree
cargo build
```

### Linux system deps

```bash
sudo apt-get install -y \
  libgtk-3-dev libglib2.0-dev libgdk-pixbuf-2.0-dev \
  libcairo2-dev libpango1.0-dev libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev libsoup-3.0-dev \
  libxdo-dev libgit2-dev pkg-config
```

### Windows

Needs Perl + NASM for OpenSSL vendored build. See `.github/workflows/test.yml` for exact setup if stuck.

## Run it

```bash
cargo run
```

## Before you open a PR

CI check three thing. Run them local first, save round trips:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --all-features
```

- `cargo fmt` — format code. CI reject unformatted code.
- `cargo clippy -- -D warnings` — CI deny all warning. Fix them, don't suppress with `#[allow]` unless real reason.
- `cargo test` — run test suite.

## PR guideline

- One PR, one purpose. Small PR review faster.
- Write clear commit message. What change, why.
- Link related issue in PR description if one exist.
- Describe what you tested (OS, steps).

## Code style

- Follow existing style in file you touch. Match it.
- Rust 2021 edition, stable toolchain.
- Keep comment short and useful, not restate the code.

## Reporting bug

Open an issue with:
- Steps to reproduce
- Expected vs actual behavior
- OS and git-tree version
- Screenshot if UI issue

## Security issue

Do **not** open public issue for security bug. See [SECURITY.md](SECURITY.md).

## License

By contributing, you agree your contribution license under MIT, same as project.
