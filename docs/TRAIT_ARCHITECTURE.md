# Trait-Based Architecture

## Overview

rtex uses a small set of traits for its extension and reporting seams. The
bulk of the codebase is concrete structs (`TexParser`, `MathFormatter`,
`NativeTexConverter`); traits are introduced only where polymorphism is
genuinely useful — the converter entry point, the plugin system, and
progress reporting.

This keeps the design lean (KISS) while preserving the testability and
extensibility benefits of a trait-based design.

## Core Traits

### 1. `TexConverter` — conversion entry point

Defined in `src/lib.rs`. The single conversion contract used by the CLI,
library callers, and WASM bindings.

```rust
pub trait TexConverter {
    fn convert(&self, input: &Path, output: &Path) -> Result<(), LatexError>;
    // convenience helpers provided as default methods
}
```

**Implementation:** `NativeTexConverter` (pure Rust, no external TeX).

**Why a trait:** allows alternative converters (e.g. a future engine
delegating to xelatex/lualatex) without changing call sites, and makes
mocking in tests trivial.

### 2. `Plugin` — command/environment/transform extension

Defined in `src/plugins.rs`. The extension point for user-supplied
commands, environments, and AST transforms.

```rust
pub trait Plugin: Send {
    fn name(&self) -> &str;
    // hooks for commands, environments, and post-parse transforms
}
```

**Built-in plugins:** text command and math command plugins live in
`src/plugins.rs` (single module, not a directory).

**Why a trait:** keeps the parser core stable while letting users add
domain-specific LaTeX commands without forking the parser.

### 3. `ProgressReporter` — streaming progress

Defined in `src/streaming.rs`. Decouples progress reporting from the
streaming converter so the same converter works headless (no-op) or with
a console/CI reporter.

```rust
pub trait ProgressReporter: Send {
    fn report(&self, stage: &str, percent: u8);
}
```

**Implementations:** `ConsoleReporter` (stderr), `NoOpReporter` (silent).

## Concrete Core (not traits)

These are structs, intentionally — they have a single implementation and
wrapping them in traits would add indirection without benefit:

- **`TexParser`** (`src/parser/mod.rs`) — recursive-descent parser producing
  `Vec<TexElement>`. 1015-line `parse_next()` is monolithic by necessity
  (borrow checker; see `TODO.md` Known Technical Debt).
- **`MathFormatter`** (`src/math_formatter.rs`) — orchestrates math
  formatting by delegating to `math/{symbols,scripts,radicals,fractions}`
  submodules. 618+ Unicode symbol mappings.
- **`NativeTexConverter`** — wires parser → `output::render_elements` →
  format-specific renderer.

## Design Principles

1. **Traits at the seams, structs at the core.** A trait earns its place
   only when there is a real second implementation or a testing seam.
2. **Single responsibility.** Each trait has one purpose (convert, extend,
   report).
3. **Thread safety where it matters.** `Plugin: Send` so plugins can be
   stored alongside the parser; `ProgressReporter: Send` for streaming.
4. **Testability.** `TexConverter` enables mock converters; `Plugin`
   enables in-process test plugins; `ProgressReporter` enables silent tests.

## Extension Points

- **New output format** — add a variant to `OutputFormat` and a renderer in
  `src/output/`.
- **New LaTeX command** — implement `Plugin` and register it, or extend
  `parser/commands.rs` for built-ins.
- **New math symbol** — add to `lookup_symbol` in `math/symbols.rs` (check
  for duplicates against existing entries).
- **Alternative converter** — implement `TexConverter` and dispatch from
  the CLI.

## References

- `src/lib.rs` — `TexConverter` trait and `NativeTexConverter`
- `src/plugins.rs` — `Plugin` trait and built-in plugins
- `src/streaming.rs` — `ProgressReporter` trait and reporters
- [ARCHITECTURE.md](../ARCHITECTURE.md) — module relationships and data flow
