//! Comprehensive unit tests for the config module

use latex_rust::config::*;
use std::collections::HashMap;

#[cfg(test)]
mod config_unit_tests {
    use super::*;

    #[test]
    fn test_latex_config_default() {
        let config = LaTeXConfig::default();
        assert!(!config.parser.strict_mode);
        assert!(config.parser.preserve_whitespace);
        assert_eq!(config.parser.max_nesting_depth(), 100);
        assert!(config.renderer.include_css_classes);
        assert!(!config.streaming.enable_streaming);
        assert!(config.plugins.enable_plugins);
    }

    #[test]
    fn test_latex_config_performance_optimized() {
        let config = LaTeXConfig::performance_optimized();
        assert!(!config.parser.preserve_whitespace);
        assert!(!config.renderer.include_css_classes);
        assert!(config.streaming.enable_streaming);
        assert!(config.streaming.parallel_processing);
        assert_eq!(config.streaming.buffer_size, 16384); // 16KB
    }

    #[test]
    fn test_latex_config_strict_mode() {
        let config = LaTeXConfig::strict_mode();
        assert!(config.parser.strict_mode);
        assert!(config.parser.preserve_whitespace);
        assert!(!config.plugins.allow_unsafe_plugins);
        assert_eq!(config.parser.max_nesting_depth(), 100); // Default value
        assert!(config.renderer.escape_html);
    }

    #[test]
    fn test_parser_config_validation() {
        let mut config = ParserConfig::default();
        
        // Test max_nesting_depth field exists
        config.set_max_nesting_depth(50).unwrap();
        assert_eq!(config.max_nesting_depth(), 50);
    }

    #[test]
    fn test_streaming_config_validation() {
        let mut config = StreamingConfig::default();
        
        // Test buffer_size field exists
        config.buffer_size = 8192; // 8KB
        assert_eq!(config.buffer_size, 8192);
    }

    #[test]
    fn test_html_config_custom_attributes() {
        let mut config = HtmlConfig::default();
        
        // Add custom attributes
        let mut div_attrs = HashMap::new();
        div_attrs.insert("class".to_string(), "custom-latex".to_string());
        div_attrs.insert("data-source".to_string(), "latex-rust".to_string());
        
        config.custom_attributes.insert("div".to_string(), div_attrs);
        
        assert_eq!(config.custom_attributes.len(), 1);
        assert!(config.custom_attributes.contains_key("div"));
        let div_attrs = config.custom_attributes.get("div").unwrap();
        assert_eq!(div_attrs.get("class"), Some(&"custom-latex".to_string()));
    }

    #[test]
    fn test_html_config_defaults() {
        let config = HtmlConfig::default();
        assert_eq!(config.self_closing_tags, true);
        assert_eq!(config.indent_output, false);
        assert_eq!(config.indent_string, "  ");
        assert_eq!(config.full_document, false);
        assert!(config.custom_attributes.is_empty());
    }

    #[test]
    fn test_parser_config_custom_commands() {
        let mut config = ParserConfig::default();
        
        // Add custom commands
        config.add_custom_command("mycommand".to_string(), "replacement".to_string());
        config.add_custom_command("highlight".to_string(), "<mark>{}</mark>".to_string());
        
        assert_eq!(config.custom_commands().len(), 2);
        assert!(config.custom_commands().contains_key("mycommand"));
    }

    #[test]
    fn test_plugin_config_auto_load() {
        let mut config = PluginConfig::default();
        
        // Test auto-load plugins
        config.auto_load_plugins = vec![
            "math".to_string(),
            "highlight".to_string(),
            "code".to_string(),
        ];
        
        assert_eq!(config.auto_load_plugins.len(), 3);
        assert!(config.auto_load_plugins.contains(&"math".to_string()));
    }

    #[test]
    fn test_plugin_config_defaults() {
        let config = PluginConfig::default();
        assert_eq!(config.enable_plugins, true); // Default is true
        assert!(config.auto_load_plugins.is_empty());
        assert!(config.plugin_settings.is_empty());
        assert_eq!(config.allow_unsafe_plugins, false);
    }

    #[test]
    fn test_config_serialization() {
        let config = LaTeXConfig::default();
        
        // Test JSON serialization
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());
        
        // Test deserialization
        let json_str = json.unwrap();
        let deserialized: Result<LaTeXConfig, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        
        let deserialized_config = deserialized.unwrap();
        assert_eq!(config.parser.strict_mode, deserialized_config.parser.strict_mode);
        assert_eq!(config.renderer.include_css_classes, deserialized_config.renderer.include_css_classes);
    }

    #[test]
    fn test_config_error_types() {
        // Test ConfigError variants
        let io_error = ConfigError::IoError("File not found".to_string());
        assert!(io_error.to_string().contains("IO error"));
        
        let parse_error = ConfigError::ParseError("Invalid syntax".to_string());
        assert!(parse_error.to_string().contains("Parse error"));
        
        let validation_error = ConfigError::ValidationError("Invalid value".to_string());
        assert!(validation_error.to_string().contains("Validation error"));
        
        let serialization_error = ConfigError::SerializationError("Cannot serialize".to_string());
        assert!(serialization_error.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_config_edge_cases() {
        // Test edge case values
        let mut config = LaTeXConfig::default();
        
        // Minimum valid values
        config.parser.set_max_nesting_depth(1).unwrap_or_else(|_| {});
        config.streaming.buffer_size = 1024; // 1KB
        config.streaming.chunk_size = 1;
        
        assert!(config.validate().is_ok());
        
        // Maximum valid values
        config.parser.set_max_nesting_depth(1000).unwrap();
        config.streaming.buffer_size = 1024 * 1024; // 1MB
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_cloning() {
        let config = LaTeXConfig::performance_optimized();
        let cloned_config = config.clone();
        
        assert_eq!(config.parser.strict_mode, cloned_config.parser.strict_mode);
        assert_eq!(config.streaming.enable_streaming, cloned_config.streaming.enable_streaming);
        assert_eq!(config.plugins.enable_plugins, cloned_config.plugins.enable_plugins);
    }

    #[test]
    fn test_config_debug_format() {
        let config = LaTeXConfig::default();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("LaTeXConfig"));
        assert!(debug_str.contains("parser"));
        assert!(debug_str.contains("renderer"));
        assert!(debug_str.contains("streaming"));
        assert!(debug_str.contains("plugins"));
    }
}