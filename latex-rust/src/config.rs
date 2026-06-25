//! Configuration system for LaTeX parsing and rendering behavior

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::common::{impl_default_config, impl_validation, Validate, is_positive_usize, is_valid_buffer_size};

/// Configuration for the LaTeX processor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct LaTeXConfig {
    /// Parser configuration
    pub parser: ParserConfig,
    /// Renderer configuration
    pub renderer: RendererConfig,
    /// Streaming configuration
    pub streaming: StreamingConfig,
    /// Plugin configuration
    pub plugins: PluginConfig,
}

/// Configuration for the parser with encapsulated fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Whether to enable strict mode (fail on unknown commands)
    pub strict_mode: bool,
    /// Whether to preserve whitespace exactly as in input
    pub preserve_whitespace: bool,
    /// Whether to enable cross-reference resolution
    pub enable_cross_references: bool,
    /// Maximum nesting depth for environments and commands
    max_nesting_depth: usize,
    /// Whether to enable math mode parsing
    pub enable_math_mode: bool,
    /// Custom command definitions
    custom_commands: HashMap<String, String>,
}

impl ParserConfig {
    /// Get the maximum nesting depth
    pub fn max_nesting_depth(&self) -> usize {
        self.max_nesting_depth
    }
    
    /// Set the maximum nesting depth (validates the value)
    pub fn set_max_nesting_depth(&mut self, depth: usize) -> Result<(), ConfigError> {
        if depth == 0 {
            return Err(ConfigError::ValidationError("max_nesting_depth must be greater than 0".to_string()));
        }
        self.max_nesting_depth = depth;
        Ok(())
    }
    
    /// Get a reference to custom commands
    pub fn custom_commands(&self) -> &HashMap<String, String> {
        &self.custom_commands
    }
    
    /// Add a custom command definition
    pub fn add_custom_command(&mut self, name: String, definition: String) {
        self.custom_commands.insert(name, definition);
    }
    
    /// Remove a custom command definition
    pub fn remove_custom_command(&mut self, name: &str) -> Option<String> {
        self.custom_commands.remove(name)
    }
    
    /// Check if a custom command exists
    pub fn has_custom_command(&self, name: &str) -> bool {
        self.custom_commands.contains_key(name)
    }
    
    /// Clear all custom commands
    pub fn clear_custom_commands(&mut self) {
        self.custom_commands.clear();
    }
}

/// Configuration for the renderer with encapsulated fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    /// HTML output format settings
    pub html: HtmlConfig,
    /// Whether to include CSS classes in output
    pub include_css_classes: bool,
    /// Custom CSS class mappings
    css_class_mappings: HashMap<String, String>,
    /// Whether to generate semantic HTML5 elements
    pub semantic_html: bool,
    /// Whether to escape HTML in text content
    pub escape_html: bool,
}

impl RendererConfig {
    /// Get a reference to CSS class mappings
    pub fn css_class_mappings(&self) -> &HashMap<String, String> {
        &self.css_class_mappings
    }
    
    /// Add a CSS class mapping
    pub fn add_css_class_mapping(&mut self, element: String, css_class: String) {
        self.css_class_mappings.insert(element, css_class);
    }
    
    /// Remove a CSS class mapping
    pub fn remove_css_class_mapping(&mut self, element: &str) -> Option<String> {
        self.css_class_mappings.remove(element)
    }
    
    /// Get CSS class for an element
    pub fn get_css_class(&self, element: &str) -> Option<&String> {
        self.css_class_mappings.get(element)
    }
    
    /// Check if CSS class mapping exists for an element
    pub fn has_css_class_mapping(&self, element: &str) -> bool {
        self.css_class_mappings.contains_key(element)
    }
    
    /// Clear all CSS class mappings
    pub fn clear_css_class_mappings(&mut self) {
        self.css_class_mappings.clear();
    }
    
    /// Set default CSS class mappings
    pub fn set_default_css_mappings(&mut self) {
        self.css_class_mappings = Self::default_css_mappings();
    }
    
    /// Create default renderer configuration with CSS mappings
    pub fn with_default_css_mappings() -> Self {
        Self {
            css_class_mappings: Self::default_css_mappings(),
            ..Default::default()
        }
    }
    
    pub(crate) fn default_css_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        mappings.insert("section".to_string(), "latex-section".to_string());
        mappings.insert("subsection".to_string(), "latex-subsection".to_string());
        mappings.insert("paragraph".to_string(), "latex-paragraph".to_string());
        mappings.insert("equation".to_string(), "latex-equation".to_string());
        mappings.insert("table".to_string(), "latex-table".to_string());
        mappings.insert("figure".to_string(), "latex-figure".to_string());
        mappings
    }
}

/// HTML-specific rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlConfig {
    /// Whether to use self-closing tags for empty elements
    pub self_closing_tags: bool,
    /// Whether to indent nested elements
    pub indent_output: bool,
    /// Indentation string (e.g., "  " or "\t")
    pub indent_string: String,
    /// Whether to include HTML5 doctype and structure
    pub full_document: bool,
    /// Custom HTML attributes for elements
    pub custom_attributes: HashMap<String, HashMap<String, String>>,
}

/// Configuration for streaming parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Buffer size for streaming operations (in bytes)
    pub buffer_size: usize,
    /// Whether to enable streaming mode by default
    pub enable_streaming: bool,
    /// Chunk size for processing large documents
    pub chunk_size: usize,
    /// Whether to enable parallel processing of chunks
    pub parallel_processing: bool,
}

