//! Lightweight display-math layout for PDF (stacked fractions, operator limits).
//!
//! This is intentionally a small subset of LaTeX — enough for common
//! Markdown math blocks without pulling in a full TeX engine.

use regex::Regex;

use super::text_support::{flatten_math_environments, parse_brace_group, render_math_text};

use std::sync::OnceLock;

static LIMITS_LOWER_RE: OnceLock<Regex> = OnceLock::new();
static LIMITS_UPPER_RE: OnceLock<Regex> = OnceLock::new();

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
    /// Square root with a vinculum (overline) over the radicand.
    Radical {
        /// Optional root index (e.g. "3" for cube root), already rendered.
        index: String,
        /// The radicand, already rendered to Unicode text.
        radicand: String,
    },
    /// `\left<delim> ... \right<delim>` with delimiters scaled to the
    /// height of the grouped content (empty string = invisible `.`).
    Group {
        open: String,
        close: String,
        inner: Vec<MathPiece>,
    },
    /// Base with superscript/subscript (`x^{i+j}`, `a_1`, `a_n^m`),
    /// scripts rendered at script size with vertical offsets. Either
    /// script may be empty; base may be empty for bare `_{...}`.
    Scripts {
        base: String,
        sup: String,
        sub: String,
    },
}

/// Parse a LaTeX-like math expression into display pieces.
pub(super) fn parse_display_math(expr: &str) -> Vec<MathPiece> {
    let mut s = flatten_math_environments(expr.trim());
    if s.is_empty() {
        return Vec::new();
    }

    // Normalize common spacing commands early so token splits stay simple.
    s = s.replace("\\,", " ");
    s = s.replace("\\;", " ");
    s = s.replace("\\!", "");
    s = s.replace("\\quad", "  ");
    s = s.replace("\\qquad", "   ");

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

        if let Some((piece, consumed)) = try_parse_group(rest) {
            pieces.push(piece);
            i += consumed;
            continue;
        }

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

        if let Some((piece, consumed)) = try_parse_radical(rest) {
            pieces.push(piece);
            i += consumed;
            continue;
        }

        // Scripts LAST: large operators and groups consume their own
        // `^`/`_` limits first.
        if let Some((piece, consumed)) = try_parse_scripts(rest) {
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
                    c.is_whitespace() || *c == '\\' || *c == '{' || *c == '_'
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
    let markers = ["\\sum", "\\prod", "\\int", "\\frac", "\\sqrt", "\\left", "^", "_"];
    let mut best = s.len();
    for m in markers {
        if let Some(pos) = s.find(m) {
            best = best.min(pos);
        }
    }
    // Also stop before a lone backslash that starts a command after content.
    if let Some(pos) = s.find('\\') {
        // If the backslash is itself one of the markers, keep that.
        // Otherwise if it appears before best and is not part of already-handled
        // content, we still want ordinary render_math_text to handle it — so
        // only stop at markers for piece boundaries.
        let _ = pos;
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
            numerator: render_math_text(&num),
            denominator: render_math_text(&den),
        },
        idx,
    ))
}

fn try_parse_radical(s: &str) -> Option<(MathPiece, usize)> {
    if !s.starts_with("\\sqrt") {
        return None;
    }
    let mut idx = 5; // \sqrt
    let mut index = String::new();
    // Optional root index [n]
    if idx < s.len()
        && s.as_bytes()[idx] == b'['
        && let Some(end) = s[idx..].find(']')
    {
        index = render_math_text(&s[idx + 1..idx + end]);
        idx += end + 1;
    }
    let (radicand, rlen) = parse_brace_group(&s[idx..])?;
    idx += rlen;
    Some((
        MathPiece::Radical {
            index,
            radicand: render_math_text(&radicand),
        },
        idx,
    ))
}

