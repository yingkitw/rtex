# Font Embedding Status

> **Historical document.** The `printpdf`/`lopdf` backends described here have been replaced by the vendored [pdfrs](https://crates.io/crates/pdfrs) engine, which handles all PDF generation including Unicode font embedding. This file is kept for reference only.

## Current Implementation

### What Was Done
1. ✅ Added DejaVu Sans TrueType font (290KB)
2. ✅ Added `rusttype` dependency for font parsing
3. ✅ Modified PDF builder to load external font
4. ✅ Font successfully embeds in PDF (verified with `pdffonts`)

### Current Issue

**Problem**: Unicode mathematical symbols are not displaying correctly in the PDF despite:
- Font being properly embedded (DejaVu Sans, TrueType, Identity-H encoding)
- Math formatter correctly converting LaTeX → Unicode (α, β, ∑, ∫, etc.)
- PDF file size increasing (indicating font is embedded)

**Root Cause**: The `printpdf` library's `use_text()` method appears to not properly encode Unicode characters when writing to the PDF text stream. When extracting text from the PDF with `pdftotext`, we see the original LaTeX commands (\alpha, \beta) instead of Unicode symbols.

### Technical Details

**Font Verification**:
```bash
$ pdffonts output/advanced_math.pdf
name     type         encoding     emb sub uni object ID
F0       CID TrueType Identity-H   yes no  yes   1  0
```

**Text Extraction Test**:
```bash
$ pdftotext output/simple_test.pdf -
Test inline math: \alpha + \beta = \gamma  # Should show: α + β = γ
Test symbols: \sum, \int, \pi              # Should show: ∑, ∫, π
```

The Unicode characters are being passed to `use_text()` but not properly encoded in the PDF.

## Limitations of printpdf

The `printpdf` crate (v0.7) has known limitations with Unicode text:
1. Limited Unicode support in text rendering
2. Character encoding issues with TrueType fonts
3. No built-in support for complex text layout
4. Identity-H encoding may not map correctly to Unicode codepoints

## Possible Solutions

### Option 1: Use lopdf directly (Low-level)
- Manually construct PDF text objects with proper Unicode encoding
- More control but significantly more complex
- Would require deep PDF format knowledge

### Option 2: Use pdf-writer crate
- More modern PDF generation library
- Better Unicode support
- Would require rewriting PDF builder

### Option 3: Use genpdf crate
- Higher-level PDF generation
- Better text handling
- May have better Unicode support

### Option 4: Wait for printpdf updates
- printpdf is actively maintained
- Future versions may improve Unicode support
- Not a solution for current needs

### Option 5: Hybrid approach
- Keep printpdf for structure
- Use raw PDF operators for Unicode text
- Moderate complexity

## Recommendation

Given the complexity of proper Unicode support in PDF generation, the most practical approaches are:

**Short-term**: Document the limitation and recommend pdflatex for math-heavy documents

**Long-term**: 
1. Evaluate switching to `pdf-writer` or `genpdf` crate
2. Or contribute Unicode text improvements to `printpdf`
3. Or implement custom PDF text encoding for Unicode

## Current Status

- ✅ Font embedding works
- ❌ Unicode text rendering doesn't work with printpdf 0.7
- ⚠️ Math symbols convert correctly but don't display in PDF

The implementation is 80% complete - we have the font and the symbols, but the PDF library doesn't properly encode them.
