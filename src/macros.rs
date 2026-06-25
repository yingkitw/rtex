//! Simple macro expansion engine for user-defined LaTeX commands.
//!
//! Supports `\newcommand` (with optional parameter count) and `\def`.
//! Macros are extracted from the source in a first pass, then all
//! invocations are expanded before the document is parsed.

use std::collections::HashMap;

/// A single macro definition.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDef {
    pub name: String,
    pub param_count: usize,
    pub body: String,
}

/// Storage and expansion engine for user-defined macros.
#[derive(Debug, Default)]
pub struct MacroStore {
    defs: HashMap<String, MacroDef>,
}

impl MacroStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan `text` for `\newcommand` and `\def` declarations and store them.
    /// The definitions themselves are removed from the returned string.
    pub fn extract_definitions(&mut self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut i = 0;

        while i < text.len() {
            let remaining = &text[i..];

            if remaining.starts_with("\\newcommand")
                && let Some(skip) = self.parse_newcommand(remaining)
            {
                i += skip;
                continue;
            }

            if remaining.starts_with("\\def")
                && let Some(skip) = self.parse_def(remaining)
            {
                i += skip;
                continue;
            }

            let ch = remaining.chars().next().unwrap_or('\0');
            result.push(ch);
            i += ch.len_utf8();
        }

        result
    }

    /// Expand every known macro invocation in `text`.
    pub fn expand_all(&self, text: &str) -> String {
        let mut result = text.to_string();
        // Limit recursion depth to avoid infinite loops.
        for _ in 0..10 {
            let next = self.expand_once(&result);
            if next == result {
                break;
            }
            result = next;
        }
        result
    }

    // ------------------------------------------------------------------
    // Parsing helpers
    // ------------------------------------------------------------------

    /// Parse `\newcommand{\name}[n]{body}` or `\newcommand\name[n]{body}`.
    /// Returns the number of bytes consumed on success.
    fn parse_newcommand(&mut self, text: &str) -> Option<usize> {
        let start = "\\newcommand".len();
        let rest = text.get(start..)?;

        // Skip optional whitespace
        let mut pos = 0;
        while pos < rest.len() && rest[pos..].starts_with(|c: char| c.is_whitespace()) {
            pos += rest[pos..].chars().next()?.len_utf8();
        }

        // Macro name: either {\name} or \name
        let (name, consumed) = if rest.get(pos..).is_some_and(|s| s.starts_with('{')) {
            let (inner, end) = extract_braced(rest, pos)?;
            let trimmed = inner.trim();
            let name = trimmed.strip_prefix('\\').unwrap_or(trimmed).to_string();
            (name, end)
        } else if rest.get(pos..).is_some_and(|s| s.starts_with('\\')) {
            let name_end = rest[pos + 1..]
                .find(|c: char| !c.is_alphabetic() && c != '*')
                .map(|i| pos + 1 + i)
                .unwrap_or(rest.len());
            (rest[pos + 1..name_end].to_string(), name_end)
        } else {
            return None;
        };
        pos = consumed;

        // Skip whitespace
        while pos < rest.len() && rest[pos..].starts_with(|c: char| c.is_whitespace()) {
            pos += rest[pos..].chars().next()?.len_utf8();
        }

        // Optional parameter count [n]
        let mut param_count = 0;
        if rest[pos..].starts_with('[') {
            let close = rest[pos..].find(']')?;
            let num_str = &rest[pos + 1..pos + close];
            param_count = num_str.parse().ok()?;
            pos += close + 1;
        }

        // Skip whitespace
        while pos < rest.len() && rest[pos..].starts_with(|c: char| c.is_whitespace()) {
            pos += rest[pos..].chars().next()?.len_utf8();
        }

        // Body
        let (body, end) = extract_braced(rest, pos)?;
        pos = end;

        self.defs.insert(
            name.clone(),
            MacroDef {
                name,
                param_count,
                body,
            },
        );

        Some(start + pos)
    }

    /// Parse `\def\name#1#2...{body}`.
    /// Returns the number of bytes consumed on success.
    fn parse_def(&mut self, text: &str) -> Option<usize> {
        let start = "\\def".len();
        let rest = text.get(start..)?;

        let mut pos = 0;
        while pos < rest.len() && rest[pos..].starts_with(|c: char| c.is_whitespace()) {
            pos += rest[pos..].chars().next()?.len_utf8();
        }

        if !rest[pos..].starts_with('\\') {
            return None;
        }

        let name_end = rest[pos + 1..]
            .find(|c: char| !c.is_alphabetic() && c != '*')
            .map(|i| pos + 1 + i)
            .unwrap_or(rest.len());
        let name = rest[pos + 1..name_end].to_string();
        pos = name_end;

        // Count parameter tokens #1, #2, ...
        let mut param_count = 0;
        while let Some(digit) = rest.get(pos + 1..).and_then(|s| s.chars().next())
            .filter(|c| c.is_ascii_digit())
        {
            let n = digit.to_digit(10).unwrap() as usize;
            param_count = param_count.max(n);
            pos += 1 + digit.len_utf8();
        }

        // Body
        let (body, end) = extract_braced(rest, pos)?;
        pos = end;

        self.defs.insert(
            name.clone(),
            MacroDef {
                name,
                param_count,
                body,
            },
        );

        Some(start + pos)
    }

    // ------------------------------------------------------------------
    // Expansion helpers
    // ------------------------------------------------------------------

    fn expand_once(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        let mut i = 0;

        while i < text.len() {
            let remaining = &text[i..];

            if remaining.starts_with("\\newcommand") || remaining.starts_with("\\def") {
                // Should have been stripped during extraction, but skip just in case.
                let cmd = if remaining.starts_with("\\newcommand") {
                    "\\newcommand"
                } else {
                    "\\def"
                };
                result.push_str(cmd);
                i += cmd.len();
                continue;
            }

            #[allow(clippy::manual_strip)]
            if remaining.starts_with('\\') {
                // Try to match a macro name.
                let name_end = if remaining.len() > 1 {
                    remaining[1..]
                        .find(|c: char| !c.is_alphabetic() && c != '*')
                        .map(|i| i + 1)
                        .unwrap_or(remaining.len())
                } else {
                    remaining.len()
                };
                let name = &remaining[1..name_end];

                if let Some(def) = self.defs.get(name) {
                    let after = &remaining[name_end..];
                    let (args, consumed) = self.collect_args(after, def.param_count);
                    let expanded = self.substitute(&def.body, &args);
                    result.push_str(&expanded);
                    i += name_end + consumed;
                    continue;
                }
            }

            let ch = remaining.chars().next().unwrap_or('\0');
            result.push(ch);
            i += ch.len_utf8();
        }

        result
    }

    /// Collect `count` braced arguments from the start of `text`.
    /// Returns `(args, bytes_consumed)`.
    fn collect_args(&self, text: &str, count: usize) -> (Vec<String>, usize) {
        let mut args = Vec::with_capacity(count);
        let mut pos = 0;

        for _ in 0..count {
            while pos < text.len() && text[pos..].starts_with(|c: char| c.is_whitespace()) {
                pos += text[pos..].chars().next().unwrap().len_utf8();
            }
            if text.get(pos..).is_some_and(|s| s.starts_with('{'))
                && let Some((inner, end)) = extract_braced(text, pos)
            {
                args.push(inner);
                pos = end;
                continue;
            }
            // Failed to collect expected arg — stop.
            break;
        }

        (args, pos)
    }

    fn substitute(&self, body: &str, args: &[String]) -> String {
        let mut result = String::with_capacity(body.len() * 2);
        let mut i = 0;
        while i < body.len() {
            if body.get(i..).is_some_and(|s| s.starts_with('#'))
                && let Some(ch) = body.get(i + 1..).and_then(|s| s.chars().next())
                && let Some(n) = ch.to_digit(10)
            {
                let idx = n as usize;
                if idx > 0 && idx <= args.len() {
                    result.push_str(&args[idx - 1]);
                }
                i += 1 + ch.len_utf8();
                continue;
            }
            let c = body[i..].chars().next().unwrap();
            result.push(c);
            i += c.len_utf8();
        }
        result
    }
}

