//! BibTeX parser and bibliography management
//!
//! This module provides functionality for parsing BibTeX files and managing
//! bibliography entries for LaTeX documents.

use crate::ast::Node;
use crate::error::{LaTeXError, LaTeXResult};
use crate::common::{HasKey, Clear};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// BibTeX entry types
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

/// BibTeX entry representation
#[derive(Debug, Clone)]
pub struct BibEntry {
    pub key: String,
    pub entry_type: BibEntryType,
    pub fields: HashMap<String, String>,
}

/// BibTeX parser
pub struct BibTeXParser {
    content: String,
    position: usize,
    current_char: Option<char>,
}

impl BibTeXParser {
    /// Create a new BibTeX parser
    pub fn new(content: String) -> Self {
        let mut parser = Self {
            content,
            position: 0,
            current_char: None,
        };
        parser.current_char = parser.content.chars().next();
        parser
    }

    /// Parse BibTeX file from path
    pub fn parse_file<P: AsRef<Path>>(path: P) -> LaTeXResult<Vec<BibEntry>> {
        let content = fs::read_to_string(path)
            .map_err(|e| LaTeXError::InvalidSyntax { message: format!("Failed to read BibTeX file: {e}") })?;
        
        let mut parser = Self::new(content);
        parser.parse()
    }

    /// Parse BibTeX content
    pub fn parse(&mut self) -> LaTeXResult<Vec<BibEntry>> {
        let mut entries = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            
            if self.is_at_end() {
                break;
            }
            
            if self.current_char == Some('@') {
                entries.push(self.parse_entry()?);
            } else {
                self.advance();
            }
        }
        
