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
  - `convert(&self, input: &Path, output: &Path) -> Result<(), TexError>`
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

### PDF Builder (`src/pdf_builder.rs`)

- `PdfBuilder`: Converts parsed elements to PDF
  - Uses printpdf crate for PDF generation
  - Handles page layout and text positioning
  - Automatic page breaks
  - Text wrapping
  - Font management (Helvetica, HelveticaBold)
  - Supports titles, sections, lists, and math

#### Error Handling

- `TexError`: Comprehensive error type using thiserror
  - `ReadError`: File I/O errors
  - `CompilationError`: LaTeX compilation failures
  - `OutputNotFound`: Missing output after compilation
  - `InvalidPath`: Path validation errors

#### Public API

- `convert_tex_to_pdf`: Convenience function for direct conversion

### CLI (`src/main.rs`)

- Uses clap with derive macros for argument parsing
- Simple command structure:
  - Required: input file path
  - Optional: output file path (defaults to input with .pdf extension)
- Minimal error handling delegation to library

### Tests (`src/tests.rs`)

- Unit tests for converter creation
- Integration tests for actual conversion
- Error case testing (invalid paths, syntax errors)
- Uses tempfile for isolated test environments

## Dependencies

- **clap**: CLI argument parsing with derive macros
- **anyhow**: Error handling in main
- **thiserror**: Custom error types
- **printpdf**: Native PDF generation
- **chrono**: Date handling for `\today` command
- **tempfile**: Temporary directory management (for tests)

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
│   ├── parser.rs           # LaTeX parser implementation
│   ├── pdf_builder.rs      # PDF generation implementation
│   ├── main.rs             # CLI entry point
│   └── tests.rs            # Test suite
├── examples/               # Example TeX files
│   ├── minimal.tex
│   ├── sample.tex
│   ├── math.tex
│   ├── table.tex
│   ├── lists.tex
│   └── code.tex
├── output/                 # Generated PDFs (gitignored)
├── tests/
│   └── integration_test.rs # Integration tests
├── README.md               # User documentation
├── SETUP.md                # Setup guide
├── TODO.md                 # Task tracking
├── ARCHITECTURE.md         # This file
└── generate_examples.sh    # Helper script
```
