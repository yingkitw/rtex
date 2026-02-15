# latex-rs

A native TeX to PDF converter CLI written in Rust with **no external dependencies**.

## Features

- **Native TeX to PDF conversion** - No pdflatex or LaTeX installation required!
- Pure Rust implementation with built-in TeX parser
- Simple CLI interface
- Trait-based architecture for extensibility
- Comprehensive error handling
- Test-friendly design
- Supports common LaTeX features:
  - Document structure (sections, subsections)
  - Text formatting (bold, italic, monospace)
  - Lists (itemize, enumerate)
  - Mathematical equations (inline and display)
  - Title, author, and date metadata

## Text Output Quality Validation

Text elements are validated before writing to PDF content streams:

- Inline math safety: unmatched `$` is preserved as literal text (not dropped)
- Text normalization: collapses excessive whitespace and removes non-printable control chars
- Wrapping robustness: long tokens are split by Unicode character count to avoid overflow

This improves final PDF text stability for noisy or mixed TeX input.

## Mathematical Symbol Support

✅ **Unicode Math Symbols**: Successfully implemented using `lopdf` with UTF-16BE encoding and DejaVu Sans font embedding.

**Supported Features**:
- 150+ mathematical symbols (Greek letters, operators, relations, arrows)
- Inline and display math equations
- Special roots (∛ cube root, ∜ fourth root)
- Full alphabet super/subscripts
- Proper Unicode text encoding

**Status**: 
- ✅ Math formatter: 150+ LaTeX commands → Unicode symbols
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

Or after building:

```bash
./target/release/latex-rs input.tex
# Creates: output/input.pdf

./target/release/latex-rs input.tex -o custom_output.pdf
```

**Note:** The `output/` directory is created automatically if it doesn't exist.

## Architecture

The project follows KISS and DRY principles with a trait-based design:

- `TexConverter` trait: Defines the conversion interface
- `NativeTexConverter`: Pure Rust implementation with native TeX parser and PDF builder
- `TexParser`: Parses LaTeX syntax into structured elements
- `PdfBuilder`: Generates PDF documents from parsed elements
- `convert_tex_to_pdf`: Convenience function for direct conversion

See `ARCHITECTURE.md` for detailed design documentation.

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
- 13 comprehensive tests
- Native PDF generation verification
- File size validation
- PDF header verification
- Multiple document types (minimal, math, tables, lists, complex)
- Error handling tests
- Output directory creation tests

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
