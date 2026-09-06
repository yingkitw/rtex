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

        result = Self::convert_old_fractions(&result);
        result = Self::format_matrices(&result);
        result = Self::format_sqrt(&result);
        result = Self::format_fractions(&result);
        result = crate::math::symbols::replace_math_symbols(&result);
        result = Self::format_math_alphabets(&result);
        result = Self::format_accents(&result);
        result = Self::format_superscripts(&result);
        result = Self::format_subscripts(&result);
        // Unicode symbols now supported with DejaVu font - no ASCII fallbacks needed

        result
    }

    /// Replace `\sqrt` constructs with Unicode radicals while preserving other LaTeX.
    ///
    /// Useful before handing mixed expressions (with `\frac`, etc.) to pdfrs display math.
    pub fn replace_sqrt_in_expression(expr: &str) -> String {
        crate::math::radicals::format_sqrt(expr, Self::format)
    }

    fn format_matrices(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;

        // (environment name, opening delimiter, closing delimiter)
        const MATRIX_ENVS: &[(&str, char, char)] = &[
            ("pmatrix", '(', ')'),
            ("bmatrix", '[', ']'),
            ("Bmatrix", '{', '}'),
            ("vmatrix", '|', '|'),
            ("Vmatrix", '\u{2016}', '\u{2016}'), // ‖ double vertical
            ("matrix", '\0', '\0'),              // no delimiters
            ("smallmatrix", '\0', '\0'),        // no delimiters
        ];

        while index < text.len() {
            let remaining = &text[index..];
            let mut matched = false;
            for (env, open, close) in MATRIX_ENVS {
                let begin = format!("\\begin{{{env}}}");
                let end = format!("\\end{{{env}}}");
                if remaining.starts_with(&begin) {
                    if let Some(rel_end) = Self::find_matching_end(remaining, &begin, &end) {
                        let body = &remaining[begin.len()..rel_end];
                        // Recurse so nested matrices of the same/other env get
                        // their own delimiters instead of leaking raw \begin/\end.
                        let formatted_body = Self::format_matrices(body);
                        if *open != '\0' {
                            result.push(*open);
                        }
                        result.push_str(&formatted_body);
                        if *close != '\0' {
                            result.push(*close);
                        }
                        index += rel_end + end.len();
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
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

    /// Find the position (relative to `text`) of the `\end{...}` marker that
    /// matches the `\begin{...}` marker at the start of `text`, accounting for
    /// nested environments of the same name. Returns `None` if unbalanced.
    fn find_matching_end(text: &str, begin: &str, end: &str) -> Option<usize> {
        let mut depth = 0;
        let mut search_from = 0;
        while let Some(rel) = text[search_from..].find('\\') {
            let pos = search_from + rel;
            if text[pos..].starts_with(begin) {
                depth += 1;
                search_from = pos + begin.len();
            } else if text[pos..].starts_with(end) {
                depth -= 1;
                if depth == 0 {
                    return Some(pos);
                }
                search_from = pos + end.len();
            } else {
                search_from = pos + 1;
            }
        }
        None
    }

    /// Find the `{` and `}` that enclose the keyword at `kw_start..kw_start+kw_len`.
    /// Returns `(open_index, close_index)` where `text[open] == '{'` and
    /// `text[close] == '}'`. Returns `None` if there is no enclosing group.
    fn find_enclosing_braces(text: &str, kw_start: usize, kw_len: usize) -> Option<(usize, usize)> {
        let bytes = text.as_bytes();
        // Scan left for the opening brace of the current group.
        let mut depth = 0;
        let mut open = None;
        let mut i = kw_start;
        while i > 0 {
            i -= 1;
            let ch = bytes[i];
            if ch == b'}' {
                depth += 1;
            } else if ch == b'{' {
                if depth == 0 {
                    open = Some(i);
                    break;
                }
                depth -= 1;
            }
        }
        let open = open?;
        // Scan right for the matching closing brace.
        let mut depth = 0;
        let mut close = None;
        let mut i = kw_start + kw_len;
        while i < bytes.len() {
            let ch = bytes[i];
            if ch == b'{' {
                depth += 1;
            } else if ch == b'}' {
                if depth == 0 {
                    close = Some(i);
                    break;
                }
                depth -= 1;
            }
            i += 1;
        }
        Some((open, close?))
    }

    /// Rewrite old-style TeX fraction/binomial operators to their modern forms.
    ///
    /// `{a \over b}` -> `\frac{a}{b}`; `{n \choose k}` (and `\brack`/`\brace`)
    /// -> `\binom{n}{k}`. Only the explicit-brace form is converted; the
    /// ambiguous brace-less form is left untouched.
    fn convert_old_fractions(text: &str) -> String {
        let ops: &[(&str, &str)] = &[
            ("\\over", "\\frac"),
            ("\\choose", "\\binom"),
            ("\\brack", "\\binom"),
            ("\\brace", "\\binom"),
        ];
        let mut result = text.to_string();
        loop {
            // Find the earliest keyword occurrence with a word boundary
            // (next char not alphabetic), so `\over` does not match `\overline`.
            let mut earliest: Option<(usize, &str, &str)> = None;
            for (op, target) in ops {
                let mut search_from = 0;
                while let Some(rel) = result[search_from..].find(op) {
                    let pos = search_from + rel;
                    let after = pos + op.len();
                    let next_is_alpha = result
                        .get(after..)
                        .and_then(|s| s.chars().next())
                        .is_some_and(|c| c.is_alphabetic());
                    if !next_is_alpha {
                        if earliest.is_none_or(|(p, _, _)| pos < p) {
                            earliest = Some((pos, op, target));
                        }
                        break;
                    }
                    search_from = pos + 1;
                }
            }
            let (pos, op, target) = match earliest {
                Some(e) => e,
                None => break,
            };
            let (open, close) = match Self::find_enclosing_braces(&result, pos, op.len()) {
                Some(b) => b,
                None => break, // no enclosing braces — leave the rest untouched
            };
            let num = result[open + 1..pos].trim();
            let den = result[pos + op.len()..close].trim();
            let replacement = format!("{target}{{{num}}}{{{den}}}");
            result.replace_range(open..close + 1, &replacement);
        }
        result
    }

    fn format_sqrt(text: &str) -> String {
        crate::math::radicals::format_sqrt(text, Self::format)
    }

    fn format_fractions(text: &str) -> String {
        crate::math::fractions::format_fractions(text, Self::format)
    }

    #[allow(clippy::type_complexity)]
    fn format_math_alphabets(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;

        let alphabets: [(&str, fn(char) -> Option<char>); 7] = [
            ("\\mathbb{", mathbb_char),
            ("\\mathcal{", mathcal_char),
            ("\\mathfrak{", mathfrak_char),
            ("\\mathsf{", mathsf_char),
            ("\\mathtt{", mathtt_char),
            ("\\mathbf{", mathbf_char),
            ("\\mathit{", mathit_char),
        ];

        while index < text.len() {
            let remaining = &text[index..];
            let mut matched = false;
            for (prefix, mapper) in &alphabets {
                if remaining.starts_with(prefix) {
                    let arg_start = index + prefix.len();
                    if let Some((arg, next_index)) =
                        crate::utils::extract_braced(text, arg_start - 1)
                    {
                        let formatted = Self::format(&arg);
                        for ch in formatted.chars() {
                            if let Some(mapped) = mapper(ch) {
                                result.push(mapped);
                            } else {
                                result.push(ch);
                            }
                        }
                        index = next_index;
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            index += ch.len_utf8();
        }

        result
    }

    fn format_accents(text: &str) -> String {
        let mut result = String::new();
        let mut index = 0;

        let accents = [
            ("\\vec{", "\u{20d7}"),          // combining right arrow above
            ("\\overrightarrow{", "\u{20d7}"), // combining right arrow above
            ("\\overleftarrow{", "\u{20d6}"),  // combining left arrow above
            ("\\hat{", "\u{0302}"),          // combining circumflex
            ("\\widehat{", "\u{0302}"),      // combining circumflex
            ("\\tilde{", "\u{0303}"),        // combining tilde
            ("\\widetilde{", "\u{0303}"),    // combining tilde
            ("\\bar{", "\u{0304}"),          // combining macron
            ("\\overline{", "\u{0305}"),     // combining overline
            ("\\underline{", "\u{0332}"),    // combining low line
            ("\\dot{", "\u{0307}"),          // combining dot above
            ("\\ddot{", "\u{0308}"),         // combining diaeresis
            ("\\dddot{", "\u{20db}"),        // combining three dots above
            ("\\ddddot{", "\u{20dc}"),       // combining four dots above
            ("\\check{", "\u{030c}"),        // combining caron
            ("\\breve{", "\u{0306}"),        // combining breve
            ("\\acute{", "\u{0301}"),        // combining acute
            ("\\grave{", "\u{0300}"),        // combining grave
            ("\\mathring{", "\u{030a}"),     // combining ring above
        ];

        while index < text.len() {
            let remaining = &text[index..];
            let mut matched = false;
            for (prefix, combining) in &accents {
                if remaining.starts_with(prefix) {
                    let arg_start = index + prefix.len();
                    if let Some((arg, next_index)) =
                        crate::utils::extract_braced(text, arg_start - 1)
                    {
                        let formatted = Self::format(&arg);
                        for ch in formatted.chars() {
                            result.push(ch);
                            result.push_str(combining);
                        }
                        index = next_index;
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            index += ch.len_utf8();
        }

        result
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
                && let Some((token, next_index)) =
                    Self::read_script_token(text, index + ch.len_utf8())
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
            } else if crate::math::scripts::to_subscript(ch).is_none() {
                return format!("_{}", normalized);
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

fn mathbb_char(ch: char) -> Option<char> {
    // Double-struck (blackboard bold): U+1D538–U+1D56B
    match ch {
        'A'..='Z' => char::from_u32(0x1D538 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D552 + (ch as u32 - 'a' as u32)),
        '0'..='9' => char::from_u32(0x1D7D8 + (ch as u32 - '0' as u32)),
        _ => None,
    }
}

fn mathcal_char(ch: char) -> Option<char> {
    // Script: U+1D49C–U+1D4CF
    match ch {
        'A'..='Z' => char::from_u32(0x1D49C + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D4B6 + (ch as u32 - 'a' as u32)),
        _ => None,
    }
}

fn mathfrak_char(ch: char) -> Option<char> {
    // Fraktur: U+1D504–U+1D537
    match ch {
        'A'..='Z' => char::from_u32(0x1D504 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D51E + (ch as u32 - 'a' as u32)),
        _ => None,
    }
}

fn mathsf_char(ch: char) -> Option<char> {
    // Sans-serif: U+1D5A0–U+1D5D3
    match ch {
        'A'..='Z' => char::from_u32(0x1D5A0 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D5BA + (ch as u32 - 'a' as u32)),
        '0'..='9' => char::from_u32(0x1D7E2 + (ch as u32 - '0' as u32)),
        _ => None,
    }
}

fn mathtt_char(ch: char) -> Option<char> {
    // Monospace: U+1D670–U+1D6A3
    match ch {
        'A'..='Z' => char::from_u32(0x1D670 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D68A + (ch as u32 - 'a' as u32)),
        '0'..='9' => char::from_u32(0x1D7F6 + (ch as u32 - '0' as u32)),
        _ => None,
    }
}

fn mathbf_char(ch: char) -> Option<char> {
    // Bold: U+1D400–U+1D433
    match ch {
        'A'..='Z' => char::from_u32(0x1D400 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D41A + (ch as u32 - 'a' as u32)),
        '0'..='9' => char::from_u32(0x1D7CE + (ch as u32 - '0' as u32)),
        _ => None,
    }
}

fn mathit_char(ch: char) -> Option<char> {
    // Italic: U+1D434–U+1D467
    match ch {
        'A'..='Z' => char::from_u32(0x1D434 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x1D44E + (ch as u32 - 'a' as u32)),
        _ => None,
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
    fn format_pmatrix_uses_parens() {
        let formatted = MathFormatter::format_matrices("\\begin{pmatrix}a & b \\\\ c & d\\end{pmatrix}");
        assert_eq!(formatted, "(a & b \\\\ c & d)");
    }

    #[test]
    fn format_bmatrix_uses_brackets() {
        let formatted = MathFormatter::format_matrices("\\begin{bmatrix}a & b \\\\ c & d\\end{bmatrix}");
        assert_eq!(formatted, "[a & b \\\\ c & d]");
    }

    #[test]
    fn format_vmatrix_uses_bars() {
        let formatted = MathFormatter::format_matrices("\\begin{vmatrix}a & b \\\\ c & d\\end{vmatrix}");
        assert_eq!(formatted, "|a & b \\\\ c & d|");
    }

    #[test]
    fn format_bmatrix_uppercase_uses_braces() {
        let formatted = MathFormatter::format_matrices("\\begin{Bmatrix}a \\\\ b\\end{Bmatrix}");
        assert_eq!(formatted, "{a \\\\ b}");
    }

    #[test]
    fn format_matrix_plain_no_delimiters() {
        let formatted = MathFormatter::format_matrices("\\begin{matrix}a & b \\\\ c & d\\end{matrix}");
        assert_eq!(formatted, "a & b \\\\ c & d");
    }

    #[test]
    fn format_vmatrix_double_bars() {
        let formatted = MathFormatter::format_matrices("\\begin{Vmatrix}x\\end{Vmatrix}");
        assert_eq!(formatted, "\u{2016}x\u{2016}");
    }

    #[test]
    fn format_nested_pmatrix_finds_matching_end() {
        // Outer pmatrix contains an inner pmatrix — the naive first-occurrence
        // finder would match the inner \end and truncate the body.
        let formatted =
            MathFormatter::format_matrices("\\begin{pmatrix}a \\begin{pmatrix}x\\end{pmatrix} b\\end{pmatrix}");
        assert_eq!(formatted, "(a (x) b)");
    }

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
        assert_eq!(MathFormatter::format("\\sqrt{x}"), "√x");
        assert_eq!(MathFormatter::format("\\sqrt{b^2}"), "√b²");
        assert_eq!(MathFormatter::format("\\sqrt{b^2-4ac}"), "√(b²-4ac)");
        assert_eq!(MathFormatter::format("\\sqrt[3]{8}"), "∛8");
        assert_eq!(MathFormatter::format("\\sqrt[4]{16}"), "∜16");
        assert_eq!(MathFormatter::format("\\sqrt[5]{32}"), "⁵√32");
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

    #[test]
    fn format_vec_accent() {
        let formatted = MathFormatter::format("\\vec{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{20d7}'));
    }

    #[test]
    fn format_hat_accent() {
        let formatted = MathFormatter::format("\\hat{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0302}'));
    }

    #[test]
    fn format_tilde_accent() {
        let formatted = MathFormatter::format("\\tilde{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0303}'));
    }

    #[test]
    fn format_bar_accent() {
        let formatted = MathFormatter::format("\\bar{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0304}'));
    }

    #[test]
    fn format_dot_accent() {
        let formatted = MathFormatter::format("\\dot{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0307}'));
    }

    #[test]
    fn format_ddot_accent() {
        let formatted = MathFormatter::format("\\ddot{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0308}'));
    }

    #[test]
    fn format_mathbb() {
        let formatted = MathFormatter::format("\\mathbb{R}");
        assert!(formatted.contains('\u{1d549}')); // 𝕉 (U+1D549) — double-struck R
    }

    #[test]
    fn format_mathcal() {
        let formatted = MathFormatter::format("\\mathcal{L}");
        assert!(formatted.contains('\u{1d4a7}')); // 𝓧 (U+1D4A7) — script L
    }

    #[test]
    fn format_mathfrak() {
        let formatted = MathFormatter::format("\\mathfrak{g}");
        assert!(formatted.contains('\u{1d524}')); // 𝔤 (U+1D524) — fraktur g
    }

    #[test]
    fn format_mathbf() {
        let formatted = MathFormatter::format("\\mathbf{F}");
        assert!(formatted.contains('\u{1d405}')); // 𝐅 (U+1D405) — bold F
    }

    #[test]
    fn format_mathit() {
        let formatted = MathFormatter::format("\\mathit{x}");
        assert!(formatted.contains('\u{1d465}')); // 𝑥 (U+1D465) — italic x
    }

    #[test]
    fn format_mathsf() {
        let formatted = MathFormatter::format("\\mathsf{x}");
        assert!(formatted.contains('\u{1d5d1}')); // 𝗑 sans-serif x
    }

    #[test]
    fn format_mathtt() {
        let formatted = MathFormatter::format("\\mathtt{x}");
        assert!(formatted.contains('\u{1d6a1}')); // 𝚡 monospace x
    }

    #[test]
    fn format_binom() {
        let formatted = MathFormatter::format("\\binom{n}{k}");
        assert!(formatted.contains('n'));
        assert!(formatted.contains('k'));
    }

    #[test]
    fn format_tfrac() {
        let formatted = MathFormatter::format("\\tfrac{1}{2}");
        assert_eq!(formatted, "½");
    }

    #[test]
    fn format_dfrac() {
        let formatted = MathFormatter::format("\\dfrac{a}{b}");
        assert!(formatted.contains('a'));
        assert!(formatted.contains('b'));
    }

    #[test]
    fn format_nested_sqrt() {
        let formatted = MathFormatter::format("\\sqrt{\\sqrt{x}}");
        assert!(formatted.contains('√'));
        assert!(formatted.contains('x'));
    }

    #[test]
    fn format_combined_accents_and_greek() {
        let formatted = MathFormatter::format("\\vec{\\alpha} + \\hat{\\beta}");
        assert!(formatted.contains('α'));
        assert!(formatted.contains('β'));
        assert!(formatted.contains('\u{20d7}')); // vec arrow
        assert!(formatted.contains('\u{0302}')); // hat
    }

    #[test]
    fn format_multiple_symbols_in_expression() {
        let formatted = MathFormatter::format("\\alpha + \\beta = \\gamma");
        assert_eq!(formatted, "α + β = γ");
    }

    #[test]
    fn format_math_alphabet_full_word() {
        let formatted = MathFormatter::format("\\mathbb{N}");
        assert!(formatted.contains('\u{1d545}')); // 𝕅 double-struck N (U+1D545)
    }

    #[test]
    fn format_empty_expression() {
        assert_eq!(MathFormatter::format(""), "");
    }

    #[test]
    fn format_plain_text_passthrough() {
        assert_eq!(MathFormatter::format("hello world"), "hello world");
    }

    #[test]
    fn format_check_accent() {
        let formatted = MathFormatter::format("\\check{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{030c}')); // caron
    }

    #[test]
    fn format_breve_accent() {
        let formatted = MathFormatter::format("\\breve{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0306}')); // breve
    }

    #[test]
    fn format_acute_accent() {
        let formatted = MathFormatter::format("\\acute{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0301}')); // acute
    }

    #[test]
    fn format_grave_accent() {
        let formatted = MathFormatter::format("\\grave{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{0300}')); // grave
    }

    #[test]
    fn format_mathring_accent() {
        let formatted = MathFormatter::format("\\mathring{x}");
        assert!(formatted.contains('x'));
        assert!(formatted.contains('\u{030a}')); // ring above
    }

    #[test]
    fn format_widehat_accent() {
        let formatted = MathFormatter::format("\\widehat{AB}");
        assert!(formatted.contains('\u{0302}')); // circumflex
    }

    #[test]
    fn format_widetilde_accent() {
        let formatted = MathFormatter::format("\\widetilde{AB}");
        assert!(formatted.contains('\u{0303}')); // tilde
    }

    #[test]
    fn format_overrightarrow_accent() {
        let formatted = MathFormatter::format("\\overrightarrow{AB}");
        assert!(formatted.contains('\u{20d7}')); // right arrow above
    }

    #[test]
    fn format_overleftarrow_accent() {
        let formatted = MathFormatter::format("\\overleftarrow{AB}");
        assert!(formatted.contains('\u{20d6}')); // left arrow above
    }

    #[test]
    fn format_overline_accent() {
        let formatted = MathFormatter::format("\\overline{x}");
        assert!(formatted.contains('\u{0305}')); // overline
    }

    #[test]
    fn format_underline_accent() {
        let formatted = MathFormatter::format("\\underline{x}");
        assert!(formatted.contains('\u{0332}')); // low line
    }

    #[test]
    fn format_boldsymbol() {
        let formatted = MathFormatter::format("\\boldsymbol{x}");
        assert!(formatted.contains('x'));
    }

    #[test]
    fn format_operatorname() {
        let formatted = MathFormatter::format("\\operatorname{Tr}");
        assert!(formatted.contains("Tr"));
    }

    #[test]
    fn format_displaystyle_noop() {
        let formatted = MathFormatter::format("\\displaystyle x^2");
        assert!(formatted.contains('x'));
        assert!(!formatted.contains("displaystyle"));
    }

    #[test]
    fn format_over_converts_to_frac() {
        // {a \over b} should render like \frac{a}{b}
        let over = MathFormatter::format("{a \\over b}");
        let frac = MathFormatter::format("\\frac{a}{b}");
        assert_eq!(over, frac, "{{a \\over b}} should convert to \\frac{{a}}{{b}}");
    }

    #[test]
    fn format_choose_converts_to_binom() {
        let choose = MathFormatter::format("{n \\choose k}");
        let binom = MathFormatter::format("\\binom{n}{k}");
        assert_eq!(choose, binom, "{{n \\choose k}} should convert to \\binom{{n}}{{k}}");
    }

    #[test]
    fn format_over_does_not_match_overline() {
        // \overline is an accent, not the \over fraction operator.
        let formatted = MathFormatter::format("\\overline{x}");
        assert!(formatted.contains('\u{0305}'), "overline combining char");
        assert!(!formatted.contains("÷"), "should not be treated as a fraction");
    }
}
