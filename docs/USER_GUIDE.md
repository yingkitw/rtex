# User Guide

## Installation

```bash
git clone https://github.com/yingkitw/rtex
cd rtex
cargo build --release
```

Binary: `target/release/rtex`

## Quick Start

```bash
# Default: PDF in output/input.pdf
rtex examples/minimal.tex

# Custom output path
rtex examples/math.tex -o my_math.pdf

# Other formats
rtex examples/minimal.tex --format html
rtex examples/sample.tex --format docx -o report.docx
rtex examples/sample.tex --format epub
```

## CLI Reference

| Flag | Description |
|------|-------------|
| `-o, --output PATH` | Output file path |
| `-f, --format FORMAT` | `pdf` (default), `html`, `docx`, `epub` |
| `--fetch-packages` | Download missing `.sty`/`.cls` from CTAN |
| `--package-cache DIR` | Package cache directory (default: `.rtex/cache`) |
| `--keep-intermediate` | Write `.expanded.tex`, `.ast.json`, `.meta.json` |
| `--watch` | Recompile on file changes (all formats) |
| `--template PATH` | TOML document template for styling |
| `--force` | Force rebuild despite incremental cache |
| `--no-incremental` | Disable incremental compilation |

### Examples

```bash
# Watch mode with template
rtex thesis.tex --watch --template thesis.toml

# Fetch packages then convert
rtex paper.tex --fetch-packages

# Debug parse tree
rtex doc.tex --keep-intermediate
# → output/doc.pdf, output/doc.ast.json, output/doc.expanded.tex
```

## Library API

```rust
use rtex::{convert_tex_file, convert_tex_string, ConversionOptions, OutputFormat};

// File conversion
let options = ConversionOptions::default()
    .with_format(OutputFormat::Html)
    .with_keep_intermediate(true);
convert_tex_file("doc.tex".as_ref(), "doc.html".as_ref(), &options)?;

// In-memory (WASM-friendly)
let pdf = rtex::convert_tex_string_to_pdf_bytes(r"\documentclass{article}...")?;
let html = convert_tex_string(tex, OutputFormat::Html)?;
```

## Supported LaTeX

### Document structure

```latex
\documentclass{article}
\title{My Document}
\author{Jane Doe}
\date{\today}
\begin{document}
\maketitle
\tableofcontents
\section{Introduction}
Content here.
\appendix
\section{Extra}
\end{document}
```

### Text formatting

`\textbf`, `\textit`, `\texttt`, `\underline`, `\emph`, `\textsc`, `\textcolor{color}{text}`, `\colorbox`, `\fcolorbox`

### Math (566+ symbols)

Inline: `$E = mc^2$` — Display: `\begin{equation}...\end{equation}`

Supported: fractions, roots, matrices, Greek letters, operators, `\mathbb`, `\mathcal`, `\mathfrak`, accents (`\vec`, `\hat`, `\bar`, …).

### Lists, tables, figures

- `itemize`, `enumerate` (with optional `\item[label]`)
- `tabular` with `l/c/r` columns and booktabs rules
- `\includegraphics` for PNG, JPEG, and SVG

### References and bibliography

```latex
\label{sec:intro}
See Section~\ref{sec:intro} on page~\pageref{sec:intro}.

\cite{smith2020}

\begin{thebibliography}{9}
\bibitem{smith2020} Smith, J. (2020). Example.
\end{thebibliography}
```

BibTeX `.bib` file parsing is available via the library (`BibliographyManager`).

### Macros

```latex
\newcommand{\kw}[1]{\textbf{#1}}
\def\laplace#1{\mathcal{L}\{#1\}}
```

### Environments

`center`, `quote`, `quotation`, `abstract`, `equation`, `lstlisting`, `table`+`tabular`

## Templates

Create a TOML template and pass `--template`:

```toml
# thesis.toml — see src/template.rs for full schema
[page]
size = "a4"
orientation = "portrait"

[margins]
top = 72.0
bottom = 72.0
left = 72.0
right = 72.0
```

## WebAssembly

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --features wasm --release
wasm-bindgen target/wasm32-unknown-unknown/release/rtex.wasm --out-dir pkg --target web
```

JavaScript export: `convertTexToPdf(texSource)` → `Uint8Array` PDF bytes.

## Examples

| File | Demonstrates |
|------|--------------|
| `examples/minimal.tex` | Smallest valid document |
| `examples/math.tex` | Equations and symbols |
| `examples/table.tex` | Tabular + booktabs |
| `examples/lists.tex` | Itemize/enumerate |
| `examples/code.tex` | lstlisting |
| `examples/sample.tex` | Combined features |

```bash
cargo test example_tests -- --nocapture   # generate example PDFs
./generate_examples.sh                    # batch script
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `Invalid file path` | Check input `.tex` exists and is readable |
| Empty PDF | Ensure `\begin{document}` / `\end{document}` present |
| Missing image | Use paths relative to `.tex` file; check PNG/JPEG/SVG |
| Unknown command | Command may be unsupported; try `--keep-intermediate` to inspect AST |
| Package not found | Run with `--fetch-packages` or install manually into cache |

## Limitations

See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) for the full list. Summary:

- Not a full LaTeX engine — no TikZ, beamer, or arbitrary package execution
- DOCX/EPUB/HTML are functional first versions, not pixel-perfect LaTeX replicas
- PDF file size ~750 bytes for ASCII-only docs; ~385 KB when Unicode/math requires embedded font
- Package fetching uses CTAN mirror heuristics; exotic layouts may fail

For production documents requiring 100% LaTeX compatibility, use Tectonic or TeX Live.
