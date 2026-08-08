# Agent Development Loop

This document defines the continuous improvement cycle for the **rtex** crate — a pure-Rust TeX/LaTeX to PDF (and HTML, EPUB, DOCX) converter with math rendering, LSP, and incremental compilation.

## Project Structure

```
.
├── src/
│   ├── lib.rs              # crate root, module declarations, public API
│   ├── main.rs             # CLI entry point (clap dispatch)
│   ├── error.rs            # RtexError + Result alias
│   ├── parser/
│   │   ├── mod.rs          # recursive-descent TeX parser → AST
│   │   ├── commands.rs     # \command parsing and expansion
│   │   ├── math.rs         # math-mode parsing helpers
│   │   └── text.rs         # text-mode parsing helpers
│   ├── intermediate.rs     # intermediate representation (IR) between AST and output
│   ├── macros.rs           # \newcommand, \renewcommand, \def macro expansion engine
│   ├── math/
│   │   ├── mod.rs          # math module root
│   │   ├── symbols.rs      # symbol table, LaTeX math symbol → Unicode/entity mapping
│   │   ├── fractions.rs    # fraction rendering logic
│   │   ├── radicals.rs     # sqrt and nth-root rendering
│   │   ├── scripts.rs      # superscript/subscript handling
│   │   └── mathml.rs       # MathML output generation
│   ├── math_formatter.rs   # math expression formatting for output backends
│   ├── table.rs            # tabular/array table parsing and rendering
│   ├── output/
│   │   ├── mod.rs          # output dispatcher (PDF, HTML, EPUB, DOCX)
│   │   ├── common.rs       # shared output utilities and helpers
│   │   ├── pdfrs_pdf.rs    # PDF generation via pdfrs (fonts, glyphs, layout, streams)
│   │   ├── html.rs         # HTML output backend
│   │   ├── epub.rs         # EPUB output backend
│   │   └── docx.rs         # DOCX output backend
│   ├── cache.rs            # incremental build cache (content hashing, invalidation)
│   ├── incremental.rs      # incremental compilation orchestration
│   ├── streaming.rs        # streaming parser/processor for large documents
│   ├── watch.rs            # file watcher for live reload / watch mode
│   ├── plugins.rs          # plugin system for custom commands and extensions
│   ├── utils.rs            # shared utility functions
│   ├── wasm.rs             # WebAssembly bindings
│   ├── lsp/                # Language Server Protocol support
│   │   ├── mod.rs          # LSP module root
│   │   ├── server.rs       # LSP server (JSON-RPC over stdio)
│   │   ├── completion.rs   # autocomplete for TeX commands
│   │   ├── diagnostics.rs  # error/warning diagnostics
│   │   ├── hover.rs        # hover documentation
│   │   ├── positions.rs    # position mapping (source ↔ AST)
│   │   └── symbols.rs      # document symbol outline
│   ├── bin/                # debug and test binaries
│   │   ├── rtex-lsp.rs     # LSP binary entry point
│   │   ├── bench.rs        # benchmarking harness
│   │   └── debug_*.rs      # debug utilities for parser, math, fonts, samples
│   ├── tests.rs            # inline unit tests
│   └── example_tests.rs    # example file compilation tests
├── examples/
│   ├── *.tex               # sample TeX documents (minimal, math, tables, lists, etc.)
│   └── README.md           # example descriptions
├── tests/
│   ├── integration_test.rs # end-to-end CLI smoke tests
│   ├── round_trip_test.rs  # parse → render → verify round-trip tests
│   ├── html_render_test.rs # HTML output verification tests
│   ├── docx_epub_test.rs   # DOCX/EPUB output verification tests
│   ├── sqrt_audit_test.rs  # radical rendering audit tests
│   └── fixtures/           # test fixture files (.tex, expected outputs)
├── fonts/                  # bundled font files
├── vendor/                 # vendored dependencies / resources
├── docs/                   # extended documentation
├── Cargo.toml              # package metadata, deps (clap, anyhow, thiserror, etc.)
└── Cargo.lock
```

