# TODO

## Completed
- [x] Initialize Rust project with Cargo.toml
- [x] Implement core TeX to PDF conversion logic
- [x] Create CLI interface with clap
- [x] Add trait-based architecture for testability
- [x] Add comprehensive error handling
- [x] Create unit tests
- [x] Create README.md
- [x] Verify cargo build success
- [x] Verify cargo test success (12/12 tests passing)
- [x] Create example TeX files (6 examples)
- [x] Add .gitignore
- [x] Add PDF generation verification in tests
- [x] Add PDF header validation
- [x] Add file size validation
- [x] Test multiple document types
- [x] Create examples README
- [x] Add output folder support (default: output/)
- [x] Create integration test for generating all examples
- [x] Create helper script for generating examples
- [x] **Implement native TeX parser (no pdflatex dependency!)**
- [x] **Implement native PDF builder using printpdf**
- [x] **Replace PdfLatexConverter with NativeTexConverter**
- [x] Update all tests to use native converter
- [x] Update documentation for native implementation
- [x] Improve text element quality validation before PDF output (normalize whitespace/control chars, preserve unmatched inline-math delimiter, robust wrapping for long tokens)
- [x] Run core example PDF tests by default (unignore minimal/math/table/lists tests)
- [x] Harden parser against unknown-command hangs (forward-progress guard + unknown command consumption + regression tests)
- [x] Improve math generation quality (fractions/sqrt/scripts parsing + richer symbol mapping + regression tests)

## Pending

### Phase 1: Foundation (Week 1) - CURRENT FOCUS

#### Architecture & Code Quality
- [x] **Font embedding for Unicode math symbols** - COMPLETED with lopdf + DejaVu Sans
- [x] **Modular math processing** - COMPLETED (150+ symbols from minitex)
- [x] **Proper error handling system** - Create `src/error.rs` with structured errors
  - Use `thiserror` for error types
  - Add error context (line numbers, positions)
  - Implement error recovery
  - Add diagnostic output
- [x] **Fix all compiler warnings** - Build produces 0 warnings; no unused Environment variant found
- [ ] **Modularize codebase** - Split into focused modules
  - Create `src/math/` directory (symbols.rs, radicals.rs, fractions.rs, scripts.rs) — IN PROGRESS (symbols.rs + scripts.rs extracted)
  - Create `src/parser/` directory (text.rs, math.rs, commands.rs)
  - Create `src/pdf/` directory (builder.rs, fonts.rs, layout.rs)
  - Move common utilities to `src/utils/`

#### Configuration System
- [x] **Create configuration system** - `src/config.rs` with builder pattern and tests
  - Quality presets (Draft, Standard, High, Print)
  - Font embedding options
  - Output format settings
  - Compression levels
  - Feature flags

#### Testing Infrastructure
- [x] **Expand test coverage** - 116 tests passing across all modules
  - Unit tests for math (symbols, scripts), pdf_core, pdf_text_renderer, parser
  - Integration tests for full workflows
  - Error cases and edge conditions covered
  - Target: 100+ tests — ACHIEVED

### Phase 2: Quality & Documentation (Week 2-3)

#### Documentation
- [x] **Expand ARCHITECTURE.md** - Documented module structure, design decisions, and current file organization
- [x] **Create CONTRIBUTING.md** - Guidelines for contributors
- [x] **Add module documentation** - Documented lib.rs, parser.rs, error.rs, math_formatter.rs, pdf_builder.rs
- [x] **Create user guide** - `docs/USER_GUIDE.md` with examples and quick start
- [x] **Add API documentation** - rustdoc added to all major public APIs (lib.rs, parser.rs, error.rs, pdf_builder.rs, pdf_core.rs, pdf_text_renderer.rs, math_formatter.rs, utils.rs)

#### Code Quality
- [x] **DRY refactoring** - Extracted `src/utils.rs` with `extract_braced`/`extract_braced_inner`
  - Replaced `parser.rs::extract_braced_content_from` and `math_formatter.rs::read_group`
  - 9 unit tests for shared utility
- [x] **Performance profiling** - Optimized math symbol replacement
  - Baseline: 37.0 ms avg per conversion
  - Hot path: `math::symbols::replace_math_symbols` with 150+ sequential `String::replace` calls
  - Optimized: single-pass O(n) scanner with `lookup_symbol` match table
  - Result: 34.3 ms avg (~7% faster), math-heavy docs ~30% faster
  - Eliminated 150+ intermediate string allocations per math expression

#### Caching System
- [ ] **Implement caching** - `src/cache.rs`
  - Cache parsed elements
  - Cache font metrics
  - Cache formatted math expressions
  - Add cache invalidation logic

### Phase 3: Core Features (Week 4-6)

#### Image Support
- [x] **Add image handling** - `src/image.rs` with PNG/JPEG loading and PDF XObject embedding
  - `\includegraphics{path}` and `\includegraphics[width=5cm]{path}` parsing
  - `ImageInfo::from_path` decodes via `image` crate, converts to RGB8
  - `PdfBuilder` creates XObjects, registers in page Resources, draws via `cm` + `Do`
  - `parse_dimension` supports cm, mm, in, pt, and raw units
  - Parser tests for basic and optional-width includegraphics

