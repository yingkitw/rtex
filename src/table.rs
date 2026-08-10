//! Table parsing and PDF rendering.
//!
//! Handles `tabular` environments, including cell alignment and horizontal rules.

use serde::Serialize;

/// Horizontal alignment for a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// A single row in a table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Row {
    pub cells: Vec<String>,
    pub is_separator: bool,
}

/// A parsed `tabular` environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub columns: Vec<Align>,
    pub rows: Vec<Row>,
}

impl Table {
    /// Parse the raw content of a `tabular` environment into a structured table.
    ///
    /// `column_spec` is the string inside the braces after `\begin{tabular}`
    /// (e.g. `"|c|c|c|"` or `"l r"`).
    pub fn parse(column_spec: &str, content: &str) -> Self {
        let columns = parse_column_spec(column_spec);
        let mut rows = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("\\toprule")
                || line.starts_with("\\midrule")
                || line.starts_with("\\bottomrule")
                || line.starts_with("\\hline")
            {
                rows.push(Row {
                    cells: vec![],
                    is_separator: true,
                });
                continue;
            }

            if line.starts_with("\\") && !line.contains('&') {
                // Skip other commands (like \caption, \label inside table env)
                continue;
            }

            if line.contains('&') {
                let cells: Vec<String> = line
                    .split('&')
                    .map(|s| {
                        s.trim()
                            .trim_end_matches("\\\\")
                            .trim()
                            .replace("\\$", "$")
                            .to_string()
                    })
                    .collect();
                rows.push(Row {
                    cells,
                    is_separator: false,
                });
            }
        }

        Table { columns, rows }
    }
}

/// Parse a LaTeX column specification string into alignment values.
///
/// Ignores `|`, `@{}`, and `p{width}` — treats `p` as left-aligned.
fn parse_column_spec(spec: &str) -> Vec<Align> {
    let mut result = Vec::new();
    let mut chars = spec
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .chars();

    while let Some(ch) = chars.next() {
        match ch {
            'l' => result.push(Align::Left),
            'c' => result.push(Align::Center),
            'r' => result.push(Align::Right),
            'p' => {
                // p{width} — consume until matching }
                let mut depth = 0;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
                result.push(Align::Left);
            }
            '|' | ' ' | '\t' => {} // ignore
            '@' => {
                // @{} column spacing — skip until }
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_column_spec() {
        assert_eq!(
            parse_column_spec("|l|c|r|"),
            vec![Align::Left, Align::Center, Align::Right]
        );
        assert_eq!(
            parse_column_spec("{l c r}"),
            vec![Align::Left, Align::Center, Align::Right]
        );
        assert_eq!(parse_column_spec("cc"), vec![Align::Center, Align::Center]);
        assert_eq!(
            parse_column_spec("p{3cm}r"),
            vec![Align::Left, Align::Right]
        );
    }

    #[test]
    fn test_table_parse_basic() {
        let spec = "lcr";
        let content = "A & B & C \\\\\n\\hline\n1 & 2 & 3 \\\\";
        let table = Table::parse(spec, content);

        assert_eq!(
            table.columns,
            vec![Align::Left, Align::Center, Align::Right]
        );
        assert_eq!(table.rows.len(), 3);
        assert!(table.rows[1].is_separator);
        assert_eq!(table.rows[0].cells, vec!["A", "B", "C"]);
        assert_eq!(table.rows[2].cells, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_table_parse_with_dollar_escape() {
        let spec = "cc";
        let content = "$x$ & $y$ \\\\";
        let table = Table::parse(spec, content);

        assert_eq!(table.rows[0].cells, vec!["$x$", "$y$"]);
    }

    #[test]
    fn test_table_parse_booktabs_rules() {
        let spec = "lcr";
        let content = "\\toprule\nA & B & C \\\\\n\\midrule\n1 & 2 & 3 \\\\\n\\bottomrule";
        let table = Table::parse(spec, content);

        assert_eq!(table.rows.len(), 5);
        assert!(table.rows[0].is_separator); // toprule
        assert!(!table.rows[1].is_separator); // data
        assert!(table.rows[2].is_separator); // midrule
        assert!(!table.rows[3].is_separator); // data
        assert!(table.rows[4].is_separator); // bottomrule
    }

    #[test]
    fn test_table_parse_empty_content() {
        let table = Table::parse("ccc", "");
        assert!(table.rows.is_empty());
        assert_eq!(table.columns.len(), 3);
    }

    #[test]
    fn test_table_parse_at_spacing() {
        let spec = "l@{}r";
        let table = Table::parse(spec, "A & B \\\\");
        assert_eq!(table.columns, vec![Align::Left, Align::Right]);
    }

    #[test]
    fn test_table_parse_skips_non_table_commands() {
        let spec = "cc";
        let content = "\\caption{Test}\nA & B \\\\\n\\label{tab:1}";
        let table = Table::parse(spec, content);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].cells, vec!["A", "B"]);
    }

    #[test]
    fn test_table_parse_empty_lines() {
        let spec = "cc";
        let content = "\n\nA & B \\\\\n\n";
        let table = Table::parse(spec, content);
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_table_parse_p_column() {
        let spec = "p{5cm}c";
        let table = Table::parse(spec, "A & B \\\\");
        assert_eq!(table.columns, vec![Align::Left, Align::Center]);
    }
}
