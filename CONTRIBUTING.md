# Contributing to latex-rs

Thank you for your interest in contributing! This guide covers how to get started.

## Development Setup

1. **Prerequisites**
   - Rust toolchain (latest stable)
   - `cargo` and `rustc`

2. **Clone and build**
   ```bash
   git clone <repository>
   cd latex-rs
   cargo build
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

## Project Structure

```
src/
  lib.rs              # Core conversion logic
  main.rs             # CLI entry point
  parser.rs           # LaTeX parser
  math_formatter.rs   # Math expression formatting
  pdf_builder.rs      # PDF document generation
  pdf_core.rs         # Low-level PDF primitives
  pdf_text_renderer.rs # Text rendering utilities
  config.rs           # Configuration system
  error.rs            # Error types
  page_layout.rs      # Page dimensions and helpers
  traits.rs           # Composable trait definitions
  math/
    symbols.rs        # LaTeX-to-Unicode mapping
    scripts.rs        # Superscript/subscript conversion
```

## Coding Guidelines

- **Simplicity over flexibility**: Solve the problem at hand, not every hypothetical future problem
- **Surgical changes**: Touch only what you must
- **Goal-driven**: Every change should have verifiable success criteria
- **Test before ship**: No feature is complete until it has passing tests
- **Docs are code**: Documentation drift is a bug

## Testing

- Add unit tests for new functions near the code (inline `#[cfg(test)]` modules)
- Add integration tests in `tests/` for end-to-end workflows
- Ensure `cargo test` passes before submitting

## Submitting Changes

1. Create a focused PR with a clear description
2. Reference any related issues
3. Ensure CI passes (tests + build)

## Questions?

Open an issue or discussion if anything is unclear.
