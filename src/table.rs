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
                || line.starts_with("\\cline")
                || line.starts_with("\\cmidrule")
            {
                rows.push(Row {
                    cells: vec![],
                    is_separator: true,
                });
                continue;
            }

            // A full-width \multicolumn row has no `&` and would otherwise be
            // skipped as a command line — handle it explicitly.
            if line.starts_with("\\multicolumn") && !line.contains('&') {
                let cells = expand_multicolumn(line);
                rows.push(Row {
                    cells,
                    is_separator: false,
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
                    .flat_map(expand_multicolumn)
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

/// Consume a balanced `{...}` group from a char iterator, assuming the next
/// char to read is `{`. Used for `p`/`m`/`b{width}` and `@`/`>`/`<{...}` column
/// spec forms so their braced argument does not leak spurious alignment
/// columns (e.g. the `c` in `m{3cm}` or the `r` in `>{\raggedright}`).
fn consume_braced(chars: &mut std::str::Chars<'_>) {
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
}

/// Parse a LaTeX column specification string into alignment values.
///
/// Ignores `|`, `@{}`, `p`/`m`/`b{width}` (treated as left-aligned), and
/// `>`/`<` decorators (argument consumed, no column produced). `X`
/// (tabularx) is approximated as left-aligned. Expands `*{n}{spec}`
/// repetition (e.g. `*{3}{l}` → `lll`) before parsing.
fn parse_column_spec(spec: &str) -> Vec<Align> {
    let expanded = expand_repetitions(spec.trim());
    let mut result = Vec::new();
    let mut chars = expanded
        .trim_start_matches('{')
        .trim_end_matches('}')
        .chars();

    while let Some(ch) = chars.next() {
        match ch {
            'l' => result.push(Align::Left),
            'c' => result.push(Align::Center),
            'r' => result.push(Align::Right),
            // Paragraph-style columns: p/m/b{width} — consume the width, treat as left.
            'p' | 'm' | 'b' => {
                consume_braced(&mut chars);
                result.push(Align::Left);
            }
            // tabularx X column — expand-to-fill; approximated as left-aligned.
            'X' => result.push(Align::Left),
            '|' | ' ' | '\t' => {} // ignore
            // Decorators and spacing that take a {...} argument: @{}, >{...}, <{...}
            '@' | '>' | '<' => {
                consume_braced(&mut chars);
            }
            _ => {}
        }
    }

    result
}

/// Read a balanced `{...}` group starting at `s[start]`.
/// Returns `(inner, end)` where `end` is the index just after the closing `}`.
fn read_braced_group(s: &str, start: usize) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    if start >= bytes.len() || bytes[start] != b'{' {
        return None;
    }
    let mut depth = 0;
    let mut content_start = None;
    for (offset, ch) in s[start..].char_indices() {
        let idx = start + offset;
        if ch == '{' {
            depth += 1;
            if depth == 1 {
                content_start = Some(idx + ch.len_utf8());
            }
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let cs = content_start?;
                return Some((s[cs..idx].to_string(), idx + ch.len_utf8()));
            }
        }
    }
    None
}

/// Expand `*{n}{spec}` repetitions in a column spec.
///
/// `*{3}{l}` becomes `lll`; `*{2}{|c|}` becomes `|c||c|`. Nesting is supported.
/// Malformed `*` forms are emitted literally so the caller can ignore them.
fn expand_repetitions(spec: &str) -> String {
    let mut result = String::with_capacity(spec.len());
    let mut i = 0;
    while i < spec.len() {
        let rest = &spec[i..];
        if rest.starts_with('*')
            && let Some((n_str, after_n)) = read_braced_group(spec, i + 1)
            && let Ok(n) = n_str.trim().parse::<usize>()
            && let Some((sub, after_sub)) = read_braced_group(spec, after_n)
        {
            let expanded = expand_repetitions(&sub);
            for _ in 0..n {
                result.push_str(&expanded);
            }
            i = after_sub;
            continue;
        }
        let ch = spec[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

/// Normalise a raw table cell: trim whitespace, strip a trailing `\\` row break,
/// and unescape `\$` to `$`.
fn normalize_cell(s: &str) -> String {
    s.trim()
        .trim_end_matches("\\\\")
        .trim()
        .replace("\\$", "$")
        .to_string()
}

/// Expand a `\multicolumn{n}{align}{content}` cell into `n` cells: the content
/// followed by `n-1` empty cells. Non-multicolumn cells are normalised as-is.
/// This keeps `row.cells.len()` aligned with the column count so subsequent
/// rows line up, without requiring colspan support in the output backends.
fn expand_multicolumn(cell: &str) -> Vec<String> {
    let trimmed = cell.trim();
    let Some(rest) = trimmed.strip_prefix("\\multicolumn") else {
        return vec![normalize_cell(cell)];
    };
    let rest = rest.trim_start();
    let Some((n_str, after_n)) = read_braced_group(rest, 0) else {
        return vec![normalize_cell(cell)];
    };
    let Ok(n) = n_str.trim().parse::<usize>() else {
        return vec![normalize_cell(cell)];
    };
    let rest = rest[after_n..].trim_start();
    let Some((_align, after_align)) = read_braced_group(rest, 0) else {
        return vec![normalize_cell(cell)];
    };
    let rest = rest[after_align..].trim_start();
    let Some((content, _)) = read_braced_group(rest, 0) else {
        return vec![normalize_cell(cell)];
    };
    let n = n.max(1);
    let mut cells = Vec::with_capacity(n);
    cells.push(content.trim().to_string());
    for _ in 1..n {
        cells.push(String::new());
    }
    cells
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
    fn test_table_parse_cline_as_separator() {
        let spec = "ll";
        let content = "A & B \\\\\n\\cline{1-2}\nC & D \\\\";
        let table = Table::parse(spec, content);

        // \cline{1-2} must become a separator row, not be silently dropped.
        assert_eq!(table.rows.len(), 3, "cline should produce a separator row");
        assert!(!table.rows[0].is_separator); // A & B
        assert!(table.rows[1].is_separator); // \cline{1-2}
        assert!(!table.rows[2].is_separator); // C & D
    }

    #[test]
    fn test_table_parse_cmidrule_as_separator() {
        let spec = "lll";
        // booktabs \cmidrule with optional (lr) trim and a brace argument.
        let content = "\\toprule\nA & B & C \\\\\n\\cmidrule(lr){1-2}\n1 & 2 & 3 \\\\\n\\bottomrule";
        let table = Table::parse(spec, content);

        assert_eq!(table.rows.len(), 5);
        assert!(table.rows[0].is_separator); // toprule
        assert!(!table.rows[1].is_separator); // data
        assert!(table.rows[2].is_separator); // \cmidrule(lr){1-2}
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

    #[test]
    fn test_parse_column_spec_repetition() {
        assert_eq!(parse_column_spec("*{3}{l}"), vec![Align::Left, Align::Left, Align::Left]);
        assert_eq!(
            parse_column_spec("l*{2}{c}r"),
            vec![Align::Left, Align::Center, Align::Center, Align::Right]
        );
        // `|` inside the repeated spec is ignored, columns still counted
        assert_eq!(
            parse_column_spec("*{2}{|c|}"),
            vec![Align::Center, Align::Center]
        );
    }

    #[test]
    fn test_parse_column_spec_nested_repetition() {
        assert_eq!(
            parse_column_spec("*{2}{*{2}{l}}"),
            vec![Align::Left, Align::Left, Align::Left, Align::Left]
        );
    }

    #[test]
    fn test_table_parse_repetition_spec() {
        let table = Table::parse("*{3}{l}", "1 & 2 & 3 \\\\");
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.rows[0].cells, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_table_parse_multicolumn_in_row() {
        // \multicolumn{2}{c}{Header} spans the first two columns; the third
        // cell lands in column 3. The row should have 3 cells.
        let spec = "lll";
        let content = "\\multicolumn{2}{c}{Header} & x \\\\\n1 & 2 & 3 \\\\";
        let table = Table::parse(spec, content);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells, vec!["Header", "", "x"]);
        assert_eq!(table.rows[1].cells, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_table_parse_multicolumn_full_width() {
        // A full-width multicolumn row (no &) used to be silently skipped.
        let spec = "lll";
        let content = "\\multicolumn{3}{c}{Full title} \\\\\na & b & c \\\\";
        let table = Table::parse(spec, content);
        assert_eq!(table.rows.len(), 2, "full-width multicolumn row must be kept");
        assert_eq!(table.rows[0].cells, vec!["Full title", "", ""]);
        assert_eq!(table.rows[1].cells, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_table_parse_multicolumn_as_last_cell() {
        let spec = "lll";
        let content = "a & \\multicolumn{2}{c}{BC} \\\\";
        let table = Table::parse(spec, content);
        assert_eq!(table.rows[0].cells, vec!["a", "BC", ""]);
    }

    #[test]
    fn test_parse_column_spec_m_b_columns() {
        // Regression: m{3cm} and b{3cm} used to leak the inner 'c' as a spurious
        // Center column. They must consume the width and produce one Left column.
        assert_eq!(parse_column_spec("m{3cm}l"), vec![Align::Left, Align::Left]);
        assert_eq!(parse_column_spec("b{2cm}r"), vec![Align::Left, Align::Right]);
    }

    #[test]
    fn test_parse_column_spec_tabularx_x() {
        assert_eq!(parse_column_spec("XXX"), vec![Align::Left, Align::Left, Align::Left]);
        assert_eq!(parse_column_spec("lXr"), vec![Align::Left, Align::Left, Align::Right]);
    }

    #[test]
    fn test_parse_column_spec_decorator_braces() {
        // >{...} and <{...} decorators used to leak inner l/c/r as spurious columns.
        assert_eq!(parse_column_spec(">{\\bfseries}l"), vec![Align::Left]);
        assert_eq!(parse_column_spec("r<{\\hline}"), vec![Align::Right]);
        assert_eq!(parse_column_spec(">{\\raggedright}p{3cm}"), vec![Align::Left]);
    }
}
