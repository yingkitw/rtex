# Architecture

## Overview

rtex is a **native** TeX to PDF converter CLI built with Rust, requiring **no external dependencies**. It follows clean architecture principles with a modular design.

## Design Principles

- **DRY**: No code duplication, reusable components
- **KISS**: Simple, straightforward implementation
- **SoC**: Clear separation between CLI, parsing, PDF generation, and error handling
- **Test-friendly**: Trait-based design enables easy mocking and testing
- **Self-contained**: No external LaTeX installation required

## Components

### Core Library (`src/lib.rs`)

#### Traits

- `TexConverter`: Defines the conversion interface
  - `convert(&self, input: &Path, output: &Path) -> Result<(), LatexError>`
  - Enables multiple converter implementations
  - Facilitates testing with mock converters

#### Implementations

- `NativeTexConverter`: Pure Rust implementation
  - Reads TeX files directly
  - Parses LaTeX syntax using custom parser
  - Generates PDF via the pdfrs engine (from crates.io)
  - No external process calls

### TeX Parser (`src/parser/`)

- `mod.rs`: Core parser struct, element types, and main dispatch
  - `TexParser`: Parses LaTeX source into structured elements
  - `TexElement`: Enum representing parsed LaTeX elements (Text, Command, Section, Math, Lists, Theorem, Table, CodeBlock, Image, ColoredText, Citation, Bibliography, Label/Ref/PageRef, Center, Quote, Abstract, LineBreak, FlushLeft, FlushRight, Footnote, Caption, TableOfContents, ListOfFigures, ListOfTables)
  - Environment parsing (itemize, enumerate, description, equation, align, gather, multline, cases, center, quote, abstract, figure, flushleft, flushright, minipage, displaymath, math, eqnarray, split, aligned, gathered, lstlisting, verbatim, tabular, table, thebibliography, theorem-like)
  - 27 special character commands (`\copyright`, `\pounds`, `\LaTeX`, `\AA`, `\ss`, etc.)
  - 40+ skip-commands silently consumed (`\setlength`, `\setcounter`, `\ignorespaces`, font declarations, dimension commands)
  - Counter formatting (`\value`, `\arabic`, `\roman`, `\alph`, `\the<counter>`)
  - `\\` line break with optional `[length]` and `*` variants
  - Plugin command/environment dispatch
- `commands.rs`: Backslash command handlers
  - Sections, text formatting, includegraphics, citations, labels/refs
  - Font size commands, page breaks, footnotes
  - `\hspace`/`\vspace`, `\mbox`/`\parbox`/`\makebox`, `\multicolumn`/`\cline`
  - `\footnotemark`/`\footnotetext`, `\textnormal`/`\enquote`
  - `\linebreak`/`\nopagebreak`/`\samepage`/`\enlargethispage`
- `math.rs`: Inline and display math delimiter parsing
- `text.rs`: Plain-text accumulation with inline-math preservation

### Math Processing (`src/math/`)

- `symbols.rs`: LaTeX-to-Unicode symbol mapping table (566+ symbols)
  - Greek letters, operators, relations, arrows, special symbols
- `scripts.rs`: Unicode superscript/subscript character conversion
- `radicals.rs`: Square-root formatting (`\sqrt{...}`)
- `fractions.rs`: Fraction formatting (`\frac{n}{d}`) with Unicode fallbacks
- `mathml.rs`: LaTeX math to presentation MathML converter
  - Recursive descent parser handling fractions, roots, scripts, accents, matrices, cases, Greek/special symbols
  - Emits `<math>` tags in HTML output alongside Unicode fallback for browser-incompatible renderers

- `MathFormatter` (`src/math_formatter.rs`): Orchestrates math formatting
  - Delegates to submodule handlers for radicals, fractions, scripts, and symbols

### PDF Generation (`src/pdf/`)

- `builder.rs`: Converts parsed elements to PDF
  - `PdfBuilder`: Page layout, text positioning, automatic page breaks
  - Font management (DejaVu Sans with Unicode support, subsetting)
  - Supports titles, sections, lists, math, images, tables
- `core.rs`: Low-level PDF generation primitives
  - `PdfGenerator`: Object management and PDF serialization
  - `DictBuilder`: PDF dictionary construction
  - `ContentStream`: PDF content stream operations
