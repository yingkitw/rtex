# MEMORY.md — Institutional Knowledge

## Parser & AST Patterns

- `TexParser` is a recursive-descent parser that produces `Vec<TexElement>`
- `TexElement` is a large enum: Text, Command, Section, Math, Lists, Theorem, Table, CodeBlock, Image, ColoredText, Citation, Bibliography, Label/Ref/PageRef, Center, Quote, Abstract, LineBreak, FlushLeft, FlushRight, Footnote, Caption, TableOfContents, ListOfFigures, ListOfTables
- `parse_next()` in `parser/mod.rs` is 668 lines — cannot be split into sub-functions due to borrow checker (`remaining` borrows from `self.content`, conflicts with `&mut self` calls); blocked until Polonius
- Macro expansion runs before structured parsing: `MacroStore::extract_definitions` strips `\newcommand`/`\def` from source, then `MacroStore::expand_all` performs iterative expansion (max 10 rounds)
- `\input` and `\include` file splicing uses shared `splice_file` helper with base_dir, search_paths, and bare path with auto `.tex` extension
- Page break commands (`\newpage`, `\clearpage`, `\pagebreak`, `\noindent`, `\indent`, `\newline`, `\raggedright`, `\raggedleft`, `\onecolumn`, `\twocolumn`) are matched as a group with word-boundary guard to avoid matching prefixes of longer commands
- `\par` is handled separately (line ~395) as `TexElement::Paragraph`, not as a page break command — must not duplicate in page break array
- `\marginpar` consumes its braced argument and produces no output (margin notes are dropped in current output backends)
- When adding new commands, always check for word-boundary issues: `\par` vs `\parbox`, `\i` vs `\it`, `\o` vs `\oe`

## Math Rendering Patterns

- `replace_math_symbols` in `math/symbols.rs` uses a single-pass O(n) scanner instead of sequential `String::replace` calls — this eliminated 150+ intermediate string allocations
- `lookup_symbol` is a large match expression (618+ entries) — Rust compiler optimizes this to a jump table
- Word-boundary matters: `\i` vs `\it` — the scanner reads all consecutive alphabetic chars + `*` before lookup
- Font style commands (`\text`, `\mathrm`, `\mathscr`, `\mathnormal`) strip their braced argument content inline
- Math alphabets (`\mathbb`, `\mathfrak`, etc.) are preserved as-is so `format_math_alphabets` can transform them later
- Function names (`\sin`, `\cos`, `\tan`, etc.) map to their plain text form — they are typeset in upright text in LaTeX
- Negated relations (`\ncong`, `\nsim`, `\nmid`, etc.) have dedicated Unicode codepoints
- Common aliases (`\le` = `\leq`, `\ge` = `\geq`, `\ne` = `\neq`, `\dots` = `\ldots`) must be in the symbol table
- `\frac`, `\sqrt`, `\binom` are handled by dedicated submodules (fractions.rs, radicals.rs) not the symbol table
- Matrix environments (`pmatrix`, `bmatrix`, `vmatrix`) are handled by pdfrs display layout, not symbol replacement
- MathML export in `math/mathml.rs` uses a recursive descent parser for HTML output
- `format_accents` in `math_formatter.rs` handles 19 accent commands with Unicode combining chars: `\vec`, `\hat`, `\tilde`, `\bar`, `\dot`, `\ddot`, `\check`, `\breve`, `\acute`, `\grave`, `\mathring`, `\widehat`, `\widetilde`, `\overrightarrow`, `\overleftarrow`, `\overline`, `\underline`, `\dddot`, `\ddddot`
- `\boldsymbol`, `\pmb`, `\operatorname` are in the text-stripping group of `replace_math_symbols` — they consume braced args and output content inline (no bold styling in Unicode fallback)
- `\displaystyle`, `\textstyle`, `\scriptstyle`, `\scriptscriptstyle` are no-ops in the symbol table (map to empty string)
- `\bar` uses U+0304 (combining macron), `\overline` uses U+0305 (combining overline) — they are distinct accents
- `\eqref`, `\autoref`, `\nameref`, `\cref`, `\Cref` are all parsed as `TexElement::Ref` by `parse_ref()` which detects the actual command name
- `\nocite` is parsed as `TexElement::Citation` (same as `\cite`) by `parse_nocite()`
- `\pagestyle`, `\thispagestyle`, `\pagenumbering`, `\definecolor` are in the skip commands list — consumed with all braced arguments, produce no output
- MathML `dispatch_command` in `mathml.rs` has explicit cases for `\mod`, `\pod`, `\boxed` (`<menclose>`), `\substack` (`<mtable>` with `MathMLParser::new` per row), `\boldsymbol`/`\pmb` (wraps in `mathvariant="bold"`), `\limits`/`\nolimits`/`\displaylimits` (no-ops), `\displaystyle`/`\textstyle`/`\scriptstyle`/`\scriptscriptstyle` (no-ops)
- When adding entries to `lookup_symbol`, always check for duplicates — the match expression has 700+ entries and Rust warns about unreachable patterns
- Sized delimiters (`\bigl`, `\bigr`, `\Bigl`, `\Bigr`, `\biggl`, `\biggr`, `\Biggl`, `\Biggr`) map to empty string in `lookup_symbol` — they're consumed without output
- `\xrightarrow`, `\xleftarrow`, `\xRightarrow`, `\xLeftarrow` etc. are aliases for their base arrows in the symbol table (the `x` prefix variants take optional args but the Unicode output is the same)
- Additional skip commands: `\let`, `\edef`, `\xdef`, `\global` consume all following braced args and optional `[...]` args via the skip_cmds loop
- `\color{name}` is a color declaration (switch) parsed as `TexElement::Command`, while `\textcolor{color}{text}` is parsed as `TexElement::ColoredText`
- **Prefix collision bug**: `\ref` check in `parse_next()` was matching `\reflectbox` because `starts_with("\\ref")` matches any command starting with `\ref`. Fixed by adding `&& !remaining.starts_with("\\reflectbox")` guard. Always check for prefix collisions when adding new commands that share a prefix with existing checks

