# TODO

## Completed

All foundational phases are done. Key milestones:

- **Native TeX parser** — recursive-descent parser, no pdflatex dependency
- **PDF generation** — vendored pdfrs engine with DejaVu Sans font embedding and subsetting
- **Multi-format output** — PDF, HTML, DOCX, EPUB from single parsed AST
- **Math support** — 618 Unicode symbols, fractions, radicals, scripts, accents, alphabets, matrix environments, function names, MathML export
- **LaTeX coverage** — 450+ commands, multi-line math (`align`/`gather`/`multline`/`cases`), theorems, `description` lists, `\href`, unnumbered sections, `\binom`/`\tfrac`/`\dfrac`, 42+ special characters, 55+ skip-commands, counter formatting, `\input`/`\include` file splicing, page break commands (`\newpage`/`\clearpage`/`\pagebreak`/`\noindent`/`\newline`/`\raggedright`/`\raggedleft`/`\onecolumn`/`\twocolumn`), `\marginpar`
- **Bibliography & cross-references** — `thebibliography`, `\cite`, `\label`/`\ref`/`\pageref` with `RefStore`
- **Macros** — `\newcommand`, `\renewcommand`, `\def` with iterative expansion
- **Images** — PNG/JPEG/SVG via `\includegraphics` with dimension parsing
- **Tables** — `tabular` with alignment, booktabs rules, `\multicolumn`/`\cline`
- **Colors** — 12 named colors, hex/RGB parsing, `\textcolor`, `\colorbox`/`\fcolorbox`
- **Layout** — automatic page breaks, multi-page PDFs, A4/Letter presets
- **Incremental compilation** — content-hash caching, source-hash + dependency mtime tracking
- **Watch mode** — file polling with dependency-aware rebuilds
- **Streaming** — chunked file reading with progress reporting
- **Plugin system** — `Plugin` trait with command/environment/transform hooks
- **LSP** — diagnostics, completion, hover, document symbols (`rtex-lsp` binary)
- **WASM** — `convert_tex_string_to_pdf_bytes()` + `wasm-bindgen` exports
- **CTAN package fetching** — on-demand `.sty`/`.cls` download from CTAN mirrors
- **Shell completions** — bash/zsh/fish/elvish/powershell via `clap_complete`
- **Intermediate artifacts** — `--keep-intermediate` writes expanded TeX, AST JSON, metadata
- **PDF optimization** — font subsetting, FlateDecode compression, minimal object graph
- **Code quality audit** — zero clippy warnings, dead code removed, `thiserror` v2, `anyhow` removed
- **Expanded math accents** — 19 accent commands (`\vec`, `\hat`, `\tilde`, `\bar`, `\dot`, `\ddot`, `\check`, `\breve`, `\acute`, `\grave`, `\mathring`, `\widehat`, `\widetilde`, `\overrightarrow`, `\overleftarrow`, `\overline`, `\underline`, `\dddot`, `\ddddot`) with Unicode combining characters
- **Expanded ref/cite commands** — `\eqref`, `\autoref`, `\nameref`, `\cref`, `\Cref`, `\nocite`
- **Expanded skip commands** — `\pagestyle`, `\thispagestyle`, `\pagenumbering`, `\definecolor`
- **Math style switches** — `\displaystyle`, `\textstyle`, `\scriptstyle`, `\scriptscriptstyle` as no-ops
- **Math text commands** — `\boldsymbol`, `\pmb`, `\operatorname` strip braces and output content inline
- **Expanded math symbol table** — 50+ new symbols: negated relations (`\nleqslant`, `\ngeqslant`, `\nsubset`, `\nsupset`), arrows (`\dashrightarrow`, `\multimap`, `\upuparrows`, `\xrightarrow`), operators (`\dotminus`, `\bmod`, `\mod`, `\pod`), function names (`\arcsec`, `\arccsc`, `\arccot`, `\arcsinh`, `\arccosh`, `\arctanh`), geometry (`\vartriangle`, `\blacktriangle`, `\Box`, `\Diamond`), misc (`\checkmark`, `\ballotbox`, `\maltese`), sized delimiters (`\bigl`, `\bigr`, `\Bigl`, `\Bigr`)
- **Expanded MathML dispatch** — `\mod`, `\pod`, `\boxed`, `\substack`, `\boldsymbol`/`\pmb` with `mathvariant="bold"`, `\displaystyle`/`\textstyle`/`\scriptstyle`/`\scriptscriptstyle` no-ops, `\limits`/`\nolimits`/`\displaylimits` no-ops, additional function names
- **Expanded parser text commands** — `\textcircled`, `\hl`, `\st`, `\uline`, `\uuline`, `\uwave`, `\dotuline`, `\color` declaration, `\normalcolor`
- **Expanded skip commands** — `\let`, `\edef`, `\xdef`, `\global`, `\frenchspacing`, `\nonfrenchspacing`, `\thicklines`, `\thinlines`, `\baselineskip`, `\topskip`, `\bottomskip`, `\parindent`, `\parskip`, `\headheight`, `\headsep`, `\footskip`, `\topmargin`, `\bottommargin`, `\leftmargin`, `\rightmargin`, `\oddsidemargin`, `\evensidemargin`, `\marginparwidth`, `\marginparsep`, `\marginparpush`, `\floatsep`, `\intextsep`, `\textfloatsep`, `\abovecaptionskip`, `\belowcaptionskip`, `\counterwithout`, `\mathversion`, `\restoremathversion`, `\sloppypar`
- **Final command batch (400+ target reached)** — `\not`, `\limits`, `\nolimits`, `\displaylimits`, `\mathclap`/`\mathllap`/`\mathrlap`, `\boxed`, `\substack`, `\cr`, `\tag`/`\tag*`, `\intertext`/`\shortintertext`, `\reflectbox`, `\resizebox`, 20+ additional skip commands (`\nonumber`, `\notag`, `\theoremstyle`, `\qedhere`, `\swapnumbers`, `\qedsymbol`, `\settowidth`, `\settodepth`, `\settoheight`, `\sbox`, `\savebox`, `\usebox`, `\adjustbox`, `\captionof`, `\subfloat`, `\subcaption`)
- **Bug fix** — `\ref` check no longer shadows `\reflectbox` (prefix collision fix)
- **Biblatex citation commands** — `\textcite`, `\parencite`, `\footcite`, `\citeauthor`, `\citeyear`, `\citetitle`, `\fullcite` with kind-aware rendering (parenthetical, footnote, author-only, etc.); `\printbibliography` with optional `[...]` arg consumption; `\addbibresource{file.bib}`
- **Missing common environments** — `tabular*`, `tabularx`, `array`, `comment`, `subfigure`/`subfig`, `table*`
- **Table and figure commands** — `\multirow`, `\rowcolor`, `\cellcolor`, `\caption*`, `\floatplacement`, `\floatbarrier`
- **Spacing and text commands** — `\fill` (spacing), `\stretch{n}` (skip), `\textellipsis` (Unicode ellipsis), HTML rendering for `\dotfill`, `\strut`, `\mathstrut`
- **Additional environments** — `multicols`, `wrapfigure`, `wraptable`, `tabbing`, `algorithm`/`algorithm*`, `algorithmic`, `alltt`
- **Penalty and hyphenation skip commands** — `\widowpenalty`, `\clubpenalty`, `\interlinepenalty`, `\hyphenpenalty`, `\exhyphenpenalty`, `\brokenpenalty`, `\floatingpenalty`, `\hyphenation`, `\tolerance`, `\pretolerance`, `\emergencystretch`, `\hbadness`, `\vbadness`
- **Text symbols** — `\textunderscore`, `\textdegree`, `\textcelsius`, `\textmu`, `\textohm`, `\textnumero`, `\textestimated`, `\textbrokenbar`, `\textordfeminine`, `\textordmasculine`, `\textacutedbl`, `\textgravedbl`, `\texttildelow`, `\textcent`, `\texteuro`, `\textyen`, `\textcurrency`
- **siunitx commands** — `\SI{num}{unit}`, `\si{unit}`, `\unit{unit}`, `\num{num}`, `\SIrange{start}{end}{unit}`, `\sisetup{opts}` (skip) with HTML and PDF rendering
- **Inline verbatim and misc commands** — `\verb<delim>text<delim>`, `\verb*`, `\lstinline<delim>text<delim>`, `\smash{text}`, `\nolinkurl{url}`