        Ok(entries)
    }

    /// Parse a single BibTeX entry
    fn parse_entry(&mut self) -> LaTeXResult<BibEntry> {
        // Consume '@'
        self.advance();
        
        // Parse entry type
        let entry_type = self.parse_identifier()?;
        
        self.skip_whitespace();
        
        // Expect '{'
        if self.current_char != Some('{') {
            return Err(LaTeXError::InvalidSyntax { 
                message: format!("Expected '{{' after entry type at position {}", self.position)
            });
        }
        self.advance();
        
        self.skip_whitespace();
        
        // Parse entry key
        let key = self.parse_identifier()?;
        
        self.skip_whitespace();
        
        // Parse fields
        let mut fields = HashMap::new();
        
        while self.current_char != Some('}') && !self.is_at_end() {
            self.skip_whitespace();
            
            if self.current_char == Some(',') {
                self.advance();
                self.skip_whitespace();
                continue;
            }
            
            if self.current_char == Some('}') {
                break;
            }
            
            // Parse field
            let field_name = self.parse_identifier()?;
            
            self.skip_whitespace();
            
            // Expect '='
            if self.current_char != Some('=') {
                return Err(LaTeXError::InvalidSyntax { 
                    message: format!("Expected '=' after field name '{}' at position {}", field_name, self.position)
                });
            }
            self.advance();
            
            self.skip_whitespace();
            
            // Parse field value
            let field_value = self.parse_field_value()?;
            
            fields.insert(field_name.to_lowercase(), field_value);
            
            self.skip_whitespace();
        }
        
        // Expect '}'
        if self.current_char != Some('}') {
            return Err(LaTeXError::InvalidSyntax { 
                message: format!("Expected '}}' to close entry at position {}", self.position)
            });
        }
        self.advance();
        
        Ok(BibEntry {
            key,
            entry_type: BibEntryType::from(entry_type.as_str()),
            fields,
        })
    }

    /// Parse an identifier (entry type, key, or field name)
    fn parse_identifier(&mut self) -> LaTeXResult<String> {
        let mut identifier = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        if identifier.is_empty() {
            return Err(LaTeXError::InvalidSyntax { 
                message: format!("Expected identifier at position {}", self.position)
            });
        }
        
        Ok(identifier)
    }

    /// Parse a field value (string, number, or concatenation)
    fn parse_field_value(&mut self) -> LaTeXResult<String> {
        let mut value = String::new();
        
        loop {
            self.skip_whitespace();
            
            match self.current_char {
                Some('"') => {
                    // Quoted string
                    self.advance(); // consume opening quote
                    
                    while let Some(ch) = self.current_char {
                        if ch == '"' {
                            self.advance(); // consume closing quote
                            break;
                        } else if ch == '\\' {
                            // Handle escape sequences
                            self.advance();
                            if let Some(escaped) = self.current_char {
                                match escaped {
                                    '"' => value.push('"'),
                                    '\\' => value.push('\\'),
                                    'n' => value.push('\n'),
                                    't' => value.push('\t'),
                                    _ => {
                                        value.push('\\');
                                        value.push(escaped);
                                    }
                                }
                                self.advance();
                            }
                        } else {
                            value.push(ch);
                            self.advance();
                        }
                    }
                }
                Some('{') => {
                    // Braced string
                    self.advance(); // consume opening brace
                    let mut brace_count = 1;
                    
                    while let Some(ch) = self.current_char {
                        if ch == '{' {
                            brace_count += 1;
                        } else if ch == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                self.advance(); // consume closing brace
                                break;
                            }
                        }
                        value.push(ch);
                        self.advance();
                    }
                }
                Some(ch) if ch.is_ascii_digit() => {
                    // Number
                    while let Some(ch) = self.current_char {
                        if ch.is_ascii_digit() {
                            value.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                Some(ch) if ch.is_alphabetic() => {
                    // String variable or abbreviation
                    let var_name = self.parse_identifier()?;
                    value.push_str(&var_name);
                }
                _ => break,
            }
            
            self.skip_whitespace();
            
            // Check for concatenation operator '#'
            if self.current_char == Some('#') {
                self.advance();
                value.push(' '); // Add space for concatenation
            } else {
                break;
            }
        }
        
        Ok(value)
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '%' {
                // Skip comment line
                while let Some(ch) = self.current_char {
                    if ch == '\n' {
                        self.advance();
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// Advance to next character
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.content.chars().nth(self.position);
    }

    /// Check if at end of input
    fn is_at_end(&self) -> bool {
        self.current_char.is_none()
    }
}

/// Bibliography manager
pub struct BibliographyManager {
    entries: HashMap<String, BibEntry>,
    style: Option<String>,
}

impl BibliographyManager {
    /// Create a new bibliography manager
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            style: None,
        }
    }

    /// Load entries from BibTeX file
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> LaTeXResult<()> {
        let entries = BibTeXParser::parse_file(path)?;
        
        for entry in entries {
            self.entries.insert(entry.key.clone(), entry);
        }
        
        Ok(())
    }

    /// Add a single entry
    pub fn add_entry(&mut self, entry: BibEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }

    /// Get entry by key
    pub fn get_entry(&self, key: &str) -> Option<&BibEntry> {
        self.entries.get(key)
    }

    /// Set bibliography style
    pub fn set_style(&mut self, style: String) {
        self.style = Some(style);
    }

    /// Get bibliography style
    pub fn get_style(&self) -> Option<&String> {
        self.style.as_ref()
    }

    /// Generate bibliography nodes for given citation keys
    pub fn generate_bibliography(&self, keys: &[String]) -> Vec<Node> {
        let mut nodes = Vec::new();
        
        for key in keys {
            if let Some(entry) = self.entries.get(key) {
                nodes.push(Node::BibliographyEntry {
                    key: entry.key.clone(),
                    entry_type: entry.entry_type.to_string(),
                    fields: entry.fields.clone(),
                });
            }
        }
        
        nodes
    }

    /// Generate complete bibliography with all entries
    pub fn generate_complete_bibliography(&self) -> Vec<Node> {
        let mut nodes = Vec::new();
        
        // Sort entries by key for consistent output
        let mut sorted_entries: Vec<_> = self.entries.values().collect();
        sorted_entries.sort_by(|a, b| a.key.cmp(&b.key));
        
        for entry in sorted_entries {
            nodes.push(Node::BibliographyEntry {
                key: entry.key.clone(),
                entry_type: entry.entry_type.to_string(),
                fields: entry.fields.clone(),
            });
        }
        
        nodes
    }

    /// Check if a citation key exists
    pub fn has_key(&self, key: &str) -> bool {
        self.entries.has_key(key)
    }

    /// Get all available keys
    pub fn get_all_keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

impl Clear for BibliographyManager {
    fn clear(&mut self) {
        self.entries.clear();
        self.style = None;
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
    fn test_parse_simple_entry() {
        let content = r#"
@article{key1,
  author = "John Doe",
  title = {A Simple Title},
  year = 2023
}
"#;
        
        let mut parser = BibTeXParser::new(content.to_string());
        let entries = parser.parse().unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "key1");
        assert_eq!(entries[0].entry_type, BibEntryType::Article);
        assert_eq!(entries[0].fields.get("author").unwrap(), "John Doe");
        assert_eq!(entries[0].fields.get("title").unwrap(), "A Simple Title");
        assert_eq!(entries[0].fields.get("year").unwrap(), "2023");
    }

    #[test]
    fn test_bibliography_manager() {
        let mut manager = BibliographyManager::new();
        
        let entry = BibEntry {
            key: "test_key".to_string(),
            entry_type: BibEntryType::Article,
            fields: {
                let mut fields = HashMap::new();
                fields.insert("author".to_string(), "Test Author".to_string());
                fields.insert("title".to_string(), "Test Title".to_string());
                fields
            },
        };
        
        manager.add_entry(entry);
        
        assert!(manager.has_key("test_key"));
        assert!(!manager.has_key("nonexistent_key"));
        
        let retrieved = manager.get_entry("test_key").unwrap();
        assert_eq!(retrieved.key, "test_key");
        assert_eq!(retrieved.fields.get("author").unwrap(), "Test Author");
    }
}