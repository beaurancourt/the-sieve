# The Sieve

A CLI tool that converts TTRPG-flavored markdown into half-letter (5.5" × 8.5") PDFs sized for booklet printing.

## Features

- **Stat blocks** via fenced code blocks (`` ```statblock ``)
- **Boxed read-aloud text** (`` ```boxed ``)
- **Multi-column layouts** (`` ```columns `` with `---` separators)
- **Manual page breaks** (`<!-- pagebreak -->`)
- Standard markdown: headings, lists, tables, images, emphasis

See [`STYLE_GUIDE.md`](STYLE_GUIDE.md) for the full set of supported features and `sample.md` for a minimal example.

## Installation

```sh
cargo build --release
# binary lands at target/release/the-sieve
```

The default PDF pipeline shells out to [WeasyPrint](https://weasyprint.org/), which must be installed separately:

```sh
brew install weasyprint   # macOS
pipx install weasyprint   # any platform with pipx
```

## Usage

```sh
the-sieve adventure.md                 # → adventure.pdf
the-sieve adventure.md -o booklet.pdf  # custom output path
the-sieve adventure.md --html-only     # emit intermediate HTML
```

## How it works

`markdown → AST → HTML → WeasyPrint → PDF`. WeasyPrint was chosen for its strong multi-column text balancing, which matters for half-letter booklet layouts.

## License

MIT — see [`LICENSE`](LICENSE).