## Pending

### Future Features

- [ ] Add support for other LaTeX engines (xelatex, lualatex)
- [ ] Advanced color management (ICC profiles)
- [ ] PDF/A compliance
- [ ] Accessibility features (tagged PDF)
- [ ] Internationalization (i18n)
- [ ] Web interface
- [ ] GUI application

### Known Technical Debt

- [ ] Refactor `parse_next()` (1015 lines) into sub-functions — blocked by borrow checker (`remaining` borrows from `self.content`, conflicts with `&mut self` calls); deferred until Polonius

## Competitive Intelligence

### Completed competitive features

- [x] Bibliography / citation support (Pandoc, Tectonic)
- [x] Cross-references (`\label`/`\ref`/`\pageref`)
- [x] Incremental / cached compilation (Tectonic)
- [x] WASM target (Typst)
- [x] Multiple output formats (Pandoc)
- [x] On-demand package fetching (Tectonic)
- [x] PDF metadata (Info dictionary)
- [x] Real-time preview / watch mode
- [x] Language Server Protocol (LSP)
- [x] Shell completions
- [x] MathML export (Typst 0.15)
- [x] `description` environment
- [x] Multi-line math environments
- [x] `\href` hyperlinks
- [x] Theorem-like environments
- [x] Unnumbered sections
- [x] Sized fractions and binomial coefficients
- [x] Comprehensive LaTeX syntax coverage (special chars, skip-commands, counter formatting, file splicing)
- [x] Divider element (`\hrulefill`)