#### Table Support
- [x] **Implement table rendering** - `src/table.rs`
  - `Table::parse` extracts column specs (l/c/r/p{}) and row/cell data from `tabular` content
  - `render_table` draws cells with alignment offsets and horizontal rules via PDF path operators
  - Parser produces `TexElement::Table` instead of flattening to text
  - `\hline`, `\toprule`, `\midrule`, `\bottomrule` recognized as separator rows
  - `\begin{table}` wrapper extracts inner `tabular` and returns same `Table` element

#### Color Support
- [x] **Add color management** - `src/color.rs`
  - `Color` struct with normalized RGB and 12 named colors
  - `Color::parse` accepts named colors, `#RRGGBB`/`RGB` hex, and `r,g,b` decimal triples
  - `\textcolor{color}{text}` parsed as `TexElement::ColoredText`; PDF builder applies `rg` operator
  - `ContentStream::set_color` added for non-stroking RGB color changes
  - `\colorbox` deferred — requires rectangle fill primitive

#### Layout Engine
- [x] **Improve layout engine** - `src/layout.rs`
  - `LayoutState` tracks current page, Y position, and content area using `PageLayout`
  - Automatic page breaks: `render_text_block` creates new pages mid-text-block
  - `ensure_space` pre-checks available height before rendering large elements
  - `PdfBuilder` now produces multi-page PDFs with proper `/Pages` tree
  - Hard-coded layout constants replaced with `PageLayout::a4_portrait()`
  - Column support and float positioning deferred to future work

#### Macro System
- [x] **Basic macro support** - `src/macros.rs`
  - `\newcommand{\name}[n]{body}` and `\newcommand\name[n]{body}` parsing
  - `\def\name#1#2...{body}` parsing with parameter counting
  - Parameter substitution (`#1`, `#2`, ...) during expansion
  - `MacroStore::extract_definitions` strips definitions from source before parsing
  - `MacroStore::expand_all` performs iterative expansion (max 10 rounds) with loop protection
  - `TexParser::new` automatically runs macro extraction + expansion before structured parsing
  - Parser integration tests for `\newcommand`, `\newcommand` with args, and `\def`

### Phase 4: Advanced Features (Month 2-3)

#### Bibliography
- [ ] **Bibliography support** - `src/bibliography.rs`
  - BibTeX parsing
  - \cite command
  - Bibliography formatting
  - Citation styles

#### Cross-References
- [ ] **Cross-reference system** - `src/references.rs`
  - \label and \ref
  - \pageref
  - Section references
  - Equation references
  - Figure/table references

#### Streaming
- [ ] **Streaming support** - `src/streaming.rs`
  - Handle large documents (>100MB)
  - Incremental parsing
  - Memory-efficient processing
  - Progress reporting

#### Plugin System
- [ ] **Plugin architecture** - `src/plugins.rs`
  - Plugin trait definition
  - Plugin loading
  - Custom command handlers
  - Extension points

### Phase 5: Professional Features (Month 3+)

#### Advanced Typography
- [ ] **OpenType features** - `src/typography.rs`
  - Ligatures
  - Kerning
  - Small caps
  - Stylistic sets

#### TeX Compatibility
- [ ] **Professional TeX features** - `src/tex/`
  - Category codes
  - Token system
  - Dimension system
  - Glue system
  - Box model
  - Line breaking (Knuth-Plass)

#### Performance
- [ ] **Parallel processing** - `src/parallel.rs`
  - Multi-threaded parsing
  - Parallel page rendering
  - Work stealing
  - Thread pool

#### Optimization
- [ ] **PDF optimization** - Reduce file sizes
  - Font subsetting
  - Image compression
  - Content stream optimization
  - Remove unused objects
  - Target: <100KB for simple documents

### Low Priority / Future

- [ ] Add support for other LaTeX engines (xelatex, lualatex)
- [ ] Add batch conversion support
- [ ] Add progress indicator for long compilations
- [ ] Add option to keep intermediate files
- [ ] SVG image support
- [ ] Advanced color management (ICC profiles)
- [ ] PDF/A compliance
- [ ] Accessibility features (tagged PDF)
- [ ] Internationalization (i18n)
- [ ] Web interface
- [ ] GUI application

## Competitive Intelligence

Research vs. Tectonic, Pandoc, Typst — capabilities we lack:

- **Bibliography / citation support** (Pandoc, Tectonic) — highest value; needed for academic docs
- **Cross-references** (\label, \ref, \cite) — basic but widely used
- **Template / style system** — allow custom document styling without editing Rust code
- **Incremental / cached compilation** — Typst achieves <500ms renders via caching
- **WASM target** (Typst) — run in browser with zero infrastructure
- **Multiple output formats** (Pandoc) — HTML, DOCX, EPUB from same source
- **On-demand package fetching** (Tectonic) — download missing packages automatically
- **Real-time preview / watch mode** — recompile on file changes

## Metrics & Goals

### Current Status
- Tests: 147 (100% passing)
- Warnings: 1 (pre-existing dead_code in pdf_core.rs)
- Math symbols: 150+
- LaTeX commands: ~50
- Documentation: ~25%
- File size: ~750KB per PDF

### Target Goals
- Tests: 100+ (80%+ coverage)
- Warnings: 0
- Math symbols: 300+
- LaTeX commands: 200+
- Documentation: 80%
- File size: <100KB (optimized)
