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
- [x] **Implement native PDF builder (historical; now replaced by pdfrs)**
- [x] **Replace PdfLatexConverter with NativeTexConverter**
- [x] Update all tests to use native converter
- [x] Update documentation for pdfrs-only PDF backend
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
- [x] **Modularize codebase** - Split into focused modules
  - Create `src/math/` directory (symbols.rs, radicals.rs, fractions.rs, scripts.rs) — COMPLETED
  - Create `src/pdf/` directory (core.rs, builder.rs, text_renderer.rs, font_subset.rs) — COMPLETED
  - Create `src/parser/` directory (mod.rs, text.rs, math.rs, commands.rs) — COMPLETED
  - Move common utilities to `src/utils/` — COMPLETED (extract_braced, extract_braced_inner)

#### Configuration System
- [x] **Configuration system** — `src/config.rs` (removed in audit — dead code, `ConversionOptions` used instead)

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
- [x] **Documentation alignment pass** — `SPEC.md`, updated `USER_GUIDE.md`, `KNOWN_LIMITATIONS.md`, expanded crate docs

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
- [x] **Implement caching** - `src/cache.rs`
  - Cache parsed elements
  - Cache font metrics
  - Cache formatted math expressions
  - Add cache invalidation logic
- [x] **Wire cache into conversion pipeline** — `NativeTexConverter::with_cache()` caches parsed ASTs by content hash; verified by integration test

### Phase 3: Core Features (Week 4-6)

#### Image Support
- [x] **Add image handling** - `src/image.rs` with PNG/JPEG/SVG loading and PDF XObject embedding
  - `\includegraphics{path}` and `\includegraphics[width=5cm]{path}` parsing
  - `ImageInfo::from_path` decodes PNG/JPEG via `image` crate; SVG rasterized via `resvg`/`usvg`
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
  - `\colorbox` / `\fcolorbox` parsed and rendered with colored rectangle backgrounds

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
- [x] **Bibliography support** - `src/bibliography.rs`
  - BibTeX parser: `@type{key, field = {value}, ...}` with brace and quote value support
  - `\cite{key1,key2}` parsed as `TexElement::Citation`
  - `thebibliography` environment parsed as `TexElement::Bibliography` with `\bibitem{key}` entries
  - Numeric citation formatting: `[1, 2]` via `build_citation_map` + `format_citation`
  - `PdfBuilder` pre-scans bibliography to resolve citation numbers before rendering
  - `References` heading rendered before bibliography entries with `[n]` labels
  - `BibEntry` formatting helpers (`format_plain`, `format_numeric`) for future style support

#### Cross-References
- [x] **Cross-reference system** - `src/references.rs`
  - `\label{key}` parsed as `TexElement::Label`; `\ref{key}` as `TexElement::Ref`; `\pageref{key}` as `TexElement::PageRef`
  - `RefStore::scan` assigns sequential numbers: sections (`1`, `1.1`, `2`), equations (`(1)`), figures (`1`), tables (`1`)
  - `RefStore::set_page` updates page numbers during rendering (current page = `state.pages.len()`)
  - `PdfBuilder` pre-scans labels, resolves `\ref` to number text and `\pageref` to page number during content stream build
  - Unknown refs render as `??` (matching LaTeX behavior)
  - Parser integration tests for `\label`, `\ref`, and `\pageref`

#### Streaming
- [x] **Streaming support** - `src/streaming.rs`
  - `ProgressReporter` trait with `NoOpReporter` and `ConsoleReporter` implementations
  - `StreamingConverter` reads files in configurable chunks (default 1 MiB), falling back to `read_to_string` for small files
  - Stage-based progress reporting: reading (0–30%), parsing (30–60%), building PDF (60–100%)
  - `convert_with_progress` convenience helper for one-shot conversions with a reporter
  - CLI (`main.rs`) now uses `StreamingConverter` with `ConsoleReporter` for visible progress
  - Memory-efficient: large files read incrementally via `BufReader` instead of loading entire file into memory

