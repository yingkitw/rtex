# Testing Guide

## Test Organization

### Fast Tests (Default)
Run with: `cargo test`

These tests run quickly and are suitable for development:
- Unit tests for error handling
- Unit tests for configuration
- Parser tests (no PDF generation)
- Quick validation tests

**Execution time:** < 1 second

### Slow Tests (Ignored by Default)
Run with: `cargo test -- --ignored`

These tests generate actual PDFs and are slower:
- `test_minimal_example` - Basic PDF generation
- `test_math_example` - Math equation rendering
- `test_table_example` - Table rendering
- `test_lists_example` - List rendering
- `test_complex_document` - Full document with all features

**Execution time:** 5-10 seconds (due to font loading and PDF generation)

### All Tests
Run with: `cargo test -- --include-ignored`

Runs both fast and slow tests.

## Test Categories

### Unit Tests
- `src/error.rs` - Error handling tests
- `src/tests.rs` - Conversion pipeline tests
- Module-level tests in `parser/`, `math/`, `output/`, `cache.rs`, `incremental.rs`, `streaming.rs`, `watch.rs`

### Integration Tests
- `src/tests.rs` - Full conversion pipeline tests
- Marked with `#[ignore]` for slow tests

## Running Tests

```bash
# Fast tests only (default)
cargo test

# Slow integration tests only
cargo test -- --ignored

# All tests
cargo test -- --include-ignored

# Specific test
cargo test test_error_display

# With output
cargo test -- --nocapture

# Single-threaded (for debugging)
cargo test -- --test-threads=1
```

## Adding New Tests

### Fast Unit Tests
Add to the module being tested:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something_fast() {
        // Test logic that doesn't generate PDFs
    }
}
```

### Slow Integration Tests
Add to `src/tests.rs` with `#[ignore]`:
```rust
#[test]
#[ignore] // Slow test - run with: cargo test -- --ignored
fn test_something_slow() {
    // Test that generates PDFs or loads fonts
}
```

## Test Coverage Goals

- **Priority:** Fast unit tests for all modules; integration tests in `tests/` for end-to-end workflows
- Run `cargo test` for the fast suite; `cargo test -- --ignored` for PDF-generating tests

## Performance Guidelines

- Fast tests should complete in < 100ms
- Slow tests can take up to 2 seconds each
- Mark any test that does PDF generation as `#[ignore]`
- Mark any test that loads fonts as `#[ignore]`

## CI/CD Recommendations

```yaml
# Fast tests on every commit
- cargo test

# Slow tests on pull requests
- cargo test -- --ignored

# All tests before release
- cargo test -- --include-ignored
```
