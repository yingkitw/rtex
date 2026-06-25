#!/bin/bash
# Development setup script for LaTeX-Rust project

set -e

echo "Setting up LaTeX-Rust development environment..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"

# Install required components
echo "Installing Rust components..."
rustup component add clippy rustfmt

# Install cargo tools for development
echo "Installing development tools..."
cargo install cargo-watch cargo-expand cargo-audit

# Create necessary directories if they don't exist
echo "Creating project directories..."
mkdir -p target
mkdir -p logs

# Run initial build to check everything works
echo "Running initial build..."
cargo check

echo "Development environment setup completed!"
echo "You can now run:"
echo "  cargo run -- --help    # Run the CLI"
echo "  cargo test             # Run tests"
echo "  cargo watch -x test    # Watch for changes and run tests"
echo "  ./scripts/build.sh     # Full build with quality checks"