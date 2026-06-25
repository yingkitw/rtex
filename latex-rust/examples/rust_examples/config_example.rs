//! Example demonstrating the configuration system for LaTeX processing

use latex_rust::{
    LaTeXProcessor, LaTeXConfig, ConfigError
};
use latex_rust::config::{ParserConfig, RendererConfig, StreamingConfig, PluginConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("LaTeX-Rust Configuration System Example");
    println!("======================================\n");
    
    // Example 1: Default configuration
    println!("1. Default Configuration:");
    let default_config = LaTeXConfig::default();
    println!("   - Strict mode: {}", default_config.parser.strict_mode);
    println!("   - Preserve whitespace: {}", default_config.parser.preserve_whitespace);
    println!("   - Include CSS classes: {}", default_config.renderer.include_css_classes);
    println!("   - Enable plugins: {}", default_config.plugins.enable_plugins);
    
    let processor = LaTeXProcessor::new();
    println!("   ✓ Created processor with default config\n");
    
    // Example 2: Performance-optimized configuration
    println!("2. Performance-Optimized Configuration:");
    let perf_config = LaTeXConfig::performance_optimized();
    println!("   - Preserve whitespace: {}", perf_config.parser.preserve_whitespace);
    println!("   - Include CSS classes: {}", perf_config.renderer.include_css_classes);
    println!("   - Enable streaming: {}", perf_config.streaming.enable_streaming);
    println!("   - Parallel processing: {}", perf_config.streaming.parallel_processing);
    println!("   - Buffer size: {} bytes", perf_config.streaming.buffer_size);
    
    let _perf_processor = LaTeXProcessor::with_config(perf_config)?;
    println!("   ✓ Created processor with performance config\n");
    
    // Example 3: Strict mode configuration
    println!("3. Strict Mode Configuration:");
    let strict_config = LaTeXConfig::strict_mode();
    println!("   - Strict mode: {}", strict_config.parser.strict_mode);
    println!("   - Preserve whitespace: {}", strict_config.parser.preserve_whitespace);
    println!("   - Escape HTML: {}", strict_config.renderer.escape_html);
    println!("   - Allow unsafe plugins: {}", strict_config.plugins.allow_unsafe_plugins);
    
    let _strict_processor = LaTeXProcessor::with_config(strict_config)?;
    println!("   ✓ Created processor with strict config\n");
    
    // Example 4: Custom configuration
    println!("4. Custom Configuration:");
    let mut custom_config = LaTeXConfig::default();
    
    // Customize parser settings
    custom_config.parser.strict_mode = true;
    custom_config.parser.set_max_nesting_depth(50).unwrap();
    custom_config.parser.enable_math_mode = true;
    
    // Add custom commands
    custom_config.parser.add_custom_command(
        "mycommand".to_string(),
        "<span class=\"custom\">{}</span>".to_string()
    );
    
    // Customize renderer settings
    custom_config.renderer.semantic_html = true;
    custom_config.renderer.html.indent_output = true;
    custom_config.renderer.html.indent_string = "    ".to_string(); // 4 spaces
    
    // Add custom CSS mappings
    custom_config.renderer.add_css_class_mapping(
        "important".to_string(),
        "text-red-500 font-bold".to_string()
    );
    
    // Customize streaming settings
    custom_config.streaming.buffer_size = 4096; // 4KB
    custom_config.streaming.chunk_size = 512;   // 512 bytes
    
    // Customize plugin settings
    custom_config.plugins.auto_load_plugins = vec![
        "highlight".to_string(),
        "code".to_string()
    ];
    
    println!("   - Max nesting depth: {}", custom_config.parser.max_nesting_depth());
    println!("   - Custom commands: {}", custom_config.parser.custom_commands().len());
    println!("   - Indent output: {}", custom_config.renderer.html.indent_output);
    println!("   - Buffer size: {} bytes", custom_config.streaming.buffer_size);
    println!("   - Auto-load plugins: {:?}", custom_config.plugins.auto_load_plugins);
    
    let _custom_processor = LaTeXProcessor::builder()
        .with_config(custom_config.clone())
        .build()?;
    println!("   ✓ Created processor with custom config\n");
    
    // Example 5: Configuration validation
    println!("5. Configuration Validation:");
    
    // Valid configuration
    let valid_config = LaTeXConfig::default();
    match valid_config.validate() {
        Ok(_) => println!("   ✓ Default configuration is valid"),
        Err(e) => println!("   ✗ Default configuration is invalid: {}", e),
    }
    
    // Invalid configuration - zero nesting depth
    let mut invalid_config = LaTeXConfig::default();
    invalid_config.parser.set_max_nesting_depth(0).unwrap_or_else(|_| {
        println!("   ✓ Correctly rejected zero nesting depth during setter");
    });
    match invalid_config.validate() {
        Ok(_) => println!("   ✗ Should have rejected zero nesting depth"),
        Err(e) => println!("   ✓ Correctly rejected invalid config: {}", e),
    }
    
    // Invalid configuration - zero buffer size
    let mut invalid_config2 = LaTeXConfig::default();
    invalid_config2.streaming.buffer_size = 0;
    match invalid_config2.validate() {
        Ok(_) => println!("   ✗ Should have rejected zero buffer size"),
        Err(e) => println!("   ✓ Correctly rejected invalid config: {}", e),
    }
    
    // Invalid configuration - chunk size larger than buffer
    let mut invalid_config3 = LaTeXConfig::default();
    invalid_config3.streaming.buffer_size = 1024;
    invalid_config3.streaming.chunk_size = 2048;
    match invalid_config3.validate() {
        Ok(_) => println!("   ✗ Should have rejected chunk size > buffer size"),
        Err(e) => println!("   ✓ Correctly rejected invalid config: {}", e),
    }
    
    println!();
    
    // Example 6: Configuration serialization
    println!("6. Configuration Serialization:");
    
    // TOML serialization
    let toml_string = custom_config.to_toml()?;
    println!("   ✓ Serialized config to TOML ({} chars)", toml_string.len());
    
    let parsed_toml_config = LaTeXConfig::from_toml(&toml_string)?;
    println!("   ✓ Parsed config from TOML");
    
    // JSON serialization
    let json_string = custom_config.to_json()?;
    println!("   ✓ Serialized config to JSON ({} chars)", json_string.len());
    
    let parsed_json_config = LaTeXConfig::from_json(&json_string)?;
    println!("   ✓ Parsed config from JSON\n");
    
    // Example 7: Configuration file operations
    println!("7. Configuration File Operations:");
    
    // Save to TOML file
    let toml_path = "/tmp/latex_config.toml";
    custom_config.to_toml_file(toml_path)?;
    println!("   ✓ Saved config to TOML file: {}", toml_path);
    
    // Load from TOML file
    let loaded_toml_config = LaTeXConfig::from_toml_file(toml_path)?;
    println!("   ✓ Loaded config from TOML file");
    
    // Save to JSON file
    let json_path = "/tmp/latex_config.json";
    custom_config.to_json_file(json_path)?;
    println!("   ✓ Saved config to JSON file: {}", json_path);
    
    // Load from JSON file
    let loaded_json_config = LaTeXConfig::from_json_file(json_path)?;
    println!("   ✓ Loaded config from JSON file\n");
    
    // Example 8: Runtime configuration updates
    println!("8. Runtime Configuration Updates:");
    
    let mut processor = LaTeXProcessor::new();
    println!("   - Initial strict mode: {}", processor.config().parser.strict_mode);
    
    // Update configuration at runtime
    let mut new_config = processor.config().clone();
    new_config.parser.strict_mode = true;
    new_config.renderer.include_css_classes = false;
    
    processor.update_config(new_config)?;
    println!("   - Updated strict mode: {}", processor.config().parser.strict_mode);
    println!("   - Updated CSS classes: {}", processor.config().renderer.include_css_classes);
    println!("   ✓ Successfully updated processor configuration\n");
    
    // Clean up temporary files
    let _ = std::fs::remove_file(toml_path);
    let _ = std::fs::remove_file(json_path);
    
    println!("======================================\n");
    println!("Configuration system example completed successfully!");
    println!("\nThe configuration system provides:");
    println!("• Comprehensive settings for all components");
    println!("• Pre-defined configuration presets");
    println!("• Configuration validation");
    println!("• TOML and JSON serialization support");
    println!("• File-based configuration loading/saving");
    println!("• Runtime configuration updates");
    println!("• Custom command and CSS class mappings");
    println!("• Streaming and plugin configuration");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_presets() {
        let default_config = LaTeXConfig::default();
        let perf_config = LaTeXConfig::performance_optimized();
        let strict_config = LaTeXConfig::strict_mode();
        let minimal_config = LaTeXConfig::minimal_html();
        
        // Test that presets have different settings
        assert_ne!(default_config.parser.preserve_whitespace, perf_config.parser.preserve_whitespace);
        assert_ne!(default_config.parser.strict_mode, strict_config.parser.strict_mode);
        assert_ne!(default_config.renderer.include_css_classes, minimal_config.renderer.include_css_classes);
    }
    
    #[test]
    fn test_processor_with_config() {
        let config = LaTeXConfig::performance_optimized();
        let processor = LaTeXProcessor::with_config(config.clone()).unwrap();
        
        assert_eq!(processor.config().streaming.enable_streaming, config.streaming.enable_streaming);
        assert_eq!(processor.config().streaming.parallel_processing, config.streaming.parallel_processing);
    }
    
    #[test]
    fn test_config_updates() {
        let mut processor = LaTeXProcessor::new();
        let initial_strict = processor.config().parser.strict_mode;
        
        let mut new_config = processor.config().clone();
        new_config.parser.strict_mode = !initial_strict;
        
        processor.set_config(new_config).unwrap();
        assert_eq!(processor.config().parser.strict_mode, !initial_strict);
    }
    
    #[test]
    fn test_config_serialization_roundtrip() {
        let original_config = LaTeXConfig::performance_optimized();
        
        // TOML roundtrip
        let toml_str = original_config.to_toml().unwrap();
        let toml_config = LaTeXConfig::from_toml(&toml_str).unwrap();
        assert_eq!(original_config.parser.strict_mode, toml_config.parser.strict_mode);
        
        // JSON roundtrip
        let json_str = original_config.to_json().unwrap();
        let json_config = LaTeXConfig::from_json(&json_str).unwrap();
        assert_eq!(original_config.renderer.include_css_classes, json_config.renderer.include_css_classes);
    }
}