## Output Backend Patterns

- PDF generation is in `src/output/pdfrs_pdf.rs` (not `src/pdf/` — that directory doesn't exist)
- pdfrs is vendored at `vendor/pdfrs` (local path dependency in Cargo.toml), not from crates.io
- Font strategy: standard Helvetica for ASCII-only docs (~750 bytes); embedded DejaVu Sans subset for Unicode/math (~385 KB)
- Font subsetting via `font-subset` crate when Unicode characters are present
- FlateDecode compression on all page content streams
- HTML/DOCX/EPUB backends are in `src/output/{html,docx,epub}.rs` with shared utilities in `common.rs`
- Public API: `render_elements(elements, OutputFormat)` and `convert_tex_file(path, output, &ConversionOptions)`

## Incremental & Streaming Patterns

- `DocumentCache` uses content-hash keys to avoid stale results
- `IncrementalCompiler` tracks source-file hashes + dependency mtime (`.tex` includes, `.bib` files)
- `StreamingConverter` reads files in configurable chunks (default 1 MiB) with `BufReader`
- Progress reporting: reading (0–30%), parsing (30–60%), building (60–100%)
- CLI uses `StreamingConverter` with `ConsoleReporter` by default

## LSP & Tooling Patterns

- LSP server runs over stdio (`rtex-lsp` binary, `--features lsp`)
- `diagnostics.rs` checks braces, environments, math delimiters, document structure
- `completion.rs` provides command and environment completion candidates
- `symbols.rs` generates section/label outline from parsed AST
- `hover.rs` provides command documentation on hover

## Testing Patterns

- 648 tests total (468 lib unit tests + 10 doc tests + 79 integration + 87 round-trip + 10 docx/epub + 1 html)
- Round-trip tests in `tests/round_trip_test.rs` verify deterministic conversion
- `tests/sqrt_audit_test.rs` specifically tests radical rendering
- All tests run with zero external dependencies
- `cargo clean` + rebuild may reveal clippy warnings that were cached away — always verify from clean state

## Domain Knowledge

- pdfrs is vendored, not from crates.io — Cargo.toml uses `path = "vendor/pdfrs"`
- `src/fonts.rs` does not exist — font embedding is handled within `output/pdfrs_pdf.rs`
- `src/pdf/` directory does not exist — PDF code is in `src/output/pdfrs_pdf.rs`
- Several modules were removed in audit (2026-07-31): traits.rs, common.rs, tex/, typography.rs, template.rs, math_processor.rs, config.rs, bibliography.rs, page_layout.rs, parallel.rs
- `parallel.rs` was removed — "parallel batch conversion" should not be claimed as a feature
