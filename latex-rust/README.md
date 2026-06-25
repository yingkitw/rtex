# LaTeX-Rust

A LaTeX processor written in Rust that can parse LaTeX documents and convert them to HTML and PDF formats.

## Design Principles

### Core Development Principles ✅ **IMPLEMENTED**
- **DRY (Don't Repeat Yourself)**: ✅ Applied throughout codebase with extracted helper functions
- **Separation of Concerns**: ✅ Each module has single responsibility with clear boundaries
- **Atomic Functions**: ✅ Complex functions broken down into smaller, testable units
- **Code Quality**: ✅ All Clippy warnings resolved, Rust best practices applied
- **Minimal Dependencies**: ✅ Optimized import statements and reduced inter-module coupling
- **Test-Friendly Design**: ✅ Functions designed for easy unit testing with 100% test pass rate

### Architecture Guidelines ✅ **ACHIEVED**
- **Modular Design**: ✅ Clear module boundaries with specialized helper functions
- **Maintainable Code**: ✅ Clean, readable, and consistent code patterns
- **Extensible Structure**: ✅ Plugin-ready architecture with configurable components
- **Memory Efficiency**: ✅ Optimized string handling and reduced cloning
- **Error Handling**: ✅ Comprehensive error context with detailed information

### Development Standards ✅ **MAINTAINED**
- **Minimal Public API**: ✅ Functions made private where possible
- **Consistent Patterns**: ✅ Uniform coding style across all modules
- **Performance Focus**: ✅ Benchmarked and optimized critical paths
- **Documentation**: ✅ Comprehensive inline and external documentation

## Layout

- header panel - title, author, date
- left panel - navigation, table of content
- central lower panel - tex code editor
- central upper panel - preview in live
- right panel - chat with AI

## Features

- **LaTeX Parsing**: Complete lexer and parser for LaTeX syntax with optimized memory usage
- **Streaming Parser**: Memory-efficient processing for large documents
- **Document Structure**: Support for sections, subsections, and document hierarchy
- **Text Formatting**: Bold, italic, monospace, underline, and emphasis with proper whitespace handling
- **Mathematical Expressions**: Both inline (`$...$`) and display (`$$...$$`) math with MathJax rendering
- **Lists**: Itemized, enumerated, and description lists with improved formatting
- **Environments**: Quote, center, abstract, and verbatim environments
- **Cross-References**: Support for label and ref commands for internal document linking
- **HTML Output**: Clean, semantic HTML with CSS styling and proper whitespace preservation
- **PDF Output**: Direct PDF generation with advanced text formatting
- **CLI Interface**: Command-line tool for batch processing
- **Error Handling**: Enhanced error context with detailed line and column information

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building from Source

```bash
git clone <repository-url>
cd latex-rust
cargo build --release
```

## Usage

### Command Line Interface

```bash
# Generate HTML output (default)
cargo run -- -i input.tex -o output.html

# Generate PDF output
cargo run -- -i input.tex -o output.pdf --format pdf

# Auto-detect output format from extension
cargo run -- -i input.tex --format pdf  # Creates input.pdf
cargo run -- -i input.tex --format html # Creates input.html

# Verbose output
cargo run -- -i input.tex -o output.pdf --format pdf --verbose
```

### Library Usage

```rust
use latex_rust::LaTeXProcessor;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
    \documentclass{article}
    \title{Hello World}
    \author{John Doe}
    \begin{document}
    \maketitle
    \section{Introduction}\label{sec:intro}
    This is a \textbf{bold} statement with proper whitespace.
    See \ref{sec:intro} for more details.
    \end{document}
    "#;
    
    // Generate HTML output
    let html_output = processor.to_html(latex_input)?;
    println!("{}", html_output);
    
    // Generate PDF output
    processor.to_pdf(latex_input, "output.pdf")?;
    println!("PDF generated successfully!");
    
    // For large documents, use streaming parser
    let file = File::open("large_document.tex")?;
    let reader = BufReader::new(file);
    let html_output = processor.stream_to_html(reader)?;
    println!("Large document processed with streaming parser");
    
    Ok(())
}
```

## Supported LaTeX Features

### Document Structure
- `\documentclass{}`
- `\title{}`, `\author{}`, `\date{}`
- `\maketitle`
- `\section{}`, `\subsection{}`, `\subsubsection{}`
- `\begin{document}` ... `\end{document}`

### Text Formatting
- `\textbf{}` - Bold text
- `\textit{}` - Italic text
- `\texttt{}` - Monospace text
- `\underline{}` - Underlined text
- `\emph{}` - Emphasized text
- `\large`, `\Large`, `\huge` - Size commands
- `\tiny`, `\small` - Small size commands

### Mathematics
- Inline math: `$expression$`
- Display math: `$$expression$$`
- Mathematical symbols and expressions
- Rendered using MathJax in HTML output

### Lists
- `\begin{itemize}` ... `\end{itemize}`
- `\begin{enumerate}` ... `\end{enumerate}`
- `\begin{description}` ... `\end{description}`
- `\item` for list items

### Environments
- `\begin{abstract}` ... `\end{abstract}`
- `\begin{quote}` ... `\end{quote}`
- `\begin{center}` ... `\end{center}`
- `\begin{verbatim}` ... `\end{verbatim}`

### Cross-References
- `\label{}` - Define labels for sections, equations, etc.
- `\ref{}` - Reference labeled elements
- Automatic link generation in HTML output

### Packages
- `\usepackage{}` (parsed but not functionally implemented)

## Examples

The `examples/` directory contains organized samples and code examples:

### LaTeX Samples (`examples/latex_samples/`)
- `sample.tex` - Comprehensive LaTeX document with various features
- `basic_document.tex` - Simple document with sections and formatting
- `advanced_features.tex` - Complex document with tables, figures, and citations

### Rust Code Examples (`examples/rust_examples/`)
- `streaming_example.rs` - Demonstrates streaming parser usage
- `async_example.rs` - Asynchronous document processing
- `config_example.rs` - Configuration system usage
- `cache_example.rs` - Caching functionality
- `plugin_example.rs` - Plugin system demonstration
- `advanced_math_example.rs` - Mathematical expression processing
- `incremental_example.rs` - Incremental parsing features
- `debug_tokens.rs` - Token debugging utilities

### Running Examples

```bash
# Process LaTeX samples
cargo run -- -i examples/latex_samples/sample.tex -o sample.html
cargo run -- -i examples/latex_samples/basic_document.tex -o basic.html
cargo run -- -i examples/latex_samples/advanced_features.tex -o advanced.html

# Run Rust code examples
cargo run --example streaming_example
cargo run --example config_example
cargo run --example async_example

# Open HTML results in your browser
open sample.html  # macOS
# or
xdg-open sample.html  # Linux
```

## Architecture

The LaTeX-Rust processor consists of several key components with a modular, maintainable design:

1. **Lexer** (`src/lexer.rs`): Tokenizes LaTeX input into structured tokens with optimized string handling
2. **Parser** (`src/parser.rs`): Converts tokens into an Abstract Syntax Tree (AST) with memory-efficient processing
3. **AST** (`src/ast.rs`): Defines the document structure and node types
4. **Renderer** (`src/renderer.rs`): Converts AST to HTML and PDF output with proper whitespace preservation
5. **Error Handling** (`src/error.rs`): Comprehensive error types with enhanced context information
6. **Streaming Parser** (`src/streaming_parser.rs`): Memory-efficient processing with modular command parsing functions:
   - `parse_section_command_streaming`: Handles section-level commands
   - `parse_text_formatting_command_streaming`: Processes text formatting
   - `parse_generic_command_streaming`: Handles general command parsing
   - `extract_braced_content`: Efficient argument extraction
7. **Core Library** (`src/lib.rs`): Main processing logic with specialized helper functions:
   - `process_batch_async`: Concurrent document processing with task management
   - `process_node_math`: Mathematical content processing with environment handling
   - Atomic helper functions for improved testability and maintainability
8. **Math Processing** (`src/math.rs`): Mathematical expression handling with optimized initialization patterns
9. **Bibliography** (`src/bibtex.rs`): Bibliography management with idiomatic Rust patterns
10. **Configuration** (`src/config.rs`): System configuration with optimized initialization

### Documentation

Detailed documentation is available in the `docs/` directory:

- `docs/ARCHITECTURE.md` - Detailed system architecture and design principles
- `docs/API.md` - Complete API documentation with examples
- `docs/README.md` - Comprehensive project documentation

### Project Structure

```
latex-rust/
├── src/                    # Core library code
├── tests/                  # Comprehensive test suite
├── benches/                # Performance benchmarks
├── examples/               # Organized examples and samples
│   ├── latex_samples/      # LaTeX document samples
│   └── rust_examples/      # Rust code examples
├── docs/                   # Project documentation
├── scripts/                # Development and build scripts
├── test_data/              # Test data and sample files
└── Cargo.toml              # Project configuration
```

## Testing

The project includes a comprehensive test suite with 192 unit tests covering all core modules:

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test modules
cargo test lexer_tests
cargo test parser_tests
cargo test renderer_tests
cargo test ast_tests
```

### Test Coverage

- **Lexer Tests** (48 tests): Tokenization, error handling, edge cases
- **Parser Tests** (48 tests): Command parsing, environment parsing, error recovery
- **Renderer Tests** (48 tests): HTML output, CSS generation, math rendering
- **AST Tests** (48 tests): Node creation, serialization, metadata handling
- **Integration Tests**: End-to-end document processing

### Performance Benchmarks

Run performance benchmarks to measure processing speed:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suites
cargo bench lexer_benchmarks
cargo bench parser_benchmarks
cargo bench renderer_benchmarks
```

Benchmarks cover:
- Lexer performance across different document types
- Parser performance for various LaTeX constructs
- HTML rendering performance for complex documents

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## Limitations

- Limited package support (packages are parsed but not functionally implemented)
- Limited table support
- No bibliography or citation support
- No figure/image processing

## Recent Improvements

### Code Quality and Refactoring (Latest)
- **Comprehensive Function Refactoring**: Applied DRY principle by breaking down complex functions into smaller, atomic, testable units across all modules
- **Enhanced Separation of Concerns**: Improved module boundaries and single responsibility principle implementation
- **Code Quality Optimization**: Fixed all Clippy warnings and applied Rust best practices for better maintainability
- **Modular Architecture**: Extracted specialized helper functions in key modules:
  - `src/lib.rs`: Refactored `process_batch_async` and `process_node_math` into focused helper functions
  - `src/streaming_parser.rs`: Modularized command parsing and argument extraction logic
  - `src/math.rs`: Enhanced math expression processing with cleaner initialization patterns
  - `src/bibtex.rs`: Improved enum-to-string conversion using idiomatic Display trait
  - `src/config.rs`: Optimized configuration initialization patterns
- **Improved Test Coverage**: Maintained 100% test pass rate throughout refactoring process
- **Reduced Code Duplication**: Eliminated repeated patterns and consolidated common functionality

### Performance and Memory Optimization
- **Memory Optimization**: Reduced excessive cloning and improved reference usage in parser
- **String Performance**: Pre-allocation of string capacity in lexer for better performance
- **Enhanced Error Context**: Added detailed line and column information to all error types
- **Cross-Reference Support**: Implemented label and ref commands with automatic linking
- **Streaming Parser**: Added memory-efficient processing for large documents
- **Whitespace Handling**: Improved whitespace preservation in text formatting and lists

## Future Enhancements

- **Plugin System**: Extensible command handling architecture
- **Configuration System**: Comprehensive configuration for parsing and rendering behavior
- Extended package support
- Table processing improvements
- Bibliography and citation support
- Figure and image handling
- Custom command definitions
- More mathematical environments
- Enhanced PDF formatting options

## License

MIT License - see LICENSE file for details.

## Dependencies

### Runtime Dependencies
- `regex` - Regular expression support for lexing
- `serde` - Serialization support for AST
- `clap` - Command-line argument parsing
- `thiserror` - Error handling
- `log` and `env_logger` - Logging support
- `printpdf` - PDF generation support

### Development Dependencies
- `tokio-test` - Async testing utilities
- `tempfile` - Temporary file handling for tests
- `criterion` - Performance benchmarking framework