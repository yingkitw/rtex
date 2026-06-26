# Converting Examples

This directory contains simple scripts to convert all LaTeX example files to PDFs.

## Python Script

The `convert_examples.py` script converts all `.tex` files in the `examples/` directory to PDFs in the `output/` directory:

```bash
python3 convert_examples.py
```

## Rust Test

The built-in test verifies that all examples can be converted:

```bash
cargo test test_convert_all_examples -- --nocapture
```

## Generated Files

After running either method, you'll find PDF files in the `output/` directory:

- `simple_test.pdf` - Basic text test
- `code.pdf` - Code listings example
- `table.pdf` - Table example
- `minimal.pdf` - Minimal LaTeX document
- `quadratic.pdf` - Quadratic equation example
- `lists.pdf` - Lists example
- `sample.pdf` - Sample document
- `math.pdf` - Math example
- `advanced_math.pdf` - Advanced math example

All PDFs are generated with compressed fonts (~375KB each) and include proper text extraction support.
