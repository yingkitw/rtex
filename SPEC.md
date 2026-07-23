# rtex Specification

## Scope

rtex is a **native** TeX-to-document converter written in Rust. It parses a practical subset of LaTeX and produces output **without** requiring an external TeX distribution (pdflatex, xelatex, etc.).

### In scope

- CLI conversion of `.tex` files to PDF, HTML, DOCX, or EPUB
- Library API for in-process and WASM embedding
- 228+ LaTeX commands, 566 math symbols, native PDF generation
- Bibliography, cross-references, tables, graphics, macros, templates
- Incremental compilation, watch mode, parallel batch conversion, LSP editor support
- On-demand CTAN package fetching for missing `.sty`/`.cls` files
- Multi-line math environments (`align`, `gather`, `multline`, `cases`)
- `description` lists for term/definition pairs
- Theorem-like environments (`theorem`, `lemma`, `proof`, etc.)
- `\href{url}{text}` for hyperlinks
- Unnumbered sections (`\section*`, `\subsection*`)
- `\binom`, `\tfrac`, `\dfrac` binomial and sized fractions
- Ellipsis symbols `\ldots`, `\cdots`, `\vdots`, `\ddots`

### Out of scope (current)

- Full TeX compatibility (TikZ, PGF, raw `\catcode` execution)
- xelatex/lualatex engine emulation
- PDF/A, tagged PDF, ICC color profiles
- Web UI and GUI (library/CLI only)

## Technical Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2024 (MSRV 1.85) |
| CLI | clap 4 |
| PDF | [pdfrs](https://crates.io/crates/pdfrs) (vendored) primary + native `src/pdf/` fallback |
| Math | Unicode mapping (`src/math/`) + pdfrs display layout (`\frac`, `\sqrt` vinculum) |
| Alt formats | HTML/DOCX/EPUB (`src/output/`) |
| HTTP (packages) | ureq + rustls |
| SVG | resvg + usvg |
| WASM | wasm32-unknown-unknown + wasm-bindgen (optional) |

## Quality Bar

- `cargo build` and `cargo test` pass (warnings noted; eliminate when practical)
- 400+ automated tests including round-trip and example PDF verification
- Public APIs documented with rustdoc
- User-facing docs in `README.md`, `docs/USER_GUIDE.md`, `ARCHITECTURE.md`
- Surgical changes: no speculative features beyond `TODO.md`
- PDF backend identifiable via `/Producer` (`rtex/pdfrs` or `rtex/native`); force with `RTEX_PDF_BACKEND`

## Success Criteria

1. A minimal `\documentclass{article}` document converts to valid PDF/HTML/DOCX/EPUB
2. Math documents render Unicode symbols via embedded DejaVu Sans
3. CLI flags (`--format`, `--watch`, `--fetch-packages`, `--keep-intermediate`) work end-to-end
4. WASM build succeeds with `cargo build --target wasm32-unknown-unknown --features wasm`
