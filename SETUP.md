# Setup Guide

## Quick Start

### 1. Build the Project

**No external dependencies required!** rtex is a native Rust implementation that doesn't need pdflatex or any LaTeX installation.

```bash
cd /Users/yingkitw/Desktop/myproject/latex-rs
cargo build --release
```

### 2. Generate Example PDFs

Generate all example PDFs using the native converter:

```bash
./generate_examples.sh
```

Or generate them individually:

```bash
cargo run --release -- examples/minimal.tex
# Creates: output/minimal.pdf

cargo run --release -- examples/math.tex
# Creates: output/math.pdf
```

## Output Directory

By default, all PDFs are generated in the `output/` directory, which is:
- Created automatically if it doesn't exist
- Ignored by git (see `.gitignore`)
- Located at: `/Users/yingkitw/Desktop/myproject/latex-rs/output/`

## Custom Output Location

You can specify a custom output path:

```bash
cargo run --release -- examples/sample.tex -o my_document.pdf
```

## Troubleshooting

### Permission denied on generate_examples.sh

Make the script executable:
```bash
chmod +x generate_examples.sh
```

### Build errors

Make sure you have the latest Rust toolchain:
```bash
rustup update
```

## Verifying Installation

Run the test suite:
```bash
cargo test
# Should show: 317 passed
```

Generate a test PDF:
```bash
cargo run -- examples/minimal.tex
ls -lh output/minimal.pdf
```

The generated PDF should be viewable in any PDF reader!
