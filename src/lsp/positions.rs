//! Byte-offset ↔ line/column conversion for LSP positions.

/// A half-open range in the source text (`start` inclusive, `end` exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TexRange {
    pub start: usize,
    pub end: usize,
}

impl TexRange {
    pub fn single(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset.saturating_add(1),
        }
    }

    pub fn span(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// 0-indexed line and character position (LSP convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TexPosition {
    pub line: u32,
    pub character: u32,
}

/// Convert a byte offset to an LSP position.
pub fn offset_to_position(text: &str, offset: usize) -> TexPosition {
    let offset = offset.min(text.len());
    let mut line = 0u32;
    let mut character = 0u32;
    let mut current = 0usize;

    for ch in text.chars() {
        if current >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
        current += ch.len_utf8();
    }

    TexPosition { line, character }
}

/// Convert a byte range to LSP start/end positions.
pub fn range_to_positions(text: &str, range: TexRange) -> (TexPosition, TexPosition) {
    (
        offset_to_position(text, range.start),
        offset_to_position(text, range.end),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_at_start() {
        let pos = offset_to_position("abc\ndef", 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn offset_after_newline() {
        let text = "abc\ndef";
        let pos = offset_to_position(text, 4);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn offset_past_end_clamps() {
        let pos = offset_to_position("hi", 99);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 2);
    }
}
