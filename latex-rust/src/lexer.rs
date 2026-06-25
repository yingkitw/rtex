//! LaTeX lexer for tokenizing input

use crate::error::{LaTeXResult, Position};
use regex::Regex;
use std::fmt;

/// Token types in LaTeX
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// LaTeX command (e.g., \section, \textbf)
    Command(String),
    /// Plain text content
    Text(String),
    /// Opening brace {
    LeftBrace,
    /// Closing brace }
    RightBrace,
    /// Opening bracket [
    LeftBracket,
    /// Closing bracket ]
    RightBracket,
    /// Dollar sign for math mode
    Dollar,
    /// Double dollar for display math
    DoubleDollar,
    /// Ampersand for table alignment
    Ampersand,
    /// Backslash for line breaks
    Backslash,
    /// Newline character
    Newline,
    /// Whitespace
    Whitespace(String),
    /// End of file
    Eof,
}

/// A token with its type and position
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Position,
}

impl Token {
    pub fn new(token_type: TokenType, position: Position) -> Self {
        Self { token_type, position }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}", self.token_type, self.position)
    }
}

/// LaTeX lexer
pub struct Lexer {
    command_regex: Regex,
}

impl Lexer {
    /// Create a new lexer
    pub fn new() -> Self {
        Self {
            command_regex: Regex::new(r"\\[a-zA-Z]+\*?").unwrap(),
        }
    }

    /// Tokenize LaTeX input
    pub fn tokenize(&self, input: &str) -> LaTeXResult<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();
        let mut position = Position::start();

        while let Some((pos, ch)) = chars.next() {
            let current_pos = position;
            
            match ch {
                '\\' => {
                    // Check for double backslash first
                    if let Some((_, next_ch)) = chars.peek() {
                        if *next_ch == '\\' {
                            chars.next(); // consume the second backslash
                            tokens.push(Token::new(TokenType::Command("\\\\".to_string()), current_pos));
                            position.advance('\\');
                            position.advance('\\');
                            continue;
                        }
                    }
                    
                    // Check if this is a command
                    if let Some(command_match) = self.command_regex.find_at(input, pos) {
                        if command_match.start() == pos {
                            let command_text = &input[command_match.start() + 1..command_match.end()];
                            tokens.push(Token::new(
                                TokenType::Command(command_text.to_string()),
                                current_pos,
                            ));
                            // Skip the characters we've already processed
                            for _ in 0..(command_match.len() - 1) {
                                let (_, next_char) = chars.next().unwrap();
                                position.advance(next_char);
                            }
                            position.advance('\\');
                            continue;
                        }
                    }
                    // If not a command, treat as backslash
                    tokens.push(Token::new(TokenType::Backslash, current_pos));
                    position.advance(ch);
                }
                '{' => {
                    tokens.push(Token::new(TokenType::LeftBrace, current_pos));
                    position.advance(ch);
                }
                '}' => {
                    tokens.push(Token::new(TokenType::RightBrace, current_pos));
                    position.advance(ch);
                }
                '[' => {
                    tokens.push(Token::new(TokenType::LeftBracket, current_pos));
                    position.advance(ch);
                }
                ']' => {
                    tokens.push(Token::new(TokenType::RightBracket, current_pos));
                    position.advance(ch);
                }
                '$' => {
                    // Check for double dollar
                    if let Some((_, next_ch)) = chars.peek() {
                        if *next_ch == '$' {
                            chars.next(); // consume the second $
                            tokens.push(Token::new(TokenType::DoubleDollar, current_pos));
                            position.advance('$');
                            position.advance('$');
                            continue;
                        }
                    }
                    tokens.push(Token::new(TokenType::Dollar, current_pos));
                    position.advance(ch);
                }
                '&' => {
                    tokens.push(Token::new(TokenType::Ampersand, current_pos));
                    position.advance(ch);
                }
                '\n' => {
                    tokens.push(Token::new(TokenType::Newline, current_pos));
                    position.advance(ch);
                }
                c if c.is_whitespace() => {
                     // Collect consecutive whitespace
                     let mut whitespace = String::new();
                     whitespace.push(c);
                     position.advance(c);

                     loop {
                         match chars.peek() {
                             Some((_, next_ch)) if next_ch.is_whitespace() && *next_ch != '\n' => {
                                 let next_char = *next_ch;
                                 whitespace.push(next_char);
                                 chars.next();
                                 position.advance(next_char);
                             }
                             _ => break,
                         }
                     }
                     tokens.push(Token::new(TokenType::Whitespace(whitespace), current_pos));
                 }
                _ => {
                    // Collect consecutive text characters
                    let mut text = String::new();
                    text.push(ch);
                    position.advance(ch);

                    loop {
                        match chars.peek() {
                            Some((_, next_ch)) if !matches!(*next_ch, '\\' | '{' | '}' | '[' | ']' | '$' | '&' | '\n') && !next_ch.is_whitespace() => {
                                let next_char = *next_ch;
                                text.push(next_char);
                                chars.next();
                                position.advance(next_char);
                            }
                            _ => break,
                        }
                    }
                    tokens.push(Token::new(TokenType::Text(text), current_pos));
                }
            }
        }

        tokens.push(Token::new(TokenType::Eof, position));
        Ok(tokens)
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\section{Hello World}").unwrap();
        
        // Expected: command, {, text("Hello"), whitespace(" "), text("World"), }, EOF
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0].token_type, TokenType::Command(ref cmd) if cmd == "section"));
        assert!(matches!(tokens[1].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[2].token_type, TokenType::Text(ref text) if text == "Hello"));
        assert!(matches!(tokens[3].token_type, TokenType::Whitespace(_)));
        assert!(matches!(tokens[4].token_type, TokenType::Text(ref text) if text == "World"));
        assert!(matches!(tokens[5].token_type, TokenType::RightBrace));
        assert!(matches!(tokens[6].token_type, TokenType::Eof));
    }

    #[test]
    fn test_math_mode() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("$x^2$ and $$y^3$$").unwrap();
        
        // Find the dollar tokens in the sequence
        let dollar_positions: Vec<usize> = tokens.iter().enumerate()
            .filter_map(|(i, token)| match token.token_type {
                TokenType::Dollar | TokenType::DoubleDollar => Some(i),
                _ => None,
            })
            .collect();
        
        // Expected: $, $, $$, $$
        assert_eq!(dollar_positions.len(), 4);
        assert!(matches!(tokens[dollar_positions[0]].token_type, TokenType::Dollar));
        assert!(matches!(tokens[dollar_positions[1]].token_type, TokenType::Dollar));
        assert!(matches!(tokens[dollar_positions[2]].token_type, TokenType::DoubleDollar));
        assert!(matches!(tokens[dollar_positions[3]].token_type, TokenType::DoubleDollar));
    }
}