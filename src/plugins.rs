//! Plugin architecture for extending parser and renderer behaviour.
//!
//! Plugins can intercept unknown commands, unknown environments, and
//! transform the element tree before PDF generation.  This provides
//! extension points without modifying the core library.

use crate::parser::TexElement;

/// A plugin that extends the LaTeX-to-PDF pipeline.
///
/// All methods have default no-op implementations so a plugin only
/// needs to override the hooks it cares about.
pub trait Plugin: Send {
    /// Human-readable plugin name.
    fn name(&self) -> &str;

    /// Called once when the plugin is registered.
    fn on_load(&mut self) {}

    /// Intercept an unknown command.  Return `Some(element)` to handle
    /// the command, or `None` to let the next plugin (or the default
    /// unknown-command handler) process it.
    fn handle_command(&mut self, _name: &str, _args: &[String]) -> Option<TexElement> {
        None
    }

    /// Intercept an unknown environment.  Return `Some(element)` to
    /// handle the environment, or `None` to fall through.
    fn handle_environment(
        &mut self,
        _name: &str,
        _content: &str,
    ) -> Option<TexElement> {
        None
    }

    /// Transform the parsed element tree before PDF generation.
    fn transform_elements(&mut self, elements: Vec<TexElement>) -> Vec<TexElement> {
        elements
    }
}

/// Container that holds registered plugins and dispatches to them.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin.  It is moved into the registry.
    pub fn register(&mut self, mut plugin: Box<dyn Plugin>) {
        plugin.on_load();
        self.plugins.push(plugin);
    }

    /// Try each plugin in registration order for `handle_command`.
    /// Returns the first non-`None` result.
    pub fn try_command(&mut self, name: &str, args: &[String]) -> Option<TexElement> {
        for p in &mut self.plugins {
            if let Some(elem) = p.handle_command(name, args) {
                return Some(elem);
            }
        }
        None
    }

    /// Try each plugin in registration order for `handle_environment`.
    pub fn try_environment(&mut self, name: &str, content: &str) -> Option<TexElement> {
        for p in &mut self.plugins {
            if let Some(elem) = p.handle_environment(name, content) {
                return Some(elem);
            }
        }
        None
    }

    /// Run `transform_elements` through every registered plugin.
    pub fn transform(&mut self, mut elements: Vec<TexElement>) -> Vec<TexElement> {
        for p in &mut self.plugins {
            elements = p.transform_elements(elements);
        }
        elements
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

/// Load plugins from a directory by looking for `.texplugin` marker
/// files (for future dynamic-loading support) and returning a
/// pre-configured registry.
pub fn load_plugins_from_dir(_dir: &std::path::Path) -> PluginRegistry {
    // Currently no dynamic loading; reserved for future work.
    PluginRegistry::new()
}

// ── Built-in example plugins ──────────────────────────────────────────

/// A plugin that turns `\today` into a `TexElement::Text` with the
/// current date string.
pub struct TodayPlugin;

impl Plugin for TodayPlugin {
    fn name(&self) -> &str {
        "today"
    }

    fn handle_command(&mut self, name: &str, _args: &[String]) -> Option<TexElement> {
        if name == "today" {
            let date = chrono::Local::now().format("%B %d, %Y").to_string();
            return Some(TexElement::Text(date));
        }
        None
    }
}

/// A plugin that resolves `\url{...}` into a plain-text element.
pub struct UrlPlugin;

impl Plugin for UrlPlugin {
    fn name(&self) -> &str {
        "url"
    }

    fn handle_command(&mut self, name: &str, args: &[String]) -> Option<TexElement> {
        if name == "url" && !args.is_empty() {
            return Some(TexElement::Text(format!("({})", args[0])));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_empty() {
        let mut reg = PluginRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.try_command("foo", &[]), None);
        let elems: Vec<TexElement> = vec![];
        assert_eq!(reg.transform(elems).len(), 0);
    }

    #[test]
    fn test_today_plugin() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(TodayPlugin));
        let result = reg.try_command("today", &[]);
        assert!(matches!(result, Some(TexElement::Text(ref s)) if !s.is_empty()));
    }

    #[test]
    fn test_url_plugin() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(UrlPlugin));
        let result = reg.try_command("url", &["https://example.com".to_string()]);
        assert_eq!(result, Some(TexElement::Text("(https://example.com)".to_string())));
    }

    #[test]
    fn test_transform_passthrough() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(TodayPlugin));
        let elems = vec![TexElement::Text("hello".to_string())];
        let out = reg.transform(elems);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], TexElement::Text("hello".to_string()));
    }

    #[test]
    fn test_try_environment_falls_through() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(TodayPlugin));
        assert_eq!(reg.try_environment("custom", "body"), None);
    }
}
