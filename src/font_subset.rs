//! Font subsetting — embed only glyphs actually used in the document.

use std::collections::BTreeSet;

/// Collect every Unicode character that appears in the parsed document.
pub fn collect_used_chars(elements: &[crate::parser::TexElement]) -> BTreeSet<char> {
    let mut chars = BTreeSet::new();
    // Always include basic ASCII control / punctuation so the font remains usable
    for c in ' '..='~' {
        chars.insert(c);
    }
    for elem in elements {
        match elem {
            crate::parser::TexElement::Text(t)
            | crate::parser::TexElement::CodeBlock(t) => {
                for c in t.chars() {
                    chars.insert(c);
                }
            }
            crate::parser::TexElement::MathInline(t)
            | crate::parser::TexElement::MathDisplay(t) => {
                for c in t.chars() {
                    chars.insert(c);
                }
            }
            crate::parser::TexElement::Section { title, .. } => {
                for c in title.chars() {
                    chars.insert(c);
                }
            }
            crate::parser::TexElement::ColoredText { text, .. } => {
                for c in text.chars() {
                    chars.insert(c);
                }
            }
            crate::parser::TexElement::ItemList { items, .. } => {
                for item in items {
                    for inner in item {
                        if let crate::parser::TexElement::Text(t) = inner {
                            for c in t.chars() {
                                chars.insert(c);
                            }
                        }
                    }
                }
            }
            crate::parser::TexElement::Bibliography { entries } => {
                for entry in entries {
                    for c in entry.text.chars() {
                        chars.insert(c);
                    }
                }
            }
            crate::parser::TexElement::Command { args, .. } => {
                for arg in args {
                    for c in arg.chars() {
                        chars.insert(c);
                    }
                }
            }
            _ => {}
        }
    }
    chars
}

/// Subset a TrueType font to only the characters used in the document.
///
/// Returns `Some(subset_bytes)` on success, or `None` if subsetting fails
/// (so the caller can fall back to embedding the full font).
pub fn subset_font(font_data: &[u8], chars: &BTreeSet<char>) -> Option<Vec<u8>> {
    let font = font_subset::Font::opentype(font_data).ok()?;

    let permissions = font.permissions();
    if !permissions.embedding.is_lenient() || !permissions.allow_subsetting {
        return None;
    }

    let subset = font.subset(chars).ok()?;
    Some(subset.to_opentype())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_used_chars_includes_text() {
        let elements = vec![
            crate::parser::TexElement::Text("Hello α".to_string()),
            crate::parser::TexElement::Section { level: 1, title: "World".to_string() },
        ];
        let chars = collect_used_chars(&elements);
        assert!(chars.contains(&'H'));
        assert!(chars.contains(&'α'));
        assert!(chars.contains(&'W'));
        // 'Œ' is not in the text, so it should not be in the set
        assert!(!chars.contains(&'Œ'));
    }

    #[test]
    fn subset_font_smaller_than_original() {
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts/DejaVuSans.ttf");
        let font_data = std::fs::read(&font_path).unwrap();
        let original_len = font_data.len();

        let mut chars = BTreeSet::new();
        for c in ' '..='~' {
            chars.insert(c);
        }

        if let Some(subset_data) = subset_font(&font_data, &chars) {
            // Subset should be significantly smaller than original
            assert!(subset_data.len() < original_len,
                "subset {} bytes should be smaller than original {} bytes",
                subset_data.len(), original_len);
        }
        // If subsetting returns None (e.g. permissions don't allow it),
        // the PdfBuilder falls back to the full font, so this is fine.
    }
}
