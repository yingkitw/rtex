# LaTeX-Rust Documentation

This directory contains comprehensive documentation for the LaTeX-Rust project.

## Documentation Files

### [ARCHITECTURE.md](ARCHITECTURE.md)
Detailed architecture overview of the LaTeX-Rust processor, including:
- Component descriptions (Lexer, Parser, AST, Renderer)
- Data flow and design principles
- Testing strategy and future enhancements

### [API.md](API.md)
Complete API documentation covering:
- Core API usage examples
- Lexer, Parser, and Renderer APIs
- AST types and structures
- Error handling patterns
- Configuration options
- Utility functions

## Quick Start

For a quick introduction to using LaTeX-Rust, see the main [README.md](../README.md) in the project root.

## Examples

Practical examples can be found in the [examples/](../examples/) directory:
- `basic_document.tex` - Simple document structure
- `advanced_features.tex` - Complex features like tables, figures, citations
- `sample.tex` - Comprehensive feature demonstration

## Testing

The project includes comprehensive test suites:
- Unit tests in `tests/` directory
- Integration tests for end-to-end functionality
- Benchmark tests in `benches/` directory

## Performance

Benchmark tests are available for performance-critical operations:
- Lexer benchmarks: Token generation performance
- Parser benchmarks: AST construction performance
- Renderer benchmarks: Output generation performance

Run benchmarks with:
```bash
cargo bench
```

## Contributing

When contributing to the project:
1. Follow the architecture patterns described in ARCHITECTURE.md
2. Use the API patterns shown in API.md
3. Add appropriate tests for new functionality
4. Update documentation as needed

## Project Structure

```
latex-rust/
├── src/           # Source code
├── tests/         # Unit and integration tests
├── benches/       # Performance benchmarks
├── examples/      # Example LaTeX documents
├── docs/          # Documentation (this directory)
└── README.md      # Main project documentation
```