# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust CLI that converts TTRPG-flavored markdown into half-letter (5.5" × 8.5") PDFs sized for booklet printing. The binary is `the-sieve`; the library crate name is `the_sieve`.

## Commands

```sh
cargo build --release            # binary at target/release/the-sieve
cargo test                       # all unit tests (inline #[test] modules; no tests/ dir)
cargo test -p the-sieve <name>   # run a single test by name substring
cargo run -- <INPUT.md>          # convert a markdown file to PDF
```

CLI flags (see `src/cli.rs`): `-o OUTPUT`, `-v`, `--html-only` (emit intermediate HTML).

The PDF pipeline shells out to `weasyprint`, which must be on `PATH` (`brew install weasyprint` on macOS). Without it, only `--html-only` works.

## Architecture

The pipeline is **markdown → AST → HTML → PDF**:

1. `src/parser/markdown.rs` walks `pulldown-cmark` events into an intermediate `Document` (see `src/ast.rs`).
2. `src/renderer/html.rs` emits HTML with embedded CSS for half-letter geometry, two-column flow, and TTRPG styling (stat blocks, boxed text, license appendix). It then spawns `weasyprint` as a subprocess to produce the PDF — WeasyPrint was chosen because it balances multi-column text well.

Library entry points (`convert_markdown_to_pdf`, `convert_markdown_to_html`, `parse_markdown`, `compile_html_to_pdf`) live in `src/lib.rs`; `src/main.rs` is a thin wrapper that picks an output path based on flags.

### Parser extensions

TTRPG-specific syntax lives in `src/parser/extensions.rs`:

- `<!-- pagebreak -->` HTML comments → `Element::PageBreak`
- `<!-- license: ogl-1.0a -->` and `<!-- license: cc-by-sa-4.0 -->` → `Element::License`. CC-BY-SA accepts optional `attribution="..."` and `changes="..."` parameters that render above the body.
- Fenced code blocks with language tags `statblock` / `stat-block` / `monster` → `Element::StatBlock` (shaded box)
- Fenced code blocks with `boxed` / `read-aloud` / `readaloud` → `Element::BoxedText`
- `<!-- 1-column -->` / `<!-- 2-column -->` HTML comments switch the page layout (the default is two-column)

When adding a new extension, the typical change touches three layers: detect it in `extensions.rs`, add an `Element` variant in `ast.rs`, and render it in `renderer/html.rs`.

### Statblock / boxed-text content

Inside a `statblock` or `boxed` fence, single newlines are soft (joined with a space, like a markdown paragraph); a blank line emits a hard `<br>`. Lines starting with `### ` / `#### ` are rendered as bold sub-headings on their own visual line.

### Licenses

`src/licenses.rs` embeds the canonical OGL 1.0a and CC-BY-SA 4.0 texts via `include_str!` from `licenses/*.txt`. The body is parsed into setext-heading and paragraph fragments so that the source files' visual underlines (`====` / `----`) become real headings instead of literal characters in the output.

### Half-letter output

Page geometry (5.5" × 8.5", two-column, narrow margins) is hardcoded in the HTML CSS — the format is the project's identity, not a parameter.