- `text_renderer.rs`: Text rendering utilities
  - Text normalization, character counting, word wrapping
- `font_subset.rs`: Font subsetting and dual-font selection
  - `requires_embedded_font` chooses standard Helvetica vs embedded DejaVu
  - Collects used Unicode characters from parsed document (formatted math)
  - Subsets TrueType fonts via the `font-subset` crate when Unicode is needed

### Error Handling (`src/error.rs`)

- `LatexError`: Comprehensive error type using `thiserror`
  - `ParseError`: Parsing errors with line/column context
  - `PdfError`: PDF generation failures
  - `IoError`: File I/O with path information
  - `FontError`, `MathError`, `ConfigError`, `UnsupportedFeature`, `InvalidPath`

### CLI (`src/main.rs`)

- Uses clap with derive macros for argument parsing
- Simple command structure:
  - Required: input file path
  - Optional: output file path (defaults to `output/<input_stem>.pdf`)
  - `--watch`: Poll for file changes and auto-rebuild (dependency-aware)
  - `--force`: Force rebuild even when source and dependencies are unchanged
  - `--no-incremental`: Disable incremental compilation and always rebuild
  - `--completions <shell>`: Generate shell completion script (bash, zsh, fish, elvish, powershell)
- Minimal error handling delegation to library

### Tests

- `src/tests.rs`: Unit and integration tests for conversion
- `src/example_tests.rs`: Example-based tests
- `tests/integration_test.rs`: Full workflow integration tests
- `tests/round_trip_test.rs`: PDF regression tests
- `tests/html_render_test.rs`: HTML rendering tests
- `tests/docx_epub_test.rs`: DOCX/EPUB structural tests
- Module-level tests in `error.rs`, `parser/tests.rs`, `math/symbols.rs`, `math/scripts.rs`, `watch.rs`, `streaming.rs`

## Dependencies

- `clap`: CLI argument parsing with derive macros
- `clap_complete`: Shell completion script generation (bash, zsh, fish, elvish, powershell)
- `thiserror`: Custom error types
- `chrono`: Date handling for `\today` command
- `serde` + **serde_json**: Serialization for intermediate artifacts
- `resvg` + **usvg** + **png**: SVG rasterization for `\includegraphics{*.svg}`
- `zip`: DOCX and EPUB packaging (ZIP archives)
- `ureq`: CTAN package downloading (`--fetch-packages`)
- `pdfrs` (from crates.io): Primary PDF layout/render engine

## Data Flow

```
User Input (CLI)
    ↓
Argument Parsing (clap)
    ↓
NativeTexConverter
    ↓
TexParser → Parse LaTeX → TexElement[]
    ↓
output::render_elements_in_dir
    ↓
PDF: pdfrs → stamp Producer rtex/pdfrs
HTML / DOCX / EPUB: format-specific renderers
    ↓
Output file
```

### PDF backend

| Backend | When used | Module |
|---------|-----------|--------|
| **pdfrs** | All PDF output | `src/output/pdfrs_pdf.rs` → crates.io |

Math policy on the pdfrs path: simple symbols → Unicode via `MathFormatter`; `\frac` / `\sqrt` → display math layout (stacked fractions, vinculum).

## Extension Points

The trait-based design allows for:
- Adding new LaTeX engines (XeLaTeX, LuaLaTeX)
- Custom preprocessing/postprocessing
- Alternative output formats
- Mock converters for testing

## WebAssembly

rtex targets `wasm32-unknown-unknown` for zero-infrastructure browser preview:

- **`convert_tex_string_to_pdf_bytes(tex)`** — parses and renders entirely in memory (no file I/O)
- **`src/fonts.rs`** — embeds DejaVu Sans at compile time via `include_bytes!`
- **`src/wasm.rs`** — optional `wasm` feature exports `convertTexToPdf` via `wasm-bindgen`
- **`pdfrs_pdf::render_pdf_bytes`** — renders parsed elements to PDF bytes entirely in memory

Build: `cargo build --target wasm32-unknown-unknown --features wasm --release`

## Multi-Format Output

The `src/output/` module renders the same parsed `TexElement` AST to multiple formats:

