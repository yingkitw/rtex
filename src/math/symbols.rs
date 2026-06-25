/// Replace LaTeX math commands with Unicode symbols.
///
/// This is the central symbol-mapping table used by `MathFormatter`.
/// It covers Greek letters, operators, relations, arrows, delimiters
/// and common spacing commands.
pub fn replace_math_symbols(text: &str) -> String {
    let mut result = text.to_string();

    // Clean up LaTeX spacing and formatting commands
    result = result.replace("\\,", " "); // thin space
    result = result.replace("\\;", " "); // medium space
    result = result.replace("\\!", ""); // negative thin space
    result = result.replace("\\quad", "  "); // quad space
    result = result.replace("\\qquad", "    "); // double quad
    result = result.replace("\\:", " "); // medium math space
    result = result.replace("\\>", " "); // medium space
    result = result.replace("\\~", " "); // non-breaking space

    // Remove matrix row separators and alignment
    result = result.replace("\\\\", " "); // row separator
    result = result.replace("&", " "); // column separator

    // Differential operators
    result = result.replace("\\mathrm{d}", "d");
    result = result.replace("\\,d", "d");
    result = result.replace("\\dx", "dx");
    result = result.replace("\\dy", "dy");
    result = result.replace("\\dt", "dt");

    // Greek letters - lowercase
    result = result.replace("\\alpha", "α");
    result = result.replace("\\beta", "β");
    result = result.replace("\\gamma", "γ");
    result = result.replace("\\delta", "δ");
    result = result.replace("\\epsilon", "ε");
    result = result.replace("\\varepsilon", "ε");
    result = result.replace("\\zeta", "ζ");
    result = result.replace("\\eta", "η");
    result = result.replace("\\theta", "θ");
    result = result.replace("\\vartheta", "θ");
    result = result.replace("\\iota", "ι");
    result = result.replace("\\kappa", "κ");
    result = result.replace("\\lambda", "λ");
    result = result.replace("\\mu", "μ");
    result = result.replace("\\nu", "ν");
    result = result.replace("\\xi", "ξ");
    result = result.replace("\\pi", "π");
    result = result.replace("\\varpi", "π");
    result = result.replace("\\rho", "ρ");
    result = result.replace("\\varrho", "ρ");
    result = result.replace("\\sigma", "σ");
    result = result.replace("\\varsigma", "σ");
    result = result.replace("\\tau", "τ");
    result = result.replace("\\upsilon", "υ");
    result = result.replace("\\phi", "φ");
    result = result.replace("\\varphi", "φ");
    result = result.replace("\\chi", "χ");
    result = result.replace("\\psi", "ψ");
    result = result.replace("\\omega", "ω");

    // Greek letters - uppercase
    result = result.replace("\\Gamma", "Γ");
    result = result.replace("\\Delta", "Δ");
    result = result.replace("\\Theta", "Θ");
    result = result.replace("\\Lambda", "Λ");
    result = result.replace("\\Xi", "Ξ");
    result = result.replace("\\Pi", "Π");
    result = result.replace("\\Sigma", "Σ");
    result = result.replace("\\Upsilon", "Υ");
    result = result.replace("\\Phi", "Φ");
    result = result.replace("\\Psi", "Ψ");
    result = result.replace("\\Omega", "Ω");

    // Math operators - order matters! Replace longer strings first
    result = result.replace("\\infty", "∞"); // Must come before \in
    result = result.replace("\\int", "∫");
    result = result.replace("\\in", "∈");
    result = result.replace("\\sum", "∑");
    result = result.replace("\\prod", "∏");
    result = result.replace("\\coprod", "∐");
    result = result.replace("\\bigcap", "⋂");
    result = result.replace("\\bigcup", "⋃");
    result = result.replace("\\bigoplus", "⊕");
    result = result.replace("\\bigotimes", "⊗");
    result = result.replace("\\bigodot", "⊙");

    // Binary operators
    result = result.replace("\\pm", "±");
    result = result.replace("\\mp", "∓");
    result = result.replace("\\times", "×");
    result = result.replace("\\cdot", "·");
    result = result.replace("\\ast", "∗");
    result = result.replace("\\star", "⋆");
    result = result.replace("\\circ", "∘");
    result = result.replace("\\bullet", "•");
    result = result.replace("\\diamond", "⋄");
    result = result.replace("\\oplus", "⊕");
    result = result.replace("\\ominus", "⊖");
    result = result.replace("\\otimes", "⊗");
    result = result.replace("\\odot", "⊙");

    // Relations
    result = result.replace("\\leq", "≤");
    result = result.replace("\\geq", "≥");
    result = result.replace("\\ll", "≪");
    result = result.replace("\\gg", "≫");
    result = result.replace("\\prec", "≺");
    result = result.replace("\\succ", "≻");
    result = result.replace("\\preceq", "≼");
    result = result.replace("\\succeq", "≽");
    result = result.replace("\\equiv", "≡");
    result = result.replace("\\sim", "∼");
    result = result.replace("\\simeq", "≃");
    result = result.replace("\\cong", "≅");
    result = result.replace("\\approx", "≈");
    result = result.replace("\\subset", "⊂");
    result = result.replace("\\subseteq", "⊆");
    result = result.replace("\\supset", "⊃");
    result = result.replace("\\supseteq", "⊇");
    result = result.replace("\\in", "∈");
    result = result.replace("\\notin", "∉");
    result = result.replace("\\neq", "≠");
    result = result.replace("\\perp", "⊥");
    result = result.replace("\\parallel", "∥");
    result = result.replace("\\mid", "|");
    result = result.replace("\\models", "⊨");
    result = result.replace("\\propto", "∝");

    // Arrows
    result = result.replace("\\rightarrow", "→");
    result = result.replace("\\leftarrow", "←");
    result = result.replace("\\leftrightarrow", "↔");
    result = result.replace("\\Rightarrow", "⇒");
    result = result.replace("\\Leftarrow", "⇐");
    result = result.replace("\\Leftrightarrow", "⇔");
    result = result.replace("\\longrightarrow", "⟶");
    result = result.replace("\\longleftarrow", "⟵");
    result = result.replace("\\Longrightarrow", "⟹");
    result = result.replace("\\Longleftarrow", "⟸");
    result = result.replace("\\uparrow", "↑");
    result = result.replace("\\downarrow", "↓");
    result = result.replace("\\Uparrow", "⇑");
    result = result.replace("\\Downarrow", "⇓");
    result = result.replace("\\updownarrow", "↕");
    result = result.replace("\\nearrow", "↗");
    result = result.replace("\\searrow", "↘");
    result = result.replace("\\swarrow", "↙");
    result = result.replace("\\nwarrow", "↖");
    result = result.replace("\\mapsto", "↦");
    result = result.replace("\\to", "→");

    // Special symbols
    result = result.replace("\\infty", "∞");
    result = result.replace("\\aleph", "ℵ");
    result = result.replace("\\hbar", "ℏ");
    result = result.replace("\\ell", "ℓ");
    result = result.replace("\\nabla", "∇");
    result = result.replace("\\partial", "∂");
    result = result.replace("\\angle", "∠");
    result = result.replace("\\emptyset", "∅");
    result = result.replace("\\forall", "∀");
    result = result.replace("\\exists", "∃");
    result = result.replace("\\neg", "¬");
    result = result.replace("\\land", "∧");
    result = result.replace("\\lor", "∨");
    result = result.replace("\\top", "⊤");
    result = result.replace("\\bot", "⊥");

    // Delimiters (remove)
    result = result.replace("\\left", "");
    result = result.replace("\\right", "");
    result = result.replace("\\big", "");
    result = result.replace("\\Big", "");
    result = result.replace("\\bigg", "");
    result = result.replace("\\Bigg", "");

    // Text commands
    result = result.replace("\\text{", "");
    result = result.replace("\\mathrm{", "");
    result = result.replace("\\mathbf{", "");
    result = result.replace("\\mathit{", "");
    result = result.replace("\\mathcal{", "");

    // Clean up extra backslashes and braces
    result = result.replace("\\\\", "");
    result = result.replace("\\{", "{");
    result = result.replace("\\}", "}");

    result
}

