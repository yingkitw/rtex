# Math Processing Learnings from MiniTeX

## Overview
This document captures key learnings from the MiniTeX project's math handling implementation that can improve latex-rs.

## Key Architecture Patterns

### 1. **Modular Math Processing**
MiniTeX separates math processing into focused modules:
- `radicals.rs` - Square root and nth root handling
- `fractions.rs` - Fraction processing with Unicode support
- `superscripts.rs` - Superscript conversion
- `subscripts.rs` - Subscript conversion
- `symbols.rs` - Mathematical symbol mappings

**Benefit**: Better maintainability, easier testing, clearer separation of concerns.

### 2. **Robust Brace Handling**
MiniTeX uses a consistent `extract_braced_content()` helper:
```rust
fn extract_braced_content(text: &str, start: usize) -> Option<(String, usize)> {
    // Returns (content, end_position)
    // Handles nested braces with depth tracking
}
```

**Current latex-rs**: We have similar logic but scattered across methods.
**Improvement**: Extract to a shared utility function.

### 3. **Comprehensive Unicode Mappings**

#### Greek Letters (47 variants)
- Lowercase: α, β, γ, δ, ε, ζ, η, θ, ι, κ, λ, μ, ν, ξ, ο, π, ρ, σ, τ, υ, φ, χ, ψ, ω
- Uppercase: Α, Β, Γ, Δ, Ε, Ζ, Η, Θ, Ι, Κ, Λ, Μ, Ν, Ξ, Ο, Π, Ρ, Σ, Τ, Υ, Φ, Χ, Ψ, Ω
- Variants: \varepsilon, \varphi, \vartheta, etc.

**Current latex-rs**: ~30 symbols
**Improvement**: Add missing Greek variants and uppercase letters.

#### Mathematical Operators (115+ symbols)
Including: ⊕, ⊖, ⊗, ⊘, ⊙, ◯, †, ‡, ⨿, ⋆, ∘, •, ⊎, ⊓, ⊔, ∨, ∧, ∖, ≀, ⋄, etc.

**Current latex-rs**: ~20 operators
**Improvement**: Expand operator coverage significantly.

#### Relations (40+ symbols)
Including: ≺, ⪯, ≪, ⊏, ⊑, ⊢, ≻, ⪰, ≫, ⊐, ⊒, ∋, ⊣, ∼, ≃, ≍, ≅, ≐, ⊨, ⊥, ∣, ∥, ⋈, ⌣, ⌢, ∝

**Current latex-rs**: ~15 relations
**Improvement**: Add comprehensive relation symbols.

#### Arrows (28+ variants)
Including: ↩, ↼, ↽, ⇌, ⟵, ⟸, ⟶, ⟹, ⟷, ⟺, ⟼, ↪, ⇀, ⇁, ⇝, ↗, ↘, ↙, ↖

**Current latex-rs**: ~10 arrows
**Improvement**: Add long arrows and harpoons.

#### Advanced Symbols
- Special fractions: ⅐ (1/7), ⅑ (1/9), ⅒ (1/10)
- Math symbols: ℵ, ℏ, ı, ȷ, ℓ, ℘, ℜ, ℑ, ℧, ′
- Geometric: △, ▽, ◁, ▷, ⊲, ⊳, ⊴, ⊵
- Card suits: ♡, ♢, ♣, ♠
- Music: ♭, ♮, ♯

### 4. **Enhanced Superscript/Subscript Handling**

#### More Complete Character Sets

**Superscripts** (MiniTeX supports):
- All digits: ⁰¹²³⁴⁵⁶⁷⁸⁹
- Letters: ᵃᵇᶜᵈᵉᶠᵍʰⁱʲᵏˡᵐⁿᵒᵖʳˢᵗᵘᵛʷˣʸᶻ
- Symbols: ⁺⁻⁼⁽⁾

**Subscripts** (MiniTeX supports):
- All digits: ₀₁₂₃₄₅₆₇₈₉
- Letters: ₐᵦᶜᵈₑᶠᵍₕᵢⱼₖₗₘₙₒₚᵣₛₜᵤᵥₓ
- Symbols: ₊₋₌₍₎

**Current latex-rs**: Limited character set
**Improvement**: Add full alphabet support for super/subscripts.

#### Fallback Strategy
MiniTeX uses `<sup>` and `<sub>` HTML tags when Unicode isn't available:
```rust
if all_converted {
    result
} else {
    format!("<sup>{}</sup>", text)
}
```

**Current latex-rs**: No fallback mechanism
**Improvement**: Consider fallback for unsupported characters.

