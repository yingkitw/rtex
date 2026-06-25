#!/bin/bash
# Build script for LaTeX-Rust project

set -e

echo "Building LaTeX-Rust project..."

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Run tests
echo "Running tests..."
cargo test

# Run clippy for code quality
echo "Running clippy..."
cargo clippy -- -D warnings

# Build in release mode
echo "Building release version..."
cargo build --release

# Run benchmarks
echo "Running benchmarks..."
cargo bench

echo "Build completed successfully!"
echo "Binary location: target/release/latex-rust"