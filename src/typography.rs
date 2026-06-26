//! Advanced typography support — ligatures and kerning.
//!
//! This module provides text-level transformations that improve
//! visual quality without requiring a full OpenType layout engine.
//!
//! # Limitations
//! - **Small caps** and **stylistic sets** require GSUB table parsing;
//!   they are not yet implemented.
//! - Kerning values are font-specific; the built-in table targets
//!   DejaVu Sans.

use std::collections::HashMap;

/// Controls which typography features are active.
#[derive(Debug, Clone, Copy)]
pub struct TypographyOptions {
    /// Replace ASCII sequences with Unicode ligature characters
    /// (e.g. "fi" → "ﬁ", U+FB01).
    pub ligatures: bool,
    /// Adjust character spacing for common glyph pairs.
    pub kerning: bool,
}

impl Default for TypographyOptions {
    fn default() -> Self {
        Self {
            ligatures: true,
            kerning: true,
        }
    }
}

/// A kerning adjustment for a specific glyph pair, expressed in
/// thousandths of an em (the same unit used by the PDF `TJ`
/// operator).
pub type KerningValue = i16;

/// Pair-to-adjustment map for kerning.
#[derive(Debug, Clone, Default)]
pub struct KerningTable {
    map: HashMap<(char, char), KerningValue>,
}

impl KerningTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a kerning pair.  Positive values move glyphs apart,
    /// negative values pull them together.
    pub fn insert(&mut self, left: char, right: char, value: KerningValue) {
        self.map.insert((left, right), value);
    }

    pub fn get(&self, left: char, right: char) -> Option<KerningValue> {
        self.map.get(&(left, right)).copied()
    }

    /// Built-in table tuned for DejaVu Sans at 11 pt.
    pub fn dejavu_sans() -> Self {
        let mut t = Self::new();
        // Common kerning pairs (negative = tighter)
        t.insert('A', 'V', -50); t.insert('V', 'A', -50);
        t.insert('A', 'W', -40); t.insert('W', 'A', -40);
        t.insert('A', 'Y', -50); t.insert('Y', 'A', -50);
        t.insert('L', 'T', -40);
        t.insert('T', 'o', -30); t.insert('T', 'a', -30);
        t.insert('T', 'e', -30); t.insert('T', 'r', -30);
        t.insert('V', 'o', -30); t.insert('V', 'a', -30);
        t.insert('W', 'o', -20); t.insert('W', 'a', -20);
        t.insert('Y', 'o', -40); t.insert('Y', 'a', -40);
        t.insert('P', 'a', -20); t.insert('P', 'e', -20);
        t.insert('F', 'a', -30); t.insert('F', 'e', -30);
        t.insert('L', 'V', -30); t.insert('L', 'W', -20);
        t.insert('D', 'A', -20);
        t.insert('O', 'X', -20);
        t.insert('C', 'A', -10);
        t.insert('K', 'o', -20); t.insert('K', 'e', -20);
        t.insert('X', 'o', -20); t.insert('X', 'a', -20);
        t.insert('r', 't', -10);
        t.insert('a', 'v', -10);
        t.insert('e', 'x', -10);
        // Slight tightening around punctuation
        t.insert('L', ' ', -20);
        t.insert('T', ' ', -15);
        t.insert(' ', 'T', -10);
        t.insert(' ', 'V', -10);
        t.insert('f', 'f', -30);
        t.insert('f', 'i', -30);
        t.insert('f', 'l', -30);
        t
    }
}

/// A text segment together with an optional kerning adjustment
/// (in thousandths of an em) that follows it.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSegment {
    pub text: String,
    /// Adjustment applied *after* this segment, before the next one.
    pub adjustment: KerningValue,
}

/// Applies ligature substitution and optionally computes kerning
/// adjustments for a string.
#[derive(Debug, Clone)]
pub struct TypographyEngine {
    pub options: TypographyOptions,
    pub kerning: KerningTable,
}

