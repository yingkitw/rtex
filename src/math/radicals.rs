//! Radical (square-root) formatting for LaTeX math.
//!
//! Scans a math expression for `\sqrt{...}` and replaces each occurrence
//! with a Unicode square-root sign wrapping the formatted radicand.

/// Replace every `\sqrt{radicand}` in `text` with `√(formatted_radicand)`.
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
            let sqrt_end = index + "\\sqrt".len();

            if let Some((radicand, next_index)) = crate::utils::extract_braced(text, sqrt_end) {
                let formatted = format_fn(&radicand);
                result.push('√');
                result.push('(');
                result.push_str(&formatted);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn simple_sqrt() {
        assert_eq!(format_sqrt("\\sqrt{x}", identity), "√(x)");
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
        assert_eq!(format_sqrt("\\sqrt{b^2}", fmt), "√(b²)");
    }
}
