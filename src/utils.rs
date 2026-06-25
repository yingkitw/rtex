//! Shared utility functions used across latex-rs modules.

/// Extract the content of a balanced brace group starting at `start`.
///
/// Returns `Some((content, end_index))` where `end_index` is the position
/// immediately after the matching `}`. Returns `None` if there is no
/// `{` at `start` or if braces are unbalanced.
///
/// # Examples
/// ```
/// let (content, end) = extract_braced("{hello}", 0).unwrap();
/// assert_eq!(content, "hello");
/// assert_eq!(end, 7);
/// ```
pub fn extract_braced(text: &str, start: usize) -> Option<(String, usize)> {
    if start >= text.len() || !text[start..].starts_with('{') {
        return None;
    }

    let mut depth = 0;
    let mut content_start = None;

    for (offset, ch) in text[start..].char_indices() {
        let index = start + offset;
        if ch == '{' {
            depth += 1;
            if depth == 1 {
                content_start = Some(index + ch.len_utf8());
            }
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let inner_start = content_start?;
                return Some((text[inner_start..index].to_string(), index + ch.len_utf8()));
            }
        }
    }

    None
}

/// Extract braced content from a string that is already inside the opening brace.
///
/// This is a convenience wrapper for cases where the caller has already
/// consumed the `{` and only needs the inner content.
pub fn extract_braced_inner(text: &str) -> Option<String> {
    let mut depth = 1; // already inside the opening brace
    let mut content = String::new();

    for ch in text.chars() {
        if ch == '{' {
            depth += 1;
            content.push(ch);
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(content);
            }
            content.push(ch);
        } else {
            content.push(ch);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_braced_simple() {
        let (content, end) = extract_braced("{hello}", 0).unwrap();
        assert_eq!(content, "hello");
        assert_eq!(end, 7);
    }

    #[test]
    fn test_extract_braced_nested() {
        let (content, end) = extract_braced("{a{b}c}", 0).unwrap();
        assert_eq!(content, "a{b}c");
        assert_eq!(end, 7);
    }

    #[test]
    fn test_extract_braced_with_offset() {
        let (content, end) = extract_braced("abc{def}ghi", 3).unwrap();
        assert_eq!(content, "def");
        assert_eq!(end, 8); // index past the closing brace
    }

    #[test]
    fn test_extract_braced_no_brace() {
        assert_eq!(extract_braced("hello", 0), None);
    }

    #[test]
    fn test_extract_braced_unbalanced() {
        assert_eq!(extract_braced("{hello", 0), None);
        assert_eq!(extract_braced("{he{llo}", 0), None);
    }

    #[test]
    fn test_extract_braced_empty() {
        let (content, end) = extract_braced("{}", 0).unwrap();
        assert_eq!(content, "");
        assert_eq!(end, 2);
    }

    #[test]
    fn test_extract_braced_inner() {
        assert_eq!(extract_braced_inner("hello}"), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_braced_inner_nested() {
        assert_eq!(extract_braced_inner("a{b}c}"), Some("a{b}c".to_string()));
    }

    #[test]
    fn test_extract_braced_inner_unbalanced() {
        assert_eq!(extract_braced_inner("hello"), None);
    }
}
