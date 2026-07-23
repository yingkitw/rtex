//! Lightweight display-math layout for PDF (stacked fractions, operator limits).
//!
//! This is intentionally a small subset of LaTeX — enough for common
//! Markdown math blocks without pulling in a full TeX engine.

use regex::Regex;

use super::text_support::{
    flatten_math_environments, normalize_math_display, parse_brace_group, render_math_text,
};

/// A laid-out piece of display mathematics.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum MathPiece {
    /// Ordinary run of already-rendered math text.
    Text(String),
    /// Large operator with limits (∑/∏: above/below; ∫: side scripts).
    Operator {
        symbol: char,
        lower: String,
        upper: String,
        /// When true, place limits to the right (integrals). Otherwise above/below.
        side_limits: bool,
    },
    /// Stacked fraction with a horizontal rule.
    Fraction {
        numerator: String,
        denominator: String,
    },
    /// Square / nth root with a vinculum over the radicand.
    Sqrt {
        index: Option<String>,
        radicand: String,
    },
    /// Matrix / grid with rows and columns, plus optional delimiters.
    Matrix {
        rows: Vec<Vec<Vec<MathPiece>>>,
        left_delim: String,
        right_delim: String,
    },
}

/// Parse a LaTeX-like math expression into display pieces, flattening matrix
/// environments to readable text (used for accessibility / plain text).
pub(super) fn parse_display_math(expr: &str) -> Vec<MathPiece> {
    parse_display_math_internal(expr, true)
}

/// Parse for actual PDF layout, keeping matrices as grids.
pub(super) fn parse_display_math_for_layout(expr: &str) -> Vec<MathPiece> {
    parse_display_math_internal(expr, false)
}

fn parse_display_math_internal(expr: &str, flatten_matrices: bool) -> Vec<MathPiece> {
    let mut s = if flatten_matrices {
        flatten_math_environments(expr.trim())
    } else {
        normalize_math_display(expr.trim())
    };
    if s.is_empty() {
        return Vec::new();
    }

    // Normalize common spacing commands early so token splits stay simple.
    s = s.replace("\\,", " ");
    s = s.replace("\\;", " ");
    s = s.replace("\\!", "");
    s = s.replace("\\quad", "  ");
    s = s.replace("\\qquad", "   ");
    s = s.replace("\\left", "");
    s = s.replace("\\right", "");

    let mut pieces = Vec::new();
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        // Skip whitespace but keep a single space as text when between tokens.
        if bytes[i].is_ascii_whitespace() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if !pieces.is_empty() {
                pieces.push(MathPiece::Text(" ".to_string()));
            }
            continue;
        }

        let rest = &s[i..];

        if let Some((piece, consumed)) = try_parse_operator(rest) {
            pieces.push(piece);
            i += consumed;
            continue;
        }

        if let Some((piece, consumed)) = try_parse_fraction(rest) {
            pieces.push(piece);
            i += consumed;
            continue;
        }

        if let Some((piece, consumed)) = try_parse_sqrt(rest) {
            pieces.push(piece);
            i += consumed;
            continue;
        }

        if let Some((piece, consumed)) = try_parse_matrix(rest) {
            pieces.push(piece);
            i += consumed;
            continue;
        }

        // Ordinary text until next special command or end.
        let next_special = find_next_special(rest);
        let chunk = &rest[..next_special];
        if !chunk.is_empty() {
            let rendered = render_math_text(chunk);
            if !rendered.is_empty() {
                pieces.push(MathPiece::Text(rendered));
            }
            i += chunk.len();
        } else {
            // Unknown backslash command: take one token and render it.
            let tok_end = rest
                .char_indices()
                .skip(1)
                .find(|(_, c)| {
                    c.is_whitespace() || *c == '\\' || *c == '{' || *c == '^' || *c == '_'
                })
                .map(|(idx, _)| idx)
                .unwrap_or(rest.len())
                .max(1);
            let tok = &rest[..tok_end];
            let rendered = render_math_text(tok);
            if !rendered.is_empty() {
                pieces.push(MathPiece::Text(rendered));
            }
            i += tok_end;
        }
    }

    // Merge adjacent text pieces and drop empty ones.
    coalesce_text_pieces(pieces)
}

