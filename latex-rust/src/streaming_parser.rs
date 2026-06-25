//! Streaming parser for large LaTeX documents
//! 
//! This module provides a streaming approach to parsing LaTeX documents,
//! which is more memory-efficient for large files by processing content
//! in chunks rather than loading everything into memory at once.

use crate::ast::*;
use crate::error::{LaTeXError, LaTeXResult};
use crate::lexer::{Lexer, Token, TokenType};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};

/// Configuration for the streaming parser
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Size of the token buffer (number of tokens to keep in memory)
    pub buffer_size: usize,
    /// Size of chunks to read from input (in bytes)
    pub chunk_size: usize,
    /// Whether to preserve whitespace tokens
    pub preserve_whitespace: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            chunk_size: 8192, // 8KB chunks
            preserve_whitespace: false,
        }
    }
}

/// Streaming LaTeX parser that processes documents in chunks
pub struct StreamingParser {
    lexer: Lexer,
    config: StreamingConfig,
    token_buffer: VecDeque<Token>,
    current_position: usize,
    input_buffer: String,
    eof_reached: bool,
}

impl StreamingParser {
    /// Create a new streaming parser with default configuration
    pub fn new() -> Self {
        Self::with_config(StreamingConfig::default())
    }
    
    /// Create a new streaming parser with custom configuration
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            lexer: Lexer::new(),
            config,
            token_buffer: VecDeque::new(),
            current_position: 0,
            input_buffer: String::new(),
            eof_reached: false,
        }
    }
    
    /// Parse a document from a reader in streaming fashion
    pub fn parse_stream<R: Read>(&mut self, reader: R) -> LaTeXResult<Document> {
        let mut buf_reader = BufReader::new(reader);
        let mut document = Document::new();
        
        // Reset state
        self.token_buffer.clear();
        self.current_position = 0;
        self.input_buffer.clear();
        self.eof_reached = false;
        
        // Process the stream in chunks
        loop {
            // Fill token buffer if needed
            if self.token_buffer.len() < self.config.buffer_size / 2 && !self.eof_reached {
                self.fill_token_buffer(&mut buf_reader)?;
            }
            
            // If no more tokens, we're done
            if self.token_buffer.is_empty() {
                break;
            }
            
            // Parse next node
            if let Some(node) = self.parse_next_node()? {
                document.add_node(node);
            }
        }
        
        Ok(document)
    }
    
    /// Fill the token buffer by reading and tokenizing more input
    pub(crate) fn fill_token_buffer<R: BufRead>(&mut self, reader: &mut R) -> LaTeXResult<()> {
        let mut chunk = vec![0u8; self.config.chunk_size];
        
        match reader.read(&mut chunk) {
            Ok(0) => {
                // EOF reached
                self.eof_reached = true;
                
                // Process any remaining input
                if !self.input_buffer.is_empty() {
                    let tokens = self.lexer.tokenize(&self.input_buffer)?;
                    self.extend_token_buffer(tokens);
                    self.input_buffer.clear();
                }
            }
            Ok(bytes_read) => {
                // Convert bytes to string and add to buffer
                let chunk_str = String::from_utf8_lossy(&chunk[..bytes_read]);
                self.input_buffer.push_str(&chunk_str);
                
                // Find a good breaking point (end of command or whitespace)
                if let Some(break_point) = self.find_break_point() {
                    let to_process = self.input_buffer[..break_point].to_string();
                    let remainder = self.input_buffer[break_point..].to_string();
                    let tokens = self.lexer.tokenize(&to_process)?;
                    self.extend_token_buffer(tokens);
                    
                    // Keep the remainder for next iteration
                    self.input_buffer = remainder;
                }
            }
            Err(e) => return Err(LaTeXError::IoError(e)),
        }
        
        Ok(())
    }
    
    /// Find a good point to break the input buffer for processing
    pub(crate) fn find_break_point(&self) -> Option<usize> {
        let input = &self.input_buffer;
        
        // Look for natural break points from the end
        for (i, ch) in input.char_indices().rev() {
            match ch {
                // Break after complete commands
                '}' | ']' => return Some(i + 1),
                // Break at whitespace
                ' ' | '\t' | '\n' | '\r' => return Some(i + 1),
                _ => continue,
            }
        }
        
        // If buffer is getting too large, force a break
        if input.len() > self.config.chunk_size * 2 {
            return Some(input.len() / 2);
        }
        
        None
    }
    
    /// Add tokens to the buffer, filtering based on configuration
    pub(crate) fn extend_token_buffer(&mut self, tokens: Vec<Token>) {
        for token in tokens {
            // Filter whitespace if configured
            if !self.config.preserve_whitespace {
                if let TokenType::Whitespace(_) = token.token_type {
                    continue;
                }
            }
            
            self.token_buffer.push_back(token);
            
            // Limit buffer size
            if self.token_buffer.len() > self.config.buffer_size {
                break;
            }
        }
    }
    
    /// Parse the next node from the token buffer
    pub(crate) fn parse_next_node(&mut self) -> LaTeXResult<Option<Node>> {
        if self.token_buffer.is_empty() {
            return Ok(None);
        }
        
        let token = self.token_buffer.pop_front().unwrap();
        
        match &token.token_type {
            TokenType::Command(name) => {
                self.parse_command_streaming(name.clone())
            }
            TokenType::Text(text) => {
                Ok(Some(Node::Text(text.clone())))
            }
            TokenType::LeftBrace => {
                self.parse_group_streaming()
            }
            TokenType::Dollar => {
                self.parse_math_streaming(false)
            }
            TokenType::DoubleDollar => {
                self.parse_math_streaming(true)
            }
            TokenType::Newline => {
                Ok(Some(Node::Text("\n".to_string())))
            }
            TokenType::Whitespace(ws) => {
                Ok(Some(Node::Text(ws.clone())))
            }
            _ => {
                // Handle other token types as text
                Ok(Some(Node::Text(format!("{:?}", token.token_type))))
            }
        }
    }
    
    /// Parse a command in streaming mode (simplified)
    pub(crate) fn parse_command_streaming(&mut self, name: String) -> LaTeXResult<Option<Node>> {
        match name.as_str() {
            "section" | "subsection" | "subsubsection" => {
                self.parse_section_command_streaming(name)
            }
            "textbf" | "textit" | "emph" => {
                self.parse_text_formatting_command_streaming(name)
            }
            _ => {
                self.parse_generic_command_streaming(name)
            }
        }
    }
    
    /// Parse section commands in streaming mode
    fn parse_section_command_streaming(&mut self, name: String) -> LaTeXResult<Option<Node>> {
        let title = self.parse_required_argument_streaming()?;
        let level = self.determine_section_level(&name);
        Ok(Some(Node::Section {
            level,
            title,
            content: vec![],
        }))
    }
    
    /// Determine section level from command name
    fn determine_section_level(&self, name: &str) -> crate::ast::SectionLevel {
        match name {
            "section" => crate::ast::SectionLevel::Section,
            "subsection" => crate::ast::SectionLevel::Subsection,
            "subsubsection" => crate::ast::SectionLevel::Subsubsection,
            _ => crate::ast::SectionLevel::Section,
        }
    }
    
    /// Parse text formatting commands in streaming mode
    fn parse_text_formatting_command_streaming(&mut self, name: String) -> LaTeXResult<Option<Node>> {
        let content = self.parse_required_argument_streaming()?;
        Ok(Some(Node::Command {
            name,
            args: vec![Argument::Required(vec![Node::Text(content)])],
            content: None,
        }))
    }
    
    /// Parse generic commands in streaming mode
    fn parse_generic_command_streaming(&mut self, name: String) -> LaTeXResult<Option<Node>> {
        Ok(Some(Node::Command {
            name,
            args: vec![],
            content: None,
        }))
    }
    
    /// Parse a required argument in streaming mode
    fn parse_required_argument_streaming(&mut self) -> LaTeXResult<String> {
        if !self.find_opening_brace()? {
            return Ok(String::new());
        }
        
        self.extract_braced_content()
    }
    
    /// Find and consume the opening brace for an argument
    fn find_opening_brace(&mut self) -> LaTeXResult<bool> {
        while let Some(token) = self.token_buffer.front() {
            match &token.token_type {
                TokenType::LeftBrace => {
                    self.token_buffer.pop_front();
                    return Ok(true);
                }
                TokenType::Whitespace(_) => {
                    self.token_buffer.pop_front();
                    continue;
                }
                _ => {
                    // No brace found
                    return Ok(false);
                }
            }
        }
        Ok(false)
    }
    
    /// Extract content between braces
    fn extract_braced_content(&mut self) -> LaTeXResult<String> {
        let mut content = String::new();
        let mut brace_count = 1;
        
        while let Some(token) = self.token_buffer.pop_front() {
            match &token.token_type {
                TokenType::LeftBrace => {
                    brace_count += 1;
                    content.push('{');
                }
                TokenType::RightBrace => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        break;
                    }
                    content.push('}');
                }
                _ => {
                    self.append_token_to_content(&mut content, &token.token_type);
                }
            }
        }
        
        Ok(content)
    }
    
    /// Append a token to the content string
    fn append_token_to_content(&self, content: &mut String, token_type: &TokenType) {
        match token_type {
            TokenType::Text(text) => {
                content.push_str(text);
            }
            TokenType::Command(cmd) => {
                content.push('\\');
                content.push_str(cmd);
            }
            _ => {
                content.push_str(&format!("{token_type:?}"));
            }
        }
    }
    
    /// Parse a group in streaming mode
    fn parse_group_streaming(&mut self) -> LaTeXResult<Option<Node>> {
        let content = self.parse_required_argument_streaming()?;
        Ok(Some(Node::Group(vec![Node::Text(content)])))
    }
    
    /// Parse math mode in streaming mode
    fn parse_math_streaming(&mut self, display: bool) -> LaTeXResult<Option<Node>> {
        let mut content = String::new();
        let end_token = if display { TokenType::DoubleDollar } else { TokenType::Dollar };
        
        while let Some(token) = self.token_buffer.pop_front() {
            if token.token_type == end_token {
                break;
            }
            
            match &token.token_type {
                TokenType::Text(text) => content.push_str(text),
                TokenType::Command(cmd) => {
                    content.push('\\');
                    content.push_str(cmd);
                }
                _ => content.push_str(&format!("{:?}", token.token_type)),
            }
        }
        
        Ok(Some(Node::Math { display, content }))
    }
    
    /// Get current buffer statistics for monitoring
    pub fn buffer_stats(&self) -> (usize, usize, bool) {
        (self.token_buffer.len(), self.input_buffer.len(), self.eof_reached)
    }
}

impl Default for StreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_streaming_parser_basic() {
        let mut parser = StreamingParser::new();
        let input = "\\section{Test}\nHello world!";
        let cursor = Cursor::new(input.as_bytes());
        
        let result = parser.parse_stream(cursor);
        assert!(result.is_ok());
        
        let document = result.unwrap();
        assert!(!document.body.is_empty());
    }
    
    #[test]
    fn test_streaming_config() {
        let config = StreamingConfig {
            buffer_size: 500,
            chunk_size: 4096,
            preserve_whitespace: true,
        };
        
        let parser = StreamingParser::with_config(config.clone());
        assert_eq!(parser.config.buffer_size, 500);
        assert_eq!(parser.config.chunk_size, 4096);
        assert!(parser.config.preserve_whitespace);
    }
}