//! TeX-style horizontal list model and total-fit line breaking.
//!
//! Paragraphs are modeled as a horizontal list of [Node]s:
//! - [`Node::Box`] is an indivisible item (a word) with a fixed width.
//! - [`Node::Glue`] is breakable space with a natural width that can
//!   stretch (inter-word justification) or shrink (tight lines).
//! - [`Node::Penalty`] marks a breakpoint; cost <= [`FORCED_BREAK`]
//!   forces a break there, cost >= [`FORBIDDEN_BREAK`] forbids one.
//!
//! [`break_lines`] chooses breakpoints by minimizing total demerits
//! (Knuth–Plass dynamic programming with fitness classes). When a line
//! breaks at glue or a penalty, that node is discarded, exactly as in
//! TeX. The final line of the paragraph is never stretched (TeX's
//! `\parfillskip` gives it infinite stretch), so its ratio is 0.
//!
//! If no sequence of lines satisfies the tolerance (e.g. a single word
//! wider than the line, or all-glue-zero paragraphs), the breaker falls
//! back to greedy first-fit, which always produces a complete covering
//! of the input.
//!
//! Line output is a sequence of [`Line`]s. Each line covers a half-open
//! range `start..end` of the node list and carries an adjustment
//! `ratio`: positive ratios stretch glue, negative ratios shrink it.

/// Penalty cost that forces a break (TeX's -10000).
pub const FORCED_BREAK: f32 = -10_000.0;
/// Penalty cost at or above which a break is forbidden (TeX's +10000).
pub const FORBIDDEN_BREAK: f32 = 10_000.0;

/// Infeasible ratios are clamped to this magnitude (instead of
/// infinity) so demerits stay finite and chains remain representable.
const RATIO_CLAMP: f32 = 1_000_000.0;

/// One item in a horizontal list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Node {
    /// An indivisible box (e.g. a word) with a fixed width.
    Box { width: f32 },
    /// Breakable space: natural `width`, may grow by `stretch` or
    /// shrink by `shrink` (both >= 0).
    Glue {
        width: f32,
        stretch: f32,
        shrink: f32,
    },
    /// A breakpoint. Breaks are allowed when `cost` <
    /// [`FORBIDDEN_BREAK`]; `cost` <= [`FORCED_BREAK`] forces a break.
    Penalty { cost: f32 },
    /// A hyphenation breakpoint (TeX's `\discretionary{-}{}{}`): a
    /// break is allowed here with demerit cost `cost`; when the break
    /// is taken, a hyphen of width `pre_width` is added to the END of
    /// the line. When no break is taken it contributes zero width and
    /// is not drawn.
    Hyphen { cost: f32, pre_width: f32 },
}

/// A line chosen by the breaker: the half-open drawn range `start..end`
/// and the adjustment ratio applied to every glue in the range. The
/// node at the breakpoint itself (glue or penalty) is discarded and is
/// not part of any line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// Index of the first drawn node of the line.
    pub start: usize,
    /// One past the index of the last drawn node of the line.
    pub end: usize,
    /// Adjustment ratio: `> 0` stretches glue, `< 0` shrinks it, `0`
    /// leaves natural spacing (always `0` for the final line).
    pub ratio: f32,
}

/// Total-fit line breaking with greedy fallback.
///
/// `line_width` is the target width of every line. `tolerance` bounds
/// the acceptable adjustment ratio on non-final lines (TeX uses values
/// around 1.0–4.0 via `\tolerance`).
pub fn break_lines(nodes: &[Node], line_width: f32, tolerance: f32) -> Vec<Line> {
    break_lines_with_first(nodes, line_width, line_width, tolerance)
}

/// Like [`break_lines`] but with a distinct first-line width (TeX's
/// `\parindent` shortens the first line of an indented paragraph).
pub fn break_lines_with_first(
    nodes: &[Node],
    line_width: f32,
    first_line_width: f32,
    tolerance: f32,
) -> Vec<Line> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let candidates = legal_breakpoints(nodes);
    if let Some(lines) = total_fit(nodes, line_width, first_line_width, tolerance, &candidates) {
        return lines;
    }
    greedy_fallback(nodes, line_width, first_line_width, &candidates)
}

