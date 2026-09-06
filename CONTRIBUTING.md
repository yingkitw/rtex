# Contributing to rtex

Thank you for your interest in contributing! This guide covers how to get started.

## Development Setup

1. **Prerequisites**
   - Rust toolchain (latest stable, edition 2024)
   - `cargo` and `rustc`

2. **Clone and build**
   ```bash
   git clone <repository>
   cd rtex
   cargo build
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

## Project Structure

```
src/
  lib.rs              # Core conversion logic, TexConverter trait, public API
  main.rs             # CLI entry point (binary: rtex)
  error.rs            # LatexError + Position (thiserror)
  table.rs            # Table parsing and rendering
  macros.rs           # \newcommand / \renewcommand / \def macro expansion
  math_formatter.rs   # Math formatting orchestrator (struct, not a trait)
  intermediate.rs     # Intermediate artifact writer (.expanded.tex, .ast.json)
  cache.rs            # DocumentCache with TTL/LRU
  incremental.rs      # Incremental compilation (source-hash tracking)
  streaming.rs        # StreamingConverter + ProgressReporter trait
  watch.rs            # File watch mode for auto-rebuild
  plugins.rs          # Plugin trait and built-in plugins
  utils.rs            # Shared utilities (extract_braced, etc.)
  wasm.rs             # wasm-bindgen exports (feature: wasm)
  tests.rs            # Inline unit tests
  example_tests.rs    # Example file compilation tests
  parser/
    mod.rs            # TexParser, TexElement enum, environment dispatch
    text.rs           # Raw text accumulation
    math.rs           # Inline and display math delimiter parsing
    commands.rs       # Backslash command handlers (sections, refs, footnotes, ...)
  math/
    symbols.rs        # 618+ LaTeX-to-Unicode mappings
    scripts.rs        # Superscript/subscript Unicode conversion
    radicals.rs       # Square root (\sqrt) formatting
    fractions.rs      # Fraction -> Unicode or parenthesized form
    mathml.rs         # LaTeX math -> presentation MathML for HTML
  output/
    mod.rs            # Output dispatcher (PDF, HTML, DOCX, EPUB)
    common.rs         # Shared output utilities (DocumentMeta)
    pdfrs_pdf.rs      # PDF generation via vendored pdfrs (sole PDF backend)
    html.rs            # HTML output backend
    docx.rs            # DOCX output backend
    epub.rs            # EPUB output backend
  packages/           # CTAN package scanning and fetching
  lsp/                # Language Server Protocol support (feature: lsp)
    server.rs         # stdio LSP loop (rtex-lsp binary)
    diagnostics.rs    # Error/warning diagnostics
    completion.rs      # Autocomplete candidates
    hover.rs           # Hover documentation
    symbols.rs         # Document symbol outline
  bin/                # Debug/test binaries (feature: dev-bins)
vendor/
  pdfrs/              # Vendored pdfrs PDF engine (local path dependency)
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
