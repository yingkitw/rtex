//! TeX category codes — the heart of the TeX lexer.
//!
//! Every character in a TeX source file has a *category code* that
//! determines its lexical meaning.  Changing category codes is how
//! TeX implements verbatim mode, active characters, and comment
//! stripping.

use std::collections::HashMap;

/// The 16 standard TeX category codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatCode {
    /// 0 — Escape character (backslash).
    Escape,
    /// 1 — Begin group (left brace).
    BeginGroup,
    /// 2 — End group (right brace).
    EndGroup,
    /// 3 — Math shift (dollar sign).
    MathShift,
    /// 4 — Alignment tab (ampersand).
    AlignmentTab,
    /// 5 — End of line (newline).
    EndOfLine,
    /// 6 — Parameter (hash / number sign).
    Parameter,
    /// 7 — Superscript (caret).
    Superscript,
    /// 8 — Subscript (underscore).
    Subscript,
    /// 9 — Ignored character.
    Ignored,
    /// 10 — Space.
    Space,
    /// 11 — Letter (a–z, A–Z).
    Letter,
    /// 12 — Other character (digits, punctuation, etc.).
    Other,
    /// 13 — Active character (treated like a control sequence).
    Active,
    /// 14 — Comment (percent sign).
    Comment,
    /// 15 — Invalid character.
    Invalid,
}

impl CatCode {
    /// Numeric value of the category code (0–15).
    pub fn as_u8(&self) -> u8 {
        match self {
            CatCode::Escape => 0,
            CatCode::BeginGroup => 1,
            CatCode::EndGroup => 2,
            CatCode::MathShift => 3,
            CatCode::AlignmentTab => 4,
            CatCode::EndOfLine => 5,
            CatCode::Parameter => 6,
            CatCode::Superscript => 7,
            CatCode::Subscript => 8,
            CatCode::Ignored => 9,
            CatCode::Space => 10,
            CatCode::Letter => 11,
            CatCode::Other => 12,
            CatCode::Active => 13,
            CatCode::Comment => 14,
            CatCode::Invalid => 15,
        }
    }
}

/// Maps characters to their current category codes.
#[derive(Debug, Clone)]
pub struct CatCodeTable {
    map: HashMap<char, CatCode>,
}

impl Default for CatCodeTable {
    fn default() -> Self {
        Self::new()
    }
}

impl CatCodeTable {
    /// Create a table with standard LaTeX category-code assignments.
    pub fn new() -> Self {
        let mut map = HashMap::new();

        // Standard defaults
        for c in 'a'..='z' {
            map.insert(c, CatCode::Letter);
        }
        for c in 'A'..='Z' {
            map.insert(c, CatCode::Letter);
        }
        for c in '0'..='9' {
            map.insert(c, CatCode::Other);
        }

        map.insert('\\', CatCode::Escape);
        map.insert('{', CatCode::BeginGroup);
        map.insert('}', CatCode::EndGroup);
        map.insert('$', CatCode::MathShift);
        map.insert('&', CatCode::AlignmentTab);
        map.insert('\n', CatCode::EndOfLine);
        map.insert('\r', CatCode::EndOfLine);
        map.insert('#', CatCode::Parameter);
        map.insert('^', CatCode::Superscript);
        map.insert('_', CatCode::Subscript);
        map.insert(' ', CatCode::Space);
        map.insert('\t', CatCode::Space);
        map.insert('%', CatCode::Comment);
        map.insert('~', CatCode::Active);

        // Remaining ASCII punctuation → Other
        for c in ['!', '"', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';',
                   '<', '=', '>', '?', '@', '[', ']', '`', '|'] {
            map.insert(c, CatCode::Other);
        }

        Self { map }
    }

    pub fn get(&self, ch: char) -> CatCode {
        self.map.get(&ch).copied().unwrap_or(CatCode::Other)
    }

    pub fn set(&mut self, ch: char, code: CatCode) {
        self.map.insert(ch, code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backslash_is_escape() {
        let table = CatCodeTable::new();
        assert_eq!(table.get('\\'), CatCode::Escape);
    }

    #[test]
    fn default_letter() {
        let table = CatCodeTable::new();
        assert_eq!(table.get('a'), CatCode::Letter);
        assert_eq!(table.get('Z'), CatCode::Letter);
    }

    #[test]
    fn default_digit_is_other() {
        let table = CatCodeTable::new();
        assert_eq!(table.get('5'), CatCode::Other);
    }

    #[test]
    fn override_category() {
        let mut table = CatCodeTable::new();
        table.set('@', CatCode::Letter);
        assert_eq!(table.get('@'), CatCode::Letter);
    }
}
