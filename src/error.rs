use std::fmt;
use std::path::PathBuf;

/// Main error type for latex-rs operations
#[derive(Debug)]
pub enum LatexError {
    /// Error during parsing
    ParseError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
        context: Option<String>,
    },
    
    /// Error during PDF generation
    PdfError {
        message: String,
        context: Option<String>,
    },
    
    /// File I/O error
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    
    /// Font loading error
    FontError {
        font_name: String,
        message: String,
    },
    
    /// Math formatting error
    MathError {
        expression: String,
        message: String,
    },
    
    /// Configuration error
    ConfigError {
        message: String,
    },
    
    /// Unsupported feature
    UnsupportedFeature {
        feature: String,
        suggestion: Option<String>,
    },
}

impl fmt::Display for LatexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LatexError::ParseError { message, line, column, context } => {
                write!(f, "Parse error: {}", message)?;
                if let Some(l) = line {
                    write!(f, " at line {}", l)?;
                    if let Some(c) = column {
                        write!(f, ", column {}", c)?;
                    }
                }
                if let Some(ctx) = context {
                    write!(f, "\nContext: {}", ctx)?;
                }
                Ok(())
            }
            LatexError::PdfError { message, context } => {
                write!(f, "PDF generation error: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, "\nContext: {}", ctx)?;
                }
                Ok(())
            }
            LatexError::IoError { path, source } => {
                write!(f, "I/O error for file '{}': {}", path.display(), source)
            }
            LatexError::FontError { font_name, message } => {
                write!(f, "Font error for '{}': {}", font_name, message)
            }
            LatexError::MathError { expression, message } => {
                write!(f, "Math error in '{}': {}", expression, message)
            }
            LatexError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            LatexError::UnsupportedFeature { feature, suggestion } => {
                write!(f, "Unsupported feature: {}", feature)?;
                if let Some(sug) = suggestion {
                    write!(f, "\nSuggestion: {}", sug)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for LatexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LatexError::IoError { source, .. } => Some(source),
            _ => None,
        }
    }
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