## The Loop

### 1. Complete Remaining TODO Items
Pick the next highest-priority item from `TODO.md` (or `ARCHITECTURE.md` if the task is architectural). Implement it with minimal, focused changes. Do not add speculative features.

### 2. Create Tests and Examples
For every new capability:
- Add inline `#[cfg(test)] mod tests` in the relevant source file — exercise the feature end-to-end
- Add unit tests for core logic (parser, math rendering, output generation)
- Provide a minimal `.tex` example in `examples/` if the feature is user-facing
- Add a CLI smoke test to `tests/integration_test.rs` if there's a CLI dispatch path
- Add a round-trip test to `tests/round_trip_test.rs` for parse → render → verify workflows

### 3. Ensure `cargo test` Passes
Run the full test suite:
```bash
cargo test                  # all inline unit tests + integration tests
cargo test --examples       # examples compile and run
cargo clippy                # lint pass (warnings acceptable but noted)
```
Fix any failures before proceeding.

### 4. Harvest to MEMORY.md
After each completed feature, extract patterns and best practices:
- **Success patterns**: What worked well and should be repeated
- **Anti-patterns**: What to avoid in future implementations
- **TeX/domain knowledge**: Parsing edge cases, macro expansion pitfalls, font/glyph mapping issues, layout quirks
- **Rust patterns**: rtex-specific conventions for the AST, parser, output backends, and CLI dispatch
- **Testing patterns**: How to assert on PDF/HTML output, fixture documents, regression cases

Add these to `MEMORY.md` with clear categories and references to specific files/lines.

### 5. Loop Back to Step 1
Return to `TODO.md` and pick the next item. Repeat until the backlog is clear.

### 6. Audit and Optimize
After each batch of features, perform a quality pass:
- **Maintainability**: Are functions small and well-named? Is the module structure logical?
- **Leanness**: Remove dead code, unused imports, and speculative abstractions
- **Wiring**: Ensure all new features are properly integrated into `lib.rs`, `main.rs` CLI dispatch, and output backends
- **Small footprint**: Avoid unnecessary crates; prefer the standard library or lightweight dependencies
- **Consistency**: Match existing code style and patterns (Rust edition, `thiserror` for errors, `clap` for CLI)
- **Output fidelity**: Verify rendered output against reference converters (Tectonic, texlive) for correctness

### 7. Competitive Intelligence
Research similar open-source TeX to PDF converters (Tectonic, Pandoc, Typst, texlive, KaTeX, MathJax). Identify capabilities they have that this project lacks. Add the most valuable ones to the `TODO.md` brainstorming section. Prioritize features that provide clear competitive advantage.

### 8. Update Documentation
Keep all project docs aligned with the current implementation. Root docs (required):

- **`README.md`**: Quick start, CLI usage, feature list, crate API summary
- **`ARCHITECTURE.md`**: Module relationships, data flow, design decisions
- **`TODO.md`**: Mark completed items, move them to Done, keep brainstorming current
- **`SPEC.md`**: CLI subcommands, supported TeX commands, output format specifications
- **`MEMORY.md`**: Harvested patterns, domain knowledge, technical conventions

Update **`AGENTS.md`** if the loop itself evolves.

## Memory System (MEMORY.md)

### Purpose
`MEMORY.md` is the institutional knowledge repository that accelerates development by:
- **Preventing wheel reinvention**: Reuse proven patterns instead of guessing
- **Domain knowledge preservation**: Capture TeX parsing and rendering rules that may be counter-intuitive
- **Onboarding acceleration**: New contributors (human or AI) can understand patterns quickly
- **Quality consistency**: Ensure all features follow established conventions

### Structure
Organize `MEMORY.md` into these sections:

#### 1. Parser & AST Patterns
- AST node design and variant structure
- Recursive-descent parser conventions (tokenization, command expansion, environment handling)
- Macro expansion engine conventions (`\newcommand`, `\renewcommand`, `\def`)
- Edge cases: nested environments, optional arguments, verbatim mode