fn find_next_special(s: &str) -> usize {
    let markers = [
        "\\sum",
        "\\prod",
        "\\int",
        "\\frac",
        "\\sqrt",
        "\\begin{pmatrix}",
        "\\begin{bmatrix}",
        "\\begin{vmatrix}",
        "\\begin{matrix}",
    ];
    let mut best = s.len();
    for m in markers {
        if let Some(pos) = s.find(m) {
            best = best.min(pos);
        }
    }
    best
}

fn try_parse_operator(s: &str) -> Option<(MathPiece, usize)> {
    let (symbol, side_limits, cmd_len) = if s.starts_with("\\sum") {
        ('∑', false, 4)
    } else if s.starts_with("\\prod") {
        ('∏', false, 5)
    } else if s.starts_with("\\int") {
        ('∫', true, 4)
    } else {
        return None;
    };

    let mut idx = cmd_len;
    let (lower, upper, consumed) = parse_limits(&s[idx..]);
    idx += consumed;

    Some((
        MathPiece::Operator {
            symbol,
            lower: render_math_text(&lower),
            upper: render_math_text(&upper),
            side_limits,
        },
        idx,
    ))
}

fn try_parse_fraction(s: &str) -> Option<(MathPiece, usize)> {
    if !s.starts_with("\\frac") {
        return None;
    }
    let mut idx = 5; // \frac
    let (num, nlen) = parse_brace_group(&s[idx..])?;
    idx += nlen;
    let (den, dlen) = parse_brace_group(&s[idx..])?;
    idx += dlen;
    Some((
        MathPiece::Fraction {
            numerator: num,
            denominator: den,
        },
        idx,
    ))
}

fn try_parse_sqrt(s: &str) -> Option<(MathPiece, usize)> {
    if !s.starts_with("\\sqrt") {
        return None;
    }
    let mut idx = 5;
    let mut index = None;
    if s[idx..].starts_with('[') {
        let close = s[idx + 1..].find(']')?;
        index = Some(s[idx + 1..idx + 1 + close].trim().to_string());
        idx += 1 + close + 1;
    }
    let (body, blen) = parse_brace_group(&s[idx..])?;
    idx += blen;
    Some((
        MathPiece::Sqrt {
            index,
            radicand: render_math_text(&body),
        },
        idx,
    ))
}

/// Parse an `align`-style expression (rows split on `\\`, columns on `&`) into a
/// delimiter-free matrix piece so it can share the matrix rendering path.
pub(super) fn parse_aligned_grid(expr: &str) -> Option<MathPiece> {
    let rows: Vec<Vec<Vec<MathPiece>>> = expr
        .split("\\\\")
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(|r| {
            r.split('&')
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .map(parse_display_math_for_layout)
                .collect()
        })
        .filter(|r: &Vec<Vec<MathPiece>>| !r.is_empty())
        .collect();
    if rows.is_empty() {
        return None;
    }
    Some(MathPiece::Matrix {
        rows,
        left_delim: String::new(),
        right_delim: String::new(),
    })
}

fn try_parse_matrix(s: &str) -> Option<(MathPiece, usize)> {
    const ENVS: [(&str, &str, &str); 4] = [
        ("pmatrix", "(", ")"),
        ("bmatrix", "[", "]"),
        ("vmatrix", "|", "|"),
        ("matrix", "", ""),
    ];
    for (env, left, right) in ENVS {
        let open = format!("\\begin{{{}}}", env);
        if !s.starts_with(&open) {
            continue;
        }
        let body_start = open.len();
        let close = format!("\\end{{{}}}", env);
        let rel_end = s[body_start..].find(&close)?;
        let body = &s[body_start..body_start + rel_end];
        let rows: Vec<Vec<Vec<MathPiece>>> = body
            .split("\\\\")
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .map(|r| {
                r.split('&')
                    .map(str::trim)
                    .filter(|c| !c.is_empty())
                    .map(parse_display_math_for_layout)
                    .collect()
            })
            .collect();
        let consumed = body_start + rel_end + close.len();
        return Some((
            MathPiece::Matrix {
                rows,
                left_delim: left.to_string(),
                right_delim: right.to_string(),
            },
            consumed,
        ));
    }
    None
}

