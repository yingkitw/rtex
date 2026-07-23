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
  - Generates PDF using native PDF builder
  - No external process calls

### TeX Parser (`src/parser/`)

- `mod.rs`: Core parser struct, element types, and main dispatch
  - `TexParser`: Parses LaTeX source into structured elements
  - `TexElement`: Enum representing parsed LaTeX elements
  - Environment parsing (itemize, enumerate, equation, center, etc.)
  - Plugin command/environment dispatch
- `commands.rs`: Backslash command handlers
  - Sections, text formatting, includegraphics, citations, labels/refs
  - Font size commands, page breaks, footnotes
- `math.rs`: Inline and display math delimiter parsing
- `text.rs`: Plain-text accumulation with inline-math preservation

### Math Processing (`src/math/`)

- `symbols.rs`: LaTeX-to-Unicode symbol mapping table (150+ symbols)
  - Greek letters, operators, relations, arrows, special symbols
- `scripts.rs`: Unicode superscript/subscript character conversion
- `radicals.rs`: Square-root formatting (`\sqrt{...}`)
- `fractions.rs`: Fraction formatting (`\frac{n}{d}`) with Unicode fallbacks

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

### Configuration (`src/config.rs`)

- `Config`: Builder-pattern configuration struct
  - Quality presets (Draft, Standard, High, Print)
  - Font embedding options
  - Compression levels
  - Caching controls

### Page Layout (`src/page_layout.rs`)

- `PageLayout`: Page dimensions and margins
  - A4 and Letter size presets
  - Portrait and landscape orientations
  - Content area calculations
  - Font size helpers

### Error Handling (`src/error.rs`)

- `LatexError`: Comprehensive error type using `thiserror`
  - `ParseError`: Parsing errors with line/column context
  - `PdfError`: PDF generation failures
  - `IoError`: File I/O with path information
  - `FontError`, `MathError`, `ConfigError`, `UnsupportedFeature`, `InvalidPath`
- `ErrorContext` trait for adding contextual messages

### Traits (`src/traits.rs`)

- Atomic, composable trait definitions
- `TexParser`, `MathFormatter`, `PdfBuilder`, `FontProvider`
- `Cache`, `TextLayout`, `Validator`, `Transform`, `ResourceManager`

### CLI (`src/main.rs`)

- Uses clap with derive macros for argument parsing
- Simple command structure:
  - Required: input file path
  - Optional: output file path (defaults to `output/<input_stem>.pdf`)
  - `--watch`: Poll for file changes and auto-rebuild (dependency-aware)
  - `--template <path>`: Apply a TOML template for styling
  - `--force`: Force rebuild even when source and dependencies are unchanged
  - `--no-incremental`: Disable incremental compilation and always rebuild
- Minimal error handling delegation to library

### Tests

- `src/tests.rs`: Unit and integration tests for conversion
- `src/example_tests.rs`: Example-based tests
- `tests/integration_test.rs`: Full workflow integration tests
- `tests/round_trip_test.rs`: Deterministic conversion and regression tests
- Module-level tests in `error.rs`, `config.rs`, `page_layout.rs`, `parser/mod.rs`, `pdf/core.rs`, `pdf/text_renderer.rs`, `math/symbols.rs`, `math/scripts.rs`, `watch.rs`

## Dependencies

- **clap**: CLI argument parsing with derive macros
- **anyhow**: Error handling in main
- **thiserror**: Custom error types
- **chrono**: Date handling for `\today` command
- **tempfile**: Temporary directory management (for tests)
- **image**: PNG/JPEG/SVG decoding for `\includegraphics` (SVG rasterized via resvg)
- **font_subset**: Character collection, subsetting, and `requires_embedded_font` heuristic for dual-font strategy (Helvetica vs DejaVu)
- **flate2**: Compression support
- **serde** + **serde_json** + **toml**: Template serialization
- **num_cpus**: Parallel worker pool sizing
- **pdfrs** (vendored at `vendor/pdfrs`): Primary PDF layout/render engine

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
PDF: pdfrs (primary) → stamp Producer rtex/pdfrs
         ↓ on error / oversize / RTEX_PDF_BACKEND=native
     native PdfBuilder → Producer rtex/native
HTML / DOCX / EPUB: format-specific renderers
    ↓
