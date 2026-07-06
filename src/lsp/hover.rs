//! Hover documentation for LaTeX commands.

use super::completion::command_documentation;

/// Return a short documentation string for a command at `offset`, if any.
pub fn hover_at(text: &str, offset: usize) -> Option<String> {
    let word = command_at_offset(text, offset)?;
    let name = word.trim_start_matches('\\');
    command_documentation(name).map(|detail| format!("`\\{name}` — {detail}"))
}

fn command_at_offset(text: &str, offset: usize) -> Option<String> {
    let safe = offset.min(text.len());
    let before = &text[..safe];
    let start = before.rfind('\\')?;
    let rest = &text[start..];
    let end = rest
        .chars()
        .skip(1)
        .take_while(|c| c.is_ascii_alphabetic() || *c == '*')
        .map(|c| c.len_utf8())
        .sum::<usize>()
        + 1;
    Some(rest[..end.min(rest.len())].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hovers_known_command() {
        let text = "\\section{Intro}";
        let hover = hover_at(text, 3).unwrap();
        assert!(hover.contains("Section heading"));
    }

    #[test]
    fn returns_none_for_unknown() {
        let text = "\\unknowncmd{x}";
        assert!(hover_at(text, 4).is_none());
    }
}
