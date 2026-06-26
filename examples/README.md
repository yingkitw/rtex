# Example TeX Files

This directory contains example TeX files for testing the rtex converter.

## Files

### `minimal.tex`
The simplest possible LaTeX document - just "Hello, World!"

### `sample.tex`
A comprehensive sample document with:
- Title, author, and date
- Multiple sections
- Itemized lists
- Mathematical equations (inline and display)

### `math.tex`
Mathematical equations showcase:
- Quadratic formula
- Calculus (fundamental theorem)
- Linear algebra (matrix operations)

### `table.tex`
Table examples using the booktabs package:
- Simple data table
- Programming languages comparison table

### `lists.tex`
List structures:
- Unordered lists (itemize)
- Ordered lists (enumerate)
- Nested lists

### `code.tex`
Code listings with syntax highlighting:
- Rust code example
- Python code example
- Uses the listings package

## Usage

Convert any example to PDF:

```bash
cargo run -- examples/minimal.tex
cargo run -- examples/math.tex -o output.pdf
```

## Testing

All examples are tested in the test suite to verify:
- PDF generation succeeds
- Output file exists
- PDF file has valid header
- File size is reasonable
- Content is properly formatted
