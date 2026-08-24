# rtex Specification

## Scope

rtex is a **native** TeX-to-document converter written in Rust. It parses a practical subset of LaTeX and produces output **without** requiring an external TeX distribution (pdflatex, xelatex, etc.).

### In scope

- CLI conversion of `.tex` files to PDF, HTML, DOCX, or EPUB
- Library API for in-process and WASM embedding
- 450+ LaTeX commands, 618 math symbols, PDF generation via vendored pdfrs engine
- Bibliography, cross-references, tables, graphics, macros
- Incremental compilation, watch mode, LSP editor support
- On-demand CTAN package fetching for missing `.sty`/`.cls` files
- Multi-line math environments (`align`, `gather`, `multline`, `cases`)
- `description` lists for term/definition pairs
- Theorem-like environments (`theorem`, `lemma`, `proof`, etc.)
- `\href{url}{text}` for hyperlinks
- Unnumbered sections (`\section*`, `\subsection*`)
- `\binom`, `\tfrac`, `\dfrac` binomial and sized fractions
- Ellipsis symbols `\ldots`, `\cdots`, `\vdots`, `\ddots`
- Cross-reference variants: `\eqref`, `\autoref`, `\nameref`, `\cref`, `\Cref`, `\nocite`
- 19 math accent commands with Unicode combining characters
- Math style switches: `\displaystyle`, `\textstyle`, `\scriptstyle`, `\scriptscriptstyle`
- Math text commands: `\boldsymbol`, `\pmb`, `\operatorname`
- 50+ additional math symbols: negated relations, arrows, operators, function names, geometry, sized delimiters
- MathML dispatch: `\mod`, `\pod`, `\boxed`, `\substack`, `\limits`/`\nolimits`, `\boldsymbol` with `mathvariant="bold"`
- Text commands: `\textcircled`, `\hl`, `\st`, `\uline`, `\uuline`, `\uwave`, `\dotuline`, `\color`, `\normalcolor`
- 30+ additional skip commands: `\let`, `\edef`, `\xdef`, `\global`, `\parindent`, `\parskip`, `\floatsep`, etc.
- Biblatex citation commands: `\textcite`, `\parencite`, `\footcite`, `\citeauthor`, `\citeyear`, `\citetitle`, `\fullcite` with kind-aware rendering
- `\printbibliography` (with optional `[...]` args), `\addbibresource{file.bib}`
- Missing common environments: `tabular*`, `tabularx`, `array`, `comment`, `subfigure`/`subfig`, `table*`
- Table and figure commands: `\multirow`, `\rowcolor`, `\cellcolor`, `\caption*`, `\floatplacement`, `\floatbarrier`
- Additional environments: `multicols`, `wrapfigure`, `wraptable`, `tabbing`, `algorithm`/`algorithm*`, `algorithmic`, `alltt`
- Penalty and hyphenation skip commands: `\widowpenalty`, `\clubpenalty`, `\interlinepenalty`, `\hyphenpenalty`, `\exhyphenpenalty`, `\brokenpenalty`, `\floatingpenalty`, `\hyphenation`, `\tolerance`, `\pretolerance`, `\emergencystretch`, `\hbadness`, `\vbadness`

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
| PDF | [pdfrs](https://crates.io/crates/pdfrs) (vendored at `vendor/pdfrs`) — sole backend |
| Math | Unicode mapping (`src/math/`) + pdfrs display layout (`\frac`, `\sqrt` vinculum) |
| Alt formats | HTML/DOCX/EPUB (`src/output/`) |
| HTTP (packages) | ureq + rustls |
| SVG | resvg + usvg |
| WASM | wasm32-unknown-unknown + wasm-bindgen (optional) |

## Quality Bar

- `cargo build` and `cargo test` pass (warnings noted; eliminate when practical)
- 717+ automated tests including round-trip and example PDF verification
- Public APIs documented with rustdoc
- User-facing docs in `README.md`, `docs/USER_GUIDE.md`, `ARCHITECTURE.md`
- Surgical changes: no speculative features beyond `TODO.md`
- PDF backend identifiable via `/Producer` (`rtex/pdfrs`)

## Success Criteria

1. A minimal `\documentclass{article}` document converts to valid PDF/HTML/DOCX/EPUB
2. Math documents render Unicode symbols via embedded DejaVu Sans
3. CLI flags (`--format`, `--watch`, `--fetch-packages`, `--keep-intermediate`) work end-to-end
4. WASM build succeeds with `cargo build --target wasm32-unknown-unknown --features wasm`