/// Configuration for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether to enable plugin system
    pub enable_plugins: bool,
    /// List of plugin names to auto-load
    pub auto_load_plugins: Vec<String>,
    /// Plugin-specific configurations
    pub plugin_settings: HashMap<String, serde_json::Value>,
    /// Whether to allow unsafe plugins
    pub allow_unsafe_plugins: bool,
}


impl_default_config!(ParserConfig, {
    strict_mode: false,
    preserve_whitespace: true,
    enable_cross_references: true,
    max_nesting_depth: 100,
    enable_math_mode: true,
    custom_commands: HashMap::new(),
});

impl_validation!(ParserConfig, ConfigError, {
    max_nesting_depth: is_positive_usize => "max_nesting_depth must be greater than 0",
});

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            html: HtmlConfig::default(),
            include_css_classes: true,
            css_class_mappings: HashMap::new(),
            semantic_html: true,
            escape_html: true,
        }
    }
}

impl_default_config!(HtmlConfig, {
    self_closing_tags: true,
    indent_output: false,
    indent_string: "  ".to_string(),
    full_document: false,
    custom_attributes: HashMap::new(),
});

impl_default_config!(StreamingConfig, {
    buffer_size: 8192, // 8KB
    enable_streaming: false,
    chunk_size: 1024,  // 1KB
    parallel_processing: false,
});

impl_validation!(StreamingConfig, ConfigError, {
    buffer_size: is_valid_buffer_size => "buffer_size must be between 1KB and 1MB",
    chunk_size: is_positive_usize => "chunk_size must be greater than 0",
});

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enable_plugins: true,
            auto_load_plugins: vec![],
            plugin_settings: HashMap::new(),
            allow_unsafe_plugins: false,
        }
    }
}

impl LaTeXConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration optimized for performance
    pub fn performance_optimized() -> Self {
        let mut config = Self::default();
        config.parser.preserve_whitespace = false;
        config.renderer.include_css_classes = false;
        config.renderer.html.indent_output = false;
        config.streaming.enable_streaming = true;
        config.streaming.parallel_processing = true;
        config.streaming.buffer_size = 16384; // 16KB
        config
    }
    
    /// Create a configuration optimized for strict LaTeX compliance
    pub fn strict_mode() -> Self {
        let mut config = Self::default();
        config.parser.strict_mode = true;
        config.parser.preserve_whitespace = true;
        config.renderer.escape_html = true;
        config.plugins.allow_unsafe_plugins = false;
        config
    }
    
    /// Create a configuration for minimal HTML output
    pub fn minimal_html() -> Self {
        let mut config = Self::default();
        config.renderer.include_css_classes = false;
        config.renderer.semantic_html = false;
        config.renderer.html.indent_output = false;
        config.renderer.html.full_document = false;
        config
    }
    
    /// Load configuration from a TOML file
    pub fn from_toml_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_toml(&content)
    }
    
    /// Load configuration from TOML string
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
    
    /// Save configuration to a TOML file
    pub fn to_toml_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = self.to_toml()?;
        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }
    
    /// Convert configuration to TOML string
    pub fn to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }
    
    /// Load configuration from a JSON file
    pub fn from_json_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::from_json(&content)
    }
    
    /// Load configuration from JSON string
    pub fn from_json(content: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
    
    /// Save configuration to a JSON file
    pub fn to_json_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = self.to_json()?;
        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }
    
    /// Convert configuration to JSON string
    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }
    
    /// Validate the configuration for consistency
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.parser.validate()?;
        self.streaming.validate()?;
        Ok(())
    }
    
    /// Merge this configuration with another, with the other taking precedence
    pub fn merge(&mut self, other: &LaTeXConfig) {
        // Note: This is a simple merge - in a real implementation,
        // you might want more sophisticated merging logic
        *self = other.clone();
    }
}

/// Configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = LaTeXConfig::default();
        assert!(!config.parser.strict_mode);
        assert!(config.parser.preserve_whitespace);
        assert!(config.renderer.include_css_classes);
        assert!(config.plugins.enable_plugins);
    }
    
    #[test]
    fn test_performance_optimized_config() {
        let config = LaTeXConfig::performance_optimized();
        assert!(!config.parser.preserve_whitespace);
        assert!(!config.renderer.include_css_classes);
        assert!(config.streaming.enable_streaming);
        assert!(config.streaming.parallel_processing);
    }
    
    #[test]
    fn test_strict_mode_config() {
        let config = LaTeXConfig::strict_mode();
        assert!(config.parser.strict_mode);
        assert!(config.parser.preserve_whitespace);
        assert!(!config.plugins.allow_unsafe_plugins);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = LaTeXConfig::default();
        assert!(config.validate().is_ok());
        
        config.parser.max_nesting_depth = 0;
        assert!(config.validate().is_err());
        
        config.parser.max_nesting_depth = 100;
        config.streaming.buffer_size = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_toml_serialization() {
        let config = LaTeXConfig::default();
        let toml_str = config.to_toml().unwrap();
        let parsed_config = LaTeXConfig::from_toml(&toml_str).unwrap();
        
        // Compare key fields
        assert_eq!(config.parser.strict_mode, parsed_config.parser.strict_mode);
        assert_eq!(config.renderer.include_css_classes, parsed_config.renderer.include_css_classes);
    }
    
    #[test]
    fn test_json_serialization() {
        let config = LaTeXConfig::default();
        let json_str = config.to_json().unwrap();
        let parsed_config = LaTeXConfig::from_json(&json_str).unwrap();
        
        // Compare key fields
        assert_eq!(config.parser.strict_mode, parsed_config.parser.strict_mode);
        assert_eq!(config.renderer.include_css_classes, parsed_config.renderer.include_css_classes);
    }
}