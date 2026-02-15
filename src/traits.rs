/// Core traits for latex-rs
/// 
/// Defines atomic, composable traits for parsing, formatting, and PDF generation
/// Following the trait-based architecture pattern from minitex

use crate::error::Result;
use crate::parser::TexElement;
use std::path::Path;

/// Trait for parsing LaTeX content
pub trait TexParser: Send + Sync {
    /// Parse LaTeX content into structured elements
    fn parse(&self, content: &str) -> Result<Vec<TexElement>>;
    
    /// Parse a single command
    fn parse_command(&self, content: &str, position: usize) -> Result<Option<TexElement>>;
}

/// Trait for formatting mathematical expressions
pub trait MathFormatter: Send + Sync {
    /// Format a mathematical expression from LaTeX to display format
    fn format(&self, math: &str) -> String;
    
    /// Format inline math (within text)
    fn format_inline(&self, math: &str) -> String {
        self.format(math)
    }
    
    /// Format display math (standalone)
    fn format_display(&self, math: &str) -> String {
        self.format(math)
    }
}

/// Trait for building PDF documents
pub trait PdfBuilder: Send + Sync {
    /// Build a PDF from parsed elements
    fn build(&mut self, elements: Vec<TexElement>, output_path: &Path) -> Result<()>;
    
    /// Set document title
    fn set_title(&mut self, title: String);
    
    /// Set document author
    fn set_author(&mut self, author: String);
    
    /// Set document date
    fn set_date(&mut self, date: String);
}

/// Trait for font management
pub trait FontProvider: Send + Sync {
    /// Load a font from a path
    fn load_font(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// Get the default font
    fn default_font(&self) -> Result<Vec<u8>>;
    
    /// Check if a font supports a character
    fn supports_character(&self, font_data: &[u8], ch: char) -> bool;
}

/// Trait for caching operations
pub trait Cache<K, V>: Send + Sync {
    /// Get a value from the cache
    fn get(&self, key: &K) -> Option<&V>;
    
    /// Insert a value into the cache
    fn insert(&mut self, key: K, value: V);
    
    /// Clear the cache
    fn clear(&mut self);
    
    /// Get cache size
    fn size(&self) -> usize;
}

/// Trait for text layout operations
pub trait TextLayout: Send + Sync {
    /// Wrap text to a specified width
    fn wrap_text(&self, text: &str, max_width: usize) -> Vec<String>;
    
    /// Calculate text width
    fn text_width(&self, text: &str) -> f32;
    
    /// Calculate line height
    fn line_height(&self, font_size: f32) -> f32;
}

/// Trait for validation operations
pub trait Validator<T>: Send + Sync {
    /// Validate an item
    fn validate(&self, item: &T) -> Result<()>;
    
    /// Check if an item is valid
    fn is_valid(&self, item: &T) -> bool {
        self.validate(item).is_ok()
    }
}

/// Trait for transformation operations
pub trait Transform<T>: Send + Sync {
    /// Transform an item
    fn transform(&self, item: T) -> Result<T>;
}

/// Trait for resource management
pub trait ResourceManager: Send + Sync {
    /// Initialize resources
    fn initialize(&mut self) -> Result<()>;
    
    /// Clean up resources
    fn cleanup(&mut self) -> Result<()>;
    
    /// Check if resources are initialized
    fn is_initialized(&self) -> bool;
}

/// Trait for configuration management
pub trait Configurable {
    /// Get configuration value
    fn get_config(&self, key: &str) -> Option<String>;
    
    /// Set configuration value
    fn set_config(&mut self, key: String, value: String);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example implementation for testing
    struct MockMathFormatter;
    
    impl MathFormatter for MockMathFormatter {
        fn format(&self, math: &str) -> String {
            math.replace("\\alpha", "α")
        }
    }

    #[test]
    fn test_math_formatter_trait() {
        let formatter = MockMathFormatter;
        assert_eq!(formatter.format("\\alpha"), "α");
        assert_eq!(formatter.format_inline("\\alpha"), "α");
    }
}