/// Line width in effect for a line STARTING at `start`: the first
/// line of the paragraph uses `first_line_width`.
fn line_target(start: usize, line_width: f32, first_line_width: f32) -> f32 {
    if start == 0 {
        first_line_width
    } else {
        line_width
    }
}

/// Indices that are legal breakpoints: glue with positive width
/// preceded by a box, or any penalty below [`FORBIDDEN_BREAK`]. The
/// end of the list is an implicit final breakpoint handled separately.
fn legal_breakpoints(nodes: &[Node]) -> Vec<usize> {
    let mut out = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        match node {
            Node::Glue { width, .. } => {
                if *width > 0.0 && i > 0 && matches!(nodes[i - 1], Node::Box { .. }) {
                    out.push(i);
                }
            }
            Node::Penalty { cost } => {
                if *cost < FORBIDDEN_BREAK {
                    out.push(i);
                }
            }
            Node::Hyphen { .. } => out.push(i),
            Node::Box { .. } => {}
        }
    }
    out
}

fn is_forced(node: &Node) -> bool {
    matches!(node, Node::Penalty { cost } if *cost <= FORCED_BREAK)
}

/// Demerit cost contributed by breaking AT node `brk` (its penalty
/// value, if any). TeX adds the break penalty to the total.
fn break_cost(nodes: &[Node], brk: usize) -> f32 {
    match nodes.get(brk) {
        Some(Node::Hyphen { cost, .. }) | Some(Node::Penalty { cost }) => *cost,
        _ => 0.0,
    }
}

/// Knuth–Plass dynamic programming over candidate breakpoints.
/// Returns `None` when no sequence of lines satisfies the tolerance.
fn total_fit(
    nodes: &[Node],
    line_width: f32,
    first_line_width: f32,
    tolerance: f32,
    candidates: &[usize],
) -> Option<Vec<Line>> {
    // Sentinel index standing for "end of paragraph".
    let end_all = nodes.len();
    let n = candidates.len() + 1; // +1 for the sentinel
    let sentinel = n - 1;

    // best[i] = (min cost, previous candidate index) for a chain whose
    // last break is candidates[i] (or the sentinel for i == sentinel).
    let mut best: Vec<(f32, Option<usize>)> = vec![(f32::INFINITY, None); n];
    let mut fitness: Vec<u8> = vec![1; n];

    let feasible = |start: usize, brk: usize, is_final: bool| -> bool {
        let target = line_target(start, line_width, first_line_width);
        if is_final {
            // The last line gains TeX's `\parfillskip` (infinite
            // stretch, no shrink): positive deficits are always
            // absorbed; negative deficits must come out of interword
            // shrink within tolerance.
            if drawn_range_has_forced(nodes, start, end_all) {
                // A forced penalty must be broken at, even on the
                // paragraph's last line.
                return false;
            }
            let ratio = final_ratio(nodes, start, end_all, target);
            return ratio >= -tolerance;
        }
        if drawn_range_has_forced(nodes, start, brk) {
            // A forced penalty inside the line must be broken at.
            return false;
        }
        if brk < end_all && is_forced(&nodes[brk]) {
            // Breaks at forced penalties are always allowed.
            return true;
        }
        let ratio = line_ratio(nodes, start, brk, target);
        ratio <= tolerance && ratio >= -tolerance
    };

    // First lines start at node 0.
    for i in 0..n {
        let brk = if i == sentinel { end_all } else { candidates[i] };
        let is_final = i == sentinel;
        if feasible(0, brk, is_final) {
            let ratio = chosen_ratio(nodes, 0, brk, first_line_width, is_final);
            best[i] = (line_demerits(ratio) + break_cost(nodes, brk), None);
            fitness[i] = fitness_class(ratio);
        }
    }

    for i in 0..n {
        if best[i].0.is_infinite() {
            continue;
        }
        let start = if i == sentinel { 0 } else { candidates[i] + 1 };
        if start > end_all {
            continue;
        }
        for j in (i + 1)..n {
            let brk = if j == sentinel { end_all } else { candidates[j] };
            let is_final = j == sentinel;
            if !feasible(start, brk, is_final) {
                continue;
            }
            let target = line_target(start, line_width, first_line_width);
            let ratio = chosen_ratio(nodes, start, brk, target, is_final);
            let mut cost = best[i].0 + line_demerits(ratio) + break_cost(nodes, brk);
            if fitness_incompatible(fitness[i], fitness_class(ratio)) {
                cost += 3_000.0;
            }
            if cost < best[j].0 {
                best[j] = (cost, Some(i));
                fitness[j] = fitness_class(ratio);
            }
        }
    }

    if best[sentinel].0.is_infinite() {
        return None;
    }

    // Walk the chain backwards collecting break indices.
    let mut idxs = Vec::new();
    let mut cur = Some(sentinel);
    while let Some(i) = cur {
        idxs.push(i);
        cur = best[i].1;
    }
    idxs.reverse();

    let mut lines = Vec::with_capacity(idxs.len());
    let mut start = 0usize;
    for &i in &idxs {
        let brk = if i == sentinel { end_all } else { candidates[i] };
        let is_final = i == sentinel;
        if start < brk {
            let target = line_target(start, line_width, first_line_width);
            let ratio = chosen_ratio(nodes, start, brk, target, is_final);
            lines.push(Line { start, end: brk, ratio });
        }
        start = brk + if is_final { 0 } else { 1 };
    }
    if lines.is_empty() {
        return None;
    }
    Some(lines)
}

