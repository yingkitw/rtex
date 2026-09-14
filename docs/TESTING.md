# Testing Guide

## Test Organization

`cargo test` runs the full suite by default — **786 passing tests, 3 ignored**, completing in ~7 seconds. PDF-generating tests are **not** marked `#[ignore]`; they run as part of the default suite because the pdfrs engine is fast and needs no external tools.

Only three tests are `#[ignore]`d, all golden-file generators that write fixtures rather than assert on them:
- `round_trip_test.rs::generate_golden_files`
- `round_trip_test.rs::test_golden_file_minimal`
- `src/utils.rs` doctest (feature-gated example)

Run them explicitly with `cargo test -- --ignored`.

### Unit Tests (inline)

Live inside each module under `#[cfg(test)] mod tests`:
- `src/parser/tests.rs`, `src/parser/text.rs` — parser coverage
- `src/math/symbols.rs`, `src/math/scripts.rs` — math formatting
- `src/output/pdfrs_pdf.rs` — PDF rendering
- `src/table.rs`, `src/macros.rs`, `src/plugins.rs`, `src/cache.rs`, `src/incremental.rs`, `src/streaming.rs`, `src/watch.rs`, `src/utils.rs`
- `src/tests.rs` — conversion pipeline (library API, in-memory PDF/HTML/DOCX/EPUB)

### Integration Tests (`tests/`)

| File | Coverage |
|------|----------|
| `integration_test.rs` | End-to-end CLI smoke test (all examples) |
| `round_trip_test.rs` | Parse → render → verify PDF regression (99 tests + 2 ignored golden generators) |
| `html_render_test.rs` | HTML output verification (96 tests) |
| `docx_epub_test.rs` | DOCX/EPUB structural tests (10 tests) |
| `sqrt_audit_test.rs` | Radical rendering audit (2 tests) |

## Running Tests

```bash
# Full suite (default — includes PDF generation)
cargo test

# Ignored golden-file generators only
cargo test -- --ignored

# Everything, including ignored
cargo test -- --include-ignored

# Specific test
cargo test test_error_display

# With output
cargo test -- --nocapture

# Single-threaded (for debugging)
cargo test -- --test-threads=1

# Generate example PDFs
cargo test --lib example_tests -- --nocapture
./generate_examples.sh
```

## Adding New Tests

### Unit Tests

Add to the module being tested:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test logic — PDF-generating tests are fine here, no #[ignore] needed
    }
}
```

### Integration Tests

Add a new file under `tests/` (one file per capability area). PDF-generating integration tests run by default; reserve `#[ignore]` only for tests that write fixtures rather than assert on them.

## Conventions

- PDF-generating tests run by default — the pdfrs engine is in-process and fast.
- Use `#[ignore]` only for golden-file generators that write artifacts, not for slow assertions.
- Keep tests focused and isolated; prefer real conversion calls over mocks.
- The full suite must stay green: `cargo test` is the CI gate.