#### 2. Math Rendering Patterns
- Symbol table conventions (LaTeX math symbol → Unicode/entity mapping)
- Fraction, radical, and script rendering approaches
- MathML generation conventions
- Font/glyph mapping for PDF output

#### 3. Output Backend Patterns
- PDF generation via pdfrs (object management, stream encoding, font embedding)
- HTML/EPUB/DOCX backend conventions and shared utilities
- Layout and page structure handling
- Fidelity verification against reference converters

#### 4. Incremental & Streaming Patterns
- Content hashing and cache invalidation strategy
- Incremental compilation orchestration
- Streaming parser for large documents
- File watcher / live reload conventions

#### 5. LSP & Tooling Patterns
- LSP server architecture (JSON-RPC over stdio)
- Completion, diagnostics, hover, and symbol providers
- Position mapping between source and AST

#### 6. Testing Patterns
- Round-trip test structure (parse → render → verify)
- Fixture document management in `tests/fixtures/`
- CLI smoke test conventions in `tests/integration_test.rs`
- Regression cases for parser and renderer edge cases

## Principles

- **Simplicity over flexibility**: Solve the problem at hand, not every hypothetical future problem
- **Surgical changes**: Touch only what you must; clean up only your own mess
- **Goal-driven**: Every change should have a verifiable success criterion
- **Test before ship**: No feature is complete until it has passing tests
- **Docs are code**: Documentation drift is a bug
- **Output fidelity**: Never compromise on rendering correctness for convenience
- **Memory first**: Always check `MEMORY.md` before starting a new feature
- **Pattern harvesting**: After success, update `MEMORY.md` to share the learning

## File Positioning and Value

### README.md
- **Value**: User-facing documentation and project overview
- **Audience**: Users, contributors, stakeholders
- **Position**: Entry point for anyone discovering the project
- **Focus**: Features, quick start, CLI usage, crate API summary, architecture summary

### TODO.md
- **Value**: Feature roadmap and backlog management
- **Audience**: Development team (human and AI agents)
- **Position**: Development planning and prioritization
- **Focus**: What to build next, what's done, competitive intelligence

### ARCHITECTURE.md
- **Value**: Module relationships, data flow, and design decisions
- **Audience**: Contributors maintaining or extending the crate
- **Position**: Structural reference for the codebase
- **Focus**: Module boundaries, data flow, deployment topology

### SPEC.md
- **Value**: Interface specification for the CLI and TeX language support
- **Audience**: Users and contributors integrating with rtex
- **Position**: Contract definition for inputs and outputs
- **Focus**: CLI subcommands, supported TeX commands, output format specifications

### MEMORY.md
- **Value**: Institutional knowledge and pattern library
- **Audience**: Development team (accelerates onboarding and consistency)
- **Position**: Development acceleration and quality consistency
- **Focus**: Proven patterns, domain knowledge, technical conventions
- **Update**: Must be updated after each completed feature to capture patterns and lessons learned

### AGENTS.md (this file)
- **Value**: Development process and workflow definition
- **Audience**: AI agents and human developers following the development loop
- **Position**: Process automation and continuous improvement
- **Focus**: How we work, the loop, memory system, principles
- **Update**: This file should be updated when the development loop itself evolves or when new process patterns emerge

## How These Files Work Together

1. **README.md** tells stakeholders what the project is and how to use it
2. **SPEC.md** defines the CLI and TeX support contract
3. **ARCHITECTURE.md** describes how the modules fit together
4. **TODO.md** tells developers what to build next (driven by competitive intelligence)
5. **AGENTS.md** tells agents how to work through the TODO items with quality and memory
6. **MEMORY.md** captures what we learned so we don't repeat mistakes

The loop reinforces these files:
- Complete TODO → Test → Harvest to MEMORY → Optimize → Research → Update TODO

This creates a flywheel of continuous improvement with institutional knowledge preservation.
