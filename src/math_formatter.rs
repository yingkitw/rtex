pub struct MathFormatter;

impl MathFormatter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn format(math: &str) -> String {
        let mut result = math.to_string();
        
        result = Self::format_matrices(&result);
        result = Self::format_sqrt(&result);
        result = Self::format_fractions(&result);
        result = Self::replace_math_symbols(&result);
        result = Self::format_superscripts(&result);
        result = Self::format_subscripts(&result);
        // Unicode symbols now supported with DejaVu font - no ASCII fallbacks needed

        result
    }
    
    fn format_matrices(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;

        while index < text.len() {
            let remaining = &text[index..];
            if remaining.starts_with("\\begin{pmatrix}") {
                let matrix_end = index + "\\begin{pmatrix}".len();
                
                // Find the matching \end{pmatrix}
                if let Some(end_pos) = remaining.find("\\end{pmatrix}") {
                    let matrix_content = &remaining[matrix_end - index..end_pos];
                    
                    // Convert matrix to bracket notation
                    result.push('[');
                    result.push_str(matrix_content);
                    result.push(']');
                    
                    index += end_pos + "\\end{pmatrix}".len();
                    continue;
                }
            }
            
            if remaining.starts_with("\\begin{bmatrix}") {
                let matrix_end = index + "\\begin{bmatrix}".len();
                
                if let Some(end_pos) = remaining.find("\\end{bmatrix}") {
                    let matrix_content = &remaining[matrix_end - index..end_pos];
                    
                    result.push('[');
                    result.push_str(matrix_content);
                    result.push(']');
                    
                    index += end_pos + "\\end{bmatrix}".len();
                    continue;
                }
            }

            let ch = remaining
                .chars()
                .next()
                .expect("remaining is never empty while index < text.len()");
            result.push(ch);
            index += ch.len_utf8();
        }

        result
    }
    
    fn format_sqrt(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;

        while index < text.len() {
            let remaining = &text[index..];
            if remaining.starts_with("\\sqrt") {
                let sqrt_end = index + "\\sqrt".len();

                if let Some((radicand, next_index)) = Self::read_group(text, sqrt_end) {
                    // Format the radicand recursively
                    let formatted_radicand = Self::format(&radicand);
                    result.push('√');
                    result.push('(');
                    result.push_str(&formatted_radicand);
                    result.push(')');
                    index = next_index;
                    continue;
                }

                result.push('√');
                index = sqrt_end;
                continue;
            }

            let ch = remaining
                .chars()
                .next()
                .expect("remaining is never empty while index < text.len()");
            result.push(ch);
            index += ch.len_utf8();
        }

        result
    }
    
    fn format_fractions(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;

        while index < text.len() {
            let remaining = &text[index..];
            if remaining.starts_with("\\frac") {
                let frac_end = index + "\\frac".len();

                if let Some((numerator, num_end)) = Self::read_group(text, frac_end) {
                    if let Some((denominator, denom_end)) = Self::read_group(text, num_end) {
                        // Format numerator and denominator recursively
                        let formatted_num = Self::format(&numerator);
                        let formatted_denom = Self::format(&denominator);

                        // Use common fraction notation for simple cases
                        if let Some(unicode_frac) = Self::to_unicode_fraction(&formatted_num, &formatted_denom) {
                            result.push_str(&unicode_frac);
                        } else {
                            // For complex fractions, use clear division notation
                            // Format: [numerator] ÷ [denominator] to avoid rendering issues
                            // This is more readable than / and doesn't require parentheses
                            result.push('[');
                            result.push_str(&formatted_num);
                            result.push_str("] ÷ [");
                            result.push_str(&formatted_denom);
                            result.push(']');
                        }

                        index = denom_end;
                        continue;
                    }
                }

                result.push_str("frac");
                index = frac_end;
                continue;
            }

            let ch = remaining
                .chars()
                .next()
                .expect("remaining is never empty while index < text.len()");
            result.push(ch);
            index += ch.len_utf8();
        }

        result
    }

    /// Convert simple fractions to Unicode fraction characters
    fn to_unicode_fraction(numerator: &str, denominator: &str) -> Option<String> {
        match (numerator, denominator) {
            ("1", "2") => Some("½".to_string()),
            ("1", "3") => Some("⅓".to_string()),
            ("2", "3") => Some("⅔".to_string()),
            ("1", "4") => Some("¼".to_string()),
            ("3", "4") => Some("¾".to_string()),
            ("1", "5") => Some("⅕".to_string()),
            ("2", "5") => Some("⅖".to_string()),
            ("3", "5") => Some("⅗".to_string()),
            ("4", "5") => Some("⅘".to_string()),
            ("1", "6") => Some("⅙".to_string()),
            ("5", "6") => Some("⅚".to_string()),
            ("1", "7") => Some("⅐".to_string()),
            ("1", "8") => Some("⅛".to_string()),
            ("3", "8") => Some("⅜".to_string()),
            ("5", "8") => Some("⅝".to_string()),
            ("7", "8") => Some("⅞".to_string()),
            ("1", "9") => Some("⅑".to_string()),
            ("1", "10") => Some("⅒".to_string()),
            _ => None,
        }
    }
    
    fn format_superscripts(text: &str) -> String {
        Self::format_script(text, '^', true)
    }
    
    fn format_subscripts(text: &str) -> String {
        Self::format_script(text, '_', false)
    }

    fn format_script(text: &str, marker: char, is_super: bool) -> String {
        let mut result = String::new();
        let mut index = 0;

        while index < text.len() {
            let remaining = &text[index..];
            let ch = remaining
                .chars()
                .next()
                .expect("remaining is never empty while index < text.len()");

            if ch == marker
                && let Some((token, next_index)) = Self::read_script_token(text, index + ch.len_utf8())
            {
                result.push_str(&Self::render_script_token(&token, is_super));
                index = next_index;
                continue;
            }

            result.push(ch);
            index += ch.len_utf8();
        }

        result
    }

    fn render_script_token(token: &str, is_super: bool) -> String {
        let normalized = Self::replace_math_symbols(token);
        if normalized.is_empty() {
            return String::new();
        }

        // If it's a single Unicode symbol that can't be converted to script,
        // return it with the appropriate marker
        if normalized.chars().count() == 1 {
            let ch = normalized.chars().next().unwrap();
            if is_super {
                if Self::to_superscript(ch).is_none() {
                    return format!("^{}", normalized);
                }
            } else {
                if Self::to_subscript(ch).is_none() {
                    return format!("_{}", normalized);
                }
            }
        }

        let mut mapped = String::new();
        for ch in normalized.chars() {
            let mapped_char = if is_super {
                Self::to_superscript(ch)
            } else {
                Self::to_subscript(ch)
            };

            if let Some(script_char) = mapped_char {
                mapped.push(script_char);
            } else {
                return if is_super {
                    format!("^({normalized})")
                } else {
                    format!("_({normalized})")
                };
            }
        }

        mapped
    }

    fn read_script_token(text: &str, start: usize) -> Option<(String, usize)> {
        if start >= text.len() {
            return None;
        }

        let first = text[start..].chars().next()?;
        if first == '{' {
            return Self::read_group(text, start);
        }

        if first == '\\' {
            let mut index = start + first.len_utf8();
            while index < text.len() {
                let ch = text[index..].chars().next()?;
                if ch.is_alphabetic() || ch == '*' {
                    index += ch.len_utf8();
                } else {
                    break;
                }
            }
            return Some((text[start..index].to_string(), index));
        }

        Some((first.to_string(), start + first.len_utf8()))
    }

    fn read_group(text: &str, start: usize) -> Option<(String, usize)> {
        if start >= text.len() || !text[start..].starts_with('{') {
            return None;
        }

        let mut depth = 0;
        let mut content_start = None;

        for (offset, ch) in text[start..].char_indices() {
            let index = start + offset;
            if ch == '{' {
                depth += 1;
                if depth == 1 {
                    content_start = Some(index + ch.len_utf8());
                }
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    let inner_start = content_start?;
                    return Some((text[inner_start..index].to_string(), index + ch.len_utf8()));
                }
            }
        }

        None
    }

    fn replace_math_symbols(text: &str) -> String {
        let mut result = text.to_string();
        
        // Clean up LaTeX spacing and formatting commands
        result = result.replace("\\,", " ");  // thin space
        result = result.replace("\\;", " ");  // medium space
        result = result.replace("\\!", "");   // negative thin space
        result = result.replace("\\quad", "  ");  // quad space
        result = result.replace("\\qquad", "    ");  // double quad
        result = result.replace("\\:", " ");  // medium math space
        result = result.replace("\\>", " ");  // medium space
        result = result.replace("\\~", " ");  // non-breaking space
        
        // Remove matrix row separators and alignment
        result = result.replace("\\\\", " ");  // row separator
        result = result.replace("&", " ");  // column separator
        
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
        result = result.replace("\\infty", "∞");  // Must come before \in
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
    
    fn to_superscript(ch: char) -> Option<char> {
        match ch {
            '0' => Some('⁰'),
            '1' => Some('¹'),
            '2' => Some('²'),
            '3' => Some('³'),
            '4' => Some('⁴'),
            '5' => Some('⁵'),
            '6' => Some('⁶'),
            '7' => Some('⁷'),
            '8' => Some('⁸'),
            '9' => Some('⁹'),
            '+' => Some('⁺'),
            '-' => Some('⁻'),
            '=' => Some('⁼'),
            '(' => Some('⁽'),
            ')' => Some('⁾'),
            'a' => Some('ᵃ'),
            'b' => Some('ᵇ'),
            'c' => Some('ᶜ'),
            'd' => Some('ᵈ'),
            'e' => Some('ᵉ'),
            'f' => Some('ᶠ'),
            'g' => Some('ᵍ'),
            'h' => Some('ʰ'),
            'i' => Some('ⁱ'),
            'j' => Some('ʲ'),
            'k' => Some('ᵏ'),
            'l' => Some('ˡ'),
            'm' => Some('ᵐ'),
            'n' => Some('ⁿ'),
            'o' => Some('ᵒ'),
            'p' => Some('ᵖ'),
            'r' => Some('ʳ'),
            's' => Some('ˢ'),
            't' => Some('ᵗ'),
            'u' => Some('ᵘ'),
            'v' => Some('ᵛ'),
            'w' => Some('ʷ'),
            'x' => Some('ˣ'),
            'y' => Some('ʸ'),
            'z' => Some('ᶻ'),
            _ => None,
        }
    }

    fn to_subscript(ch: char) -> Option<char> {
        match ch {
            '0' => Some('₀'),
            '1' => Some('₁'),
            '2' => Some('₂'),
            '3' => Some('₃'),
            '4' => Some('₄'),
            '5' => Some('₅'),
            '6' => Some('₆'),
            '7' => Some('₇'),
            '8' => Some('₈'),
            '9' => Some('₉'),
            '+' => Some('₊'),
            '-' => Some('₋'),
            '=' => Some('₌'),
            '(' => Some('₍'),
            ')' => Some('₎'),
            'a' => Some('ₐ'),
            'e' => Some('ₑ'),
            'h' => Some('ₕ'),
            'i' => Some('ᵢ'),
            'j' => Some('ⱼ'),
            'k' => Some('ₖ'),
            'l' => Some('ₗ'),
            'm' => Some('ₘ'),
            'n' => Some('ₙ'),
            'o' => Some('ₒ'),
            'p' => Some('ₚ'),
            'r' => Some('ᵣ'),
            's' => Some('ₛ'),
            't' => Some('ₜ'),
            'u' => Some('ᵤ'),
            'v' => Some('ᵥ'),
            'x' => Some('ₓ'),
            _ => None,
        }
    }
}