/// Parse `<atom>^<arg>` / `<atom>_<arg>` (TeX scripts). The base atom
/// is a braced group, a backslash command, or a single non-operator
/// character; the base itself may be empty (`_{n}` alone). Either
/// script may be absent and both may appear (`a_n^m`).
fn try_parse_scripts(s: &str) -> Option<(MathPiece, usize)> {
    fn script_arg(s: &str) -> Option<(String, usize)> {
        if let Some(rest) = s.strip_prefix('{') {
            let end = rest.find('}')?;
            Some((rest[..end].to_string(), end + 2))
        } else if let Some(rest) = s.strip_prefix('\\') {
            let len: usize = rest
                .char_indices()
                .take_while(|(_, c)| c.is_alphabetic())
                .map(|(_, c)| c.len_utf8())
                .sum();
            Some((s[..1 + len].to_string(), 1 + len))
        } else {
            let c = s.chars().next()?;
            Some((c.to_string(), c.len_utf8()))
        }
    }

    let mut idx = 0usize;
    let mut base = String::new();
    // Optional base atom — never an operator/relation/punct char
    // (those belong to the surrounding expression, e.g. `x = y_1`).
    if let Some(c) = s.chars().next() {
        let is_op = "+-=<>,();[].|/!~".contains(c);
        if c == '{' {
            let rest = &s[1..];
            let end = rest.find('}')?;
            base = render_math_text(&rest[..end]);
            idx = 1 + end + 1;
        } else if c == '\\' {
            let len: usize = s[1..]
                .char_indices()
                .take_while(|(_, ch)| ch.is_alphabetic())
                .map(|(_, ch)| ch.len_utf8())
                .sum();
            idx = 1 + len;
            base = render_math_text(&s[..idx]);
        } else if !is_op && !c.is_whitespace() && c != '^' && c != '_' {
            idx = c.len_utf8();
            base = render_math_text(&s[..idx]);
        }
    }

    let mut sup = String::new();
    let mut sub = String::new();
    let mut seen = false;
    loop {
        if s[idx..].starts_with('^') {
            let (arg, len) = script_arg(&s[idx + 1..])?;
            // Strip operator spacing: script style is tight.
            sup = render_math_text(&arg).replace(' ', "");
            idx += 1 + len;
            seen = true;
        } else if s[idx..].starts_with('_') {
            let (arg, len) = script_arg(&s[idx + 1..])?;
            sub = render_math_text(&arg).replace(' ', "");
            idx += 1 + len;
            seen = true;
        } else {
            break;
        }
    }
    if !seen {
        return None;
    }
    Some((MathPiece::Scripts { base, sup, sub }, idx))
}

/// Parse `\left<delim> ... \right<delim>` into a scaled-delimiter
/// [`MathPiece::Group`]. Nesting is honored via depth counting;
/// unmatched `\left` yields `None` (caller falls back to text).
fn try_parse_group(s: &str) -> Option<(MathPiece, usize)> {
    if !s.starts_with("\\left") {
        return None;
    }
    let mut idx = 5; // \left
    let (open, olen) = parse_delim_token(&s[idx..]);
    idx += olen;

    let (right_pos, right_word_end) = find_matching_right(&s[idx..])?;
    let inner = parse_display_math(&s[idx..idx + right_pos]);
    idx += right_word_end;
    let (close, clen) = parse_delim_token(&s[idx..]);
    idx += clen;

    Some((MathPiece::Group { open, close, inner }, idx))
}

/// The delimiter token following `\left`/`\right`: a symbol, an
/// escaped brace, `.` (invisible), or a small set of commands.
fn parse_delim_token(s: &str) -> (String, usize) {
    for (tok, glyph, len) in [
        ("\\{", "{", 2),
        ("\\}", "}", 2),
        ("\\langle", "\u{27e8}", 7),
        ("\\rangle", "\u{27e9}", 7),
        ("\\lvert", "|", 6),
        ("\\rvert", "|", 6),
    ] {
        if s.starts_with(tok) {
            return (glyph.to_string(), len);
        }
    }
    match s.chars().next() {
        Some(c @ ('(' | '[' | '{' | '|' | '.' | ')' | ']' | '}')) => {
            let glyph: String = if c == '.' { String::new() } else { c.to_string() };
            (glyph, c.len_utf8())
        }
        _ => (String::new(), 0),
    }
}

/// Find the `\right` matching the `\left` at the start of `s`.
/// Returns (offset of `\right`, offset past the word `\right`).
fn find_matching_right(s: &str) -> Option<(usize, usize)> {
    let mut i = 0usize;
    let mut depth = 1i32;
    while i < s.len() {
        if s[i..].starts_with("\\left") {
            depth += 1;
            i += 5;
        } else if s[i..].starts_with("\\right") {
            depth -= 1;
            if depth == 0 {
                return Some((i, i + 6));
            }
            i += 6;
        } else {
            i += 1;
        }
    }
    None
}

