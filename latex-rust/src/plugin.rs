//! Plugin system for extensible LaTeX command handling
//!
//! This module provides the foundation for a plugin architecture that allows
//! custom command handlers to be registered and executed during LaTeX processing.

use crate::ast::Node;
use crate::error::LaTeXError;
use crate::common::HasKey;
use std::collections::HashMap;
use std::fmt;

/// Trait for implementing custom LaTeX command handlers
pub trait CommandPlugin: Send + Sync {
    /// Returns the name of the command this plugin handles (without backslash)
    fn command_name(&self) -> &str;
    
    /// Process the command with given arguments and return the resulting AST node
    fn process_command(&self, args: Vec<String>) -> Result<Node, LaTeXError>;
    
    /// Optional: Validate command arguments before processing
    fn validate_args(&self, args: &[String]) -> Result<(), LaTeXError> {
        // Default implementation accepts any arguments
        Ok(())
    }
    
    /// Optional: Get help text for this command
    fn help_text(&self) -> Option<&str> {
        None
    }
}

/// Plugin registry for managing command plugins
#[derive(Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn CommandPlugin>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a new command plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn CommandPlugin>) -> Result<(), PluginError> {
        let command_name = plugin.command_name().to_string();
        
        if self.plugins.contains_key(&command_name) {
            return Err(PluginError::DuplicateCommand(command_name));
        }
        
        self.plugins.insert(command_name, plugin);
        Ok(())
    }
    
    /// Unregister a command plugin
    pub fn unregister_plugin(&mut self, command_name: &str) -> Option<Box<dyn CommandPlugin>> {
        self.plugins.remove(command_name)
    }
    
    /// Check if a command is handled by a plugin
    pub fn has_plugin(&self, command_name: &str) -> bool {
        self.plugins.has_key(command_name)
    }
    
    /// Process a command using the appropriate plugin
    pub fn process_command(&self, command_name: &str, args: Vec<String>) -> Result<Node, PluginError> {
        let plugin = self.plugins.get(command_name)
            .ok_or_else(|| PluginError::CommandNotFound(command_name.to_string()))?;
        
        // Validate arguments first
        plugin.validate_args(&args)
            .map_err(PluginError::ValidationError)?;
        
        // Process the command
        plugin.process_command(args)
            .map_err(PluginError::ProcessingError)
    }
    
    /// Get list of registered command names
    pub fn registered_commands(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get help text for a specific command
    pub fn get_help(&self, command_name: &str) -> Option<&str> {
        self.plugins.get(command_name)
            .and_then(|plugin| plugin.help_text())
    }
}

/// Errors that can occur during plugin operations
#[derive(Debug)]
pub enum PluginError {
    /// Command already registered
    DuplicateCommand(String),
    /// Command not found in registry
    CommandNotFound(String),
    /// Argument validation failed
    ValidationError(LaTeXError),
    /// Command processing failed
    ProcessingError(LaTeXError),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::DuplicateCommand(cmd) => {
                write!(f, "Command '{cmd}' is already registered")
            }
            PluginError::CommandNotFound(cmd) => {
                write!(f, "No plugin found for command '{cmd}'")
            }
            PluginError::ValidationError(err) => {
                write!(f, "Plugin validation error: {err}")
            }
            PluginError::ProcessingError(err) => {
                write!(f, "Plugin processing error: {err}")
            }
        }
    }
}

impl std::error::Error for PluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PluginError::ValidationError(err) | PluginError::ProcessingError(err) => Some(err),
            _ => None,
        }
    }
}

/// Built-in plugin for custom text formatting
pub struct CustomFormatPlugin {
    command: String,
    format_type: FormatType,
}

#[derive(Debug, Clone)]
pub enum FormatType {
    Bold,
    Italic,
    Underline,
    Custom(String), // Custom CSS class
}

impl CustomFormatPlugin {
    pub fn new(command: String, format_type: FormatType) -> Self {
        Self { command, format_type }
    }
}

impl CommandPlugin for CustomFormatPlugin {
    fn command_name(&self) -> &str {
        &self.command
    }
    
    fn process_command(&self, args: Vec<String>) -> Result<Node, LaTeXError> {
        if args.is_empty() {
            return Err(LaTeXError::ParserError {
                position: crate::error::Position::start(),
                message: format!("Command '{}' requires at least one argument", self.command),
            });
        }
        
        let content = args.join(" ");
        
        match &self.format_type {
            FormatType::Bold => {
                Ok(Node::Command {
                    name: "textbf".to_string(),
                    args: vec![crate::ast::Argument::Required(vec![Node::Text(content)])],
                    content: None,
                })
            }
            FormatType::Italic => {
                Ok(Node::Command {
                    name: "textit".to_string(),
                    args: vec![crate::ast::Argument::Required(vec![Node::Text(content)])],
                    content: None,
                })
            }
            FormatType::Underline => {
                Ok(Node::Command {
                    name: "underline".to_string(),
                    args: vec![crate::ast::Argument::Required(vec![Node::Text(content)])],
                    content: None,
                })
            }
            FormatType::Custom(class) => {
                // Create a custom formatted node
                Ok(Node::Text(format!("<span class=\"{class}\">{content}</span>")))
            }
        }
    }
    
    fn validate_args(&self, args: &[String]) -> Result<(), LaTeXError> {
        if args.is_empty() {
            return Err(LaTeXError::ParserError {
                position: crate::error::Position::start(),
                message: format!("Command '{}' requires at least one argument", self.command),
            });
        }
        Ok(())
    }
    
    fn help_text(&self) -> Option<&str> {
        Some("Custom formatting command that applies specified formatting to text")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.registered_commands().len(), 0);
    }
    
    #[test]
    fn test_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(CustomFormatPlugin::new(
            "highlight".to_string(),
            FormatType::Custom("highlight".to_string())
        ));
        
        assert!(registry.register_plugin(plugin).is_ok());
        assert!(registry.has_plugin("highlight"));
        assert_eq!(registry.registered_commands().len(), 1);
    }
    
    #[test]
    fn test_duplicate_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin1 = Box::new(CustomFormatPlugin::new(
            "test".to_string(),
            FormatType::Bold
        ));
        let plugin2 = Box::new(CustomFormatPlugin::new(
            "test".to_string(),
            FormatType::Italic
        ));
        
        assert!(registry.register_plugin(plugin1).is_ok());
        assert!(matches!(registry.register_plugin(plugin2), Err(PluginError::DuplicateCommand(_))));
    }
    
    #[test]
    fn test_plugin_command_processing() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(CustomFormatPlugin::new(
            "bold".to_string(),
            FormatType::Bold
        ));
        
        registry.register_plugin(plugin).unwrap();
        
        let result = registry.process_command("bold", vec!["test".to_string()]);
        assert!(result.is_ok());
        
        if let Ok(Node::Command { name, args, .. }) = result {
            assert_eq!(name, "textbf");
            if let Some(crate::ast::Argument::Required(nodes)) = args.first() {
                if let Some(Node::Text(content)) = nodes.first() {
                    assert_eq!(content, "test");
                } else {
                    panic!("Expected text content in bold command");
                }
            } else {
                panic!("Expected required argument in bold command");
            }
        } else {
            panic!("Expected Command node");
        }
    }
    
    #[test]
    fn test_plugin_unregistration() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(CustomFormatPlugin::new(
            "test".to_string(),
            FormatType::Bold
        ));
        
        registry.register_plugin(plugin).unwrap();
        assert!(registry.has_plugin("test"));
        
        let removed = registry.unregister_plugin("test");
        assert!(removed.is_some());
        assert!(!registry.has_plugin("test"));
    }
}