impl Default for MathFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MathFormatter;

    #[test]
    fn format_fraction_with_groups() {
        let formatted = MathFormatter::format("\\frac{a+b}{c-d}");
        assert_eq!(formatted, "[a+b] ÷ [c-d]");
    }

    #[test]
    fn format_simple_fractions_as_unicode() {
        assert_eq!(MathFormatter::format("\\frac{1}{2}"), "½");
        assert_eq!(MathFormatter::format("\\frac{1}{4}"), "¼");
        assert_eq!(MathFormatter::format("\\frac{3}{4}"), "¾");
        assert_eq!(MathFormatter::format("\\frac{1}{3}"), "⅓");
        assert_eq!(MathFormatter::format("\\frac{2}{3}"), "⅔");
    }

    #[test]
    fn format_root_and_exponents() {
        let formatted = MathFormatter::format("\\sqrt{b^2-4ac}");
        assert_eq!(formatted, "√(b²-4ac)");
    }

    #[test]
    fn format_subscript_and_greek() {
        let formatted = MathFormatter::format("x_{i+1}=\\alpha");
        assert_eq!(formatted, "xᵢ₊₁=α");
    }

    #[test]
    fn format_unmappable_script_falls_back_to_parenthesized_form() {
        let formatted = MathFormatter::format("x^\\infty");
        assert_eq!(formatted, "x^∞"); // Infinity as Unicode symbol with DejaVu font
    }
}
