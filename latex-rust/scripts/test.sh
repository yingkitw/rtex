#!/bin/bash
# Test runner script for LaTeX-Rust project

set -e

# Default to running all tests
TEST_TYPE="all"

if [ $# -gt 0 ]; then
    TEST_TYPE="$1"
fi

case "$TEST_TYPE" in
    "unit")
        echo "Running unit tests..."
        cargo test --lib
        ;;
    "integration")
        echo "Running integration tests..."
        cargo test --test '*'
        ;;
    "bench")
        echo "Running benchmark tests..."
        cargo bench
        ;;
    "coverage")
        echo "Running tests with coverage..."
        if command -v cargo-tarpaulin &> /dev/null; then
            cargo tarpaulin --out Html --output-dir coverage
            echo "Coverage report generated in coverage/tarpaulin-report.html"
        else
            echo "cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"
            exit 1
        fi
        ;;
    "all")
        echo "Running all tests..."
        cargo test
        echo "Running clippy..."
        cargo clippy -- -D warnings
        echo "Checking formatting..."
        cargo fmt -- --check
        ;;
    *)
        echo "Usage: $0 [unit|integration|bench|coverage|all]"
        echo "  unit        - Run unit tests only"
        echo "  integration - Run integration tests only"
        echo "  bench       - Run benchmark tests"
        echo "  coverage    - Generate test coverage report"
        echo "  all         - Run all tests and quality checks (default)"
        exit 1
        ;;
esac

echo "Test run completed successfully!"