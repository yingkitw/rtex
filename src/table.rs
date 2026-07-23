//! Table parsing and PDF rendering.
//!
//! Handles `tabular` environments, including cell alignment and horizontal rules.

use crate::pdf::core::ContentStream;
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

/// Render a table into PDF content stream operations.
///
/// Returns the new Y position after the table.
pub fn render_table(
    table: &Table,
    stream: &mut ContentStream,
    left_margin: f32,
    y_position: f32,
    line_height: f32,
    content_width: f32,
) -> f32 {
    let mut current_y = y_position - 5.0;

    if table.columns.is_empty() || table.rows.is_empty() {
        return current_y - 10.0;
    }

    let col_width = content_width / table.columns.len() as f32;

    for row in &table.rows {
        if row.is_separator {
            // Draw a horizontal rule
            let rule_y = current_y + line_height * 0.4;
            stream.move_to(left_margin, rule_y);
            stream.line_to(left_margin + content_width, rule_y);
            stream.stroke();
            current_y -= line_height * 0.5;
            continue;
        }

        for (i, cell) in row.cells.iter().enumerate() {
            if i >= table.columns.len() {
                break;
            }
            if cell.is_empty() {
                continue;
            }

            let x_pos = left_margin + (i as f32 * col_width);
            let align = table.columns.get(i).copied().unwrap_or(Align::Left);

            // Calculate x offset for alignment
            let cell_width = estimate_text_width(cell);
            let x_offset = match align {
                Align::Left => 0.0,
                Align::Center => (col_width - cell_width) / 2.0,
                Align::Right => col_width - cell_width,
            };

            stream.begin_text();
            stream.set_font("F1", 10.0);
            stream.set_position(x_pos + x_offset, current_y);
            stream.show_text(cell);
            stream.end_text();
        }

        current_y -= line_height;
    }

    current_y - 10.0
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

/// Rough text width estimate in PDF points (1 char ≈ 5pt at 10pt font).
fn estimate_text_width(text: &str) -> f32 {
    text.chars().count() as f32 * 5.0
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
}
