//! Example demonstrating the plugin system for custom LaTeX commands

use latex_rust::{
    LaTeXProcessor, CommandPlugin, CustomFormatPlugin, FormatType, 
    Node, LaTeXError, PluginError
};

/// Custom plugin for handling \highlight{} commands
struct HighlightPlugin;

impl CommandPlugin for HighlightPlugin {
    fn command_name(&self) -> &str {
        "highlight"
    }
    
    fn process_command(&self, args: Vec<String>) -> Result<Node, LaTeXError> {
        if args.is_empty() {
            return Err(LaTeXError::ParserError {
                position: latex_rust::Position::start(),
                message: "highlight command requires text argument".to_string(),
            });
        }
        
        let content = args.join(" ");
        Ok(Node::Text(format!(
            "<mark class=\"highlight\">{}</mark>", 
            content
        )))
    }
    
    fn help_text(&self) -> Option<&str> {
        Some("Highlights text with a yellow background")
    }
}

/// Custom plugin for handling \code{} commands
struct CodePlugin;

impl CommandPlugin for CodePlugin {
    fn command_name(&self) -> &str {
        "code"
    }
    
    fn process_command(&self, args: Vec<String>) -> Result<Node, LaTeXError> {
        if args.is_empty() {
            return Err(LaTeXError::ParserError {
                position: latex_rust::Position::start(),
                message: "code command requires code text".to_string(),
            });
        }
        
        let content = args.join(" ");
        Ok(Node::Text(format!(
            "<code class=\"inline-code\">{}</code>", 
            content
        )))
    }
    
    fn validate_args(&self, args: &[String]) -> Result<(), LaTeXError> {
        if args.is_empty() {
            return Err(LaTeXError::ParserError {
                position: latex_rust::Position::start(),
                message: "code command requires at least one argument".to_string(),
            });
        }
        
        // Additional validation: check for potentially dangerous code
        let content = args.join(" ");
        if content.contains("<script>") {
            return Err(LaTeXError::ParserError {
                position: latex_rust::Position::start(),
                message: "script tags are not allowed in code blocks".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn help_text(&self) -> Option<&str> {
        Some("Formats text as inline code with monospace font")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("LaTeX-Rust Plugin System Example");
    println!("=================================\n");
    
    // Create a new LaTeX processor
    let mut processor = LaTeXProcessor::new();
    
    // Register built-in plugins
    println!("Registering plugins...");
    
    // Register a custom highlight plugin
    let highlight_plugin = Box::new(HighlightPlugin);
    processor.register_plugin(highlight_plugin)?;
    println!("✓ Registered highlight plugin");
    
    // Register a custom code plugin
    let code_plugin = Box::new(CodePlugin);
    processor.register_plugin(code_plugin)?;
    println!("✓ Registered code plugin");
    
    // Register a built-in custom format plugin for warnings
    let warning_plugin = Box::new(CustomFormatPlugin::new(
        "warning".to_string(),
        FormatType::Custom("warning-text".to_string())
    ));
    processor.register_plugin(warning_plugin)?;
    println!("✓ Registered warning plugin");
    
    // Show registered commands
    println!("\nRegistered plugin commands:");
    for command in processor.registered_commands() {
        if let Some(help) = processor.get_plugin_help(command) {
            println!("  \\{}: {}", command, help);
        } else {
            println!("  \\{}: (no help available)", command);
        }
    }
    
    // Test plugin functionality
    println!("\nTesting plugin functionality:");
    
    // Test if plugins are registered
    assert!(processor.has_plugin("highlight"));
    assert!(processor.has_plugin("code"));
    assert!(processor.has_plugin("warning"));
    println!("✓ All plugins are properly registered");
    
    // Test plugin processing (this would normally be called internally by the parser)
    println!("\nTesting plugin command processing:");
    
    // Test highlight plugin
    match processor.process_plugin_command("highlight", vec!["important".to_string(), "text".to_string()]) {
        Ok(node) => {
            if let Node::Text(html) = node {
                println!("✓ Highlight plugin output: {}", html);
            }
        }
        Err(e) => println!("✗ Highlight plugin error: {}", e),
    }
    
    // Test code plugin
    match processor.process_plugin_command("code", vec!["println!(\"Hello\");".to_string()]) {
        Ok(node) => {
            if let Node::Text(html) = node {
                println!("✓ Code plugin output: {}", html);
            }
        }
        Err(e) => println!("✗ Code plugin error: {}", e),
    }
    
    // Test validation (should fail)
    match processor.process_plugin_command("code", vec!["<script>alert('xss')</script>".to_string()]) {
        Ok(_) => println!("✗ Code plugin should have rejected script tags"),
        Err(e) => println!("✓ Code plugin correctly rejected dangerous input: {}", e),
    }
    
    // Test warning plugin
    match processor.process_plugin_command("warning", vec!["This".to_string(), "is".to_string(), "a".to_string(), "warning".to_string()]) {
        Ok(node) => {
            if let Node::Text(html) = node {
                println!("✓ Warning plugin output: {}", html);
            }
        }
        Err(e) => println!("✗ Warning plugin error: {}", e),
    }
    
    // Test unregistering a plugin
    println!("\nTesting plugin unregistration:");
    if let Some(_) = processor.unregister_plugin("warning") {
        println!("✓ Successfully unregistered warning plugin");
        assert!(!processor.has_plugin("warning"));
        println!("✓ Warning plugin is no longer registered");
    }
    
    // Test duplicate registration (should fail)
    println!("\nTesting duplicate registration:");
    let duplicate_plugin = Box::new(HighlightPlugin);
    match processor.register_plugin(duplicate_plugin) {
        Ok(_) => println!("✗ Should not allow duplicate registration"),
        Err(e) => println!("✓ Correctly rejected duplicate registration: {}", e),
    }
    
    println!("\n=================================\n");
    println!("Plugin system example completed successfully!");
    println!("\nThe plugin system allows you to:");
    println!("• Register custom command handlers");
    println!("• Validate command arguments");
    println!("• Process commands with custom logic");
    println!("• Provide help text for commands");
    println!("• Unregister plugins when no longer needed");
    println!("• Prevent duplicate command registration");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_highlight_plugin() {
        let plugin = HighlightPlugin;
        let result = plugin.process_command(vec!["test".to_string()]);
        assert!(result.is_ok());
        
        if let Ok(Node::Text(html)) = result {
            assert!(html.contains("highlight"));
            assert!(html.contains("test"));
        }
    }
    
    #[test]
    fn test_code_plugin_validation() {
        let plugin = CodePlugin;
        
        // Valid input should pass
        let result = plugin.validate_args(&["println!(\"hello\")".to_string()]);
        assert!(result.is_ok());
        
        // Script tags should be rejected
        let result = plugin.validate_args(&["<script>alert('xss')</script>".to_string()]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_plugin_integration() {
        let mut processor = LaTeXProcessor::new();
        
        // Register plugins
        let highlight_plugin = Box::new(HighlightPlugin);
        assert!(processor.register_plugin(highlight_plugin).is_ok());
        
        let code_plugin = Box::new(CodePlugin);
        assert!(processor.register_plugin(code_plugin).is_ok());
        
        // Test plugin functionality
        assert!(processor.has_plugin("highlight"));
        assert!(processor.has_plugin("code"));
        assert_eq!(processor.registered_commands().len(), 2);
        
        // Test command processing
        let result = processor.process_plugin_command("highlight", vec!["test".to_string()]);
        assert!(result.is_ok());
    }
}