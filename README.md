# rtex

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](Cargo.toml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/)
[![Tests](https://img.shields.io/badge/tests-520%2B-brightgreen.svg)](TODO.md)
[![LaTeX commands](https://img.shields.io/badge/LaTeX%20commands-300%2B-informational.svg)](TODO.md)
[![Math symbols](https://img.shields.io/badge/math%20symbols-618%2B-informational.svg)](TODO.md)
[![Outputs](https://img.shields.io/badge/outputs-PDF%20%7C%20HTML%20%7C%20DOCX%20%7C%20EPUB-purple.svg)](#usage)
[![LSP](https://img.shields.io/badge/LSP-rtex--lsp-blueviolet.svg)](docs/LSP.md)
[![No TeX Live](https://img.shields.io/badge/TeX%20Live-not%20required-critical.svg)](#why-rtex)
[![Repository](https://img.shields.io/badge/github-yingkitw%2Frtex-181717?logo=github)](https://github.com/yingkitw/rtex)

A native TeX-to-document converter written in Rust. **No LaTeX installation required.**

Convert `.tex` to PDF, HTML, DOCX, or EPUB from a single parser. Ship as a CLI, library, WASM module, or language server.

## Why rtex?

Most LaTeX workflows assume a full TeX distribution — gigabytes of packages, slow cold starts, and fragile CI setups. rtex takes a different path: a self-contained Rust toolchain that understands common LaTeX and produces documents without `pdflatex`, `xelatex`, or TeX Live.

| | **rtex** | **TeX Live / pdflatex** | **Tectonic** | **Pandoc** | **Typst** |
|---|:---:|:---:|:---:|:---:|:---:|
| No TeX install | ✅ | ❌ | ✅ | ✅* | ✅ |
| PDF from LaTeX (pdfrs) | ✅ | ✅ | ✅ | ⚠️ via LaTeX | ❌ (own syntax) |
| HTML / DOCX / EPUB | ✅ | ❌ | ❌ | ✅ | ⚠️ export |
| LSP editor support | ✅ | ⚠️ third-party | ❌ | ❌ | ✅ |
| WASM / embeddable | ✅ | ❌ | ❌ | ❌ | ✅ |
| Incremental + watch | ✅ | ❌ | ✅ | ❌ | ✅ |
| CTAN package fetch | ✅ | manual | ✅ | ❌ | bundled |

\*Pandoc often delegates PDF to an external LaTeX engine.

**Choose rtex when you want:**

- **CI and servers without TeX Live** — one `cargo build`, no 4 GB install step
- **Multi-format output from one source** — same AST → PDF, HTML, DOCX, or EPUB
- **Editor integration** — `rtex-lsp` gives diagnostics, completions, outline, and hover
- **Fast iteration** — incremental compilation and `--watch` mode
- **Small, predictable artifacts** — ASCII-only PDFs can be under 1 KB; tests run with zero external tools
- **Embedding** — use as a Rust library, compile to WASM for browser preview, or run headless in pipelines

**Know the trade-off:** rtex is not a full TeX engine. Complex documents (TikZ, exotic packages, fine math spacing) may still need TeX Live. rtex targets everyday LaTeX — articles, reports, homework, notes — where speed, portability, and multi-format output matter more than 100% LaTeX compatibility.

See [docs/KNOWN_LIMITATIONS.md](docs/KNOWN_LIMITATIONS.md) for the full honesty list.

## Features

- **Multiple output formats** — PDF (default), HTML, DOCX, EPUB from the same parsed AST
- **Language Server (LSP)** — diagnostics, completions, outline, and hover for `.tex` editors (`rtex-lsp` binary)
- **On-demand package fetching** — download missing `.sty`/`.cls` files from CTAN mirrors
- Pure Rust implementation with built-in TeX parser
- Simple CLI interface
- Trait-based architecture for extensibility
- Comprehensive error handling
- Test-friendly design
- Supports common LaTeX features:
  - Document structure (\part, \chapter, \section, \subsection, \appendix)
  - Text formatting (\textbf, \textit, \texttt, \underline, \emph, \textsc, \sout, \overline)
  - Lists (itemize, enumerate with optional \item[label], description-style labels)
  - Mathematical equations (inline and display, 566+ Unicode symbols)
  - Math alphabets (\mathbb, \mathcal, \mathfrak, \mathbf, \mathit, \mathsf, \mathtt)
  - Math accents (\vec, \hat, \tilde, \bar, \dot, \ddot)
  - Tables (tabular with alignment, booktabs rules)
  - Environments (center, quote, quotation, abstract, equation, lstlisting)
  - Graphics (\includegraphics with PNG/JPEG/SVG)
  - References (\cite, \label, \ref, \pageref, \index, \glossary)
  - Bibliography (\thebibliography, \bibliography, \bibliographystyle)
  - PDF transformations (\rotatebox, \scalebox, \raisebox, \phantom)
  - Title, author, and date metadata
  - Table of contents, list of figures, list of tables
  - Font size commands, colors, alignment, page breaks, horizontal/vertical rules
  - **300+ LaTeX commands supported**
- Multi-line math (`align`, `align*`, `gather`, `gather*`, `multline`, `cases`) including cases nested inside `\[…\]`
- `description` environment for definition lists
- Theorem-like blocks (`theorem`, `lemma`, `proof`, `definition`, `corollary`, `proposition`, `remark`, `example`)
- `\href{url}{text}` hyperref-style links
- `\section*` / `\subsection*` / `\subsubsection*` unnumbered headings
- `\binom` / `\dbinom` / `\tbinom` binomial coefficients
- `\tfrac` / `\dfrac` sized fractions
- Ellipsis: `\ldots`, `\cdots`, `\vdots`, `\ddots`

## Math Support

PDF generation uses the vendored [pdfrs](https://crates.io/crates/pdfrs) engine with DejaVu Sans TrueType embedding for Unicode math symbols.

- **618 symbols** — Greek letters, operators, relations, arrows, integrals, summation, function names
- Inline and display math, multi-line environments (`align`, `gather`, `multline`, `cases`)
- Math alphabets (`\mathbb`, `\mathcal`, `\mathfrak`, `\mathbf`, `\mathit`, `\mathsf`, `\mathtt`)
- Math accents (`\vec`, `\hat`, `\tilde`, `\bar`, `\dot`, `\ddot`)
- Fractions, radicals (`\sqrt`), binomials (`\binom`), sized fractions (`\tfrac`, `\dfrac`)
- Matrix environments (`pmatrix`, `bmatrix`, `vmatrix`)
- MathML export in HTML output for accessible, selectable math
- Font strategy: standard Helvetica for ASCII-only docs (~750 bytes); embedded DejaVu subset for Unicode/math (~385 KB)

Text elements are validated before PDF output: unmatched `$` preserved as literal text, whitespace normalized, long tokens split by Unicode count to avoid overflow.

## Prerequisites

- Rust (edition 2024)
- **No LaTeX installation required!**

## Installation

```bash
cargo build --release
```

## Usage

Convert a TeX file to PDF (outputs to `output/` folder by default):

```bash
cargo run -- input.tex
# Creates: output/input.pdf
```

Specify custom output path:

```bash
cargo run -- input.tex -o custom_output.pdf
```

Incremental compilation skips unchanged sources (enabled by default). Force a rebuild or disable incremental mode:

```bash
cargo run -- input.tex --force
cargo run -- input.tex --no-incremental
```

Watch mode recompiles when the source or `\input` dependencies change:

```bash
cargo run -- input.tex --watch
```

Convert to HTML, DOCX, or EPUB from the same parsed AST:

```bash
cargo run -- input.tex --format html
cargo run -- input.tex --format docx -o report.docx
cargo run -- input.tex --format epub
```

Fetch missing LaTeX packages from CTAN before conversion:

```bash
cargo run -- input.tex --fetch-packages
cargo run -- input.tex --fetch-packages --package-cache ~/.cache/rtex/texmf
```

Keep intermediate build artifacts for debugging:

```bash
cargo run -- input.tex --keep-intermediate
# Also writes: output/input.expanded.tex, output/input.ast.json, output/input.meta.json
```

### Shell Completions

Generate completion scripts for your shell:

```bash
cargo run --bin rtex -- --completions bash > /etc/bash_completion.d/rtex
cargo run --bin rtex -- --completions zsh > ~/.zsh/completions/_rtex
cargo run --bin rtex -- --completions fish > ~/.config/fish/completions/rtex.fish
```

Supported shells: bash, zsh, fish, elvish, powershell.

### Language Server (editor integration)

Build and run the LSP server for VS Code, Cursor, or Neovim:

```bash
cargo build --release --features lsp
./target/release/rtex-lsp
```

See [docs/LSP.md](docs/LSP.md) for editor configuration.

Or after building:

```bash
./target/release/rtex input.tex
# Creates: output/input.pdf

./target/release/rtex input.tex -o custom_output.pdf
```

**Note:** The `output/` directory is created automatically if it doesn't exist.

## Architecture

The project follows KISS and DRY principles with a trait-based design:

- `TexConverter` trait: Defines the conversion interface
- `NativeTexConverter`: Pure Rust implementation with native TeX parser and pdfrs PDF engine
- `TexParser`: Parses LaTeX syntax into structured elements
- `pdfrs_pdf`: Generates PDF documents from parsed elements via the vendored pdfrs engine
- `convert_tex_to_pdf`: Convenience function for direct conversion
- `convert_tex_string_to_pdf_bytes`: In-memory PDF conversion (WASM-ready)
- `convert_tex_string` / `convert_tex_file`: Multi-format conversion (PDF, HTML, DOCX, EPUB)
- `OutputFormat`, `PackageFetcher`: Format selection and CTAN package cache

See [ARCHITECTURE.md](ARCHITECTURE.md) for design documentation and [docs/USER_GUIDE.md](docs/USER_GUIDE.md) for full usage. Documentation index: [docs/README.md](docs/README.md).

## WebAssembly

rtex compiles to `wasm32-unknown-unknown` for browser-side preview UIs with no server-side LaTeX installation.

```bash
# Install the WASM target (once)
rustup target add wasm32-unknown-unknown

# Build the library for WASM with JS bindings
cargo build --target wasm32-unknown-unknown --features wasm --release

# Generate JavaScript glue (requires wasm-bindgen-cli)
wasm-bindgen target/wasm32-unknown-unknown/release/rtex.wasm \
  --out-dir pkg --target web
```

Use `convert_tex_string_to_pdf_bytes(tex)` from Rust, or the exported `convertTexToPdf(tex)` function from JavaScript when built with the `wasm` feature.

## Examples

The `examples/` directory contains various TeX files for testing:

- `minimal.tex` - Simplest possible document
- `sample.tex` - Comprehensive sample with multiple features
- `math.tex` - Mathematical equations
- `table.tex` - Tables with booktabs
- `lists.tex` - Itemized and enumerated lists
- `code.tex` - Code listings with syntax highlighting

Try them:

```bash
# Generates output/minimal.pdf
cargo run -- examples/minimal.tex

# Generates my_math.pdf in current directory
cargo run -- examples/math.tex -o my_math.pdf

# Generate all examples at once
./generate_examples.sh
```

## Testing

Run tests:

```bash
cargo test
```

The test suite includes:
- 520+ comprehensive tests (520 passing, 3 ignored)
- PDF generation verification
- File size validation
- PDF header verification
- Multiple document types (minimal, math, tables, lists, complex)
- Error handling tests
- Output directory creation tests
- Round-trip tests for deterministic conversion
- Math formatter tests (fractions, roots, symbols, accents, alphabets)
- Parser tests for 300+ LaTeX commands

All tests run without any external dependencies!

## Development

The codebase follows these principles:
- DRY (Don't Repeat Yourself)
- KISS (Keep It Simple, Stupid)
- SoC (Separation of Concerns)
- Test-friendly trait-based design
- Minimal public surface area

## License

Apache-2.0 — see [Cargo.toml](Cargo.toml).
