//! Math parsing — inline and display math delimiters.

use super::{TexElement, TexParser};

impl TexParser {
    /// Parse inline math (`$...$`) or display math (`$$...$$` / `\begin{equation}`).
    pub(super) fn parse_math(&mut self) -> Option<TexElement> {
        self.position += 1;

        let remaining = &self.content[self.position..];

        if remaining.starts_with('$') {
            self.position += 1;
            let content = self.read_until_str("$$");
            self.position += 2;
            Some(TexElement::MathDisplay(content))
        } else {
            let content = self.read_until('$');
            self.position += 1;
            Some(TexElement::MathInline(content))
        }
    }
}
