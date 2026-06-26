//! PDF text rendering operations — separation of concerns.
//!
//! Handles all text rendering operations for PDF generation.

/// Utilities for normalizing and wrapping text before PDF rendering.
pub struct PdfTextRenderer;

impl PdfTextRenderer {
    // Note: encode_utf16_be removed - now using simpler literal strings with Helvetica font

    /// Remove control characters and collapse consecutive whitespace.
    pub fn normalize_text(text: &str) -> Option<String> {
        let mut normalized = String::with_capacity(text.len());
        let mut previous_was_whitespace = false;

        for ch in text.chars() {
            if ch.is_control() && !ch.is_whitespace() {
                continue;
            }

            if ch.is_whitespace() {
                if !previous_was_whitespace {
                    normalized.push(' ');
                    previous_was_whitespace = true;
                }
            } else {
                normalized.push(ch);
                previous_was_whitespace = false;
            }
        }

        let trimmed = normalized.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Count Unicode scalar values in `text`.
    pub fn char_count(text: &str) -> usize {
        text.chars().count()
    }

    /// Split `text` into chunks of at most `max_chars` Unicode scalars.
    pub fn split_by_char_count(text: &str, max_chars: usize) -> Vec<String> {
        if max_chars == 0 {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        let mut current_len = 0;

        for ch in text.chars() {
            if current_len == max_chars {
                chunks.push(current);
                current = String::new();
                current_len = 0;
            }
            current.push(ch);
            current_len += 1;
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    /// Word-wrap `text` so that no line exceeds `max_chars` characters.
    pub fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_line_chars = 0;

        for word in text.split_whitespace() {
            let word_chars = Self::char_count(word);

            // If word is longer than max_chars, split it
            if word_chars > max_chars {
                // Finish current line if not empty
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_line_chars = 0;
                }

                // Split the long word
                for chunk in Self::split_by_char_count(word, max_chars) {
                    lines.push(chunk);
                }
                continue;
            }

            // Check if we can add this word to current line
            if current_line_chars == 0 {
                // First word on line
                current_line = word.to_string();
                current_line_chars = word_chars;
            } else if current_line_chars + 1 + word_chars <= max_chars {
                // Can add to current line
                current_line.push(' ');
                current_line.push_str(word);
                current_line_chars += 1 + word_chars;
            } else {
                // Need new line
                lines.push(current_line);
                current_line = word.to_string();
                current_line_chars = word_chars;
            }
        }

        // Add last line if not empty
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let text = "Hello   world  \n  test";
        let normalized = PdfTextRenderer::normalize_text(text).unwrap();
        assert_eq!(normalized, "Hello world test");
    }

    #[test]
    fn test_char_count() {
        assert_eq!(PdfTextRenderer::char_count("hello"), 5);
        assert_eq!(PdfTextRenderer::char_count("日本語"), 3);
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a long line that needs wrapping";
        let lines = PdfTextRenderer::wrap_text(text, 20);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(PdfTextRenderer::char_count(line) <= 20);
        }
    }

    #[test]
    fn test_normalize_text_removes_control_chars() {
        let text = "Hello\x00world\x01test";
        let normalized = PdfTextRenderer::normalize_text(text).unwrap();
        assert_eq!(normalized, "Helloworldtest");
    }

    #[test]
    fn test_normalize_text_returns_none_for_empty() {
        assert_eq!(PdfTextRenderer::normalize_text("   "), None);
        assert_eq!(PdfTextRenderer::normalize_text("\n\n\t"), None);
    }

    #[test]
    fn test_split_by_char_count() {
        let chunks = PdfTextRenderer::split_by_char_count("HelloWorld", 3);
        assert_eq!(chunks, vec!["Hel", "loW", "orl", "d"]);
    }

    #[test]
    fn test_split_by_char_count_zero_max() {
        let chunks = PdfTextRenderer::split_by_char_count("hello", 0);
        assert_eq!(chunks, vec!["hello"]);
    }

    #[test]
    fn test_wrap_text_splits_long_word() {
        let text = "supercalifragilisticexpialidocious";
        let lines = PdfTextRenderer::wrap_text(text, 10);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(PdfTextRenderer::char_count(line) <= 10);
        }
    }

    #[test]
    fn test_wrap_text_empty() {
        let lines = PdfTextRenderer::wrap_text("", 10);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_wrap_text_single_short_word() {
        let lines = PdfTextRenderer::wrap_text("hello", 10);
        assert_eq!(lines, vec!["hello"]);
    }
}
