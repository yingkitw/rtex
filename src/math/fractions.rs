//! Fraction formatting for LaTeX math.
//!
//! Scans a math expression for `\frac{numerator}{denominator}` and
//! converts simple numeric fractions to Unicode fraction characters
//! (e.g. `\frac{1}{2}` → `½`).  Complex fractions are rendered as
//! `[num] ÷ [denom]`.

/// Replace every `\frac{num}{denom}` in `text` with a formatted fraction.
///
/// `format_fn` is called recursively on the extracted numerator and
/// denominator so that nested math constructs are processed in the
/// correct order.
pub fn format_fractions<F>(text: &str, format_fn: F) -> String
where
    F: Fn(&str) -> String,
{
    let mut result = String::new();
    let mut index = 0;

    while index < text.len() {
        let remaining = &text[index..];
        if remaining.starts_with("\\frac") {
            let frac_end = index + "\\frac".len();

            if let Some((numerator, num_end)) = crate::utils::extract_braced(text, frac_end)
                && let Some((denominator, denom_end)) = crate::utils::extract_braced(text, num_end)
            {
                let formatted_num = format_fn(&numerator);
                let formatted_denom = format_fn(&denominator);

                if let Some(unicode_frac) = to_unicode_fraction(&formatted_num, &formatted_denom) {
                    result.push_str(&unicode_frac);
                } else {
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

/// Convert simple numeric fractions to single Unicode fraction characters.
pub fn to_unicode_fraction(numerator: &str, denominator: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn simple_fraction_unicode() {
        assert_eq!(format_fractions("\\frac{1}{2}", identity), "½");
        assert_eq!(format_fractions("\\frac{3}{4}", identity), "¾");
    }

    #[test]
    fn complex_fraction_division_form() {
        assert_eq!(
            format_fractions("\\frac{a+b}{c-d}", identity),
            "[a+b] ÷ [c-d]"
        );
    }

    #[test]
    fn to_unicode_fraction_table() {
        assert_eq!(to_unicode_fraction("1", "2"), Some("½".to_string()));
        assert_eq!(to_unicode_fraction("2", "2"), None);
    }
}
