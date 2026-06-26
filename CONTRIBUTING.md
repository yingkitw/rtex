# Contributing to rtex

Thank you for your interest in contributing! This guide covers how to get started.

## Development Setup

1. **Prerequisites**
   - Rust toolchain (latest stable, edition 2024)
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
  lib.rs              # Core conversion logic and public API
  main.rs             # CLI entry point (binary: rtex)
  error.rs            # Structured error types with Position tracking
  config.rs           # Configuration system with quality presets
  common.rs           # Shared traits (Clear, Stats)
  color.rs            # Color struct with RGB and named color parsing
  table.rs            # Table parsing and PDF rendering
  macros.rs           # Macro definition and expansion system
  layout.rs           # LayoutState for text alignment and indentation
  page_layout.rs      # Page dimensions, margins, orientation helpers
  math_formatter.rs   # Math expression formatting orchestrator
  parser/
    mod.rs            # TexParser, TexElement enum, environment dispatch
    text.rs           # Raw text accumulation
    math.rs           # Inline/display math parsing
    commands.rs       # Section, URL, rule, footnote, caption parsers
    plugin.rs         # Plugin trait for extensible commands
  math/
    symbols.rs        # 566+ LaTeX-to-Unicode mappings
    scripts.rs        # Superscript/subscript Unicode conversion
    radicals.rs       # Square root formatting with Unicode
    fractions.rs      # Fraction → Unicode fraction or parenthesized form
  pdf/
    mod.rs            # PDF module re-exports
    core.rs           # ContentStream, PdfGenerator, PdfObj, HEX_TABLE
    builder.rs        # PdfBuilder, LayoutState, text block rendering
    text_renderer.rs  # Text normalization and wrapping utilities
    font_subset.rs    # Font subsetting to only used characters
  bin/
    debug_*.rs          # Debug/test binaries
  plugins/
    text_commands.rs  # Built-in text command plugins
    math_commands.rs  # Built-in math command plugins
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
