//! Structured error types for latex-rs.
//!
//! Provides rich error variants with context, line numbers, and suggestions.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for latex-rs operations
#[derive(Error, Debug)]
pub enum LatexError {
    /// Error during parsing
    #[error("Parse error: {message}{line_info}{column_info}{context_info}",
        line_info = line.map(|l| format!(" at line {}", l)).unwrap_or_default(),
        column_info = column.map(|c| format!(", column {}", c)).unwrap_or_default(),
        context_info = context.as_ref().map(|ctx| format!("\nContext: {}", ctx)).unwrap_or_default()
    )]
    ParseError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
        context: Option<String>,
    },

    /// Error during PDF generation
    #[error("PDF generation error: {message}{context_info}",
        context_info = context.as_ref().map(|ctx| format!("\nContext: {}", ctx)).unwrap_or_default()
    )]
    PdfError {
        message: String,
        context: Option<String>,
    },

    /// File I/O error
    #[error("I/O error for file '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Font loading error
    #[error("Font error for '{font_name}': {message}")]
    FontError {
        font_name: String,
        message: String,
    },

    /// Math formatting error
    #[error("Math error in '{expression}': {message}")]
    MathError {
        expression: String,
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
    },

    /// Unsupported feature
    #[error("Unsupported feature: {feature}{suggestion_info}",
        suggestion_info = suggestion.as_ref().map(|s| format!("\nSuggestion: {}", s)).unwrap_or_default()
    )]
    UnsupportedFeature {
        feature: String,
        suggestion: Option<String>,
    },

    /// Invalid file path
    #[error("Invalid file path")]
    InvalidPath,
}

/// Result type for latex-rs operations
pub type Result<T> = std::result::Result<T, LatexError>;

/// Helper trait for adding context to errors
pub trait ErrorContext<T> {
    fn context(self, message: impl Into<String>) -> Result<T>;
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<LatexError>,
{
    fn context(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let err: LatexError = e.into();
            match err {
                LatexError::ParseError { message: msg, line, column, .. } => {
                    LatexError::ParseError {
                        message: msg,
                        line,
                        column,
                        context: Some(message.into()),
                    }
                }
                LatexError::PdfError { message: msg, .. } => {
                    LatexError::PdfError {
                        message: msg,
                        context: Some(message.into()),
                    }
                }
                other => other,
            }
        })
    }
    
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let err: LatexError = e.into();
            let context_msg = f();
            match err {
                LatexError::ParseError { message: msg, line, column, .. } => {
                    LatexError::ParseError {
                        message: msg,
                        line,
                        column,
                        context: Some(context_msg),
                    }
                }
                LatexError::PdfError { message: msg, .. } => {
                    LatexError::PdfError {
                        message: msg,
                        context: Some(context_msg),
                    }
                }
                other => other,
            }
        })
    }
}

// Conversion from String to LatexError
impl From<String> for LatexError {
    fn from(message: String) -> Self {
        LatexError::PdfError {
            message,
            context: None,
        }
    }
}

// Conversion from &str to LatexError
impl From<&str> for LatexError {
    fn from(message: &str) -> Self {
        LatexError::PdfError {
            message: message.to_string(),
            context: None,
        }
    }
}

// Conversion from std::io::Error
impl From<std::io::Error> for LatexError {
    fn from(error: std::io::Error) -> Self {
        LatexError::IoError {
            path: PathBuf::from("<unknown>"),
            source: error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = LatexError::ParseError {
            message: "Unexpected token".to_string(),
            line: Some(42),
            column: Some(10),
            context: Some("\\begin{document}".to_string()),
        };
        let display = format!("{}", err);
        assert!(display.contains("Parse error"));
        assert!(display.contains("line 42"));
        assert!(display.contains("column 10"));
    }

    #[test]
    fn test_unsupported_feature() {
        let err = LatexError::UnsupportedFeature {
            feature: "\\usepackage{tikz}".to_string(),
            suggestion: Some("Use external LaTeX for TikZ diagrams".to_string()),
        };
        let display = format!("{}", err);
        assert!(display.contains("Unsupported feature"));
        assert!(display.contains("Suggestion"));
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), String> = Err("test error".to_string());
        let with_context = result.context("while processing file");
        assert!(with_context.is_err());
    }
}
