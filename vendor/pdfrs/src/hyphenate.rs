//! Liang hyphenation with the embedded American English patterns.
//!
//! Implements Frank Liang's pattern algorithm as used by TeX: patterns
//! like `.ach4` assign odd/even inter-letter values; a break between
//! letters is allowed where the merged value is odd. `\lefthyphenmin`
//! = 2 and `\righthyphenmin` = 3 restrict how close a break may come
//! to the word's ends. The `\hyphenation` exception list overrides the
//! patterns wholesale for listed words.
//!
//! Only ASCII-alphabetic words are hyphenated; other scripts and words
//! containing digits or punctuation are returned unbreakable.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The pattern file: `[patterns]` section (Liang patterns) then
/// `[exceptions]` section (hyphenated words like `as-so-ciate`).
static PATTERNS_TXT: &str = include_str!("hyph_en_us.txt");

/// TeX's `\lefthyphenmin` for English: at least 2 letters before a break.
const LEFT_MIN: usize = 2;
/// TeX's `\righthyphenmin` for English: at least 3 letters after a break.
const RIGHT_MIN: usize = 3;
/// TeX's `\hyphenpenalty`: demerit cost of breaking at a hyphen.
pub const HYPHEN_PENALTY: f32 = 50.0;

/// Parsed pattern table: letters (including `.` boundary marks) ->
/// digit levels, `digits[k]` being the value between `letters[k-1]`
/// and `letters[k]` (`0` = unset, odd = break allowed).
struct Tables {
    patterns: HashMap<String, Vec<u8>>,
    /// Exception word (no hyphens, lowercase) -> break-before char indices.
    exceptions: HashMap<String, Vec<usize>>,
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut section = 0;
        let mut patterns = HashMap::new();
        let mut exceptions = HashMap::new();
        for line in PATTERNS_TXT.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('%') {
                continue;
            }
            if line == "[patterns]" {
                section = 1;
                continue;
            }
            if line == "[exceptions]" {
                section = 2;
                continue;
            }
            match section {
                1 => {
                    let (letters, digits) = parse_pattern(line);
                    patterns.insert(letters, digits);
                }
                2 => {
                    let word = line.replace('-', "");
                    let breaks: Vec<usize> = line
                        .char_indices()
                        .filter(|&(_, c)| c == '-')
                        .map(|(i, _)| {
                            // char index in the de-hyphenated word
                            line[..i].chars().filter(|&c| c != '-').count()
                        })
                        .collect();
                    exceptions.insert(word, breaks);
                }
                _ => {}
            }
        }
        Tables {
            patterns,
            exceptions,
        }
    })
}

/// Split a Liang pattern into letters (the map key) and inter-letter
/// digit levels (`letters.len() + 1` slots, `gaps[k]` sitting between
/// `letters[k-1]` and `letters[k]`, i.e. after `k` letters). Example:
/// `.ach4` -> letters `.ach`, gaps `[0, 0, 0, 0, 4]`; `4ab.` ->
/// letters `ab.`, gaps `[4, 0, 0, 0]`; `1ca` -> letters `ca`, gaps
/// `[1, 0, 0]`.
fn parse_pattern(pattern: &str) -> (String, Vec<u8>) {
    let mut letters = String::with_capacity(pattern.len());
    // gaps[k] is the value after `k` letters (0 = unset).
    let mut gaps: Vec<u8> = vec![0];
    for ch in pattern.chars() {
        match ch {
            '0'..='9' => {
                gaps[letters.len()] = ch as u8 - b'0';
            }
            _ => {
                letters.push(ch);
                gaps.push(0);
            }
        }
    }
    (letters, gaps)
}

