#!/bin/bash

# Script to generate example PDFs using native Rust converter

echo "Building latex-rs (native TeX to PDF converter)..."
cargo build --release

echo ""
echo "Generating PDFs in output/ directory..."
echo ""

for tex_file in examples/*.tex; do
    filename=$(basename "$tex_file" .tex)
    echo "Converting $tex_file -> output/$filename.pdf"
    ./target/release/latex-rs "$tex_file"
done

echo ""
echo "Done! Check the output/ directory for generated PDFs:"
ls -lh output/*.pdf 2>/dev/null || echo "No PDFs generated"