| Format | Module | Notes |
|--------|--------|-------|
| PDF | `output/pdfrs_pdf.rs` → crates.io | pdfrs only |
| HTML | `output/html.rs` | Semantic HTML with embedded CSS |
| DOCX | `output/docx.rs` | Minimal OOXML packaged as ZIP |
| EPUB | `output/epub.rs` | EPUB 3 package with XHTML chapter |

Public API: `render_elements(elements, OutputFormat)` and `convert_tex_file(path, output, &ConversionOptions)`.

## Package Fetching

The `src/packages/` module provides Tectonic-style on-demand fetching:

- `PackageFetcher::scan_dependencies(tex)` — finds `\documentclass`, `\usepackage`, `\RequirePackage`
- `PackageFetcher::ensure_packages(...)` — downloads missing `.cls`/`.sty` from CTAN mirrors into `.rtex/cache`
- Downloaded paths are added to `TexParser` search paths for `\input` resolution

## Intermediate Artifacts

When `--keep-intermediate` is set (or `ConversionOptions::keep_intermediate`), sibling files are written next to the output:

| File | Contents |
|------|----------|
| `{stem}.expanded.tex` | Source after `\newcommand`/`\def` expansion |
| `{stem}.ast.json` | Serialized `Vec<TexElement>` parse tree |
| `{stem}.meta.json` | Format, element count, title, author |

### Language Server (`src/lsp/`, feature `lsp`)

- `diagnostics.rs` — brace, environment, math, and document-structure checks
- `completion.rs` — command and environment completion candidates
- `symbols.rs` — section/label outline from parsed AST
- `hover.rs` — command documentation on hover
- `server.rs` — stdio LSP loop (`rtex-lsp` binary)

## File Organization

```
rtex/
├── Cargo.toml              # Dependencies and metadata (crate name: rtex)
├── src/
│   ├── lib.rs              # Core conversion logic & public API
│   ├── wasm.rs             # wasm-bindgen exports (feature: wasm)
│   ├── output/             # HTML, DOCX, EPUB renderers + pdfrs PDF
│   ├── packages/           # CTAN package scanning and fetching
│   ├── main.rs             # CLI entry point (binary: rtex)
│   ├── bin/rtex-lsp.rs     # LSP server entry (feature: lsp)
│   ├── lsp/                # LSP analysis + server (feature: lsp for server)
│   ├── error.rs            # Structured error types with Position tracking
│   ├── table.rs            # Table parsing and PDF rendering
│   ├── macros.rs           # Macro definition and expansion system
│   ├── math_formatter.rs   # Math formatting orchestrator
│   ├── parser/             # LaTeX parser implementation
│   │   ├── mod.rs          # TexParser, TexElement enum, environment dispatch
│   │   ├── tests.rs        # Parser unit tests
│   │   ├── text.rs         # Raw text accumulation
│   │   ├── math.rs         # Inline and display math delimiter parsing
│   │   └── commands.rs     # Backslash Command handlers (section, URL, rule, etc.)
│   ├── math/               # Math formatting submodules
│   │   ├── symbols.rs      # 566+ LaTeX-to-Unicode mappings
│   │   ├── scripts.rs      # Superscript/subscript Unicode conversion
│   │   ├── radicals.rs     # Square root formatting with Unicode
│   │   ├── fractions.rs    # Fraction → Unicode fraction or parenthesized form
│   │   └── mathml.rs       # LaTeX math → presentation MathML for HTML
│   ├── plugins.rs          # Plugin trait and built-in plugins
│   ├── cache.rs            # Document cache with TTL and LRU eviction
│   ├── incremental.rs     # Incremental compilation (source hash tracking)
│   ├── intermediate.rs     # Intermediate artifact writer
│   ├── streaming.rs       # Streaming converter with progress reporting
│   ├── watch.rs            # File watch mode for auto-rebuild
│   └── bin/                # Debug/test binaries (feature: dev-bins)
├── examples/               # Example TeX files
├── tests/
│   ├── integration_test.rs   # Integration tests
│   ├── round_trip_test.rs  # PDF regression tests
│   ├── html_render_test.rs # HTML rendering tests
│   └── docx_epub_test.rs  # DOCX/EPUB structural tests
├── output/                 # Generated PDFs (gitignored)
├── README.md               # User documentation
├── TODO.md                 # Task tracking
└── ARCHITECTURE.md         # This file
```