/// Parse `_lower^upper`, `_{lower}^{upper}`, or mixed forms. Returns (lower, upper, bytes_consumed).
fn parse_limits(s: &str) -> (String, String, usize) {
    let mut idx = 0;
    let mut lower = String::new();
    let mut upper = String::new();

    for _ in 0..2 {
        let rest = &s[idx..];
        if rest.starts_with("_{") {
            if let Some((content, len)) = parse_brace_group(&rest[1..]) {
                lower = content;
                idx += 1 + len;
                continue;
            }
        }
        if rest.starts_with("^{") {
            if let Some((content, len)) = parse_brace_group(&rest[1..]) {
                upper = content;
                idx += 1 + len;
                continue;
            }
        }
        if let Some(caps) = Regex::new(r"^_([A-Za-z0-9+\-*/=]+)")
            .unwrap()
            .captures(rest)
        {
            lower = caps[1].to_string();
            idx += caps.get(0).unwrap().end();
            continue;
        }
        if let Some(caps) = Regex::new(r"^\^([A-Za-z0-9+\-*/=]+)")
            .unwrap()
            .captures(rest)
        {
            upper = caps[1].to_string();
            idx += caps.get(0).unwrap().end();
            continue;
        }
        break;
    }

    (lower, upper, idx)
}

