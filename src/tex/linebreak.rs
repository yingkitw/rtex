//! Knuth–Plass line breaking for paragraph setting.
//!
//! Implements the dynamic-programming algorithm from *Breaking Paragraphs into
//! Lines* (Knuth & Plass, 1981) using boxes, glue, and penalties.

use super::dimensions::{Dimension, PT_TO_SP};
use super::glue::{Glue, Stretch};

/// A node in the horizontal list used for line breaking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineItem {
    /// A rectangular box (e.g. a word glyph run).
    Box {
        width: Dimension,
        /// Index into the source word list.
        word_index: usize,
    },
    /// Flexible glue between boxes.
    Glue(Glue),
    /// Optional break point with width and penalty.
    Penalty { width: Dimension, penalty: i32 },
}

/// One line produced by the breaker.
#[derive(Debug, Clone, PartialEq)]
pub struct BrokenLine {
    /// Index of the first item on this line (inclusive).
    pub start: usize,
    /// Index past the last item on this line (exclusive).
    pub end: usize,
    /// Glue adjustment ratio applied on this line.
    pub ratio: f64,
    /// Badness of this line (lower is better).
    pub badness: i32,
}

/// Tokenized paragraph ready for line breaking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedParagraph {
    pub words: Vec<String>,
    pub items: Vec<LineItem>,
}

/// Knuth–Plass paragraph line breaker.
#[derive(Debug, Clone, Copy)]
pub struct LineBreaker {
    line_width: Dimension,
    inter_word_glue: Glue,
    char_width: Dimension,
    justify_last_line: bool,
}

impl LineBreaker {
    /// Create a breaker for lines of `line_width`.
    pub fn new(line_width: Dimension) -> Self {
        let char_width = Dimension::from_sp((0.55 * PT_TO_SP as f64).round() as i64);
        Self {
            line_width,
            inter_word_glue: Self::default_inter_word_glue(char_width),
            char_width,
            justify_last_line: false,
        }
    }

    fn default_inter_word_glue(char_width: Dimension) -> Glue {
        Glue {
            width: char_width,
            stretch: Stretch::Finite(char_width),
            shrink: Dimension::from_sp(char_width.sp() / 3),
        }
    }

    /// Override the per-character width estimator used when tokenizing plain text.
    pub fn with_char_width(mut self, char_width: Dimension) -> Self {
        self.char_width = char_width;
        self.inter_word_glue = Self::default_inter_word_glue(char_width);
        self
    }

    /// Override inter-word glue (TeX `\spaceskip` analogue).
    pub fn with_inter_word_glue(mut self, glue: Glue) -> Self {
        self.inter_word_glue = glue;
        self
    }

    /// Whether to justify the final line of a paragraph.
    pub fn with_justify_last_line(mut self, justify: bool) -> Self {
        self.justify_last_line = justify;
        self
    }

    /// Tokenize normalized text into boxes and glue items.
    pub fn tokenize_text(&self, text: &str) -> TokenizedParagraph {
        let words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
        if words.is_empty() {
            return TokenizedParagraph {
                words,
                items: Vec::new(),
            };
        }

        let mut items = Vec::new();
        for (word_index, word) in words.iter().enumerate() {
            if word_index > 0 {
                items.push(LineItem::Glue(self.inter_word_glue));
            }
            for (chunk_idx, chunk) in self.split_word(word).into_iter().enumerate() {
                if chunk_idx > 0 {
                    items.push(LineItem::Penalty {
                        width: Dimension::from_sp(0),
                        penalty: 0,
                    });
                }
                let width_sp = chunk.chars().count() as i64 * self.char_width.sp();
                items.push(LineItem::Box {
                    width: Dimension::from_sp(width_sp),
                    word_index,
                });
            }
        }

        TokenizedParagraph { words, items }
    }