/// Greedy first-fit fallback: within each segment between forced
/// breaks, fill lines to the target width. Always covers the input.
fn greedy_fallback(
    nodes: &[Node],
    line_width: f32,
    first_line_width: f32,
    candidates: &[usize],
) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    while start < nodes.len() {
        // A forced penalty caps the line (the break must occur there).
        let forced = (start..nodes.len()).find(|&k| is_forced(&nodes[k]));
        let target = line_target(start, line_width, first_line_width);
        let end = match forced {
            Some(k) if k == start => {
                // Zero-width forced break; emit no line.
                start = k + 1;
                continue;
            }
            Some(k) => k,
            None => {
                let end_all = nodes.len();
                if line_natural(nodes, start, end_all) <= target {
                    end_all
                } else {
                    candidates
                        .iter()
                        .copied()
                        .filter(|&b| b > start)
                        .take_while(|&b| line_natural(nodes, start, b) <= target)
                        .last()
                        .unwrap_or_else(|| {
                            // Single overwide box: dump it on its own line.
                            candidates
                                .iter()
                                .copied()
                                .find(|&b| b > start)
                                .unwrap_or(end_all)
                        })
                }
            }
        };
        let is_final = end == nodes.len();
        let ratio = if is_final {
            final_ratio(nodes, start, end, target)
        } else {
            line_ratio(nodes, start, end, target)
        };
        lines.push(Line { start, end, ratio });
        start = end + 1;
    }
    lines
}

/// Natural width of the drawn range `start..end` (penalties count 0).
/// If the break at `end` is a hyphen, its width is included: taking a
/// hyphen break appends the hyphen glyph to the line.
fn line_natural(nodes: &[Node], start: usize, end: usize) -> f32 {
    let mut w: f32 = nodes[start..end.min(nodes.len())]
        .iter()
        .map(|node| match node {
            Node::Box { width } => *width,
            Node::Glue { width, .. } => *width,
            Node::Penalty { .. } | Node::Hyphen { .. } => 0.0,
        })
        .sum();
    if let Some(Node::Hyphen { pre_width, .. }) = nodes.get(end) {
        w += pre_width;
    }
    w
}

