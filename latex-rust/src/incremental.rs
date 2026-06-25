//! Incremental parsing module for efficient document updates
//!
//! This module provides functionality to track changes in LaTeX documents
//! and only reprocess the sections that have been modified, significantly
//! improving performance for large documents with small changes.

use crate::ast::*;
use crate::lexer::*;
use crate::parser::*;
use crate::error::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Represents a section of a document with its hash for change detection
#[derive(Debug, Clone)]
pub struct DocumentSection {
    /// Starting position in the original document
    pub start: usize,
    /// Ending position in the original document
    pub end: usize,
    /// Hash of the section content
    pub content_hash: u64,
    /// Parsed AST for this section
    pub ast: Option<Vec<Node>>,
    /// Section type (e.g., "preamble", "section", "subsection", "paragraph")
    pub section_type: String,
}

/// Configuration for incremental parsing
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// Minimum section size to consider for incremental parsing
    pub min_section_size: usize,
    /// Maximum number of sections to track
    pub max_sections: usize,
    /// Whether to enable fine-grained paragraph-level tracking
    pub paragraph_level: bool,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            min_section_size: 100,
            max_sections: 1000,
            paragraph_level: true,
        }
    }
}

/// Incremental parser that tracks document changes
pub struct IncrementalParser {
    /// Configuration for incremental parsing
    config: IncrementalConfig,
    /// Cached document sections
    sections: HashMap<String, DocumentSection>,
    /// Full document hash for quick change detection
    document_hash: Option<u64>,
    /// Lexer instance
    lexer: Lexer,
    /// Parser instance
    parser: Parser,
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new() -> Self {
        Self {
            config: IncrementalConfig::default(),
            sections: HashMap::new(),
            document_hash: None,
            lexer: Lexer::new(),
            parser: Parser::new(),
        }
    }

    /// Create a new incremental parser with custom configuration
    pub fn with_config(config: IncrementalConfig) -> Self {
        Self {
            config,
            sections: HashMap::new(),
            document_hash: None,
            lexer: Lexer::new(),
            parser: Parser::new(),
        }
    }

    /// Calculate hash of a string
    fn calculate_hash(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Split document into logical sections
    fn split_into_sections(&self, content: &str) -> Vec<(String, usize, usize)> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_start = 0;
        let mut current_section = "preamble".to_string();
        
        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if let Some(new_section_type) = self.detect_section_boundary(trimmed) {
                self.finalize_current_section(&mut sections, &current_section, current_start, line_idx, &lines);
                current_start = self.calculate_line_position(&lines, line_idx);
                current_section = new_section_type;
            } else if self.should_split_paragraph(trimmed, line_idx, &lines) {
                self.handle_paragraph_split(&mut sections, &current_section, &mut current_start, line_idx, &lines);
            }
        }
        
        // Add final section
        if current_start < content.len() {
            sections.push((current_section, current_start, content.len()));
        }
        