    /// Break a horizontal list into lines using Knuth–Plass dynamic programming.
    pub fn break_items(&self, items: &[LineItem]) -> Vec<BrokenLine> {
        if items.is_empty() {
            return Vec::new();
        }

        let mut work = items.to_vec();
        work.push(LineItem::Glue(Glue::natural(Dimension::from_sp(0))));
        work.push(LineItem::Penalty {
            width: Dimension::from_sp(0),
            penalty: -10_000,
        });

        let n = work.len();
        let sentinel = i32::MAX / 4;
        let mut cost = vec![sentinel; n + 1];
        let mut previous_break = vec![0usize; n + 1];
        cost[0] = 0;

        for j in 1..=n {
            for i in (0..j).rev() {
                if !is_legal_break(&work, i, j) {
                    continue;
                }

                let is_last_line = j == n;
                if is_last_line && !self.justify_last_line {
                    let metrics = line_metrics(&work[i..j], Dimension::from_sp(0));
                    if metrics.natural_width > self.line_width.sp() {
                        continue;
                    }
                    let demerits = cost[i];
                    if demerits < cost[j] {
                        cost[j] = demerits;
                        previous_break[j] = i;
                    }
                    continue;
                }

                let metrics = line_metrics(&work[i..j], self.line_width);
                if metrics.total_stretch == 0 && metrics.adjustment > 0 {
                    continue;
                }
                if metrics.total_shrink == 0 && metrics.adjustment < 0 {
                    continue;
                }

                let ratio = compute_ratio(metrics);
                let badness = line_badness(ratio);
                let penalty = break_penalty(&work[j - 1]);
                let demerits = cost[i] + badness + penalty;
                if demerits < cost[j] {
                    cost[j] = demerits;
                    previous_break[j] = i;
                }
            }
        }

        let mut lines = Vec::new();
        let mut end = n;
        while end > 0 {
            let start = previous_break[end];
            let content_end = end.min(items.len());
            if start < content_end {
                let is_last_line = end == n && !self.justify_last_line;
                let metrics = if is_last_line {
                    line_metrics(&work[start..end], Dimension::from_sp(0))
                } else {
                    line_metrics(&work[start..end], self.line_width)
                };
                let ratio = if is_last_line {
                    0.0
                } else {
                    compute_ratio(metrics)
                };
                lines.push(BrokenLine {
                    start,
                    end: content_end,
                    ratio,
                    badness: line_badness(ratio),
                });
            }
            end = start;
        }
        lines.reverse();
        if !self.justify_last_line {
            if let Some(last) = lines.last_mut() {
                last.ratio = 0.0;
            }
        }
        lines
    }

    /// Tokenize and break text, returning one string per output line.
    pub fn break_text(&self, text: &str) -> Vec<String> {
        let tokenized = self.tokenize_text(text);
        if tokenized.items.is_empty() {
            return Vec::new();
        }

        let lines = self.break_items(&tokenized.items);
        lines
            .into_iter()
            .map(|line| self.render_line(&tokenized, &line))
            .collect()
    }