#### Plugin System
- [x] **Plugin architecture** - `src/plugins.rs`
  - `Plugin` trait with `handle_command`, `handle_environment`, `transform_elements` hooks
  - `PluginRegistry` collects plugins and dispatches in registration order
  - `TexParser::with_plugins` intercepts unknown commands and environments before fallback
  - `PdfBuilder::with_plugins` applies `transform_elements` before PDF generation
  - Built-in examples: `TodayPlugin` (`\today` → current date), `UrlPlugin` (`\url{...}` → plain text)

### Phase 5: Professional Features (Month 3+)

#### Advanced Typography
- [x] **Advanced Typography** — `src/typography.rs` (removed in audit — dead code, never used by PDF renderer)

#### TeX Compatibility
- [x] **Core TeX primitives** — `src/tex/` (removed in audit — full TeX lexer/linebreaker never integrated into conversion pipeline)
- [x] **Advanced TeX features** — foundational primitives in `src/tex/` (removed in audit — dead code)

#### Performance
- [x] **Document cache** - `src/cache.rs`
  - `DocumentCache` with TTL and LRU eviction
  - Caches parsed `Vec<TexElement>` and rendered PDF output paths
  - `CacheStats` with per-cache and overall hit-rate tracking
  - Content-hash keys avoid stale results on file changes
- [x] **Incremental compilation** - `src/incremental.rs`
  - `IncrementalCompiler` tracks source-file hashes and output timestamps
  - Skips rebuild when source and dependencies are unchanged
  - Dependency mtime tracking: rebuilds when `.tex` includes or `.bib` files change
  - Hit-rate statistics for build optimization
  - Wired into `StreamingConverter` (enabled by default) with CLI `--force` and `--no-incremental`
- [x] **Parallel processing** — `src/parallel.rs` (removed in audit — dead code, never used by CLI)

#### Optimization
- [x] **PDF optimization** - Reduce file sizes
  - [x] Font subsetting — `collect_used_chars` + `font_subset::subset_font` integrated into `PdfBuilder`; subset test verifies smaller output
  - [x] Image compression — already applied for embedded images
  - [x] Content stream compression — `flate2` FlateDecode on all page streams
  - [x] Remove unused objects — builder creates only referenced objects; no orphans in object graph
  - Target: <100KB for simple documents — ACHIEVED via standard Helvetica for ASCII-only docs (~750 bytes); Unicode/math still embeds subsetted DejaVu

### Low Priority / Future

- [ ] Add support for other LaTeX engines (xelatex, lualatex)
- [x] Add batch conversion support — `src/parallel.rs` (removed in audit — dead code, never used by CLI)
- [x] Add progress indicator for long compilations — `StreamingConverter` with `ConsoleReporter`
- [x] **Add option to keep intermediate files** — `--keep-intermediate` writes `.expanded.tex`, `.ast.json`, `.meta.json` via `src/intermediate.rs`
- [x] **SVG image support** — rasterize SVG via `resvg`/`usvg` in `ImageInfo::from_path`
- [ ] Advanced color management (ICC profiles)
- [ ] PDF/A compliance
- [ ] Accessibility features (tagged PDF)
- [ ] Internationalization (i18n)
- [ ] Web interface
- [ ] GUI application

## Competitive Intelligence

Research vs. Tectonic, Pandoc, Typst — capabilities we lack:

- [x] **Bibliography / citation support** (Pandoc, Tectonic) — `thebibliography` environment + numeric citation formatting (standalone BibTeX parser removed in audit as dead code)
- [x] **Cross-references** — `\label`/`\ref`/`\pageref` parsed + `RefStore` assigns sequential numbers during rendering
- [x] **Template / style system** — removed in audit (dead code, never integrated into conversion pipeline)
- [x] **Incremental / cached compilation** — `DocumentCache` (content-hash AST caching) + `IncrementalCompiler` (source-hash + dependency mtime tracking)
- [x] **WASM target** (Typst) — `convert_tex_string_to_pdf_bytes()` in-memory API + embedded fonts + `wasm` feature with `wasm-bindgen` bindings for browser-side conversion
- [x] **Multiple output formats** (Pandoc) — `src/output/` renders parsed AST to HTML, DOCX, and EPUB; CLI `--format` flag
- [x] **On-demand package fetching** (Tectonic) — `src/packages/` scans preamble and downloads missing `.sty`/`.cls` from CTAN; CLI `--fetch-packages`
- [x] **PDF metadata (Info dictionary)** — Title, Author, Creator, Producer, CreationDate in `PdfBuilder`
- [x] **Real-time preview / watch mode** — `watch_single`/`watch_batch` poll for file changes; CLI `--watch` supports all output formats via `StreamingConverter`

