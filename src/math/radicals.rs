//! Radical (square-root) formatting for LaTeX math.
//!
//! Scans a math expression for `\sqrt[n]{...}` / `\sqrt{...}` and replaces
//! each occurrence with Unicode radical notation.

use crate::math::scripts::to_superscript;

/// Replace every `\sqrt` occurrence in `text`.
///
/// `format_fn` is called recursively on the extracted radicand so that
/// nested math constructs are processed in the correct order.
pub fn format_sqrt<F>(text: &str, format_fn: F) -> String
where
    F: Fn(&str) -> String,
{
    let mut result = String::new();
    let mut index = 0;

    while index < text.len() {
        let remaining = &text[index..];
        if remaining.starts_with("\\sqrt") {
            let after_cmd = index + "\\sqrt".len();
            let (root_index, brace_start) = parse_optional_root_index(text, after_cmd);

            if let Some((radicand, next_index)) = crate::utils::extract_braced(text, brace_start) {
                let formatted = format_fn(&radicand);
                write_root(&mut result, root_index.as_deref(), &formatted);
                index = next_index;
                continue;
            }

            result.push('√');
            index = after_cmd;
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

fn parse_optional_root_index(text: &str, start: usize) -> (Option<String>, usize) {
    if start >= text.len() || !text[start..].starts_with('[') {
        return (None, start);
    }

    let Some(close) = text[start + 1..].find(']') else {
        return (None, start);
    };
    let index = text[start + 1..start + 1 + close].trim().to_string();
    (Some(index), start + 1 + close + 1)
}

fn write_root(out: &mut String, index: Option<&str>, formatted_radicand: &str) {
    match index {
        Some("3") => out.push('∛'),
        Some("4") => out.push('∜'),
        Some(n) => {
            for ch in n.chars() {
                if let Some(s) = to_superscript(ch) {
                    out.push(s);
                } else {
                    out.push(ch);
                }
            }
            out.push('√');
        }
        None => out.push('√'),
    }

    if needs_radicand_parens(formatted_radicand) {
        out.push('(');
        out.push_str(formatted_radicand);
        out.push(')');
    } else {
        out.push_str(formatted_radicand);
    }
}

fn needs_radicand_parens(formatted: &str) -> bool {
    formatted.contains('+')
        || formatted.contains('-')
        || formatted.contains(',')
        || formatted.contains(' ')
        || formatted.contains('÷')
        || formatted.contains('[')
        || formatted.contains(']')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn simple_sqrt() {
        assert_eq!(format_sqrt("\\sqrt{x}", identity), "√x");
    }

    #[test]
    fn sqrt_without_braces() {
        assert_eq!(format_sqrt("\\sqrt x", identity), "√ x");
    }

    #[test]
    fn sqrt_with_recursive_formatting() {
        let fmt = |s: &str| {
            if s == "b^2" {
                "b²".to_string()
            } else {
                s.to_string()
            }
        };
        assert_eq!(format_sqrt("\\sqrt{b^2}", fmt), "√b²");
    }

    #[test]
    fn sqrt_with_sum_needs_parens() {
        let fmt = |s: &str| s.replace("x^2 + y^2", "x² + y²");
        assert_eq!(format_sqrt("\\sqrt{x^2 + y^2}", fmt), "√(x² + y²)");
    }

    #[test]
    fn cube_and_fourth_root() {
        assert_eq!(format_sqrt("\\sqrt[3]{8}", identity), "∛8");
        assert_eq!(format_sqrt("\\sqrt[4]{16}", identity), "∜16");
    }

    #[test]
    fn nth_root_uses_superscript_index() {
        assert_eq!(format_sqrt("\\sqrt[5]{32}", identity), "⁵√32");
    }
}
