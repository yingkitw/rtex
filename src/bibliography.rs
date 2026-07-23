//! BibTeX parsing and citation formatting.
//!
//! Provides a lightweight BibTeX parser that turns raw `.bib` file content
//! into structured entries, plus helpers for formatting citations in a
//! handful of common styles.

use std::collections::HashMap;
use std::path::Path;

/// Standard BibTeX entry types.
#[derive(Debug, Clone, PartialEq)]
pub enum BibEntryType {
    Article,
    Book,
    Booklet,
    Conference,
    Inbook,
    Incollection,
    Inproceedings,
    Manual,
    Mastersthesis,
    Misc,
    Phdthesis,
    Proceedings,
    Techreport,
    Unpublished,
    Custom(String),
}

impl From<&str> for BibEntryType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "article" => BibEntryType::Article,
            "book" => BibEntryType::Book,
            "booklet" => BibEntryType::Booklet,
            "conference" => BibEntryType::Conference,
            "inbook" => BibEntryType::Inbook,
            "incollection" => BibEntryType::Incollection,
            "inproceedings" => BibEntryType::Inproceedings,
            "manual" => BibEntryType::Manual,
            "mastersthesis" => BibEntryType::Mastersthesis,
            "misc" => BibEntryType::Misc,
            "phdthesis" => BibEntryType::Phdthesis,
            "proceedings" => BibEntryType::Proceedings,
            "techreport" => BibEntryType::Techreport,
            "unpublished" => BibEntryType::Unpublished,
            _ => BibEntryType::Custom(s.to_string()),
        }
    }
}

impl std::fmt::Display for BibEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BibEntryType::Article => "article",
            BibEntryType::Book => "book",
            BibEntryType::Booklet => "booklet",
            BibEntryType::Conference => "conference",
            BibEntryType::Inbook => "inbook",
            BibEntryType::Incollection => "incollection",
            BibEntryType::Inproceedings => "inproceedings",
            BibEntryType::Manual => "manual",
            BibEntryType::Mastersthesis => "mastersthesis",
            BibEntryType::Misc => "misc",
            BibEntryType::Phdthesis => "phdthesis",
            BibEntryType::Proceedings => "proceedings",
            BibEntryType::Techreport => "techreport",
            BibEntryType::Unpublished => "unpublished",
            BibEntryType::Custom(s) => s,
        };
        write!(f, "{s}")
    }
}

/// A single BibTeX entry (article, book, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct BibEntry {
    pub key: String,
    pub entry_type: String,
    pub fields: HashMap<String, String>,
}

impl BibEntry {
    /// Convenience accessor for a field value.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.fields.get(field).map(|s| s.as_str())
    }

    /// A short "plain" citation string: *Author(s). Title. Journal. Year.*
    pub fn format_plain(&self) -> String {
        let mut parts = Vec::new();
        if let Some(author) = self.get("author") {
            parts.push(author.to_string());
        }
        if let Some(title) = self.get("title") {
            parts.push(format!("*{}*.", title));
        }
        if let Some(journal) = self.get("journal") {
            parts.push(format!("{}.", journal));
        } else if let Some(booktitle) = self.get("booktitle") {
            parts.push(format!("In *{}*.", booktitle));
        }
        if let Some(year) = self.get("year") {
            parts.push(format!("{}.", year));
        }
        parts.join(" ")
    }

    /// Format for a numeric bibliography list.
    pub fn format_numeric(&self) -> String {
        let mut parts = Vec::new();
        if let Some(author) = self.get("author") {
            parts.push(author.to_string());
        }
        if let Some(title) = self.get("title") {
            parts.push(format!("\"{}\".", title));
        }
        if let Some(journal) = self.get("journal") {
            parts.push(format!("{}.", journal));
        } else if let Some(booktitle) = self.get("booktitle") {
            parts.push(format!("In {}.", booktitle));
        }
        if let Some(publisher) = self.get("publisher") {
            parts.push(format!("{}.", publisher));
        }
        if let Some(year) = self.get("year") {
            parts.push(format!("{}.", year));
        }
        parts.join(" ")
    }
}

