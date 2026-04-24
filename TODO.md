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
- [ ] **Proper error handling system** - Create `src/error.rs` with structured errors
  - Use `thiserror` for error types
  - Add error context (line numbers, positions)
  - Implement error recovery
  - Add diagnostic output
- [ ] **Fix all compiler warnings** - Remove unused Environment variant or implement it
- [ ] **Modularize codebase** - Split into focused modules
  - Create `src/math/` directory (symbols.rs, radicals.rs, fractions.rs, scripts.rs)
  - Create `src/parser/` directory (text.rs, math.rs, commands.rs)
  - Create `src/pdf/` directory (builder.rs, fonts.rs, layout.rs)
  - Move common utilities to `src/utils/`

#### Configuration System
- [ ] **Create configuration system** - `src/config.rs`
  - Quality presets (Draft, Standard, High, Print)
  - Font embedding options
  - Output format settings
  - Compression levels
  - Feature flags

#### Testing Infrastructure
- [ ] **Expand test coverage** - Target 80%+ coverage
  - Create `src/testing/fixtures.rs` with common test data
  - Add unit tests for each module (math, parser, pdf)
  - Add integration tests for full workflows
  - Test error cases and edge conditions
  - Current: 13 tests → Target: 100+ tests

### Phase 2: Quality & Documentation (Week 2-3)

#### Documentation
- [ ] **Expand ARCHITECTURE.md** - Document module structure and design decisions
- [ ] **Create CONTRIBUTING.md** - Guidelines for contributors
- [ ] **Add module documentation** - Document all public modules and functions
- [ ] **Create user guide** - `docs/USER_GUIDE.md` with examples
- [ ] **Add API documentation** - Use rustdoc for all public APIs

#### Code Quality
- [ ] **DRY refactoring** - Extract common patterns
  - Shared brace extraction utility
  - Common text wrapping logic
  - Shared font handling
- [ ] **Performance profiling** - Identify and optimize bottlenecks
  - Profile parsing performance
  - Profile PDF generation
  - Optimize hot paths
  - Measure memory usage

#### Caching System
- [ ] **Implement caching** - `src/cache.rs`
  - Cache parsed elements
  - Cache font metrics
  - Cache formatted math expressions
  - Add cache invalidation logic

### Phase 3: Core Features (Week 4-6)

#### Image Support
- [ ] **Add image handling** - `src/image.rs`
  - PNG support
  - JPEG support
  - Image scaling and positioning
  - \includegraphics command

#### Table Support
- [ ] **Implement table rendering** - `src/table.rs`
  - Basic tabular environment
  - Cell alignment (left, center, right)
  - Borders and lines
  - Multi-column cells
  - Table positioning

#### Color Support
- [ ] **Add color management** - `src/color.rs`
  - RGB colors
  - Named colors
  - \textcolor command
  - \colorbox command
  - Color spaces (RGB, CMYK)

#### Layout Engine
- [ ] **Improve layout engine** - `src/layout.rs`
  - Better text positioning
  - Margin management
  - Column support
  - Float positioning
  - Page breaks

#### Macro System
- [ ] **Basic macro support** - `src/macros.rs`
  - \newcommand
  - \def
  - Parameter substitution
  - Macro expansion

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

## Metrics & Goals

### Current Status
- Tests: 13 (100% passing)
- Warnings: 1
- Math symbols: 150+
- LaTeX commands: ~50
- Documentation: ~20%
- File size: ~750KB per PDF

### Target Goals
- Tests: 100+ (80%+ coverage)
- Warnings: 0
- Math symbols: 300+
- LaTeX commands: 200+
- Documentation: 80%
- File size: <100KB (optimized)
