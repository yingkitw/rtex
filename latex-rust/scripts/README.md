# Scripts

This directory contains development and maintenance scripts for the LaTeX-Rust project.

## Available Scripts

### `build.sh`
Comprehensive build script that:
- Cleans previous builds
- Runs all tests
- Performs code quality checks with clippy
- Builds release version
- Runs benchmarks

```bash
./scripts/build.sh
```

### `dev-setup.sh`
Development environment setup script that:
- Checks Rust installation
- Installs required Rust components (clippy, rustfmt)
- Installs useful development tools
- Creates necessary directories
- Runs initial build verification

```bash
./scripts/dev-setup.sh
```

### `test.sh`
Flexible test runner with multiple options:
- `unit` - Run unit tests only
- `integration` - Run integration tests only
- `bench` - Run benchmark tests
- `coverage` - Generate test coverage report
- `all` - Run all tests and quality checks (default)

```bash
./scripts/test.sh [unit|integration|bench|coverage|all]
```

## Usage Examples

```bash
# Set up development environment
./scripts/dev-setup.sh

# Run all tests
./scripts/test.sh

# Run only unit tests
./scripts/test.sh unit

# Generate coverage report
./scripts/test.sh coverage

# Full build with quality checks
./scripts/build.sh
```

## Adding New Scripts

When adding new scripts:
1. Make them executable: `chmod +x scripts/new-script.sh`
2. Add proper error handling with `set -e`
3. Include help/usage information
4. Document the script purpose in this README
5. Follow the existing naming convention