/// Total stretch/shrink of glue in the drawn range `start..end`.
fn line_flex(nodes: &[Node], start: usize, end: usize) -> (f32, f32) {
    nodes[start..end.min(nodes.len())]
        .iter()
        .fold((0.0, 0.0), |(stretch, shrink), node| match node {
            Node::Glue { stretch: s, shrink: h, .. } => (stretch + s, shrink + h),
            _ => (stretch, shrink),
        })
}

/// Adjustment ratio for the drawn range. Infeasible ratios (no
/// stretch/shrink available) are clamped to +-`RATIO_CLAMP` rather than
/// infinities so demerits stay finite and `ratio * 0.0` glue widths
/// never become NaN when rendering.
fn line_ratio(nodes: &[Node], start: usize, end: usize, line_width: f32) -> f32 {
    let natural = line_natural(nodes, start, end);
    let (stretch, shrink) = line_flex(nodes, start, end);
    let deficit = line_width - natural;
    let ratio = if deficit > 0.0 {
        if stretch > 0.0 {
            deficit / stretch
        } else {
            f32::INFINITY
        }
    } else if deficit < 0.0 {
        if shrink > 0.0 {
            deficit / shrink
        } else {
            f32::NEG_INFINITY
        }
    } else {
        0.0
    };
    if ratio.is_infinite() {
        ratio.signum() * RATIO_CLAMP
    } else {
        ratio
    }
}

/// Adjustment ratio for the paragraph's final line: positive deficits
/// are absorbed by `\parfillskip` (ratio 0); negative deficits must
/// come out of interword shrink (TeX gives parfillskip no shrink).
fn final_ratio(nodes: &[Node], start: usize, end: usize, line_width: f32) -> f32 {
    if line_natural(nodes, start, end) <= line_width {
        0.0
    } else {
        line_ratio(nodes, start, end, line_width)
    }
}

/// The ratio to record for a chosen line. Lines ending at a forced
/// break that fit naturally are visually perfect (the author asked for
/// the break), so their ratio is 0 instead of a clamped underfull
/// ratio that would poison the demerit total.
fn chosen_ratio(
    nodes: &[Node],
    start: usize,
    brk: usize,
    line_width: f32,
    is_final: bool,
) -> f32 {
    if is_final {
        return final_ratio(nodes, start, brk, line_width);
    }
    if brk < nodes.len()
        && is_forced(&nodes[brk])
        && line_natural(nodes, start, brk) <= line_width
    {
        return 0.0;
    }
    line_ratio(nodes, start, brk, line_width)
}

/// True when a forced penalty lies strictly inside the drawn range
/// `start..brk` (such a line is invalid: the break must occur there).
fn drawn_range_has_forced(nodes: &[Node], start: usize, brk: usize) -> bool {
    nodes[start..brk.min(nodes.len())].iter().any(is_forced)
}

/// TeX-style line demerits: `(line_penalty + ratio * 10)^2`.
fn line_demerits(ratio: f32) -> f32 {
    const LINE_PENALTY: f32 = 10.0;
    let badness = LINE_PENALTY + ratio * 10.0;
    badness * badness
}

/// Fitness class: 0 tight, 1 decent, 2 loose, 3 very loose (TeX order).
fn fitness_class(ratio: f32) -> u8 {
    if ratio < -0.5 {
        0
    } else if ratio <= 0.5 {
        1
    } else if ratio <= 1.0 {
        2
    } else {
        3
    }
}

fn fitness_incompatible(a: u8, b: u8) -> bool {
    (a == 0 && b == 3) || (a == 3 && b == 0)
}