        sections
    }
    
    /// Detect if a line represents a section boundary
    fn detect_section_boundary(&self, trimmed: &str) -> Option<String> {
        if trimmed.starts_with("\\documentclass") {
            Some("documentclass".to_string())
        } else if trimmed.starts_with("\\begin{document}") {
            Some("document_begin".to_string())
        } else if trimmed.starts_with("\\section{") {
            Some("section".to_string())
        } else if trimmed.starts_with("\\subsection{") {
            Some("subsection".to_string())
        } else {
            None
        }
    }
    
    /// Finalize the current section and add it to the sections list
    fn finalize_current_section(
        &self,
        sections: &mut Vec<(String, usize, usize)>,
        current_section: &str,
        current_start: usize,
        line_idx: usize,
        lines: &[&str]
    ) {
        if current_start < line_idx {
            let end_pos = self.calculate_line_position(lines, line_idx);
            sections.push((current_section.to_string(), current_start, end_pos));
        }
    }
    
    /// Calculate the byte position of a line in the document
    fn calculate_line_position(&self, lines: &[&str], line_idx: usize) -> usize {
        lines[..line_idx].join("\n").len()
    }
    
    /// Check if we should split at a paragraph boundary
    fn should_split_paragraph(&self, trimmed: &str, line_idx: usize, lines: &[&str]) -> bool {
        if !self.config.paragraph_level || !trimmed.is_empty() || line_idx == 0 {
            return false;
        }
        
        let prev_line = lines.get(line_idx.saturating_sub(1)).unwrap_or(&"").trim();
        let next_line = lines.get(line_idx + 1).unwrap_or(&"").trim();
        
        !prev_line.is_empty() && !next_line.is_empty() && !next_line.starts_with("\\")
    }
    
    /// Handle paragraph-level splitting
    fn handle_paragraph_split(
        &self,
        sections: &mut Vec<(String, usize, usize)>,
        current_section: &str,
        current_start: &mut usize,
        line_idx: usize,
        lines: &[&str]
    ) {
        if *current_start < line_idx {
            let end_pos = self.calculate_line_position(lines, line_idx);
            if end_pos - *current_start >= self.config.min_section_size {
                sections.push((format!("{current_section}_para"), *current_start, end_pos));
                *current_start = end_pos;
            }
        }
    }

    /// Parse document incrementally
    pub fn parse_incremental(&mut self, content: &str) -> Result<Document, LaTeXError> {
        let new_hash = Self::calculate_hash(content);
        
        // Quick check: if document hash hasn't changed, return cached result
        if let Some(old_hash) = self.document_hash {
            if old_hash == new_hash {
                return self.reconstruct_document();
            }
        }
        
        self.document_hash = Some(new_hash);
        
        // Split document into sections
        let section_boundaries = self.split_into_sections(content);
        let mut changed_sections = Vec::new();
        let mut new_sections = HashMap::new();
        
        // Check each section for changes
        for (section_type, start, end) in section_boundaries {
            let section_content = &content[start..end];
            let section_hash = Self::calculate_hash(section_content);
            let section_key = format!("{section_type}_{start}_{end}");
            
            // Check if section has changed
            let needs_reparse = if let Some(cached_section) = self.sections.get(&section_key) {
                cached_section.content_hash != section_hash
            } else {
                true // New section
            };
            
            if needs_reparse {
                // Parse this section
                let tokens = self.lexer.tokenize(section_content)?;
                let nodes = self.parser.parse_tokens_to_nodes(tokens)?;
                
                let section = DocumentSection {
                    start,
                    end,
                    content_hash: section_hash,
                    ast: Some(nodes),
                    section_type: section_type.clone(),
                };
                
                new_sections.insert(section_key.clone(), section);
                changed_sections.push(section_key);
            } else {
                // Reuse cached section
                if let Some(cached_section) = self.sections.get(&section_key) {
                    new_sections.insert(section_key, cached_section.clone());
                }
            }
        }
        
        // Update sections cache
        self.sections = new_sections;
        
        // Reconstruct full document
        self.reconstruct_document()
    }
    
    /// Reconstruct document from cached sections
    fn reconstruct_document(&self) -> Result<Document, LaTeXError> {
        let mut all_nodes = Vec::new();
        
        // Sort sections by start position
        let mut sorted_sections: Vec<_> = self.sections.values().collect();
        sorted_sections.sort_by_key(|s| s.start);
        
        // Combine all AST nodes
        for section in sorted_sections {
            if let Some(ref nodes) = section.ast {
                all_nodes.extend(nodes.clone());
            }
        }
        
        Ok(Document {
            preamble: Vec::new(),
            body: all_nodes,
            metadata: DocumentMetadata::default(),
        })
    }
    
    /// Get statistics about incremental parsing
    pub fn get_stats(&self) -> IncrementalStats {
        IncrementalStats {
            total_sections: self.sections.len(),
            cached_sections: self.sections.values().filter(|s| s.ast.is_some()).count(),
            document_hash: self.document_hash,
        }
    }
    
    /// Clear all cached sections
    pub fn clear_cache(&mut self) {
        self.sections.clear();
        self.document_hash = None;
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: IncrementalConfig) {
        self.config = config;
        // Clear cache when config changes as section boundaries might change
        self.clear_cache();
    }
}

/// Statistics for incremental parsing
#[derive(Debug, Clone)]
pub struct IncrementalStats {
    pub total_sections: usize,
    pub cached_sections: usize,
    pub document_hash: Option<u64>,
}

/// Extension trait for Parser to support incremental parsing
trait ParserExt {
    fn parse_tokens_to_nodes(&mut self, tokens: Vec<Token>) -> Result<Vec<Node>, LaTeXError>;
}

impl ParserExt for Parser {
    fn parse_tokens_to_nodes(&mut self, tokens: Vec<Token>) -> Result<Vec<Node>, LaTeXError> {
        // This is a simplified implementation - in practice, you'd want to
        // extract the node parsing logic from the main parse method
        let document = self.parse(tokens)?;
        Ok(document.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_parser_creation() {
        let parser = IncrementalParser::new();
        assert_eq!(parser.sections.len(), 0);
        assert!(parser.document_hash.is_none());
    }

    #[test]
    fn test_section_splitting() {
        let parser = IncrementalParser::new();
        let content = r#"
\documentclass{article}
\begin{document}
\section{Introduction}
This is the introduction.

\subsection{Background}
Some background information.
\end{document}
        "#;
        
        let sections = parser.split_into_sections(content);
        assert!(!sections.is_empty());
    }

    #[test]
    fn test_hash_calculation() {
        let hash1 = IncrementalParser::calculate_hash("test content");
        let hash2 = IncrementalParser::calculate_hash("test content");
        let hash3 = IncrementalParser::calculate_hash("different content");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}