### 5. **Fraction Processing Improvements**

#### Unicode Fraction Library
MiniTeX has 15 Unicode fractions:
- ½, ⅓, ⅔, ¼, ¾, ⅕, ⅖, ⅗, ⅘, ⅙, ⅚, ⅐, ⅛, ⅜, ⅝, ⅞, ⅑, ⅒

**Current latex-rs**: Basic Unicode fractions
**Improvement**: Add 1/7, 1/9, 1/10.

#### Smart Parenthesization
```rust
let num_str = if num.len() == 1 || num.chars().all(|c| c.is_alphanumeric()) {
    num.clone()
} else {
    format!("({})", num)
};
```

**Current latex-rs**: Always adds parentheses
**Improvement**: Only add parentheses when needed.

### 6. **Radical (Square Root) Improvements**

#### Special Unicode Roots
- ∛ (cube root) for `\sqrt[3]{x}`
- ∜ (fourth root) for `\sqrt[4]{x}`

**Current latex-rs**: Uses `3√` and `4√`
**Improvement**: Use proper Unicode symbols.

#### Smart Content Wrapping
```rust
if content.len() == 1 {
    content.to_string()
} else {
    format!("({})", content)
}
```

**Current latex-rs**: Always wraps in parentheses
**Improvement**: Only wrap multi-character content.

### 7. **Function Name Support**
MiniTeX handles 25+ function names:
- Trig: arccos, arcsin, arctan, cos, sin, tan, cosh, sinh, tanh, cot, coth, csc, sec
- Other: exp, log, ln, lim, max, min, sup, inf, det, dim, gcd, hom, ker, deg, arg, Pr

**Current latex-rs**: No function name handling
**Improvement**: Add function name recognition.

## Implementation Priorities

### High Priority (Immediate Impact)
1. ✅ **Square root improvements** - Use ∛ and ∜, smart wrapping
2. **Expand Greek letters** - Add uppercase and variants
3. **More superscript/subscript chars** - Full alphabet support
4. **Extract brace handling utility** - DRY principle

### Medium Priority (Enhanced Features)
5. **Mathematical operators** - Expand from 20 to 100+ symbols
6. **Relation symbols** - Add comprehensive set
7. **Arrow variants** - Long arrows, harpoons
8. **Function names** - Recognize common math functions

### Low Priority (Nice to Have)
9. **Fallback mechanism** - HTML tags for unsupported chars
10. **Advanced symbols** - Card suits, music notation, etc.
11. **LazyLock/HashMap** - Performance optimization for symbol lookups

## Code Quality Observations

### What MiniTeX Does Well
1. **Comprehensive testing** - Each module has thorough unit tests
2. **Clear documentation** - Good inline comments and module docs
3. **Modular design** - Each concern is a separate file
4. **Performance** - Uses `LazyLock` for static symbol maps
5. **DRY principle** - Shared utilities like `extract_braced_content()`

### What We Can Improve in latex-rs
1. **Test coverage** - Add more math-specific tests
2. **Documentation** - Document the math formatter module
3. **Modularity** - Consider splitting math_formatter into submodules
4. **Symbol coverage** - Expand from ~60 to 200+ symbols

## Recommended Next Steps

1. **Refactor math_formatter.rs**:
   - Extract `extract_braced_content()` as shared utility
   - Split into submodules: `radicals.rs`, `fractions.rs`, `scripts.rs`, `symbols.rs`

2. **Expand symbol mappings**:
   - Add comprehensive Greek letter support (uppercase + variants)
   - Add 100+ mathematical operators
   - Add 40+ relation symbols
   - Add 25+ arrow variants

3. **Enhance super/subscripts**:
   - Add full alphabet support (a-z for both)
   - Implement fallback mechanism for unsupported chars

4. **Add function name support**:
   - Recognize \sin, \cos, \log, etc.
   - Convert to proper formatting

5. **Improve testing**:
   - Add comprehensive test suite for each math feature
   - Test edge cases (nested expressions, malformed input)

## Conclusion

MiniTeX demonstrates a mature, well-architected approach to math processing. The key takeaways are:
- **Modularity**: Separate concerns into focused modules
- **Completeness**: Comprehensive symbol coverage (200+ symbols)
- **Robustness**: Proper brace handling and edge case management
- **Testing**: Thorough unit tests for each feature
- **Performance**: Efficient symbol lookups with static maps

By applying these patterns, latex-rs can significantly improve its math rendering capabilities while maintaining code quality and maintainability.