// Re-use the local helper for balanced braces.
fn extract_braced(text: &str, start: usize) -> Option<(String, usize)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newcommand_no_args() {
        let mut store = MacroStore::new();
        let cleaned = store.extract_definitions("\\newcommand{\\hello}{Hello World} begin");
        assert_eq!(cleaned, " begin");
        assert_eq!(store.defs["hello"].body, "Hello World");
        assert_eq!(store.defs["hello"].param_count, 0);
    }

    #[test]
    fn test_newcommand_with_args() {
        let mut store = MacroStore::new();
        let cleaned = store.extract_definitions("\\newcommand{\\greet}[1]{Hello, #1!} text");
        assert_eq!(cleaned, " text");
        assert_eq!(store.defs["greet"].body, "Hello, #1!");
        assert_eq!(store.defs["greet"].param_count, 1);
    }

    #[test]
    fn test_def_with_params() {
        let mut store = MacroStore::new();
        let cleaned = store.extract_definitions("\\def\\foo#1#2{#1 loves #2} end");
        assert_eq!(cleaned, " end");
        assert_eq!(store.defs["foo"].body, "#1 loves #2");
        assert_eq!(store.defs["foo"].param_count, 2);
    }

    #[test]
    fn test_expand_no_args() {
        let mut store = MacroStore::new();
        store.extract_definitions("\\newcommand{\\hi}{Hi}");
        assert_eq!(store.expand_all("Say \\hi."), "Say Hi.");
    }

    #[test]
    fn test_expand_with_args() {
        let mut store = MacroStore::new();
        store.extract_definitions("\\newcommand{\\greet}[1]{Hello, #1!}");
        assert_eq!(store.expand_all("\\greet{World}"), "Hello, World!");
    }

    #[test]
    fn test_expand_def() {
        let mut store = MacroStore::new();
        store.extract_definitions("\\def\\twice#1{#1 #1}");
        assert_eq!(store.expand_all("\\twice{hello}"), "hello hello");
    }

    #[test]
    fn test_multiple_macros() {
        let mut store = MacroStore::new();
        store.extract_definitions("\\newcommand{\\a}{A}\\newcommand{\\b}{B}");
        assert_eq!(store.expand_all("\\a and \\b"), "A and B");
    }

    #[test]
    fn test_unknown_command_preserved() {
        let store = MacroStore::new();
        assert_eq!(store.expand_all("\\unknown{test}"), "\\unknown{test}");
    }
}
