# Round-Trip Testing Strategy for LaTeX → PDF Conversion

## Overview
Since this project currently supports LaTeX → PDF conversion (but not PDF → LaTeX), round-trip testing focuses on:

1. **Deterministic conversion**: Same LaTeX input always produces identical PDF output
2. **Structural validation**: PDFs maintain proper structure and contain expected content
3. **Regression testing**: Golden files detect unintended changes in PDF generation
4. **Edge case handling**: Proper behavior with malformed, minimal, or complex LaTeX inputs

## Round-Trip Definition

**Forward path**: LaTeX → PDF conversion via `convert_tex_to_pdf()`
**Inverse path**: Not currently supported (would require PDF → LaTeX converter)
**Meaning preservation criteria**:
- Same LaTeX input produces bitwise-identical PDF output across multiple runs
- PDF structure is valid (proper headers, xref tables, trailers)
- PDF contains expected text content where verifiable
- File size remains within expected bounds
- No non-deterministic elements (timestamps, random IDs, etc.)

## Test Categories

### 1. Happy Path Tests
Representative production-like LaTeX documents:
- Minimal document structure
- Text content with various formatting
- Mathematical equations (inline and display)
- Lists (ordered and unordered)
- Tables
- Document metadata (title, author, date)
- Complex nested structures

### 2. Edge Case Tests
Boundary conditions and unusual but valid inputs:
- Empty/minimal documents
- Very long text content
- Special characters and Unicode
- Deep nesting of elements
- Multiple sections and subsections
- Mixed content types

### 3. Deterministic Conversion Tests
Verify reproducibility:
- Same document converted multiple times produces identical PDFs
- Multiple conversions of different documents don't interfere
- File contents are bitwise identical (MD5/SHA256 verification)

### 4. Structural Validation Tests
PDF structure verification:
- Valid PDF header (`%PDF-x.x`)
- Proper xref table
- Valid trailer
- EOF marker (`%%EOF`)
- Font embedding
- Content stream structure

### 5. Malformed Input Tests
Behavior with invalid LaTeX:
- Unclosed commands
- Invalid math expressions
- Missing document structure
- Unknown commands
- Unmatched braces

## Fixture Organization

```
tests/fixtures/
├── latex/
│   ├── happy_path/
│   │   ├── minimal.tex
│   │   ├── text_formatting.tex
│   │   ├── mathematics.tex
│   │   ├── lists.tex
│   │   ├── tables.tex
│   │   ├── metadata.tex
│   │   └── complex_document.tex
│   ├── edge_cases/
│   │   ├── empty_document.tex
│   │   ├── long_text.tex
│   │   ├── special_chars.tex
│   │   ├── deep_nesting.tex
│   │   └── mixed_content.tex
│   └── malformed/
│       ├── unclosed_command.tex
│       ├── invalid_math.tex
│       ├── missing_structure.tex
│       └── unknown_commands.tex
└── golden_pdfs/
    ├── minimal.pdf
    ├── text_formatting.pdf
    ├── mathematics.pdf
    └── ...
```

## Test Implementation

### Deterministic Conversion Test
```rust
#[test]
fn test_deterministic_conversion() {
    let latex = r#"\documentclass{article}
\begin{document}
Test content
\end{document}"#;
    
    let pdf1 = convert_latex_to_bytes(latex);
    let pdf2 = convert_latex_to_bytes(latex);
    
    assert_eq!(pdf1, pdf2, "Same input must produce identical output");
}
```

### Structural Validation Test
```rust
#[test]
fn test_pdf_structure() {
    let pdf = convert_latex_to_bytes(latex);
    
    assert!(pdf.starts_with(b"%PDF"), "Missing PDF header");
    assert!(contains_xref_table(&pdf), "Missing xref table");
    assert!(contains_trailer(&pdf), "Missing trailer");
    assert!(pdf.ends_with(b"%%EOF") || contains_eof_marker(&pdf), "Missing EOF marker");
}
```

### Golden File Test
```rust
#[test]
fn test_golden_file_regression() {
    let latex = std::fs::read_to_string("tests/fixtures/latex/happy_path/minimal.tex").unwrap();
    let actual_pdf = convert_latex_to_bytes(&latex);
    let expected_pdf = std::fs::read("tests/fixtures/golden_pdfs/minimal.pdf").unwrap();
    
    assert_eq!(actual_pdf, expected_pdf, "PDF output differs from golden file");
}
```

## Reproducible Commands

### Generate golden files (one-time setup)
```bash
cargo test --test round_trip -- --ignored generate-golden
```

### Run round-trip tests
```bash
cargo test --test round_trip
```

### Update golden files (after intentional changes)
```bash
cargo test --test round_trip -- --ignored update-golden
```

## Success Criteria

✅ All happy path LaTeX files convert to valid PDFs  
✅ Same LaTeX produces identical PDF across multiple runs (deterministic)  
✅ All PDFs pass structural validation  
✅ Golden file tests pass (no regression)  
✅ Edge cases handled gracefully  
✅ Malformed inputs produce appropriate errors or safe fallbacks  

## Known Limitations

- No PDF → LaTeX conversion (true round-trip not possible)
- Text content validation limited (requires PDF parsing library)
- Some visual aspects not verifiable programmatically
- Timestamp in `\today` command causes non-determinism (handled by normalization)