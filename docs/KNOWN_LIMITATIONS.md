# Known Limitations

## What rtex Is

A **native subset LaTeX converter** — not a replacement for pdflatex/xelatex/lualatex. It parses common LaTeX constructs into an internal AST and renders to PDF/HTML/DOCX/EPUB without shelling out to an external TeX engine.

## Font and PDF Size

- **Dual font strategy**: ASCII-only documents use PDF standard Helvetica (no embedding, ~750 bytes). Documents with Unicode or formatted math embed a subsetted **DejaVu Sans**.
- Math-heavy PDFs remain ~385 KB due to the embedded font subset.

## Unsupported or Partial Features

| Area | Status |
|------|--------|
| TikZ / PGF / PGFPlots | Not supported |
| Beamer / complex document classes | Partial (`article`-style only) |
| Raw TeX `\catcode` / glue / boxes | Primitives exist in `src/tex/` but not wired to layout |
| Full package execution | `.sty` files are cached/fetched but not interpreted as TeX macros |
| Algorithm2e, minted, etc. | Not supported unless parsed as plain text |
| PDF/A, tagged PDF, ICC profiles | Not implemented |
| Perfect HTML/DOCX/EPUB fidelity | Functional export; not LaTeX-identical layout |

## Math

- 566 LaTeX math commands map to Unicode via `MathFormatter`
- Multi-line math environments (`align`, `align*`, `gather`, `gather*`, `multline`,
  `multline*`, `cases`) are parsed and rendered as centered line blocks;
  the `&` alignment marker is stripped rather than used for true column
  alignment, so the visual fidelity is approximate. Numbering is not yet
  emitted.
- `\begin{cases}` nested inside `\[…\]` or `\(…\)` is recognised and
  rendered as cases; the surrounding text (e.g. `f(x) =`) is merged onto
  the first case line.
- `\binom`, `\dbinom`, `\tbinom` render as `C(n, k)` (TeX-style binomials
  require AMS extensions we do not implement).
- Some constructs fall back to Unicode approximations rather than TeX-quality spacing

## Package Fetching

`--fetch-packages` downloads `.sty`/`.cls` from CTAN mirrors using URL heuristics:

- Works for common packages in `macros/latex/base`, `required/`, and `contrib/`
- Does **not** execute downloaded files — only makes them available for `\input`
- Offline mode fails if packages are missing from cache

## Comparison

| Feature | rtex | pdflatex / Tectonic |
|---------|------|---------------------|
| Installation | `cargo install` / single binary | Full TeX Live or bundle |
| Speed | Fast (native Rust) | Slower (full engine) |
| Package support | Subset + fetch/cache | Thousands, fully executed |
| Math quality | Unicode in embedded font | TeX-quality spacing |
| Output formats | PDF, HTML, DOCX, EPUB | Primarily PDF (Pandoc for others) |
| Best for | Quick conversion, WASM preview, CI | Production LaTeX documents |

## Recommendations

- **rtex**: simple articles, math notes, CI pipelines, browser preview (WASM), multi-format export
- **Tectonic / TeX Live**: final publications, complex packages, bibliographies with BibTeX styles, TikZ figures
