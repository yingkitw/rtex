# Tests

This directory contains the test suite for the LaTeX-Rust project, organized for comprehensive coverage and maintainability.

## Test Files

### Unit Tests
Tests for individual modules and components:
- `ast_tests.rs` - Abstract Syntax Tree functionality
- `config_tests.rs` - Configuration system tests
- `lexer_tests.rs` - Lexical analysis tests
- `math_tests.rs` - Mathematical expression processing
- `parser_tests.rs` - Parser functionality tests
- `renderer_tests.rs` - HTML/PDF rendering tests
- `syntax_highlighting_tests.rs` - Code syntax highlighting

### Integration Tests
End-to-end system tests:
- `integration_tests.rs` - Complete workflow and system interaction tests

### Performance Tests
Benchmark tests are located in the `benches/` directory in the project root.

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --lib
# or
./scripts/test.sh unit
```

### Integration Tests Only
```bash
cargo test --test '*'
# or
./scripts/test.sh integration
```

### Specific Test Module
```bash
cargo test lexer_tests
cargo test parser_tests
```

### With Coverage
```bash
./scripts/test.sh coverage
```

## Test Organization Principles

1. **Unit Tests**: Test individual functions and modules in isolation
2. **Integration Tests**: Test complete workflows and system interactions
3. **Benchmark Tests**: Measure performance characteristics

## Adding New Tests

When adding new tests:
1. Place unit tests in the appropriate `unit/*.rs` file
2. Place integration tests in `integration/`
3. Follow existing naming conventions
4. Include both positive and negative test cases
5. Add performance tests to `benches/` if needed
6. Ensure tests are deterministic and isolated

## Test Coverage

The project maintains comprehensive test coverage across all modules:
- **Lexer Tests**: Tokenization, error handling, edge cases
- **Parser Tests**: Command parsing, environment parsing, error recovery
- **Renderer Tests**: HTML output, CSS generation, math rendering
- **AST Tests**: Node creation, serialization, metadata handling
- **Integration Tests**: End-to-end document processing