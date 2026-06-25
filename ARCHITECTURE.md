# Architecture

## Overview

latex-rs is a **native** TeX to PDF converter CLI built with Rust, requiring **no external dependencies**. It follows clean architecture principles with a modular design.

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

### TeX Parser (`src/parser.rs`)

- `TexParser`: Parses LaTeX source into structured elements
  - Handles document structure (sections, subsections)
  - Parses environments (itemize, enumerate, equation)
  - Extracts metadata (title, author, date)
  - Processes text formatting commands
  - Supports inline and display math

- `TexElement`: Enum representing parsed LaTeX elements
  - Text, Command, Environment
  - Section, Paragraph
  - MathInline, MathDisplay
  - ItemList (ordered/unordered)

### Math Processing (`src/math/`)

- `symbols.rs`: LaTeX-to-Unicode symbol mapping table (150+ symbols)
  - Greek letters, operators, relations, arrows, special symbols
- `scripts.rs`: Unicode superscript/subscript character conversion

- `MathFormatter` (`src/math_formatter.rs`): Orchestrates math formatting
  - Fractions, square roots, superscripts, subscripts, matrices

### PDF Generation (`src/pdf/` and `src/pdf_builder.rs`)

- `PdfBuilder` (`src/pdf_builder.rs`): Converts parsed elements to PDF
  - Page layout and text positioning
  - Automatic page breaks
  - Text wrapping
  - Font management (DejaVu Sans with Unicode support)
  - Supports titles, sections, lists, and math

- `pdf_core.rs`: Low-level PDF generation primitives
  - `PdfGenerator`: Object management and PDF serialization
  - `DictBuilder`: PDF dictionary construction
  - `ContentStream`: PDF content stream operations

- `pdf_text_renderer.rs`: Text rendering utilities
  - Text normalization, character counting, word wrapping

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
  - Optional: output file path (defaults to input with .pdf extension)
- Minimal error handling delegation to library

### Tests

- `src/tests.rs`: Unit and integration tests for conversion
- `src/example_tests.rs`: Example-based tests
- `tests/integration_test.rs`: Full workflow integration tests
- `tests/round_trip_test.rs`: Deterministic conversion and regression tests
- Module-level tests in `error.rs`, `config.rs`, `page_layout.rs`, `parser.rs`, `pdf_core.rs`, `pdf_text_renderer.rs`, `math/symbols.rs`, `math/scripts.rs`

## Dependencies

- **clap**: CLI argument parsing with derive macros
- **anyhow**: Error handling in main
- **thiserror**: Custom error types
- **chrono**: Date handling for `\today` command
- **tempfile**: Temporary directory management (for tests)
- **rusttype**: Font metrics (deprecated, may be removed)
- **flate2**: Compression support

## Data Flow

```
User Input (CLI)
    ↓
Argument Parsing (clap)
    ↓
Conversion Request
    ↓
NativeTexConverter
    ↓
TexParser → Parse LaTeX → TexElement[]
    ↓
PdfBuilder → Generate PDF → PDF Document
    ↓
PDF Output (saved to file)
```

## Extension Points

The trait-based design allows for:
- Adding new LaTeX engines (XeLaTeX, LuaLaTeX)
- Custom preprocessing/postprocessing
- Alternative output formats
- Mock converters for testing

## File Organization

```
latex-rs/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── lib.rs              # Core conversion logic & trait definitions
│   ├── main.rs             # CLI entry point
│   ├── parser.rs           # LaTeX parser implementation
│   ├── math_formatter.rs   # Math formatting orchestrator
│   ├── image.rs            # Image loading and PDF embedding
│   ├── table.rs            # Table parsing and PDF rendering
│   ├── color.rs            # Color parsing and PDF RGB color operators
│   ├── layout.rs           # Multi-page layout engine and page break management
│   ├── macros.rs           # User-defined macro expansion (\newcommand, \def)
│   ├── bibliography.rs     # BibTeX parsing and citation formatting
│   ├── references.rs       # Cross-reference engine (\label, \ref, \pageref)
│   ├── pdf_builder.rs      # PDF generation implementation
│   ├── pdf_core.rs         # Low-level PDF primitives
│   ├── pdf_text_renderer.rs # Text rendering utilities
│   ├── config.rs           # Configuration system
│   ├── error.rs            # Structured error types
│   ├── page_layout.rs      # Page layout and font helpers
│   ├── traits.rs           # Composable trait definitions
│   ├── utils.rs            # Shared utilities (brace extraction)
│   ├── math/
│   │   ├── mod.rs          # Math module re-exports
│   │   ├── symbols.rs      # LaTeX-to-Unicode symbol mapping
│   │   └── scripts.rs      # Superscript/subscript conversion
│   ├── pdf/
│   │   └── mod.rs          # PDF module re-exports
│   ├── tests.rs            # Unit/integration tests
│   ├── example_tests.rs    # Example-based tests
│   └── bin/                # Auxiliary binaries (benchmarks, debug scripts)
│       ├── bench.rs
│       └── debug_*.rs
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
