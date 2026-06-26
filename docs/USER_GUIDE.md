# User Guide

## Installation

### From Source

```bash
git clone <repository>
cd latex-rs
cargo build --release
```

The binary will be at `target/release/rtex`.

## Quick Start

Convert a TeX file to PDF:

```bash
rtex input.tex
```

Output is written to `output/input.pdf` by default.

Specify an output path:

```bash
rtex input.tex --output my_document.pdf
```

## Supported LaTeX

### Document Structure

```latex
\documentclass{article}
\title{My Document}
\author{Jane Doe}
\date{2024-01-01}
\begin{document}
\maketitle
\section{Introduction}
Hello, world!
\end{document}
```

### Sections

```latex
\section{Main Section}
\subsection{Subsection}
\subsubsection{Sub-subsection}
```

### Lists

Unordered:
```latex
\begin{itemize}
\item First item
\item Second item
\end{itemize}
```

Ordered:
```latex
\begin{enumerate}
\item Step one
\item Step two
\end{enumerate}
```

### Math

Inline math: `$E = mc^2$`

Display math:
```latex
\begin{equation}
x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\end{equation}
```

### Text Formatting

```latex
\textbf{bold text}
\textit{italic text}
\texttt{monospace text}
```

### Tables (Basic)

```latex
\begin{tabular}{lcc}
\hline
Item & Quantity & Price \\
Apples & 5 & \$2.50 \\
\hline
\end{tabular}
```

### Code Blocks

```latex
\begin{lstlisting}[language=Rust]
fn main() {
    println!("Hello, World!");
}
\end{lstlisting}
```

## Examples

See the `examples/` directory for complete sample documents:

- `minimal.tex` — simplest possible document
- `math.tex` — equations and symbols
- `lists.tex` — itemize and enumerate
- `table.tex` — tabular environments
- `code.tex` — code listings

Generate all examples:

```bash
cargo test example_tests -- --nocapture
```

## Troubleshooting

### "Invalid file path"
Ensure the input `.tex` file exists and is readable.

### Empty or corrupted PDF
Check that your document has `\begin{document}` and `\end{document}`.

### Unsupported commands
Unknown commands are skipped with a warning. If a command is critical, open an issue.

## Limitations

rtex is a native converter and does not require an external LaTeX installation. However, it supports a subset of LaTeX features. Notable limitations:

- No TikZ or PGFPlots graphics
- No BibTeX / bibliography support
- No cross-references (`\label`, `\ref`)
- Limited macro support

For complex documents requiring full LaTeX compatibility, consider using a traditional TeX distribution.