/// Parse raw BibTeX file content into a list of entries.
pub fn parse_bibtex(text: &str) -> Vec<BibEntry> {
    let mut entries = Vec::new();
    let mut i = 0;
    let text = text.trim();

    while i < text.len() {
        // Skip whitespace and comments
        while i < text.len() {
            let ch = text[i..].chars().next().unwrap_or('\0');
            if ch.is_whitespace() {
                i += ch.len_utf8();
            } else if ch == '%' {
                // Skip comment line
                while i < text.len() && !text[i..].starts_with('\n') {
                    i += text[i..].chars().next().unwrap().len_utf8();
                }
            } else {
                break;
            }
        }

        if i >= text.len() {
            break;
        }

        if text[i..].starts_with('@') {
            if let Some(entry) = parse_entry(text, &mut i) {
                entries.push(entry);
            } else {
                // Failed to parse this entry, skip to next @
                i += 1;
                if let Some(next) = text[i..].find('@') {
                    i += next;
                } else {
                    break;
                }
            }
        } else {
            i += text[i..].chars().next().unwrap().len_utf8();
        }
    }

    entries
}

fn parse_entry(text: &str, pos: &mut usize) -> Option<BibEntry> {
    let start = *pos;
    let at = text[start..].find('@')?;
    *pos = start + at + 1;

    // Entry type
    let type_end = text[*pos..].find('{')?;
    let entry_type = text[*pos..*pos + type_end].trim().to_lowercase();
    *pos += type_end + 1;

    // Key (before first comma)
    let comma = text[*pos..].find(',')?;
    let key = text[*pos..*pos + comma].trim().to_string();
    *pos += comma + 1;

    // Fields until matching }
    let mut fields = HashMap::new();
    let mut brace_depth = 1;
    let field_start = *pos;

    while *pos < text.len() && brace_depth > 0 {
        let ch = text[*pos..].chars().next().unwrap();
        match ch {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    // Parse fields in the segment [field_start, pos)
                    let segment = &text[field_start..*pos];
                    parse_fields(segment, &mut fields);
                    *pos += ch.len_utf8();
                    break;
                }
            }
            _ => {}
        }
        *pos += ch.len_utf8();
    }

    Some(BibEntry {
        key,
        entry_type,
        fields,
    })
}

fn parse_fields(segment: &str, fields: &mut HashMap<String, String>) {
    let mut i = 0;
    while i < segment.len() {
        // Skip whitespace
        while i < segment.len() {
            let ch = segment[i..].chars().next().unwrap();
            if ch.is_whitespace() || ch == ',' {
                i += ch.len_utf8();
            } else {
                break;
            }
        }
        if i >= segment.len() {
            break;
        }

        // Read field name
        let name_end = segment[i..]
            .find(|c: char| c == '=' || c == '{' || c == '"' || c.is_whitespace())
            .unwrap_or(segment.len() - i);
        let name = segment[i..i + name_end].trim().to_lowercase();
        i += name_end;

        // Skip to =
        while i < segment.len() {
            let ch = segment[i..].chars().next().unwrap();
            if ch == '=' {
                i += ch.len_utf8();
                break;
            } else if ch.is_whitespace() || ch == ',' {
                i += ch.len_utf8();
            } else {
                break;
            }
        }

        // Read value
        let value = read_value(segment, &mut i);
        if !name.is_empty() {
            fields.insert(name, value);
        }

        // Skip trailing comma
        while i < segment.len() {
            let ch = segment[i..].chars().next().unwrap();
            if ch == ',' {
                i += ch.len_utf8();
                break;
            } else if ch.is_whitespace() {
                i += ch.len_utf8();
            } else {
                break;
            }
        }
    }
}