/// Parse `_lower^upper`, `_{lower}^{upper}`, or mixed forms. Returns (lower, upper, bytes_consumed).
fn parse_limits(s: &str) -> (String, String, usize) {
    let mut idx = 0;
    let mut lower = String::new();
    let mut upper = String::new();
    let lower_re = LIMITS_LOWER_RE.get_or_init(|| Regex::new(r"^_([A-Za-z0-9+\-*/=]+)").unwrap());
    let upper_re = LIMITS_UPPER_RE.get_or_init(|| Regex::new(r"^\^([A-Za-z0-9+\-*/=]+)").unwrap());

    for _ in 0..2 {
        let rest = &s[idx..];
        if rest.starts_with("_{")
            && let Some((content, len)) = parse_brace_group(&rest[1..])
        {
            lower = content;
            idx += 1 + len;
            continue;
        }
        if rest.starts_with("^{")
            && let Some((content, len)) = parse_brace_group(&rest[1..])
        {
            upper = content;
            idx += 1 + len;
            continue;
        }
        if let Some(caps) = lower_re.captures(rest) {
            lower = caps[1].to_string();
            idx += caps.get(0).unwrap().end();
            continue;
        }
        if let Some(caps) = upper_re.captures(rest) {
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
            measure(numerator, script).max(measure(denominator, script)) + 6.0
        }
        MathPiece::Radical { index, radicand } => {
            let sym_w = measure("√", font_size * 1.3);
            let idx_w = if index.is_empty() { 0.0 } else { measure(index, font_size * 0.55) };
            let rad_w = measure(radicand, font_size);
            sym_w + idx_w + rad_w
        }
        MathPiece::Group { open, close, inner } => {
            let inner_h = line_height_for_pieces(inner, font_size);
            // Delimiter glyph visual height ≈ 0.95 em.
            let delim_size = (inner_h / 0.95).clamp(font_size, font_size * 3.5);
            let delim_w = if open.is_empty() { 0.0 } else { measure(open, delim_size) };
            let close_w = if close.is_empty() { 0.0 } else { measure(close, delim_size) };
            let inner_w: f32 = inner.iter().map(|p| piece_width(p, font_size, measure)).sum();
            delim_w + inner_w + close_w
        }
        MathPiece::Scripts { base, sup, sub } => {
            let script = font_size * 0.62;
            let base_w = measure(base, font_size);
            let script_w = measure(sup, script).max(measure(sub, script));
            base_w + script_w + 1.0
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
            MathPiece::Radical { .. } => {
                ascent = ascent.max(font_size * 1.05);
            }
            MathPiece::Group { inner, .. } => {
                // Delimiters extend slightly beyond the grouped content.
                let inner_h = line_height_for_pieces(inner, font_size);
                ascent = ascent.max(inner_h * 0.62);
                descent = descent.max(inner_h * 0.48);
            }
            MathPiece::Scripts { sup, sub, .. } => {
                let script = font_size * 0.62;
                if !sup.is_empty() {
                    ascent = ascent.max(script * 1.3);
                }
                if !sub.is_empty() {
                    descent = descent.max(script * 0.9);
                }
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
                out.push_str(numerator);
                out.push(')');
                out.push('/');
                out.push('(');
                out.push_str(denominator);
                out.push(')');
            }
            MathPiece::Radical { index, radicand } => {
                if !index.is_empty() {
                    out.push_str(index);
                }
                out.push('√');
                out.push('(');
                out.push_str(radicand);
                out.push(')');
            }
            MathPiece::Group { open, close, inner } => {
                out.push_str(open);
                out.push_str(&pieces_to_plain_text(inner));
                out.push_str(close);
            }
            MathPiece::Scripts { base, sup, sub } => {
                out.push_str(base);
                if !sup.is_empty() {
                    out.push('^');
                    out.push_str(sup);
                }
                if !sub.is_empty() {
                    out.push('_');
                    out.push_str(sub);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn plain_text_is_readable() {
        let pieces = parse_display_math(r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}");
        let plain = pieces_to_plain_text(&pieces);
        assert!(plain.contains('∑'), "{}", plain);
        assert!(plain.contains('→') || plain.contains('k'), "{}", plain);
        assert!(plain.contains('/') || plain.contains("n+1"), "{}", plain);
    }
}

/// Insert TeX-style spacing around math operators at group depth 0:
/// relations (`= < > ≤ ≥ ≠ ≈ ≡ ← → ↔ ⇒ ⇐ ⇔ ∝`) get a thick space
/// (two spaces), binary operators (`+ - ± ∗ × ·`) a medium space (one
/// space, only between atoms — unary minus stays tight), and
/// punctuation (`, ;`) a thin space after. Content inside groups is
/// left untouched: `{...}` always; `(...)`/`[...]` too when
/// `parens_as_groups` is set (inline math uses that conservative mode
/// so `lim(x→0)` and `∑[i=1→n]` limit notation stays tight).
pub fn apply_operator_spacing(expr: &str, parens_as_groups: bool) -> String {
    const RELATIONS: &[char] = &[
        '=', '<', '>', '≤', '≥', '≠', '≈', '≡', '∼', '←', '→', '↔', '⇒', '⇐', '⇔', '∝', '⊂',
        '⊃', '⊆', '⊇', '∈', '∉',
    ];
    const BINOPS: &[char] = &['+', '-', '±', '∗', '×', '·', '∖', '∪', '∩'];
    const ATOM_BREAK: &[char] = &['^', '_', '(', '[', '{', ',', ';'];

    let chars: Vec<char> = expr.chars().collect();
    let prev_atom = |i: usize| -> Option<char> {
        chars[..i].iter().rev().find(|c| !c.is_whitespace()).copied()
    };
    let next_atom = |i: usize| -> Option<char> {
        chars[i + 1..].iter().find(|c| !c.is_whitespace()).copied()
    };
    // A command start (`\`) behaves like an atom boundary; a sign
    // before an operand (`x=-y`) still allows relation spacing.
    let is_atomish = |c: char| c.is_alphanumeric() || c == ')' || c == '}' || c == '\\';

    let group_open = |c: char| parens_as_groups && (c == '(' || c == '[');
    let group_close = |c: char| parens_as_groups && (c == ')' || c == ']');

    // Depth tracks parens/brackets in inline mode only; braces do NOT
    // suppress spacing — TeX spaces binaries inside groups too (thin
    // script spacing), e.g. `\sqrt{b^2-4ac}` renders `b^2 - 4ac`.
    let mut out = String::with_capacity(expr.len() + 16);
    let mut depth = 0usize;
    for (i, &c) in chars.iter().enumerate() {
        match c {
            _ if group_open(c) => {
                depth += 1;
                out.push(c);
            }
            _ if group_close(c) => {
                depth = depth.saturating_sub(1);
                out.push(c);
            }
            _ if depth > 0 => out.push(c),
            _ if RELATIONS.contains(&c) => {
                let prev = prev_atom(i);
                let next = next_atom(i);
                let spaced = prev.is_some_and(is_atomish)
                    && next.is_some_and(|n| is_atomish(n) || n == '-' || n == '+');
                if spaced {
                    out.push_str("  ");
                    out.push(c);
                    out.push_str("  ");
                } else {
                    out.push(c);
                }
            }
            _ if BINOPS.contains(&c) => {
                let prev = prev_atom(i);
                let next = next_atom(i);
                let unary_left = prev.is_none()
                    || matches!(prev, Some('(' | '[' | '{' | '^' | '_' | ','))
                    || prev.is_some_and(|p| RELATIONS.contains(&p));
                let binary = !unary_left && next.is_some_and(is_atomish);
                if binary {
                    out.push(' ');
                    out.push(c);
                    out.push(' ');
                } else {
                    out.push(c);
                }
            }
            ',' | ';' => {
                out.push(c);
                if let Some(next) = next_atom(i)
                    && !ATOM_BREAK.contains(&next)
                {
                    out.push(' ');
                }
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod spacing_tests {
    use super::apply_operator_spacing;

    /// Display-math mode: parens are content, not groups.
    fn apply_operator_spacing_display(expr: &str) -> String {
        apply_operator_spacing(expr, false)
    }

    /// Inline-math mode: parens/brackets are groups (limit notation
    /// like `lim(x→0)` stays tight).
    fn apply_operator_spacing_inline(expr: &str) -> String {
        apply_operator_spacing(expr, true)
    }

    #[test]
    fn relations_get_thick_space() {
        assert_eq!(apply_operator_spacing_display("a+b=c"), "a + b  =  c");
        assert_eq!(apply_operator_spacing_display("x<y"), "x  <  y");
        assert_eq!(apply_operator_spacing_display("a≥b"), "a  ≥  b");
    }

    #[test]
    fn binary_ops_get_medium_space() {
        assert_eq!(apply_operator_spacing_display("x-y"), "x - y");
        assert_eq!(apply_operator_spacing_display("2×3"), "2 × 3");
        assert_eq!(apply_operator_spacing_display("a±b"), "a ± b");
    }

    #[test]
    fn unary_minus_stays_tight() {
        assert_eq!(apply_operator_spacing_display("-x"), "-x");
        assert_eq!(apply_operator_spacing_display("(-x)"), "(-x)");
        assert_eq!(apply_operator_spacing_display("f(-x)"), "f(-x)");
        assert_eq!(apply_operator_spacing_display("x=-y"), "x  =  -y");
    }

    #[test]
    fn comma_gets_thin_space_after() {
        assert_eq!(apply_operator_spacing_display("f(x,y)"), "f(x, y)");
        assert_eq!(apply_operator_spacing_display("a,b,c"), "a, b, c");
    }

    #[test]
    fn braces_do_not_block_spacing() {
        // TeX spaces binaries inside groups (thin script spacing).
        assert_eq!(apply_operator_spacing_display("a^{b+c}"), "a^{b + c}");
        assert_eq!(apply_operator_spacing_inline("x_{i+j}=y"), "x_{i + j}  =  y");
    }

    #[test]
    fn commands_and_greek_survive() {
        assert_eq!(apply_operator_spacing_display("\\alpha+\\beta=\\gamma"), "\\alpha + \\beta  =  \\gamma");
        assert_eq!(apply_operator_spacing_display("E=mc^2"), "E  =  mc^2");
    }

    #[test]
    fn no_spacing_at_boundaries() {
        assert_eq!(apply_operator_spacing_display("+x"), "+x");
        assert_eq!(apply_operator_spacing_display("x+"), "x+");
        assert_eq!(apply_operator_spacing_display(""), "");
    }

    #[test]
    fn inline_mode_keeps_limit_notation_tight() {
        assert_eq!(apply_operator_spacing_inline("lim(x→0)"), "lim(x→0)");
        assert_eq!(apply_operator_spacing_inline("∑[i=1→n]"), "∑[i=1→n]");
        assert_eq!(apply_operator_spacing_inline("f(x,y)"), "f(x,y)");
        assert_eq!(apply_operator_spacing_inline("a+b"), "a + b");
    }
}

#[cfg(test)]
mod group_tests {
    use super::*;

    #[test]
    fn parses_left_right_as_group() {
        let pieces = parse_display_math(r"\left( a+b \right)");
        assert_eq!(pieces.len(), 1, "got {pieces:?}");
        match &pieces[0] {
            MathPiece::Group { open, close, inner } => {
                assert_eq!(open, "(");
                assert_eq!(close, ")");
                assert_eq!(inner.len(), 1, "inner: {inner:?}");
            }
            other => panic!("expected Group, got {other:?}"),
        }
    }

    #[test]
    fn group_with_fraction_inner() {
        let pieces = parse_display_math(r"\left[\frac{1}{2}\right]");
        match &pieces[0] {
            MathPiece::Group { open, close, inner } => {
                assert_eq!(open, "[");
                assert_eq!(close, "]");
                assert!(matches!(inner[0], MathPiece::Fraction { .. }), "{inner:?}");
            }
            other => panic!("expected Group, got {other:?}"),
        }
    }

    #[test]
    fn nested_groups() {
        let pieces = parse_display_math(r"\left[\left(a\right)\right]");
        match &pieces[0] {
            MathPiece::Group { inner, .. } => match &inner[0] {
                MathPiece::Group { open, .. } => assert_eq!(open, "("),
                other => panic!("expected nested Group, got {other:?}"),
            },
            other => panic!("expected Group, got {other:?}"),
        }
    }

    #[test]
    fn invisible_delimiters() {
        let pieces = parse_display_math(r"\left. a \right.");
        match &pieces[0] {
            MathPiece::Group { open, close, .. } => {
                assert_eq!(open, "");
                assert_eq!(close, "");
            }
            other => panic!("expected Group, got {other:?}"),
        }
    }

    #[test]
    fn group_is_wider_than_inner_text() {
        let plain = parse_display_math("a+b");
        let grouped = parse_display_math(r"\left( a+b \right)");
        let measure = |t: &str, s: f32| t.chars().count() as f32 * s;
        let w_plain: f32 = plain.iter().map(|p| piece_width(p, 10.0, &measure)).sum();
        let w_group: f32 = grouped.iter().map(|p| piece_width(p, 10.0, &measure)).sum();
        assert!(w_group > w_plain, "{w_group} vs {w_plain}");
    }

    #[test]
    fn plain_text_recurses_into_groups() {
        let pieces = parse_display_math(r"\left( x \right)");
        let text = pieces_to_plain_text(&pieces);
        assert_eq!(text, "(x)");
    }

    #[test]
    fn unmatched_left_falls_back_without_crash() {
        let pieces = parse_display_math(r"\left(a + b");
        assert!(!pieces.is_empty());
    }
}

#[cfg(test)]
mod scripts_tests {
    use super::*;

    #[test]
    fn braced_superscript_parses() {
        let pieces = parse_display_math("x^{i+j}");
        match &pieces[0] {
            MathPiece::Scripts { base, sup, sub } => {
                assert_eq!(base, "x");
                assert_eq!(sup, "i+j");
                assert_eq!(sub, "");
            }
            other => panic!("expected Scripts, got {other:?}"),
        }
    }

    #[test]
    fn simple_subscript_parses() {
        let pieces = parse_display_math("a_1");
        match &pieces[0] {
            MathPiece::Scripts { base, sup, sub } => {
                assert_eq!(base, "a");
                assert_eq!(sub, "1");
                assert_eq!(sup, "");
            }
            other => panic!("expected Scripts, got {other:?}"),
        }
    }

    #[test]
    fn both_scripts_parse() {
        let pieces = parse_display_math("a_n^m");
        match &pieces[0] {
            MathPiece::Scripts { base, sup, sub } => {
                assert_eq!(base, "a");
                assert_eq!(sup, "m");
                assert_eq!(sub, "n");
            }
            other => panic!("expected Scripts, got {other:?}"),
        }
    }

    #[test]
    fn relations_keep_scripts_attached_to_operands() {
        // `x = y_1`: the `=` must NOT become a script base.
        let pieces = parse_display_math("x = y_1");
        assert!(
            pieces.iter().any(|p| matches!(p, MathPiece::Text(t) if t.contains('='))),
            "relation must stay text: {pieces:?}"
        );
        assert!(
            pieces.iter().any(|p| matches!(p, MathPiece::Scripts { sub, .. } if sub == "1")),
            "subscript must parse: {pieces:?}"
        );
    }

    #[test]
    fn scripts_are_wider_than_base_alone() {
        let with = parse_display_math("x^{i+j}");
        let without = parse_display_math("x");
        let measure = |t: &str, s: f32| t.chars().count() as f32 * s;
        let w = |ps: &[MathPiece]| ps.iter().map(|p| piece_width(p, 10.0, &measure)).sum::<f32>();
        assert!(w(&with) > w(&without));
    }

    #[test]
    fn plain_text_includes_scripts() {
        let pieces = parse_display_math("a_n^m");
        assert_eq!(pieces_to_plain_text(&pieces), "a^m_n");
    }

    #[test]
    fn operator_limits_still_win_over_scripts() {
        // `\sum_i^n` must parse as an Operator piece (limits), not Scripts.
        let pieces = parse_display_math(r"\sum_i^n a_i");
        assert!(matches!(pieces[0], MathPiece::Operator { .. }), "{pieces:?}");
    }
}
