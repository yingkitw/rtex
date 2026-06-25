//! Error types for the LaTeX processor

use thiserror::Error;

/// Position information for enhanced error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in the input (0-indexed)
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }
    
    /// Create a position at the start of input
    pub fn start() -> Self {
        Self::new(1, 1, 0)
    }
    
    /// Advance position by one character
    pub fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.offset += ch.len_utf8();
    }
    
    /// Advance position by multiple characters
    pub fn advance_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.advance(ch);
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Main error type for LaTeX processing
#[derive(Error, Debug)]
pub enum LaTeXError {
    #[error("Lexer error at {position}: {message}")]
    LexerError { position: Position, message: String },

    #[error("Parser error at {position}: {message}")]
    ParserError { position: Position, message: String },

    #[error("Renderer error: {message}")]
    RendererError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown command: {command}")]
    UnknownCommand { command: String },

    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String },

    #[error("Missing argument for command: {command}")]
    MissingArgument { command: String },
}

/// Result type alias for LaTeX operations
pub type LaTeXResult<T> = Result<T, LaTeXError>;