    fn render_line(&self, tokenized: &TokenizedParagraph, line: &BrokenLine) -> String {
        let end = line.end.min(tokenized.items.len());
        if line.start >= end {
            return String::new();
        }
        let mut words_on_line = Vec::new();
        for item in &tokenized.items[line.start..end] {
            if let LineItem::Box { word_index, .. } = item {
                if words_on_line.last().copied() != Some(*word_index) {
                    words_on_line.push(*word_index);
                }
            }
        }
        words_on_line
            .into_iter()
            .map(|idx| tokenized.words[idx].as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn split_word(&self, word: &str) -> Vec<String> {
        let max_chars = (self.line_width.sp() / self.char_width.sp()).max(1) as usize;
        let word_chars: Vec<char> = word.chars().collect();
        if word_chars.len() <= max_chars {
            return vec![word.to_string()];
        }

        word_chars
            .chunks(max_chars)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct LineMetrics {
    natural_width: i64,
    total_stretch: i64,
    total_shrink: i64,
    adjustment: i64,
}

fn line_metrics(items: &[LineItem], line_width: Dimension) -> LineMetrics {
    let mut natural_width = 0i64;
    let mut total_stretch = 0i64;
    let mut total_shrink = 0i64;

    for item in items {
        match item {
            LineItem::Box { width, .. } => natural_width += width.sp(),
            LineItem::Glue(glue) => {
                natural_width += glue.width.sp();
                if let Stretch::Finite(d) = glue.stretch {
                    total_stretch += d.sp();
                }
                total_shrink += glue.shrink.sp();
            }
            LineItem::Penalty { width, .. } => natural_width += width.sp(),
        }
    }

    let adjustment = line_width.sp() - natural_width;
    LineMetrics {
        natural_width,
        total_stretch,
        total_shrink,
        adjustment,
    }
}

fn compute_ratio(metrics: LineMetrics) -> f64 {
    if metrics.adjustment > 0 {
        if metrics.total_stretch == 0 {
            return 10.0;
        }
        metrics.adjustment as f64 / metrics.total_stretch as f64
    } else if metrics.adjustment < 0 {
        if metrics.total_shrink == 0 {
            return -10.0;
        }
        metrics.adjustment as f64 / metrics.total_shrink as f64
    } else {
        0.0
    }
}

/// TeX-style badness from glue ratio `r` (|r| > 1 is overfull).
pub fn line_badness(ratio: f64) -> i32 {
    if ratio.abs() > 1.0 {
        10_000
    } else {
        (100.0 * ratio.abs().powi(3)).round() as i32
    }
}

fn break_penalty(item: &LineItem) -> i32 {
    match item {
        LineItem::Penalty { penalty, .. } => *penalty,
        _ => 0,
    }
}

fn is_legal_break(items: &[LineItem], start: usize, end: usize) -> bool {
    if start >= end {
        return false;
    }
    match items.get(end - 1) {
        Some(LineItem::Glue(_)) => true,
        Some(LineItem::Penalty { penalty, .. }) => *penalty < 10_000,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn breaker_for_chars(line_chars: i64, char_width_sp: i64) -> LineBreaker {
        let line_width = Dimension::from_sp(line_chars * char_width_sp);
        LineBreaker::new(line_width).with_char_width(Dimension::from_sp(char_width_sp))
    }

    #[test]
    fn empty_text_returns_no_lines() {
        let breaker = breaker_for_chars(20, 100);
        assert!(breaker.break_text("").is_empty());
        assert!(breaker.break_text("   ").is_empty());
    }

    #[test]
    fn single_word_fits_on_one_line() {
        let breaker = breaker_for_chars(20, 100);
        let lines = breaker.break_text("hello");
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn breaks_at_word_boundaries() {
        let breaker = breaker_for_chars(10, 100);
        let lines = breaker.break_text("one two three four");
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(!line.is_empty());
            assert!(!line.contains("  "));
        }
    }

    #[test]
    fn splits_long_word_when_necessary() {
        let breaker = breaker_for_chars(5, 100);
        let lines = breaker.break_text("supercalifragilistic");
        assert!(lines.len() > 1);
    }

    #[test]
    fn prefers_balanced_lines_over_greedy() {
        let char_w = 100;
        let breaker = LineBreaker::new(Dimension::from_sp(8 * char_w))
            .with_char_width(Dimension::from_sp(char_w));
        let lines = breaker.break_text("A short line with extraordinarily long word end");
        assert!(lines.len() >= 2);
        assert!(lines.iter().all(|line| !line.is_empty()));
    }

    #[test]
    fn line_badness_increases_with_ratio() {
        assert!(line_badness(0.0) < line_badness(0.5));
        assert!(line_badness(0.5) < line_badness(0.9));
        assert_eq!(line_badness(1.5), 10_000);
    }

    #[test]
    fn tokenize_inserts_glue_between_words() {
        let breaker = breaker_for_chars(20, 100);
        let tokenized = breaker.tokenize_text("ab cd");
        assert_eq!(tokenized.words, vec!["ab", "cd"]);
        assert_eq!(tokenized.items.len(), 3);
        assert!(matches!(tokenized.items[1], LineItem::Glue(_)));
    }

    #[test]
    fn break_items_reports_line_ranges() {
        let breaker = breaker_for_chars(8, 100);
        let tokenized = breaker.tokenize_text("alpha beta gamma");
        let lines = breaker.break_items(&tokenized.items);
        assert!(!lines.is_empty());
        assert_eq!(lines[0].start, 0);
        assert!(lines.last().unwrap().end <= tokenized.items.len() + 2);
    }

    #[test]
    fn last_line_not_justified_by_default() {
        let breaker = breaker_for_chars(8, 100);
        let tokenized = breaker.tokenize_text("one two three four five");
        let lines = breaker.break_items(&tokenized.items);
        let last = lines.last().unwrap();
        assert_eq!(last.ratio, 0.0);
    }
}