fn read_value(text: &str, pos: &mut usize) -> String {
    // Skip whitespace
    while *pos < text.len() {
        let ch = text[*pos..].chars().next().unwrap();
        if ch.is_whitespace() {
            *pos += ch.len_utf8();
        } else {
            break;
        }
    }

    if *pos >= text.len() {
        return String::new();
    }

    if text[*pos..].starts_with('{') {
        let (inner, end) = extract_braced(text, *pos).unwrap_or((String::new(), *pos + 1));
        *pos = end;
        return inner;
    }

    if text[*pos..].starts_with('"') {
        *pos += 1; // skip opening quote
        let start = *pos;
        while *pos < text.len() && !text[*pos..].starts_with('"') {
            *pos += text[*pos..].chars().next().unwrap().len_utf8();
        }
        let val = text[start..*pos].to_string();
        if *pos < text.len() {
            *pos += 1; // skip closing quote
        }
        return val;
    }

    // Unquoted value (number or simple token)
    let start = *pos;
    while *pos < text.len() {
        let ch = text[*pos..].chars().next().unwrap();
        if ch == ',' || ch == '}' || ch.is_whitespace() {
            break;
        }
        *pos += ch.len_utf8();
    }
    text[start..*pos].trim().to_string()
}

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

/// Build a key→index map from a bibliography list.
pub fn build_citation_map(entries: &[(String, String)]) -> HashMap<String, usize> {
    entries
        .iter()
        .enumerate()
        .map(|(i, (key, _))| (key.clone(), i + 1))
        .collect()
}

/// Format citation keys as a numeric bracket string: `[1, 2, 3]`.
pub fn format_citation(keys: &[String], cmap: &HashMap<String, usize>) -> String {
    let mut nums: Vec<usize> = keys.iter().filter_map(|k| cmap.get(k).copied()).collect();
    nums.sort_unstable();
    nums.dedup();

    if nums.is_empty() {
        return String::new();
    }

    let s = nums
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", s)
}

/// Manages a collection of bibliography entries from one or more
/// `.bib` files.
#[derive(Debug, Clone)]
pub struct BibliographyManager {
    entries: HashMap<String, BibEntry>,
    style: Option<String>,
}

impl BibliographyManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            style: None,
        }
    }

    /// Load entries from a `.bib` file on disk.
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        for entry in parse_bibtex(&content) {
            self.entries.insert(entry.key.clone(), entry);
        }
        Ok(())
    }

    /// Add a single entry directly.
    pub fn add_entry(&mut self, entry: BibEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }

    /// Look up an entry by its citation key.
    pub fn get(&self, key: &str) -> Option<&BibEntry> {
        self.entries.get(key)
    }

    /// Check whether a key is known.
    pub fn has_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// All loaded keys, sorted for stable output.
    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.entries.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Set the bibliography style (e.g. `"plain"`, `"numeric"`).
    pub fn set_style(&mut self, style: String) {
        self.style = Some(style);
    }

    /// Get the current style, if any.
    pub fn style(&self) -> Option<&str> {
        self.style.as_deref()
    }

    /// Format entries matching `keys` as bibliography lines.
    pub fn format_bibliography(&self, keys: &[String]) -> Vec<String> {
        let mut out = Vec::new();
        for key in keys {
            if let Some(e) = self.entries.get(key) {
                out.push(e.format_plain());
            }
        }
        out
    }

    /// Format *all* entries in key-sorted order.
    pub fn format_all(&self) -> Vec<String> {
        self.format_bibliography(&self.keys())
    }

    /// Remove all entries and reset style.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.style = None;
    }

    /// Number of loaded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for BibliographyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_article() {
        let bib = r#"