fn coalesce_text_pieces(pieces: Vec<MathPiece>) -> Vec<MathPiece> {
    let mut out: Vec<MathPiece> = Vec::new();
    for piece in pieces {
        match piece {
            MathPiece::Text(t) if t.is_empty() => {}
            MathPiece::Text(t) => {
                if let Some(MathPiece::Text(prev)) = out.last_mut() {
                    prev.push_str(&t);
                } else {
                    out.push(MathPiece::Text(t));
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Estimated width of a display piece at the given body font size.
pub(super) fn piece_width(
    piece: &MathPiece,
    font_size: f32,
    measure: &dyn Fn(&str, f32) -> f32,
) -> f32 {
    match piece {
        MathPiece::Text(t) => measure(t, font_size),
        MathPiece::Operator {
            symbol,
            lower,
            upper,
            side_limits,
        } => {
            let op_size = font_size * 1.55;
            let script = font_size * 0.62;
            let sym = measure(&symbol.to_string(), op_size);
            if *side_limits {
                let lim_w = measure(lower, script).max(measure(upper, script));
                sym + 2.0 + lim_w
            } else {
                let lim_w = measure(lower, script).max(measure(upper, script));
                sym.max(lim_w)
            }
        }
        MathPiece::Fraction {
            numerator,
            denominator,
        } => {
            let script = font_size * 0.85;
            let num_pieces = parse_display_math(numerator);
            let den_pieces = parse_display_math(denominator);
            let nw: f32 = num_pieces
                .iter()
                .map(|p| piece_width(p, script, measure))
                .sum();
            let dw: f32 = den_pieces
                .iter()
                .map(|p| piece_width(p, script, measure))
                .sum();
            nw.max(dw) + 6.0
        }
        MathPiece::Sqrt { index, radicand } => {
            let script = font_size * 0.85;
            let rw = measure(radicand, script);
            let radical_w = font_size * 0.45;
            let index_w = index
                .as_ref()
                .map(|i| measure(&render_math_text(i), font_size * 0.55))
                .unwrap_or(0.0);
            index_w + radical_w + rw + 4.0
        }
        MathPiece::Matrix {
            rows,
            left_delim,
            right_delim,
        } => {
            if rows.is_empty() || rows.iter().all(|r| r.is_empty()) {
                return measure(left_delim, font_size) + measure(right_delim, font_size);
            }
            let script = font_size * 0.85;
            let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            let mut col_widths = vec![0.0f32; col_count];
            let mut row_heights = Vec::new();
            for row in rows {
                let mut row_height = 0.0f32;
                for (col_idx, cell) in row.iter().enumerate() {
                    let cell_width: f32 =
                        cell.iter().map(|p| piece_width(p, script, measure)).sum();
                    col_widths[col_idx] = col_widths[col_idx].max(cell_width);
                    let cell_height = line_height_for_pieces(cell, script);
                    row_height = row_height.max(cell_height);
                }
                row_heights.push(row_height);
            }
            let cell_pad = 8.0f32;
            let matrix_width =
                col_widths.iter().sum::<f32>() + cell_pad * (col_count.saturating_sub(1)) as f32;
            let matrix_height =
                row_heights.iter().sum::<f32>() + cell_pad * (rows.len().saturating_sub(1)) as f32;
            let delim_size = matrix_height.max(font_size);
            let delim_w = measure(left_delim, delim_size).max(measure(right_delim, delim_size));
            matrix_width + delim_w * 2.0 + cell_pad * 2.0
        }
    }
}

/// Total height of a display line of pieces (above + below math axis).
pub(super) fn line_height_for_pieces(pieces: &[MathPiece], font_size: f32) -> f32 {
    let mut ascent = font_size * 0.75;
    let mut descent = font_size * 0.35;
    for piece in pieces {
        match piece {
            MathPiece::Text(_) => {}
            MathPiece::Operator { side_limits, .. } => {
                if *side_limits {
                    ascent = ascent.max(font_size * 1.1);
                    descent = descent.max(font_size * 0.55);
                } else {
                    ascent = ascent.max(font_size * 1.85);
                    descent = descent.max(font_size * 1.15);
                }
            }
            MathPiece::Fraction { .. } => {
                ascent = ascent.max(font_size * 1.15);
                descent = descent.max(font_size * 1.15);
            }
            MathPiece::Sqrt { .. } => {
                ascent = ascent.max(font_size * 1.05);
                descent = descent.max(font_size * 0.35);
            }
            MathPiece::Matrix {
                rows,
                left_delim,
                right_delim,
            } => {
                let mut matrix_height = 0.0f32;
                for row in rows {
                    let mut row_height = 0.0f32;
                    for cell in row {
                        row_height = row_height.max(line_height_for_pieces(cell, font_size * 0.85));
                    }
                    matrix_height += row_height + 4.0;
                }
                if !rows.is_empty() {
                    matrix_height -= 4.0;
                }
                let delim_height = if left_delim.is_empty() && right_delim.is_empty() {
                    0.0
                } else {
                    font_size * 0.85
                };
                let total = matrix_height.max(delim_height);
                ascent = ascent.max(total * 0.55);
                descent = descent.max(total * 0.55);
            }
        }
    }
    ascent + descent + 4.0
}

/// Flatten display pieces to a readable single-line string (for extraction / a11y).
pub(super) fn pieces_to_plain_text(pieces: &[MathPiece]) -> String {
    let mut out = String::new();
    for piece in pieces {
        match piece {
            MathPiece::Text(t) => out.push_str(t),
            MathPiece::Operator {
                symbol,
                lower,
                upper,
                ..
            } => {
                out.push(*symbol);
                if !lower.is_empty() || !upper.is_empty() {
                    out.push('[');
                    out.push_str(lower);
                    out.push('→');
                    out.push_str(upper);
                    out.push(']');
                }
            }
            MathPiece::Fraction {
                numerator,
                denominator,
            } => {
                out.push('(');
                out.push_str(&pieces_to_plain_text(&parse_display_math(numerator)));
                out.push(')');
                out.push('/');
                out.push('(');
                out.push_str(&pieces_to_plain_text(&parse_display_math(denominator)));
                out.push(')');
            }
            MathPiece::Sqrt { index, radicand } => {
                if let Some(n) = index {
                    out.push_str("root(");
                    out.push_str(n);
                    out.push(',');
                    out.push_str(radicand);
                    out.push(')');
                } else {
                    out.push('√');
                    out.push('(');
                    out.push_str(radicand);
                    out.push(')');
                }
            }
            MathPiece::Matrix {
                rows,
                left_delim,
                right_delim,
            } => {
                out.push_str(left_delim);
                for (row_idx, row) in rows.iter().enumerate() {
                    if row_idx > 0 {
                        out.push_str("; ");
                    }
                    for (col_idx, cell) in row.iter().enumerate() {
                        if col_idx > 0 {
                            out.push(' ');
                        }
                        out.push_str(&pieces_to_plain_text(cell));
                    }
                }
                out.push_str(right_delim);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sqrt_with_vinculum_radicand() {
        let pieces = parse_display_math(r"\sqrt{x^2 + y^2}");
        assert!(
            pieces.iter().any(|p| matches!(
                p,
                MathPiece::Sqrt { radicand, index: None } if radicand.contains('x')
            )),
            "{pieces:?}"
        );
    }

    #[test]
    fn parses_nth_root() {
        let pieces = parse_display_math(r"\sqrt[3]{8}");
        assert!(
            pieces.iter().any(|p| matches!(
                p,
                MathPiece::Sqrt { index: Some(n), .. } if n == "3"
            )),
            "{pieces:?}"
        );
    }

    #[test]
    fn parses_integral_fraction_sum() {
        let pieces = parse_display_math(r"\int_{0}^{1} x^{2}\, dx = \frac{1}{3}");
        assert!(
            pieces
                .iter()
                .any(|p| matches!(p, MathPiece::Operator { symbol: '∫', .. })),
            "{:?}",
            pieces
        );
        assert!(
            pieces.iter().any(|p| matches!(
                p,
                MathPiece::Fraction {
                    numerator,
                    denominator
                } if numerator == "1" && denominator == "3"
            )),
            "{:?}",
            pieces
        );
    }

    #[test]
    fn parses_sum_with_complex_lower_limit() {
        let pieces = parse_display_math(r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}");
        assert!(
            pieces.iter().any(|p| matches!(
                p,
                MathPiece::Operator {
                    symbol: '∑',
                    lower,
                    upper,
                    side_limits: false
                } if lower.contains('k') && upper.contains('n')
            )),
            "{:?}",
            pieces
        );
    }

    #[test]
    fn flattens_bmatrix_environment() {
        let pieces = parse_display_math("\\begin{bmatrix}\na & b \\\\\nc & d\n\\end{bmatrix}");
        let plain = pieces_to_plain_text(&pieces);
        assert!(plain.contains('[') && plain.contains(']'), "{}", plain);
        assert!(plain.contains('a') && plain.contains('d'), "{}", plain);
        assert!(!plain.contains("begin"), "{}", plain);
    }

    #[test]
    fn parses_pmatrix_for_layout() {
        let pieces =
            parse_display_math_for_layout("\\begin{pmatrix} a & b \\\\\nc & d \\end{pmatrix}");
        assert!(
            pieces.iter().any(|p| matches!(
                p,
                MathPiece::Matrix { rows, left_delim, right_delim }
                if rows.len() == 2 && left_delim == "(" && right_delim == ")"
            )),
            "{pieces:?}"
        );
    }

    #[test]
    fn parses_aligned_grid() {
        let grid = parse_aligned_grid("x &= 1 \\\\\ny &= 2").expect("should parse aligned grid");
        match grid {
            MathPiece::Matrix {
                rows,
                left_delim,
                right_delim,
            } => {
                assert_eq!(rows.len(), 2);
                assert!(left_delim.is_empty());
                assert!(right_delim.is_empty());
            }
            _ => panic!("expected Matrix grid piece"),
        }
    }

    #[test]
    fn plain_text_is_readable() {
        let pieces = parse_display_math(r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}");
        let plain = pieces_to_plain_text(&pieces);
        assert!(plain.contains('∑'), "{}", plain);
        assert!(plain.contains('→') || plain.contains('k'), "{}", plain);
        assert!(plain.contains('/') || plain.contains("n+1"), "{}", plain);
    }
}