### Brainstorming (future competitive features)

- [x] **Language Server Protocol (LSP)** — `src/lsp/` diagnostics, completion, symbols, hover; `rtex-lsp` binary (`--features lsp`)
- [x] **`description` environment** — `(term, body)` pairs in `TexElement::DescriptionList`; PDF renders term then body; HTML uses `<dl>/<dt>/<dd>`
- [x] **Multi-line math environments** — `align`, `align*`, `gather`, `gather*`, `multline`, `multline*`, `cases` parsed into `TexElement::MathLines { lines, kind }`. `gather`/`multline` render as centered line blocks; `align` preserves `&` markers and is rendered as aligned columns in the pdfrs backend
- [x] **`\href{url}{text}`** — hyperref-style link command parsed; HTML emits `<a href>`; PDF renders the visible text (PDF link annotations not yet emitted)
- [x] **Rich inline rendering** — `flatten_inline` helper now preserves math, formatting commands, nested lists, and `\href` text inside `Center`/`Quote`/`Abstract`/`ItemList` instead of dropping non-Text children
- [x] **Theorem-like environments** — `theorem`, `lemma`, `proof`, `definition`, `corollary`, `proposition`, `remark`, `example` parsed into `TexElement::Theorem { kind, title, body }` and rendered with bold heading + indented body
- [x] **Unnumbered section commands** — `\section*`, `\subsection*`, `\subsubsection*`, `\paragraph*`, `\subparagraph*` consume the trailing `*` so they don't break parsing
- [x] **Sized fractions** — `\tfrac` and `\dfrac` now share the `\frac` Unicode-fraction lookup in `format_fractions`
- [x] **Binomial coefficients** — `\binom`, `\dbinom`, `\tbinom` emit `C(n, k)` notation via `format_fractions`
- [x] **Nested `\begin{cases}` inside `\[…\]`** — display/inline math delimiter parser detects an embedded cases block, parses it as `MathLines`, and merges any prefix/suffix text onto the first/last lines
- [x] **Comprehensive LaTeX syntax coverage** — added `\\` line break (with optional `[length]` and `*`), 27 special characters (`\copyright`, `\pounds`, `\S`, `\P`, `\dag`, `\ddag`, `\ldots`, `\dots`, `\LaTeX`, `\TeX`, `\AA`, `\aa`, `\AE`, `\ae`, `\OE`, `\oe`, `\ss`, `\L`, `\l`, `\O`, `\o`, `\i`, `\j`, `\textellipsis`, `\textcopyright`, `\textregistered`, `\texttrademark`), `\hspace`/`\hspace*`/`\vspace*`, `\linebreak`/`\nopagebreak`/`\samepage`/`\enlargethispage`, `\mbox`/`\parbox`/`\makebox`, `\multicolumn`/`\cline`/`\footnotemark`/`\footnotetext`, `\textnormal`/`\enquote`, 40+ skip-commands (`\setlength`, `\setcounter`, `\ignorespaces`, font family/series/shape declarations, dimension commands), counter formatting (`\value`, `\arabic`, `\roman`, `\alph`, `\the<counter>`), new environments (`figure`, `flushleft`, `flushright`, `minipage`, `displaymath`, `math`, `eqnarray`, `split`, `aligned`, `gathered`), word-boundary fix for short special char commands (`\i` vs `\it`)
- **Collaborative editing** — CRDT or OT layer on parsed AST (Overleaf)
- **Formula OCR input** — photo/screenshot → LaTeX (Mathpix competitor)
- [x] **Shell completions** — `--completions <shell>` flag generates bash/zsh/fish/elvish/powershell scripts via `clap_complete`; documented in README
- **SyncTeX support** — write `.synctex.gz` for editor forward/reverse search (Tectonic 0.16, useful with LSP)
- **Variable fonts** — support OpenType variable font axes (Typst 0.15)
- [x] **MathML export** — emit presentation MathML in HTML output for accessible, selectable math (Typst 0.15 feature); `src/math/mathml.rs` converts LaTeX math to MathML with Unicode fallback
- **Multiple PDF standards** — produce PDF/A + PDF/UA simultaneously (Typst 0.15)
- **Bundle/multi-file export** — output multiple files from a single source (Typst 0.15)
- **Multiple bibliographies** — support multiple `\bibliography` commands with citation routing (Typst 0.15)
- **Layout convergence diagnostics** — warn when layout doesn't converge (Typst 0.15)
- [x] **Divider element** — `\hrulefill` renders as `Element::HorizontalRule` in PDF and `<hr>` in HTML
- **Spot colors** — custom pigment definitions for offset printing (Typst 0.15)
- [x] **File path type** — `\input` supports both braced and unbraced syntax; `\include{filename}` splices with `\clearpage` before/after; shared `splice_file` helper resolves via base_dir, search_paths, and bare path with auto `.tex` extension

