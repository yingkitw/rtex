# Examples

This directory contains various examples demonstrating the LaTeX-Rust processor capabilities.

## Directory Structure

### `latex_samples/`
Contains LaTeX document samples that can be processed by the LaTeX-Rust processor:
- `advanced_features.tex` - Complex document with tables, figures, and citations
- `basic_document.tex` - Simple document with sections and formatting
- `sample.tex` - Comprehensive example with various LaTeX features

### `rust_examples/`
Contains Rust code examples showing how to use the LaTeX-Rust library:
- `advanced_math_example.rs` - Mathematical expression processing
- `async_example.rs` - Asynchronous document processing
- `cache_example.rs` - Caching functionality demonstration
- `config_example.rs` - Configuration system usage
- `debug_tokens.rs` - Token debugging utilities
- `incremental_example.rs` - Incremental parsing features
- `plugin_example.rs` - Plugin system demonstration
- `streaming_example.rs` - Streaming parser usage

## Running Examples

### Processing LaTeX Samples
```bash
# Process a LaTeX sample to HTML
cargo run -- -i examples/latex_samples/sample.tex -o output.html

# Process to PDF
cargo run -- -i examples/latex_samples/basic_document.tex -o output.pdf --format pdf
```

### Running Rust Examples
```bash
# Run a specific example
cargo run --example streaming_example

# Run with verbose output
cargo run --example config_example -- --verbose
```

## Creating New Examples

When adding new examples:
1. Place LaTeX files in `latex_samples/`
2. Place Rust code examples in `rust_examples/`
3. Update this README with descriptions
4. Ensure examples are well-documented with comments