/// Width of a line when rendered with `line.ratio` applied to glue.
pub fn rendered_width(nodes: &[Node], line: &Line) -> f32 {
    let mut w: f32 = nodes[line.start..line.end.min(nodes.len())]
        .iter()
        .map(|node| match node {
            Node::Box { width } => *width,
            Node::Glue {
                width,
                stretch,
                shrink,
            } => {
                let adjust = if line.ratio > 0.0 {
                    line.ratio * stretch
                } else {
                    line.ratio * shrink
                };
                width + adjust
            }
            Node::Penalty { .. } | Node::Hyphen { .. } => 0.0,
        })
        .sum();
    if let Some(Node::Hyphen { pre_width, .. }) = nodes.get(line.end) {
        w += pre_width;
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bx(w: f32) -> Node {
        Node::Box { width: w }
    }

    fn glue(w: f32, stretch: f32, shrink: f32) -> Node {
        Node::Glue {
            width: w,
            stretch,
            shrink,
        }
    }

    fn interword(w: f32) -> Node {
        glue(w, w * 0.5, w / 3.0)
    }

    fn words(ws: &[f32], space: f32) -> Vec<Node> {
        let mut nodes = Vec::new();
        for (i, w) in ws.iter().enumerate() {
            if i > 0 {
                nodes.push(interword(space));
            }
            nodes.push(bx(*w));
        }
        nodes
    }

    #[test]
    fn empty_input_produces_no_lines() {
        assert!(break_lines(&[], 100.0, 2.0).is_empty());
    }

    #[test]
    fn single_short_word_is_one_line() {
        let nodes = vec![bx(30.0)];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 1);
        assert_eq!(lines[0].ratio, 0.0, "final line ratio must be 0");
    }

    #[test]
    fn single_line_paragraph_is_never_stretched() {
        // 70 units in a 100-unit line: parfillskip absorbs the rest.
        let nodes = words(&[30.0, 30.0], 10.0);
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].ratio, 0.0);
    }

    #[test]
    fn breaks_at_glue_when_line_overflows() {
        // 40 + 10 + 40 = 90 fits; the third word would overflow 100.
        let nodes = words(&[40.0, 40.0, 40.0], 10.0);
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 3, "break discards the second glue (idx 3)");
        assert_eq!(lines[1].start, 4, "next line starts after the glue");
        assert_eq!(lines[1].end, 5);
        assert_eq!(lines[1].ratio, 0.0);
    }

    #[test]
    fn stretched_glue_reaches_line_width() {
        // Line 1: natural 30+10+30 = 70; deficit 30 over stretch 15
        // gives ratio 2 (within tolerance 2.5).
        let nodes = vec![
            bx(30.0),
            glue(10.0, 15.0, 0.0),
            bx(30.0),
            glue(10.0, 15.0, 0.0),
            bx(30.0),
            Node::Penalty {
                cost: FORCED_BREAK,
            },
        ];
        let lines = break_lines(&nodes, 100.0, 2.5);
        assert_eq!(lines.len(), 2);
        assert!((lines[0].ratio - 2.0).abs() < 1e-4);
        let w = rendered_width(&nodes, &lines[0]);
        assert!((w - 100.0).abs() < 1e-3, "justified width {w} != 100");
    }

    #[test]
    fn shrunk_glue_absorbs_small_overflow() {
        let nodes = vec![bx(50.0), glue(10.0, 0.0, 6.0), bx(50.0)];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1, "natural 110 shrinks to 100 via shrink 6");
        assert!((lines[0].ratio + 10.0 / 6.0).abs() < 1e-4);
        let w = rendered_width(&nodes, &lines[0]);
        assert!((w - 100.0).abs() < 1e-3, "shrunk width {w} != 100");
    }

    #[test]
    fn forced_penalty_splits_lines() {
        let nodes = vec![bx(20.0), Node::Penalty { cost: FORCED_BREAK }, bx(20.0)];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 1, "line ends before the forced penalty");
        assert_eq!(lines[1].start, 2, "penalty consumed; next line at 2");
        assert_eq!(lines[1].end, 3);
    }

    #[test]
    fn forbidden_penalty_is_not_a_breakpoint() {
        let nodes = vec![
            bx(60.0),
            Node::Penalty {
                cost: FORBIDDEN_BREAK,
            },
            bx(60.0),
        ];
        let lines = break_lines(&nodes, 100.0, 2.0);
        // No legal glue or penalty exists: TeX would report an overfull
        // hbox and keep the content on one overflowing line.
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, nodes.len());
    }

    #[test]
    fn single_overwide_word_is_allowed_to_overflow() {
        let nodes = vec![bx(150.0)];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].end, 1);
    }

    #[test]
    fn infeasible_paragraph_falls_back_to_greedy_and_covers_all() {
        // Zero-stretch glue and overflowing lines: tolerance can never
        // be met, so the greedy fallback must take over.
        let nodes = vec![
            bx(60.0),
            glue(5.0, 0.0, 0.0),
            bx(60.0),
            glue(5.0, 0.0, 0.0),
            bx(60.0),
        ];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert!(!lines.is_empty());
        assert_eq!(lines.last().unwrap().end, nodes.len());
        assert_eq!(lines.last().unwrap().ratio, 0.0);
        // First line holds one word (two would overflow: 60+5+60=125).
        assert_eq!(lines[0].end, 1);
    }

    #[test]
    fn long_paragraph_lines_are_contiguous_and_complete() {
        let ws: Vec<f32> = (1..=30).map(|i| 10.0 + (i % 5) as f32 * 7.0).collect();
        let nodes = words(&ws, 8.0);
        let lines = break_lines(&nodes, 120.0, 1.5);
        assert!(lines.len() >= 2);
        let mut prev_end = 0usize;
        for (k, line) in lines.iter().enumerate() {
            let expected_start = if k == 0 { 0 } else { prev_end + 1 };
            assert_eq!(line.start, expected_start, "lines must chain via +1");
            assert!(line.end > line.start, "no empty lines");
            if k + 1 < lines.len() {
                assert!(line.ratio.is_finite());
            } else {
                assert_eq!(line.ratio, 0.0);
                assert_eq!(line.end, nodes.len());
            }
            prev_end = line.end;
        }
    }

    #[test]
    fn total_fit_keeps_nonfinal_lines_within_tolerance() {
        let nodes = words(&[45.0, 45.0, 20.0, 20.0], 10.0);
        let lines = break_lines(&nodes, 100.0, 1.0);
        assert!(lines.len() >= 2);
        for line in &lines[..lines.len() - 1] {
            assert!(
                (-1.0..=1.0).contains(&line.ratio),
                "ratio {} outside tolerance",
                line.ratio
            );
        }
    }

    #[test]
    fn leading_glue_is_not_a_breakpoint() {
        let nodes = vec![glue(10.0, 5.0, 5.0), bx(50.0), glue(10.0, 5.0, 5.0), bx(50.0)];
        let lines = break_lines(&nodes, 60.0, 2.0);
        assert_eq!(lines[0].start, 0);
    }

    #[test]
    fn penalties_between_boxes_do_not_count_as_width() {
        let nodes = vec![bx(30.0), Node::Penalty { cost: -50.0 }, bx(30.0)];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1, "60 units fits in 100 despite penalty");
        assert_eq!(lines[0].end, 3);
    }

    #[test]
    fn forced_break_inside_overflowing_word_chain_is_honored() {
        // Greedy fits two words per line; the forced penalty at the end
        // of the third word must still break exactly there.
        let nodes = vec![
            bx(45.0),
            interword(10.0),
            bx(45.0),
            interword(10.0),
            bx(45.0),
            Node::Penalty { cost: FORCED_BREAK },
            bx(20.0),
        ];
        let lines = break_lines(&nodes, 100.0, 1.0);
        let last = lines.last().unwrap();
        assert_eq!(last.end, nodes.len());
        // The line ending at the forced penalty must end at index 5.
        assert!(
            lines.iter().any(|l| l.end == 5),
            "forced break must land on the penalty, got {lines:?}"
        );
    }

    #[test]
    fn hyphen_break_appends_hyphen_width_to_line() {
        // word parts: "algo"(40) + Hyphen(10) + "rithm"(40); space-less.
        let nodes = vec![
            bx(40.0),
            Node::Hyphen {
                cost: 50.0,
                pre_width: 10.0,
            },
            bx(40.0),
        ];
        // Without a hyphen break the word is 80 wide and fits.
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1, "80-unit word fits in 100: no break");
        assert_eq!(lines[0].end, 3);
        assert_eq!(rendered_width(&nodes, &lines[0]), 80.0);

        // In a 70-unit line the unbroken word overflows; the hyphen
        // break lets line 1 end with the hyphen (40 + 10 = 50 <= 70).
        let lines = break_lines(&nodes, 70.0, 2.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 1, "line 1's break index is the hyphen node");
        assert_eq!(lines[1].start, 2, "hyphen consumed; part 2 starts line 2");
        assert_eq!(rendered_width(&nodes, &lines[0]), 50.0, "part + hyphen");
        assert_eq!(rendered_width(&nodes, &lines[1]), 40.0);
    }

    #[test]
    fn hyphen_not_taken_costs_nothing() {
        let nodes = vec![
            bx(30.0),
            Node::Hyphen {
                cost: 50.0,
                pre_width: 10.0,
            },
            bx(30.0),
            glue(10.0, 5.0, 3.0),
            bx(30.0),
        ];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert_eq!(lines.len(), 1, "everything fits: no hyphen drawn");
        assert_eq!(rendered_width(&nodes, &lines[0]), 100.0, "30+30+10+30");
    }

    #[test]
    fn hyphen_break_is_preferred_over_loose_glue() {
        // A tight fit where hyphenating the first word avoids a very
        // loose line: KP should pick the hyphen break despite its cost.
        let nodes = vec![
            bx(60.0),
            Node::Hyphen {
                cost: 50.0,
                pre_width: 6.0,
            },
            bx(60.0),
            glue(10.0, 4.0, 2.0),
            bx(60.0),
        ];
        let lines = break_lines(&nodes, 100.0, 2.0);
        assert!(lines.len() >= 2);
        // First line must end at the hyphen (part 1 fits with hyphen;
        // without it, 60+10+60 = 130 overflows).
        assert_eq!(lines[0].end, 1);
        assert_eq!(rendered_width(&nodes, &lines[0]), 66.0);
    }
}