#[cfg(test)]
mod tests {
    use super::replace_math_symbols;

    #[test]
    fn test_greek_letters() {
        assert_eq!(replace_math_symbols("\\alpha"), "α");
        assert_eq!(replace_math_symbols("\\beta"), "β");
        assert_eq!(replace_math_symbols("\\Gamma"), "Γ");
        assert_eq!(replace_math_symbols("\\Delta"), "Δ");
    }

    #[test]
    fn test_operators() {
        assert_eq!(replace_math_symbols("\\int"), "∫");
        assert_eq!(replace_math_symbols("\\sum"), "∑");
        assert_eq!(replace_math_symbols("\\infty"), "∞");
        assert_eq!(replace_math_symbols("\\pm"), "±");
        assert_eq!(replace_math_symbols("\\times"), "×");
    }

    #[test]
    fn test_relations() {
        assert_eq!(replace_math_symbols("\\leq"), "≤");
        assert_eq!(replace_math_symbols("\\geq"), "≥");
        assert_eq!(replace_math_symbols("\\neq"), "≠");
        assert_eq!(replace_math_symbols("\\in"), "∈");
    }

    #[test]
    fn test_arrows() {
        assert_eq!(replace_math_symbols("\\rightarrow"), "→");
        assert_eq!(replace_math_symbols("\\Leftarrow"), "⇐");
        assert_eq!(replace_math_symbols("\\to"), "→");
    }

    #[test]
    fn test_spacing_removed() {
        assert_eq!(replace_math_symbols("a\\,b"), "a b");
        assert_eq!(replace_math_symbols("a\\;b"), "a b");
        assert_eq!(replace_math_symbols("a\\!b"), "ab");
    }

    #[test]
    fn test_multiple_symbols() {
        let result = replace_math_symbols("\\alpha + \\beta = \\gamma");
        assert_eq!(result, "α + β = γ");
    }

    #[test]
    fn test_infinity_before_in() {
        // \infty must be replaced before \in to avoid partial matches
        assert_eq!(replace_math_symbols("\\infty"), "∞");
    }
}
