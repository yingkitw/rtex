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
| PDF | Custom generator (`src/pdf/`) + DejaVu Sans embedding |
| Math | Unicode mapping (`src/math/`) |
| Alt formats | HTML/DOCX/EPUB (`src/output/`) |
| HTTP (packages) | ureq + rustls |
| SVG | resvg + usvg |
| WASM | wasm32-unknown-unknown + wasm-bindgen (optional) |

## Quality Bar

- `cargo build` and `cargo test` pass with 0 warnings
- 340+ automated tests including round-trip and example PDF verification
- Public APIs documented with rustdoc
- User-facing docs in `README.md`, `docs/USER_GUIDE.md`, `ARCHITECTURE.md`
- Surgical changes: no speculative features beyond `TODO.md`

## Success Criteria

1. A minimal `\documentclass{article}` document converts to valid PDF/HTML/DOCX/EPUB
2. Math documents render Unicode symbols via embedded DejaVu Sans
3. CLI flags (`--format`, `--watch`, `--fetch-packages`, `--keep-intermediate`) work end-to-end
4. WASM build succeeds with `cargo build --target wasm32-unknown-unknown --features wasm`
