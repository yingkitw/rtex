/// Replace LaTeX math commands with Unicode symbols.
///
/// Uses a single-pass scanner for O(n) performance instead of
/// O(n·m) sequential `String::replace` calls.
pub fn replace_math_symbols(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut iter = text.chars().peekable();

    while let Some(ch) = iter.next() {
        if ch == '\\' {
            match iter.peek() {
                Some(&'\\') => {
                    iter.next();
                    result.push(' ');
                }
                Some(&'{') => {
                    iter.next();
                    result.push('{');
                }
                Some(&'}') => {
                    iter.next();
                    result.push('}');
                }
                Some(&',') | Some(&';') | Some(&':') | Some(&'>') | Some(&'~') => {
                    iter.next();
                    result.push(' ');
                }
                Some(&'!') => {
                    iter.next(); // negative space — remove
                }
                Some(&next_ch) if next_ch.is_alphabetic() => {
                    let mut name = String::new();
                    name.push(next_ch);
                    iter.next();

                    while let Some(&c) = iter.peek() {
                        if c.is_alphabetic() || c == '*' {
                            name.push(c);
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    // Text commands: strip the command and consume the opening brace
                    if matches!(name.as_str(), "text" | "mathrm" | "mathbf" | "mathit" | "mathcal") {
                        if let Some(&'{') = iter.peek() {
                            iter.next();
                        }
                        continue;
                    }

                    match lookup_symbol(&name) {
                        Some(repl) => result.push_str(repl),
                        None => {
                            result.push('\\');
                            result.push_str(&name);
                        }
                    }
                }
                _ => result.push('\\'),
            }
        } else if ch == '&' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    result
}

fn lookup_symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        // Differential operators
        "mathrm" => return Some(""), // handled above, but fallback
        "dx" => "dx",
        "dy" => "dy",
        "dt" => "dt",

        // Greek — lowercase
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "varepsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "θ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "varpi" => "π",
        "rho" => "ρ",
        "varrho" => "ρ",
        "sigma" => "σ",
        "varsigma" => "σ",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "varphi" => "φ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",

        // Greek — uppercase
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",

        // Operators — longest first to avoid shadowing
        "infty" => "∞",
        "int" => "∫",
        "in" => "∈",
        "sum" => "∑",
        "prod" => "∏",
        "coprod" => "∐",
        "bigcap" => "⋂",
        "bigcup" => "⋃",
        "bigoplus" => "⊕",
        "bigotimes" => "⊗",
        "bigodot" => "⊙",

        // Binary operators
        "pm" => "±",
        "mp" => "∓",
        "times" => "×",
        "cdot" => "·",
        "ast" => "∗",
        "star" => "⋆",
        "circ" => "∘",
        "bullet" => "•",
        "diamond" => "⋄",
        "oplus" => "⊕",
        "ominus" => "⊖",
        "otimes" => "⊗",
        "odot" => "⊙",

        // Relations
        "leq" => "≤",
        "geq" => "≥",
        "ll" => "≪",
        "gg" => "≫",
        "prec" => "≺",
        "succ" => "≻",
        "preceq" => "≼",
        "succeq" => "≽",
        "equiv" => "≡",
        "sim" => "∼",
        "simeq" => "≃",
        "cong" => "≅",
        "approx" => "≈",
        "subset" => "⊂",
        "subseteq" => "⊆",
        "supset" => "⊃",
        "supseteq" => "⊇",
        "notin" => "∉",
        "neq" => "≠",
        "perp" => "⊥",
        "parallel" => "∥",
        "mid" => "|",
        "models" => "⊨",
        "propto" => "∝",

        // Arrows
        "longrightarrow" => "⟶",
        "longleftarrow" => "⟵",
        "Longrightarrow" => "⟹",
        "Longleftarrow" => "⟸",
        "leftrightarrow" => "↔",
        "Leftrightarrow" => "⇔",
        "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "uparrow" => "↑",
        "downarrow" => "↓",
        "Uparrow" => "⇑",
        "Downarrow" => "⇓",
        "updownarrow" => "↕",
        "nearrow" => "↗",
        "searrow" => "↘",
        "swarrow" => "↙",
        "nwarrow" => "↖",
        "mapsto" => "↦",
        "to" => "→",

        // Special
        "aleph" => "ℵ",
        "hbar" => "ℏ",
        "ell" => "ℓ",
        "nabla" => "∇",
        "partial" => "∂",
        "angle" => "∠",
        "emptyset" => "∅",
        "forall" => "∀",
        "exists" => "∃",
        "neg" => "¬",
        "land" => "∧",
        "lor" => "∨",
        "top" => "⊤",
        "bot" => "⊥",

        // Delimiters (remove)
        "left" => "",
        "right" => "",
        "big" => "",
        "Big" => "",
        "bigg" => "",
        "Bigg" => "",

        // Quad spacing
        "quad" => "  ",
        "qquad" => "    ",

        _ => return None,
    })
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