#[cfg(test)]
mod first_line_tests {
    use super::*;

    fn bx(w: f32) -> Node {
        Node::Box { width: w }
    }

    fn interword(w: f32) -> Node {
        Node::Glue {
            width: w,
            stretch: w * 0.5,
            shrink: w / 3.0,
        }
    }

    #[test]
    fn shorter_first_line_breaks_earlier() {
        // Words of 40 units + 10 glue: a 70-unit first line holds ONE
        // word (90 > 70), while a 130-unit body line holds two.
        let nodes = vec![
            bx(40.0),
            interword(10.0),
            bx(40.0),
            interword(10.0),
            bx(40.0),
        ];
        let lines = break_lines_with_first(&nodes, 130.0, 70.0, 2.0);
        assert!(lines.len() >= 2);
        assert_eq!(lines[0].end, 1, "first line holds one word only");
        assert_eq!(lines[1].start, 2);
        assert_eq!(lines[1].end, nodes.len(), "body line holds the rest");
    }

    #[test]
    fn equal_widths_match_break_lines() {
        let words = [45.0f32, 45.0, 20.0, 20.0];
        let mut nodes = Vec::new();
        for (i, w) in words.iter().enumerate() {
            if i > 0 {
                nodes.push(interword(10.0));
            }
            nodes.push(bx(*w));
        }
        let a = break_lines(&nodes, 100.0, 1.0);
        let b = break_lines_with_first(&nodes, 100.0, 100.0, 1.0);
        assert_eq!(a, b);
    }
}
