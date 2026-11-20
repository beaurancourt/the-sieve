# Contributing

Thanks for your interest in The Sieve! This is a small Rust project, and the notes below cover the basics.

## Setup

You need a stable Rust toolchain ([rustup.rs](https://rustup.rs/)). The CLI has no system dependencies — fonts and licenses are embedded at build time.

```sh
git clone https://github.com/beaurancourt/the-sieve.git
cd the-sieve
cargo build --release          # CLI at target/release/the-sieve
cargo run -- sample.md         # renders sample.pdf
```

`cargo run -- sample.md -o out.pdf` writes to a custom path, and `--html-only` emits the intermediate HTML for debugging.

The desktop app in `app/` is a Tauri 2 wrapper around the same library. It is not needed for most changes, but building it requires the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform.

## Tests

Tests are inline unit tests (`#[cfg(test)] mod tests`) next to the code they cover — there is no `tests/` directory. Run them all with:

```sh
cargo test
```

To run a single test, pass part of its name:

```sh
cargo test -p the-sieve license_directive
```

Add or update tests whenever you change parser or extension behavior. As a smoke test, `scripts/build-modules.sh` renders every module under `modules/` using the release binary.

## Code style

- Run `cargo fmt` before sending a change; the code uses standard rustfmt formatting.
- Run `cargo clippy --all-targets` and address warnings that your change introduces.
- Follow the existing pipeline: markdown → AST → PDF. A new extension usually touches `src/parser/extensions.rs` (detection), `src/ast.rs` (element variant), and `src/renderer/pdf.rs` (layout).
- Keep the build self-contained. Do not add runtime dependencies; fonts and license texts are embedded via `include_bytes!` / `include_str!`.
- Match the surrounding code: small functions, doc comments on public library items, and comments explaining intent rather than restating the code.

## Pull requests

Branch off `main` and keep each pull request focused on one change. In the description, include:

- What the change does and why, and the issue it closes (for example, `Fixes #123`).
- How you tested it — the commands you ran and anything you checked by eye in the rendered PDF.
- Screenshots of before/after pages for visual or layout changes.
- Updates to [`STYLE_GUIDE.md`](STYLE_GUIDE.md) when you add user-facing markdown syntax.

All tests should pass before you send the pull request. If you're unsure whether a change fits, open an issue first and we can talk it over.
