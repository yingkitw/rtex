# rtex

A native TeX to PDF converter CLI written in Rust with **no external dependencies**.

## Features

- **Multiple output formats** — PDF (default), HTML, DOCX, EPUB from the same parsed AST
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
  - **201+ LaTeX commands supported**

## Text Output Quality Validation

Text elements are validated before writing to PDF content streams:

- Inline math safety: unmatched `$` is preserved as literal text (not dropped)
- Text normalization: collapses excessive whitespace and removes non-printable control chars
- Wrapping robustness: long tokens are split by Unicode character count to avoid overflow

This improves final PDF text stability for noisy or mixed TeX input.

## Mathematical Symbol Support

✅ **Unicode Math Symbols**: Successfully implemented using `lopdf` with UTF-16BE encoding and DejaVu Sans font embedding.

**Supported Features**:
- 566 mathematical symbols (Greek letters, operators, relations, arrows, integrals, summation)
- Inline and display math equations
- Special roots (∛ cube root, ∜ fourth root)
- Full alphabet super/subscripts
- Math alphabets: \mathbb, \mathcal, \mathfrak, \mathbf, \mathit, \mathsf, \mathtt
- Math accents: \vec, \hat, \tilde, \bar, \dot, \ddot
- Fractions with Unicode fraction characters and parenthesized form
- Proper Unicode text encoding

**Status**: 
- ✅ Math formatter: 566 LaTeX commands → Unicode symbols
- ✅ Font embedding: DejaVu Sans TrueType with full Unicode support
- ✅ PDF generation: lopdf with UTF-16BE encoding
- ⚠️ File size: ~750KB per PDF (due to embedded font)

**For production documents**, this native converter now provides good math support. For complex documents with advanced features (TikZ, complex tables, etc.), use pdflatex.

See `docs/MATH_LEARNINGS_FROM_MINITEX.md` for implementation details.

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
- `NativeTexConverter`: Pure Rust implementation with native TeX parser and PDF builder
- `TexParser`: Parses LaTeX syntax into structured elements
- `PdfBuilder`: Generates PDF documents from parsed elements
- `convert_tex_to_pdf`: Convenience function for direct conversion
- `convert_tex_string_to_pdf_bytes`: In-memory PDF conversion (WASM-ready)
- `convert_tex_string` / `convert_tex_file`: Multi-format conversion (PDF, HTML, DOCX, EPUB)
- `OutputFormat`, `PackageFetcher`: Format selection and CTAN package cache

See `ARCHITECTURE.md` for detailed design documentation.

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
- 317 comprehensive tests
- Native PDF generation verification
- File size validation
- PDF header verification
- Multiple document types (minimal, math, tables, lists, complex)
- Error handling tests
- Output directory creation tests
- Round-trip tests for deterministic conversion
- Math formatter tests (fractions, roots, symbols, accents, alphabets)
- Parser tests for 201+ LaTeX commands

All tests run without any external dependencies!

## Development

The codebase follows these principles:
- DRY (Don't Repeat Yourself)
- KISS (Keep It Simple, Stupid)
- SoC (Separation of Concerns)
- Test-friendly trait-based design
- Minimal public surface area

## License

MIT
