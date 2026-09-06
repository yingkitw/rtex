//! Plain-text parsing — accumulates characters until a control sequence,
//! paragraph break, or math delimiter is encountered.

use super::{TexElement, TexParser};

impl TexParser {
    /// Accumulate raw text, preserving inline math (`$...$`) as-is.
    pub(super) fn parse_text(&mut self) -> Option<TexElement> {
        let mut text = String::new();

        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];

            if remaining.starts_with('\\')
                || remaining.starts_with("\n\n")
                || remaining.starts_with('}')
            {
                break;
            }

            if remaining.starts_with('$') {
                self.position += 1;

                if self.position < self.content.len()
                    && self.content[self.position..].starts_with('$')
                {
                    self.position -= 1;
                    break;
                }

                let math_content = self.read_until('$');
                self.position += 1;

                // Keep $ delimiters so PDF builder can format the math
                text.push_str(&format!("${}$", math_content));
            } else if let Some(ch) = remaining.chars().next() {
                text.push(ch);
                self.position += ch.len_utf8();
            }
        }

        if text.trim().is_empty() {
            None
        } else {
            Some(TexElement::Text(convert_text_ligatures(&text)))
        }
    }
}

/// Convert LaTeX text-mode ligatures to their Unicode equivalents.
///
/// Handles the four unambiguous multi-character ligatures:
/// - `---` → em-dash (—, U+2014)
/// - `--`  → en-dash (–, U+2013)
/// - `` `` ``  → left double quote (", U+201C)
/// - `''` → right double quote (", U+201D)
///
/// Inline math spans (`$...$`) are passed through untouched — ligatures do
/// not apply in math mode (e.g. `--` there is two minus signs). Single
/// quotes/apostrophes are intentionally left alone to avoid altering
/// contractions and possessives.
fn convert_text_ligatures(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let remaining = &text[index..];

        // Pass inline math spans through untouched.
        if remaining.starts_with('$') {
            if let Some(rel) = remaining[1..].find('$') {
                let end = 1 + rel;
                result.push_str(&remaining[..=end]);
                index += end + 1;
                continue;
            }
        }

        if remaining.starts_with("---") {
            result.push('\u{2014}'); // —
            index += 3;
        } else if remaining.starts_with("--") {
            result.push('\u{2013}'); // –
            index += 2;
        } else if remaining.starts_with("``") {
            result.push('\u{201C}'); // "
            index += 2;
        } else if remaining.starts_with("''") {
            result.push('\u{201D}'); // "
            index += 2;
        } else {
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            index += ch.len_utf8();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::convert_text_ligatures;

    #[test]
    fn em_dash_ligature() {
        assert_eq!(convert_text_ligatures("a---b"), "a\u{2014}b");
    }

    #[test]
    fn en_dash_ligature() {
        assert_eq!(convert_text_ligatures("pages 10--20"), "pages 10\u{2013}20");
    }

    #[test]
    fn em_dash_takes_precedence_over_en_dash() {
        // `---` must match as em-dash, not `--` + `-`.
        assert_eq!(convert_text_ligatures("x---y"), "x\u{2014}y");
        assert_eq!(convert_text_ligatures("x--y"), "x\u{2013}y");
    }

    #[test]
    fn double_quote_ligatures() {
        assert_eq!(convert_text_ligatures("``hello''"), "\u{201C}hello\u{201D}");
    }

    #[test]
    fn ligatures_skipped_in_inline_math() {
        // `--` inside math is two minus signs, not an en-dash.
        assert_eq!(convert_text_ligatures("$a -- b$"), "$a -- b$");
        assert_eq!(
            convert_text_ligatures("see $x--y$ here"),
            "see $x--y$ here"
        );
    }

    #[test]
    fn single_quotes_preserved() {
        // Contractions and possessives keep their ASCII apostrophe.
        assert_eq!(convert_text_ligatures("don't"), "don't");
        assert_eq!(convert_text_ligatures("it's"), "it's");
    }
}
