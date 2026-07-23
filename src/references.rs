//! Cross-reference engine for labels, refs, and page refs.
//!
//! Collects `\label{key}` placements during a pre-scan of the document,
//! assigning sequential numbers to sections, equations, figures, and
//! tables.  `\ref{key}` and `\pageref{key}` are then resolved against the
//! stored map.

use crate::parser::TexElement;
use std::collections::HashMap;

/// Stores label→(number, page) mappings.
#[derive(Debug, Default)]
pub struct RefStore {
    labels: HashMap<String, (String, usize)>,
}

impl RefStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan `elements` and record every `\label` with the number of the
    /// preceding numbered element (section, equation, figure, or table).
    /// Page numbers are initialised to 1 and can be updated during rendering.
    pub fn scan(&mut self, elements: &[TexElement]) {
        let mut section_counters = [0usize; 3];
        let mut equation_counter = 0;
        let mut figure_counter = 0;
        let mut table_counter = 0;
        let mut last_number = String::new();

        for elem in elements {
            match elem {
                TexElement::Section { level, .. } => {
                    let idx = level.saturating_sub(1).min(2);
                    section_counters[idx] += 1;
                    for counter in section_counters.iter_mut().skip(idx + 1) {
                        *counter = 0;
                    }
                    last_number = format_section_number(&section_counters[..=idx]);
                }
                TexElement::MathDisplay(_) => {
                    equation_counter += 1;
                    last_number = format!("({})", equation_counter);
                }
                TexElement::Image { .. } => {
                    figure_counter += 1;
                    last_number = format!("{}", figure_counter);
                }
                TexElement::Table(_) => {
                    table_counter += 1;
                    last_number = format!("{}", table_counter);
                }
                TexElement::Label { key } => {
                    self.labels.insert(key.clone(), (last_number.clone(), 1));
                }
                _ => {}
            }
        }
    }

    /// Update the page number for a label (called during rendering).
    pub fn set_page(&mut self, key: &str, page: usize) {
        if let Some(entry) = self.labels.get_mut(key) {
            entry.1 = page;
        }
    }

    /// Resolve `\ref{key}` to its assigned number string.
    pub fn resolve_ref(&self, key: &str) -> Option<&str> {
        self.labels.get(key).map(|(num, _)| num.as_str())
    }

    /// Resolve `\pageref{key}` to its page number.
    pub fn resolve_pageref(&self, key: &str) -> Option<usize> {
        self.labels.get(key).map(|(_, page)| *page)
    }
}

fn format_section_number(counters: &[usize]) -> String {
    counters
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::TexElement;

    #[test]
    fn test_section_labels() {
        let elements = vec![
            TexElement::Section {
                level: 1,
                title: "Intro".to_string(),
            },
            TexElement::Label {
                key: "sec:intro".to_string(),
            },
            TexElement::Section {
                level: 2,
                title: "Details".to_string(),
            },
            TexElement::Label {
                key: "sec:details".to_string(),
            },
            TexElement::Section {
                level: 1,
                title: "Next".to_string(),
            },
            TexElement::Label {
                key: "sec:next".to_string(),
            },
        ];
        let mut store = RefStore::new();
        store.scan(&elements);
        assert_eq!(store.resolve_ref("sec:intro"), Some("1"));
        assert_eq!(store.resolve_ref("sec:details"), Some("1.1"));
        assert_eq!(store.resolve_ref("sec:next"), Some("2"));
    }

    #[test]
    fn test_equation_label() {
        let elements = vec![
            TexElement::MathDisplay("x = y".to_string()),
            TexElement::Label {
                key: "eq:1".to_string(),
            },
        ];
        let mut store = RefStore::new();
        store.scan(&elements);
        assert_eq!(store.resolve_ref("eq:1"), Some("(1)"));
    }

    #[test]
    fn test_figure_label() {
        let elements = vec![
            TexElement::Image {
                path: "a.png".to_string(),
                width: None,
                height: None,
            },
            TexElement::Label {
                key: "fig:1".to_string(),
            },
        ];
        let mut store = RefStore::new();
        store.scan(&elements);
        assert_eq!(store.resolve_ref("fig:1"), Some("1"));
    }

    #[test]
    fn test_table_label() {
        let table = crate::table::Table {
            columns: vec![crate::table::Align::Left],
            rows: vec![],
        };
        let elements = vec![
            TexElement::Table(table),
            TexElement::Label {
                key: "tab:1".to_string(),
            },
        ];
        let mut store = RefStore::new();
        store.scan(&elements);
        assert_eq!(store.resolve_ref("tab:1"), Some("1"));
    }

    #[test]
    fn test_pageref() {
        let elements = vec![
            TexElement::Section {
                level: 1,
                title: "A".to_string(),
            },
            TexElement::Label {
                key: "sec:a".to_string(),
            },
        ];
        let mut store = RefStore::new();
        store.scan(&elements);
        assert_eq!(store.resolve_pageref("sec:a"), Some(1));
        store.set_page("sec:a", 3);
        assert_eq!(store.resolve_pageref("sec:a"), Some(3));
    }

    #[test]
    fn test_unknown_label() {
        let store = RefStore::new();
        assert_eq!(store.resolve_ref("missing"), None);
        assert_eq!(store.resolve_pageref("missing"), None);
    }
}