/// Hyphenation points for `word`: char indices where a line break may
/// replace the implicit letter join with a hyphen (`break before
/// word[i]`). Empty for words TeX would not hyphenate (too short,
/// non-alphabetic, or exception-listed as unbreakable).
pub fn hyphenation_points(word: &str) -> Vec<usize> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < LEFT_MIN + RIGHT_MIN {
        return Vec::new();
    }
    if !chars.iter().all(|c| c.is_ascii_alphabetic()) {
        return Vec::new();
    }

    let tables = tables();
    let lower: String = word.to_ascii_lowercase();
    if let Some(breaks) = tables.exceptions.get(&lower) {
        return breaks
            .iter()
            .copied()
            .filter(|&i| i >= LEFT_MIN && chars.len() - i >= RIGHT_MIN)
            .collect();
    }

    // Liang: wrap the lowercase word in boundary dots and merge the
    // digit levels of every matching pattern substring.
    let w: Vec<char> = std::iter::once('.')
        .chain(lower.chars())
        .chain(std::iter::once('.'))
        .collect();
    let mut levels = vec![0u8; w.len() + 1];
    for start in 0..w.len() {
        let mut key = String::new();
        for end in (start + 1)..=w.len() {
            key.push(w[end - 1]);
            if let Some(digits) = tables.patterns.get(&key) {
                for (offset, &value) in digits.iter().enumerate() {
                    if value > levels[start + offset] {
                        levels[start + offset] = value;
                    }
                }
            }
        }
    }

    // Odd levels between word letters are candidate breaks.
    let mut points = Vec::new();
    for (c, &level) in levels.iter().enumerate().skip(LEFT_MIN + 1) {
        // Position c in `w` sits between w[c-1] and w[c]; w[c] is
        // word[c-1] (offset by the leading dot).
        let word_index = c - 1;
        if word_index >= LEFT_MIN
            && word_index < chars.len()
            && chars.len() - word_index >= RIGHT_MIN
            && level % 2 == 1
        {
            points.push(word_index);
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pattern_digits() {
        let (letters, gaps) = parse_pattern(".ach4");
        assert_eq!(letters, ".ach");
        assert_eq!(gaps, vec![0, 0, 0, 0, 4]);

        let (letters, gaps) = parse_pattern("4ab.");
        assert_eq!(letters, "ab.");
        assert_eq!(gaps, vec![4, 0, 0, 0]);

        let (letters, gaps) = parse_pattern("1ca");
        assert_eq!(letters, "ca");
        assert_eq!(gaps, vec![1, 0, 0]);

        let (letters, gaps) = parse_pattern("o2n");
        assert_eq!(letters, "on");
        assert_eq!(gaps, vec![0, 2, 0]);
    }

    #[test]
    fn classic_words_hyphenate_like_tex() {
        // Reference hyphenations from TeX with ushyphmax patterns.
        // hy-phen-ation, al-go-rithm, doc-u-men-ta-tion, jus-ti-fi-ca-tion.
        assert_eq!(hyphenation_points("hyphenation"), vec![2, 6]);
        assert_eq!(hyphenation_points("algorithm"), vec![2, 4]);
        assert_eq!(hyphenation_points("documentation"), vec![3, 4, 7, 9]);
        assert_eq!(hyphenation_points("justification"), vec![3, 5, 7, 9]);
        assert!(
            hyphenation_points("supercalifragilisticexpialidocious").len() > 3,
            "long nonsense word should hyphenate"
        );
    }

    #[test]
    fn exceptions_override_patterns() {
        // `ta-ble` and `reci-procity` are explicit exceptions; `present`
        // and `project` are exception-listed as NOT hyphenated.
        assert_eq!(hyphenation_points("table"), vec![2]);
        assert_eq!(hyphenation_points("reciprocity"), vec![4]);
        assert!(hyphenation_points("present").is_empty());
        assert!(hyphenation_points("project").is_empty());
    }

    #[test]
    fn short_words_are_never_hyphenated() {
        assert!(hyphenation_points("the").is_empty());
        assert!(hyphenation_points("table").len() == 1); // exception wins
        assert!(hyphenation_points("go").is_empty());
    }

    #[test]
    fn non_alphabetic_words_are_never_hyphenated() {
        assert!(hyphenation_points("x86").is_empty());
        assert!(hyphenation_points("don't").is_empty());
        assert!(hyphenation_points("über").is_empty());
        assert!(hyphenation_points("hy-phen").is_empty());
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(hyphenation_points("Hyphenation"), vec![2, 6]);
        assert_eq!(hyphenation_points("HYPHENATION"), vec![2, 6]);
    }

    #[test]
    fn points_respect_minima() {
        // Every point must leave >= 2 letters before and >= 3 after.
        for word in ["justification", "documentation", "reciprocity"] {
            for &p in &hyphenation_points(word) {
                assert!(p >= LEFT_MIN, "break too early in {word} at {p}");
                assert!(
                    word.chars().count() - p >= RIGHT_MIN,
                    "break too late in {word} at {p}"
                );
            }
        }
    }
}