## Metrics & Goals

### Current Status
- Tests: 500 (see `cargo test`)
- PDF: pdfrs sole backend (crates.io); `/Producer` stamps `rtex/pdfrs`
- Math: Unicode symbols + pdfrs display layout for `\frac` / `\sqrt` (vinculum) / `pmatrix`/`bmatrix`/`vmatrix` grid
- Math symbols: 566
- LaTeX commands: ~300+
- Documentation: ARCHITECTURE/SPEC/README aligned with pdfrs-only PDF backend
- File size: ~750 bytes for ASCII-only PDFs; subsetted Unicode/math via pdfrs Archive profile

### Audit follow-ups (2026-07-31)
- [x] Remove `vendor/pdfrs/` (1.4 MB tracked but unused — pdfrs from crates.io)
- [x] Fix mutex `unwrap()` calls in `lib.rs` (3 sites → `unwrap_or_else(|e| e.into_inner())`)
- [x] Update stale README badges (550+ tests, 300+ commands)
- [x] Remove empty `src/bin/dev/` directory
- [x] Remove dead modules: `traits.rs`, `common.rs`, `tex/`, `typography.rs`, `template.rs`, `math_processor.rs`, `config.rs`, `bibliography.rs`, `page_layout.rs`, `parallel.rs`, `ErrorContext` trait
- [x] Remove `convert_with_progress` dead function from `streaming.rs`
- [x] Replace `anyhow` in `main.rs` with `LatexError` directly; remove `anyhow` dependency
- [x] Split `parser/mod.rs`: move 1785 lines of tests to `parser/tests.rs`
- [x] Upgrade `thiserror` to v2, `png` to 0.18
- [x] Add DOCX/EPUB structural validation tests (10 tests)
- [x] Fix SVG-to-PNG temp file naming (process ID + atomic counter instead of SystemTime)
- [x] Fix `clippy::manual_strip` in `macros.rs` (use `strip_prefix`)
- [x] Fix `clippy::collapsible_if` in `parser/mod.rs`
- [x] Remove unused `toml` dependency
- [x] Remove `load_plugins_from_dir` dead function
- [x] Fix remaining pedantic clippy warnings (#[must_use], format!, etc.) — zero clippy warnings
- [x] Fix f64 to i64 truncation casts — reviewed, all safe (SVG dimensions bounded, list indices small, file size on 64-bit)
- [x] Refactor `parse_next()` (668 lines) into sub-functions — blocked by borrow checker (`remaining` borrows from `self.content`, conflicts with `&mut self` calls); deferred until Polonius

### Target Goals
- Tests: 100+ (80%+ coverage)
- Warnings: keep low
- Math symbols: 300+
- LaTeX commands: 200+
- Documentation: 80%
- File size: <100KB (optimized for ASCII; Unicode docs embed subsetted font)