### Brainstorming (future competitive features)

- **Collaborative editing** — CRDT or OT layer on parsed AST (Overleaf)
- **Formula OCR input** — photo/screenshot → LaTeX (Mathpix competitor)
- **SyncTeX support** — write `.synctex.gz` for editor forward/reverse search (Tectonic 0.16, useful with LSP)
- **Variable fonts** — support OpenType variable font axes (Typst 0.15)
- **Multiple PDF standards** — produce PDF/A + PDF/UA simultaneously (Typst 0.15)
- **Bundle/multi-file export** — output multiple files from a single source (Typst 0.15)
- **Multiple bibliographies** — support multiple `\bibliography` commands with citation routing (Typst 0.15)
- **Layout convergence diagnostics** — warn when layout doesn't converge (Typst 0.15)
- **Spot colors** — custom pigment definitions for offset printing (Typst 0.15)

## Metrics & Goals

### Current Status
- Tests: 717 (see `cargo test`)
- PDF: vendored pdfrs (`vendor/pdfrs`); `/Producer` stamps `rtex/pdfrs`
- Math: 618 Unicode symbols + pdfrs display layout for `\frac` / `\sqrt` (vinculum) / `pmatrix`/`bmatrix`/`vmatrix` grid
- LaTeX commands: 450+
- Documentation: README / ARCHITECTURE / SPEC aligned with current implementation
- File size: ~750 bytes for ASCII-only PDFs; ~385 KB for Unicode/math (DejaVu subset)
- Clippy: 10 warnings (pre-existing, in pdfrs and parser)

### Target Goals
- Tests: 717 ✓ (target was 600+)
- Math symbols: 618 ✓ (target was 600+)
- LaTeX commands: 450+ ✓ (target was 400+)
- File size: <100 KB for ASCII-only; <500 KB for Unicode/math
- PDF standards: PDF/A compliance
- Math: MathML in all output formats, not just HTML