impl Default for TypographyEngine {
    fn default() -> Self {
        Self {
            options: TypographyOptions::default(),
            kerning: KerningTable::dejavu_sans(),
        }
    }
}

impl TypographyEngine {
    pub fn new(options: TypographyOptions) -> Self {
        Self {
            options,
            kerning: KerningTable::dejavu_sans(),
        }
    }

    /// Replace common ligature sequences with Unicode ligature characters.
    ///
    /// Replacements (in order to avoid overlapping):
    /// - "ffi" → "ﬃ" (U+FB03)
    /// - "ffl" → "ﬄ" (U+FB04)
    /// - "ff"  → "ﬀ" (U+FB00)
    /// - "fi"  → "ﬁ" (U+FB01)
    /// - "fl"  → "ﬂ" (U+FB02)
    pub fn apply_ligatures(text: &str) -> String {
        let mut s = text.to_string();
        // Longer sequences first to avoid partial matches.
        s = s.replace("ffi", "\u{FB03}");
        s = s.replace("ffl", "\u{FB04}");
        s = s.replace("ff",  "\u{FB00}");
        s = s.replace("fi",  "\u{FB01}");
        s = s.replace("fl",  "\u{FB02}");
        s
    }

    /// Build a sequence of `TextSegment`s for `text`.  If kerning is
    /// disabled the result is a single segment with zero adjustment.
    pub fn segment(&self, text: &str) -> Vec<TextSegment> {
        if text.is_empty() {
            return Vec::new();
        }
        if !self.options.kerning {
            return vec![TextSegment {
                text: text.to_string(),
                adjustment: 0,
            }];
        }

        let mut segments = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();

        for i in 0..chars.len() {
            current.push(chars[i]);
            if i + 1 < chars.len()
                && let Some(adj) = self.kerning.get(chars[i], chars[i + 1])
            {
                segments.push(TextSegment {
                    text: current,
                    adjustment: adj,
                });
                current = String::new();
            }
        }

        if !current.is_empty() {
            segments.push(TextSegment {
                text: current,
                adjustment: 0,
            });
        }

        segments
    }

    /// Convenience: apply ligatures then segment for kerning.
    pub fn process(&self, text: &str) -> Vec<TextSegment> {
        let text = if self.options.ligatures {
            Self::apply_ligatures(text)
        } else {
            text.to_string()
        };
        self.segment(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_ligatures() {
        assert_eq!(
            TypographyEngine::apply_ligatures("fi fl ff ffi ffl"),
            "\u{FB01} \u{FB02} \u{FB00} \u{FB03} \u{FB04}"
        );
    }

    #[test]
    fn test_apply_ligatures_office() {
        assert_eq!(
            TypographyEngine::apply_ligatures("office"),
            "o\u{FB03}ce"
        );
    }

    #[test]
    fn test_kerning_table() {
        let table = KerningTable::dejavu_sans();
        assert_eq!(table.get('A', 'V'), Some(-50));
        assert_eq!(table.get('X', 'Y'), None);
    }

    #[test]
    fn test_segment_no_kerning() {
        let engine = TypographyEngine::new(TypographyOptions {
            ligatures: false,
            kerning: false,
        });
        let segs = engine.segment("hello");
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "hello");
        assert_eq!(segs[0].adjustment, 0);
    }

    #[test]
    fn test_segment_with_kerning() {
        let engine = TypographyEngine::default();
        let segs = engine.segment("AV");
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "A");
        assert_eq!(segs[0].adjustment, -50);
        assert_eq!(segs[1].text, "V");
        assert_eq!(segs[1].adjustment, 0);
    }

    #[test]
    fn test_process_ligatures_and_kerning() {
        let engine = TypographyEngine::default();
        let segs = engine.process("ffi");
        // ligatures turn "ffi" → "ﬃ" (single glyph)
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "\u{FB03}");
    }
}
