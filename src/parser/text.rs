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

        if text.is_empty() {
            None
        } else {
            Some(TexElement::Text(text.trim().to_string()))
        }
    }
}
