//! Math formatting orchestrator.
//!
//! Applies successive transformations to LaTeX math expressions:
//! matrices, square roots, fractions, symbol replacement,
//! superscripts, and subscripts.

pub struct MathFormatter;

impl MathFormatter {
    /// Create a new formatter (stateless; mostly for API consistency).
    pub fn new() -> Self {
        Self
    }
    
    /// Convert a LaTeX math expression to a Unicode-rich display string.
    pub fn format(math: &str) -> String {
        let mut result = math.to_string();
        
        result = Self::format_matrices(&result);
        result = Self::format_sqrt(&result);
        result = Self::format_fractions(&result);
        result = crate::math::symbols::replace_math_symbols(&result);
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

                if let Some((radicand, next_index)) = crate::utils::extract_braced(text, sqrt_end) {
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

                if let Some((numerator, num_end)) = crate::utils::extract_braced(text, frac_end)
                    && let Some((denominator, denom_end)) = crate::utils::extract_braced(text, num_end) {
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
        let normalized = crate::math::symbols::replace_math_symbols(token);
        if normalized.is_empty() {
            return String::new();
        }

        // If it's a single Unicode symbol that can't be converted to script,
        // return it with the appropriate marker
        if normalized.chars().count() == 1 {
            let ch = normalized.chars().next().unwrap();
            if is_super {
                if crate::math::scripts::to_superscript(ch).is_none() {
                    return format!("^{}", normalized);
                }
            } else {
                if crate::math::scripts::to_subscript(ch).is_none() {
                    return format!("_{}", normalized);
                }
            }
        }

        let mut mapped = String::new();
        for ch in normalized.chars() {
            let mapped_char = if is_super {
                crate::math::scripts::to_superscript(ch)
            } else {
                crate::math::scripts::to_subscript(ch)
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
            return crate::utils::extract_braced(text, start);
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
