# Known Limitations

## Font Support for Mathematical Symbols

### Current Limitation
The native TeX to PDF converter uses built-in PDF fonts (Helvetica) which have **limited Unicode support**. While the math formatter correctly converts LaTeX commands to Unicode symbols (e.g., `\alpha` → α, `\sum` → ∑), these symbols may not display correctly in the generated PDF.

### What Works
- Basic ASCII characters and numbers
- Simple superscripts and subscripts (digits)
- Common symbols that exist in Helvetica

### What May Not Display
- Greek letters (α, β, γ, etc.)
- Advanced mathematical operators (∑, ∏, ∫, etc.)
- Special Unicode symbols (⊕, ⊗, ∇, etc.)
- Full alphabet super/subscripts

### Why This Happens
Built-in PDF fonts (Type 1 fonts like Helvetica) have limited character sets. They were designed for Western European languages and don't include the full Unicode mathematical symbol range.

## Solutions

### Option 1: Use External LaTeX (Recommended for Production)
For production use with full mathematical typesetting, use a proper LaTeX distribution:
```bash
pdflatex your_document.tex
```

### Option 2: Accept Limitations (Current Implementation)
The current native converter is suitable for:
- Simple documents without complex math
- Quick previews
- Documents with minimal mathematical notation
- Testing and development

### Option 3: Future Enhancement - External Fonts
To fully support Unicode math symbols, we would need to:
1. Embed TrueType/OpenType fonts that support mathematical symbols
2. Use fonts like:
   - STIX fonts (comprehensive math support)
   - Latin Modern Math
   - Cambria Math
   - DejaVu fonts

This requires:
- Adding font file dependencies
- Implementing font embedding in PDF
- Significantly larger PDF file sizes
- More complex build process

## Workaround for Current Version

For documents that need to be readable, you can:

1. **Use LaTeX command names in text**:
   ```latex
   The sum from i=1 to n equals n(n+1)/2
   ```
   Instead of: `$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$`

2. **Use ASCII approximations**:
   - α → alpha
   - ∑ → SUM
   - ∫ → INT
   - → → ->

3. **Use external LaTeX for final documents**:
   - Use latex-rs for quick drafts
   - Use pdflatex for final output

## Comparison

| Feature | latex-rs (native) | pdflatex |
|---------|------------------|----------|
| Installation | Rust only | Full LaTeX distribution |
| Speed | Fast | Slower |
| File size | Small | Larger |
| Math symbols | Limited (font dependent) | Full support |
| Packages | None | Thousands |
| Use case | Simple docs, previews | Production documents |

## Future Roadmap

Potential improvements (not yet implemented):
1. Add option to embed Unicode-capable fonts
2. Implement font subsetting to reduce file size
3. Add configuration for font selection
4. Support external font files
5. Implement proper font metrics and kerning

## Recommendation

**For now**: Use latex-rs for simple documents and quick previews. For documents with mathematical notation, use a proper LaTeX distribution (pdflatex, xelatex, or lualatex) for the final output.

The native converter's strength is its simplicity and zero external dependencies, not comprehensive mathematical typesetting.
