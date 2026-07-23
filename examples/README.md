# Example TeX Files

This directory contains example TeX files for testing the rtex converter.

## Files

### `minimal.tex`
The simplest possible LaTeX document — just "Hello, World!"

### `sample.tex`
A comprehensive sample document with title/author/date, sections, lists, and math.

### `math.tex`
Mathematical equations showcase (quadratic formula, calculus, matrices).

### `table.tex`
Table examples using the booktabs package.

### `lists.tex`
Unordered, ordered, and nested lists.

### `code.tex`
Code listings (Rust / Python) via the listings package.

### `simple_test.tex`
Minimal inline-math smoke test (`\alpha`, `\sum`, `\int`).

### `advanced_math.tex`
Greek letters, operators, roots (vinculum), scripts, and complex expressions.

### `features.tex`
Broader feature tour (formatting, href, description lists, display math).

### `quadratic.tex`
Focused quadratic-formula / fraction layout check.

## Usage

```bash
cargo run --bin rtex -- examples/minimal.tex
cargo run --bin rtex -- examples/advanced_math.tex -o output/advanced_math.pdf --force
```

Force a PDF backend:

```bash
RTEX_PDF_BACKEND=pdfrs cargo run --bin rtex -- examples/math.tex -o output/math.pdf --force
RTEX_PDF_BACKEND=native cargo run --bin rtex -- examples/math.tex -o output/math.pdf --force
```

## Testing

```bash
cargo test --test integration_test
cargo test --lib example_tests
```