@article{smith2024,
  author = {John Smith},
  title  = {A Great Paper},
  journal = {Journal of Testing},
  year   = 2024,
  volume = 42,
}
"#;
        let entries = parse_bibtex(bib);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "smith2024");
        assert_eq!(entries[0].entry_type, "article");
        assert_eq!(entries[0].get("author"), Some("John Smith"));
        assert_eq!(entries[0].get("title"), Some("A Great Paper"));
        assert_eq!(entries[0].get("year"), Some("2024"));
    }

    #[test]
    fn test_parse_book() {
        let bib = r#"@book{knuth1984, author = "Donald E. Knuth", title = {The TeXbook}, publisher = {Addison-Wesley}, year = 1984}"#;
        let entries = parse_bibtex(bib);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "knuth1984");
        assert_eq!(entries[0].get("title"), Some("The TeXbook"));
        assert_eq!(entries[0].get("publisher"), Some("Addison-Wesley"));
    }

    #[test]
    fn test_parse_multiple() {
        let bib = r#"
@article{a1, title={A}}
@article{a2, title={B}}
"#;
        let entries = parse_bibtex(bib);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].get("title"), Some("A"));
        assert_eq!(entries[1].get("title"), Some("B"));
    }

    #[test]
    fn test_citation_map() {
        let entries = vec![
            ("smith2024".to_string(), "Smith 2024".to_string()),
            ("jones2023".to_string(), "Jones 2023".to_string()),
        ];
        let cmap = build_citation_map(&entries);
        assert_eq!(cmap.get("smith2024"), Some(&1));
        assert_eq!(cmap.get("jones2023"), Some(&2));
    }

    #[test]
    fn test_format_citation() {
        let mut cmap = HashMap::new();
        cmap.insert("a".to_string(), 1);
        cmap.insert("b".to_string(), 2);
        cmap.insert("c".to_string(), 3);

        assert_eq!(
            format_citation(&["a".to_string(), "b".to_string()], &cmap),
            "[1, 2]"
        );
        assert_eq!(
            format_citation(&["b".to_string(), "a".to_string()], &cmap),
            "[1, 2]"
        );
        assert_eq!(format_citation(&["unknown".to_string()], &cmap), "");
    }

    #[test]
    fn test_format_numeric() {
        let entry = BibEntry {
            key: "e1".to_string(),
            entry_type: "article".to_string(),
            fields: [
                ("author".to_string(), "A. Author".to_string()),
                ("title".to_string(), "The Title".to_string()),
                ("journal".to_string(), "The Journal".to_string()),
                ("year".to_string(), "2024".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let text = entry.format_numeric();
        assert!(text.contains("A. Author"));
        assert!(text.contains("\"The Title\""));
        assert!(text.contains("The Journal"));
        assert!(text.contains("2024"));
    }

    #[test]
    fn test_entry_type_from_str() {
        assert_eq!(BibEntryType::from("article"), BibEntryType::Article);
        assert_eq!(BibEntryType::from("BOOK"), BibEntryType::Book);
        assert_eq!(
            BibEntryType::from("unknown"),
            BibEntryType::Custom("unknown".to_string())
        );
    }

    #[test]
    fn test_entry_type_display() {
        assert_eq!(BibEntryType::Article.to_string(), "article");
        assert_eq!(BibEntryType::Book.to_string(), "book");
        assert_eq!(
            BibEntryType::Custom("thesis".to_string()).to_string(),
            "thesis"
        );
    }

    #[test]
    fn test_bibliography_manager_add_and_get() {
        let mut mgr = BibliographyManager::new();
        let entry = BibEntry {
            key: "test".to_string(),
            entry_type: "article".to_string(),
            fields: [
                ("author".to_string(), "A. Author".to_string()),
                ("title".to_string(), "Test Title".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        mgr.add_entry(entry);
        assert!(mgr.has_key("test"));
        assert_eq!(mgr.get("test").unwrap().get("author"), Some("A. Author"));
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn test_bibliography_manager_format_all() {
        let mut mgr = BibliographyManager::new();
        mgr.add_entry(BibEntry {
            key: "b".to_string(),
            entry_type: "article".to_string(),
            fields: [("title".to_string(), "B".to_string())]
                .into_iter()
                .collect(),
        });
        mgr.add_entry(BibEntry {
            key: "a".to_string(),
            entry_type: "article".to_string(),
            fields: [("title".to_string(), "A".to_string())]
                .into_iter()
                .collect(),
        });
        let out = mgr.format_all();
        assert_eq!(out.len(), 2);
        // Should be sorted by key: a then b
        assert!(out[0].contains("A"));
        assert!(out[1].contains("B"));
    }

    #[test]
    fn test_bibliography_manager_clear() {
        let mut mgr = BibliographyManager::new();
        mgr.add_entry(BibEntry {
            key: "k".to_string(),
            entry_type: "article".to_string(),
            fields: HashMap::new(),
        });
        mgr.set_style("plain".to_string());
        mgr.clear();
        assert!(mgr.is_empty());
        assert_eq!(mgr.style(), None);
    }
}