Output file
```

### PDF backends

| Backend | When used | Module |
|---------|-----------|--------|
| **pdfrs** (primary) | Default for documents under size/element limits | `src/output/pdfrs_pdf.rs` → `vendor/pdfrs` |
| **native** (fallback) | pdfrs error, output > 5 MB, or `RTEX_PDF_BACKEND=native` | `src/pdf/builder.rs` |

Force a backend for tests/CI: `RTEX_PDF_BACKEND=pdfrs` or `RTEX_PDF_BACKEND=native` (fails if that backend cannot produce output).

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
- **`PdfBuilder::build_to_bytes`** — returns raw PDF bytes; `build()` writes them to disk on native targets

Build: `cargo build --target wasm32-unknown-unknown --features wasm --release`

## Multi-Format Output

The `src/output/` module renders the same parsed `TexElement` AST to multiple formats:

| Format | Module | Notes |
|--------|--------|-------|
| PDF | `output/pdfrs_pdf.rs` + `pdf/builder.rs` | pdfrs primary; native fallback |
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

### TeX Primitives (`src/tex/`)

- `catcodes.rs`, `tokens.rs`, `dimensions.rs` — lexical foundation
- `glue.rs` — `Glue::parse`, infinite `fil`/`fill`/`filll`, `hfill` preset
- `boxes.rs` — `TeXBox` width/height/depth for hbox/vbox layout
- `linebreak.rs` — Knuth–Plass DP line breaking; used by PDF `wrap_text_by_width`

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
│   ├── fonts.rs            # Embedded DejaVu Sans font data
│   ├── output/             # HTML, DOCX, EPUB renderers
│   ├── packages/           # CTAN package scanning and fetching
│   ├── main.rs             # CLI entry point (binary: rtex)
│   ├── bin/rtex-lsp.rs     # LSP server entry (feature: lsp)
│   ├── lsp/                # LSP analysis + server (feature: lsp for server)
│   ├── error.rs            # Structured error types with Position tracking
│   ├── config.rs           # Configuration system with quality presets
│   ├── common.rs           # Shared traits (Clear, Stats)
│   ├── color.rs            # Color struct with RGB and named color parsing
│   ├── table.rs            # Table parsing and PDF rendering
│   ├── macros.rs           # Macro definition and expansion system
│   ├── layout.rs           # LayoutState for text alignment and indentation
│   ├── page_layout.rs      # Page dimensions, margins, orientation helpers
│   ├── math_formatter.rs   # Math formatting orchestrator
│   ├── parser/             # LaTeX parser implementation
│   │   ├── mod.rs          # TexParser, TexElement enum, environment dispatch
│   │   ├── text.rs         # Raw text accumulation
│   │   ├── math.rs         # Inline and display math delimiter parsing
│   │   ├── commands.rs     # Backslash command handlers (section, URL, rule, etc.)
│   │   └── plugin.rs       # Plugin trait for extensible commands
│   ├── math/               # Math formatting submodules
│   │   ├── symbols.rs      # 566+ LaTeX-to-Unicode mappings
│   │   ├── scripts.rs      # Superscript/subscript Unicode conversion
│   │   ├── radicals.rs     # Square root formatting with Unicode
│   │   └── fractions.rs    # Fraction → Unicode fraction or parenthesized form
│   ├── pdf/                # PDF generation modules
│   │   ├── mod.rs          # PDF module re-exports
│   │   ├── core.rs         # ContentStream, PdfGenerator, PdfObj, HEX_TABLE
│   │   ├── builder.rs      # PdfBuilder, LayoutState, text block rendering
│   │   ├── text_renderer.rs # Text normalization and wrapping utilities
│   │   └── font_subset.rs  # Font subsetting to only used characters
│   ├── plugins/            # Built-in plugin implementations
│   │   ├── text_commands.rs  # Text formatting command plugins
│   │   └── math_commands.rs  # Math command plugins
│   └── bin/                # Debug/test binaries
│       ├── debug_parser.rs
│       ├── debug_font.rs
│       ├── debug_sample.rs
│       ├── debug_math.rs
│       ├── debug_texttt.rs
│       └── bench.rs
├── examples/               # Example TeX files
│   ├── minimal.tex
│   ├── sample.tex
│   ├── math.tex
│   ├── table.tex
│   ├── lists.tex
│   └── code.tex
├── tests/
│   ├── fixtures/             # Test fixtures
│   ├── integration_test.rs   # Integration tests
│   └── round_trip_test.rs  # Regression tests
├── output/                 # Generated PDFs (gitignored)
├── README.md               # User documentation
├── SETUP.md                # Setup guide
├── TODO.md                 # Task tracking
├── ARCHITECTURE.md         # This file
└── generate_examples.sh    # Helper script
```
