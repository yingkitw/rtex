//! LaTeX-Rust: A LaTeX processor written in Rust
//!
//! This library provides functionality to parse, process, and render LaTeX documents.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod renderer;
pub mod error;
pub mod streaming_parser;
pub mod plugin;
pub mod config;
pub mod cache;
pub mod incremental;
pub mod math;
pub mod bibtex;
pub mod common;

#[cfg(feature = "async")]
use std::sync::Arc;
#[cfg(feature = "async")]
use tokio::sync::RwLock;

// Specific exports from modules
pub use ast::{Document, DocumentMetadata, Node, Argument, ListType, ListItem, TableRow, TableType, ColumnAlignment, TableRule, MultiColumn, MultiRow, TableCell, MathEnvironmentType, ReferenceType, SpacingType, CitationType, SectionLevel, CodeStyle};
pub use error::{LaTeXError, LaTeXResult, Position};
pub use lexer::{Token, TokenType, Lexer};
pub use parser::Parser;
pub use renderer::{HtmlRenderer, MathRenderer, PdfRenderer};
pub use config::{LaTeXConfig, ParserConfig, RendererConfig, HtmlConfig, PluginConfig, StreamingConfig, ConfigError};
pub use cache::{LaTeXCache, CacheStats, CacheConfig};
pub use streaming_parser::{StreamingParser, StreamingConfig as StreamingParserConfig};
pub use incremental::{IncrementalConfig, IncrementalParser, IncrementalStats};
pub use math::{MathProcessor, MathArgument};
pub use bibtex::{BibEntry, BibliographyManager};
pub use plugin::{CommandPlugin, PluginRegistry, PluginError, CustomFormatPlugin, FormatType};

// Internal modules (not re-exported)

/// Main LaTeX processor with encapsulated components
pub struct LaTeXProcessor {
    lexer: Lexer,
    parser: Parser,
    plugin_registry: PluginRegistry,
    config: LaTeXConfig,
    cache: LaTeXCache,
    incremental_parser: Option<IncrementalParser>,
    math_processor: MathProcessor,
    bibliography_manager: BibliographyManager,
}

/// Builder for LaTeXProcessor with fluent interface
pub struct LaTeXProcessorBuilder {
    config: LaTeXConfig,
    cache_config: Option<CacheConfig>,
    incremental_config: Option<IncrementalConfig>,
    plugins: Vec<Box<dyn CommandPlugin>>,
}

impl LaTeXProcessorBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: LaTeXConfig::default(),
            cache_config: None,
            incremental_config: None,
            plugins: Vec::new(),
        }
    }
    
    /// Set the main configuration
    pub fn with_config(mut self, config: LaTeXConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Configure caching behavior
    pub fn with_cache_config(mut self, cache_config: CacheConfig) -> Self {
        self.cache_config = Some(cache_config);
        self
    }
    
    /// Enable incremental parsing with configuration
    pub fn with_incremental_parsing(mut self, config: IncrementalConfig) -> Self {
        self.incremental_config = Some(config);
        self
    }
    
    /// Add a plugin to the processor
    pub fn with_plugin(mut self, plugin: Box<dyn CommandPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }
    
    /// Build the LaTeX processor
    pub fn build(self) -> Result<LaTeXProcessor, ConfigError> {
        self.config.validate()?;
        
        let cache = match self.cache_config {
            Some(config) => LaTeXCache::with_config(config),
            None => LaTeXCache::new(),
        };
        
        let incremental_parser = self.incremental_config.map(IncrementalParser::with_config);
        
        let mut plugin_registry = PluginRegistry::new();
        for plugin in self.plugins {
            plugin_registry.register_plugin(plugin)
                .map_err(|e| ConfigError::ValidationError(format!("Plugin registration failed: {e}")))?;
        }
        
        Ok(LaTeXProcessor {
            lexer: Lexer::new(),
            parser: Parser::new(),
            plugin_registry,
            config: self.config,
            cache,
            incremental_parser,
            math_processor: MathProcessor::new(),
            bibliography_manager: BibliographyManager::new(),
        })
    }
}

impl Default for LaTeXProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LaTeXProcessor {
    /// Create a new LaTeX processor with default configuration
    /// 
    /// # Deprecated
    /// Use `LaTeXProcessorBuilder::new().build()` instead for better configuration control
    pub fn new() -> Self {
        LaTeXProcessorBuilder::new().build().expect("Default configuration should be valid")
    }
    
    /// Create a new LaTeX processor with custom configuration
    /// 
    /// # Deprecated
    /// Use `LaTeXProcessorBuilder::new().with_config(config).build()` instead
    pub fn with_config(config: LaTeXConfig) -> Result<Self, ConfigError> {
        LaTeXProcessorBuilder::new().with_config(config).build()
    }
    
    /// Create a builder for configuring the LaTeX processor
    pub fn builder() -> LaTeXProcessorBuilder {
        LaTeXProcessorBuilder::new()
    }
    
    /// Get a reference to the current configuration
    pub fn config(&self) -> &LaTeXConfig {
        &self.config
    }
    
    /// Update the configuration (validates before applying)
    pub fn update_config(&mut self, config: LaTeXConfig) -> Result<(), ConfigError> {
        config.validate()?;
        self.config = config;
        Ok(())
    }
    
    /// Get parser configuration
    pub fn parser_config(&self) -> &ParserConfig {
        &self.config.parser
    }
    
    /// Get renderer configuration
    pub fn renderer_config(&self) -> &RendererConfig {
        &self.config.renderer
    }
    
    /// Check if strict mode is enabled
    pub fn is_strict_mode(&self) -> bool {
        self.config.parser.strict_mode
    }

    /// Process LaTeX input and return the AST
    pub fn process(&mut self, input: &str) -> Result<Document, LaTeXError> {
        // Check cache first
        if let Some(cached_document) = self.cache.get_ast(input) {
            return Ok(cached_document);
        }
        
        // Process if not cached
        let tokens = self.lexer.tokenize(input)?;
        let document = self.parser.parse(tokens)?;
        
        // Cache the result
        self.cache.put_ast(input, document.clone());
        
        Ok(document)
    }

    /// Process LaTeX input and render to HTML
    pub fn to_html(&mut self, input: &str) -> Result<String, LaTeXError> {
        // Check HTML cache first
        if let Some(cached_html) = self.cache.get_html(input) {
            return Ok(cached_html);
        }
        
        // Process and render if not cached
        let document = self.process(input)?;
        let renderer = HtmlRenderer::new();
        let html = renderer.render(&document);
        
        // Cache the HTML result
        self.cache.put_html(input, html.clone());
        
        Ok(html)
    }

    /// Process LaTeX input and generate PDF output to file
    pub fn to_pdf(&mut self, input: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let document = self.process(input)?;
        let mut renderer = crate::renderer::PdfRenderer::new();
        renderer.render_to_file(&document, output_path)?;
        Ok(())
    }

    /// Process LaTeX input from a reader using streaming parser (memory efficient)
    pub fn process_stream<R: std::io::Read>(&mut self, reader: R) -> Result<Document, LaTeXError> {
        let mut streaming_parser = StreamingParser::new();
        streaming_parser.parse_stream(reader)
    }

    /// Process LaTeX input from a reader and render to HTML using streaming parser
    pub fn stream_to_html<R: std::io::Read>(&mut self, reader: R) -> Result<String, LaTeXError> {
        let document = self.process_stream(reader)?;
        let renderer = HtmlRenderer::new();
        Ok(renderer.render(&document))
    }
    
    /// Register a new command plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn CommandPlugin>) -> Result<(), PluginError> {
        self.plugin_registry.register_plugin(plugin)
    }
    
    /// Unregister a command plugin
    pub fn unregister_plugin(&mut self, command_name: &str) -> Option<Box<dyn CommandPlugin>> {
        self.plugin_registry.unregister_plugin(command_name)
    }
    
    /// Check if a command is handled by a plugin
    pub fn has_plugin(&self, command_name: &str) -> bool {
        self.plugin_registry.has_plugin(command_name)
    }
    
    /// Get list of registered plugin commands
    pub fn registered_commands(&self) -> Vec<&str> {
        self.plugin_registry.registered_commands()
    }
    
    /// Get help text for a plugin command
    pub fn get_plugin_help(&self, command_name: &str) -> Option<&str> {
        self.plugin_registry.get_help(command_name)
    }
    
    /// Process a command using plugins (internal method for parser integration)
    pub fn process_plugin_command(&self, command_name: &str, args: Vec<String>) -> Result<Node, LaTeXError> {
        self.plugin_registry.process_command(command_name, args)
            .map_err(|e| match e {
                PluginError::CommandNotFound(cmd) => LaTeXError::ParserError {
                    position: Position::start(),
                    message: format!("Unknown command: {cmd}"),
                },
                PluginError::DuplicateCommand(cmd) => LaTeXError::ParserError {
                    position: Position::start(),
                    message: format!("Duplicate command registration: {cmd}"),
                },
                PluginError::ValidationError(err) => err,
                PluginError::ProcessingError(err) => err,
            })
    }
    
    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats().clone()
    }

    /// Update cache configuration
    pub fn update_cache_config(&mut self, config: CacheConfig) {
        self.cache.update_config(config);
    }

    /// Enable incremental parsing
    pub fn enable_incremental_parsing(&mut self) {
        self.incremental_parser = Some(IncrementalParser::new());
    }

    /// Enable incremental parsing with custom configuration
    pub fn enable_incremental_parsing_with_config(&mut self, config: IncrementalConfig) {
        self.incremental_parser = Some(IncrementalParser::with_config(config));
    }

    /// Disable incremental parsing
    pub fn disable_incremental_parsing(&mut self) {
        self.incremental_parser = None;
    }

    /// Process LaTeX input using incremental parsing if enabled
    pub fn process_incremental(&mut self, input: &str) -> Result<Document, LaTeXError> {
        if let Some(ref mut incremental_parser) = self.incremental_parser {
            incremental_parser.parse_incremental(input)
        } else {
            // Fall back to regular processing
            self.process(input)
        }
    }

    /// Get incremental parsing statistics
    pub fn incremental_stats(&self) -> Option<IncrementalStats> {
        self.incremental_parser.as_ref().map(|p| p.get_stats())
    }

    /// Clear incremental cache
    pub fn clear_incremental_cache(&mut self) {
        if let Some(ref mut parser) = self.incremental_parser {
            parser.clear_cache();
        }
    }
    
    /// Enable equation numbering
    pub fn enable_equation_numbering(&mut self) {
        // Equation numbering is enabled by default in MathProcessor
    }
    
    /// Disable equation numbering
    pub fn disable_equation_numbering(&mut self) {
        self.math_processor.reset_equation_counter();
    }
    
    /// Get equation number by label
    pub fn get_equation_number(&self, label: &str) -> Option<usize> {
        self.math_processor.get_equation_number(label)
    }
    
    /// Reset equation counter
    pub fn reset_equation_counter(&mut self) {
        self.math_processor.reset_equation_counter();
    }
    
    /// Check if command is a math command
    pub fn is_math_command(&self, name: &str) -> bool {
        self.math_processor.is_math_command(name)
    }
    
    /// Get supported math environments
    pub fn get_supported_math_environments() -> Vec<&'static str> {
        MathProcessor::get_supported_environments()
    }
    
    /// Process math with advanced features
    pub fn process_math_advanced(&mut self, content: &str) -> Result<String, LaTeXError> {
        // Parse the LaTeX content
        let tokens = self.lexer.tokenize(content)?;
        let mut ast = self.parser.parse(tokens)?;
        
        // Process math environments with equation numbering
        self.process_math_environments(&mut ast);
        
        // Render to HTML
        let renderer = HtmlRenderer::new();
        Ok(renderer.render(&ast))
    }
    
    /// Load bibliography from BibTeX file
    pub fn load_bibliography<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), LaTeXError> {
        self.bibliography_manager.load_file(path)
            .map_err(|e| LaTeXError::InvalidSyntax { message: format!("Failed to load bibliography: {e}") })
    }
    
    /// Add a bibliography entry
    pub fn add_bibliography_entry(&mut self, entry: BibEntry) {
        self.bibliography_manager.add_entry(entry);
    }
    
    /// Set bibliography style
    pub fn set_bibliography_style(&mut self, style: String) {
        self.bibliography_manager.set_style(style);
    }
    
    /// Get bibliography style
    pub fn get_bibliography_style(&self) -> Option<&String> {
        self.bibliography_manager.get_style()
    }
    
    /// Check if a citation key exists
    pub fn has_citation_key(&self, key: &str) -> bool {
        self.bibliography_manager.has_key(key)
    }
    
    /// Get all available citation keys
    pub fn get_citation_keys(&self) -> Vec<String> {
        self.bibliography_manager.get_all_keys()
    }
    
    /// Generate bibliography for specific citation keys
    pub fn generate_bibliography(&self, keys: &[String]) -> Vec<Node> {
        self.bibliography_manager.generate_bibliography(keys)
    }
    
    /// Generate complete bibliography with all entries
    pub fn generate_complete_bibliography(&self) -> Vec<Node> {
        self.bibliography_manager.generate_complete_bibliography()
    }
    
    // === Private Internal Methods ===
    
    /// Process math environments in AST
    fn process_math_environments(&mut self, ast: &mut Document) {
        for node in &mut ast.body {
            self.process_node_math(node);
        }
    }
    
    /// Process math in a single node
    fn process_node_math(&mut self, node: &mut Node) {
        match node {
            Node::Environment { content, .. } => {
                self.process_environment_math(content);
            }
            Node::MathEnvironment { env_type: _, content, label, numbered, equation_number } => {
                self.process_math_environment_numbering(label, numbered, equation_number);
                self.process_environment_math(content);
            }
            Node::Section { content, .. } => {
                self.process_environment_math(content);
            }
            _ => {}
        }
    }
    
    /// Process math environments in content recursively
    fn process_environment_math(&mut self, content: &mut Vec<Node>) {
        for child in content {
            self.process_node_math(child);
        }
    }
    
    /// Handle equation numbering for math environments
    fn process_math_environment_numbering(
        &mut self, 
        label: &Option<String>, 
        numbered: &bool, 
        equation_number: &mut Option<usize>
    ) {
        if *numbered {
            let eq_num = self.math_processor.next_equation_number();
            *equation_number = Some(eq_num);
            if let Some(ref label_str) = label {
                self.math_processor.add_equation_label(label_str.clone(), eq_num);
            }
        }
    }
}

impl Default for LaTeXProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "async")]
impl LaTeXProcessor {
    /// Process LaTeX input and return the AST (async)
    pub async fn process_async(&self, input: &str) -> Result<Document, LaTeXError> {
        // Check cache first
        if let Some(cached_document) = self.cache.get_ast(input) {
            return Ok(cached_document);
        }
        
        let input = input.to_string();
        let lexer = self.lexer.clone();
        let parser = self.parser.clone();
        let cache = self.cache.clone();
        
        let document = tokio::task::spawn_blocking(move || {
            let mut lexer = lexer;
            let mut parser = parser;
            
            let tokens = lexer.tokenize(&input)?;
            let document = parser.parse(tokens)?;
            
            // Cache the result
            cache.put_ast(&input, document.clone());
            
            Ok(document)
        }).await.map_err(|e| LaTeXError::ParserError {
             position: Position::start(),
             message: format!("Async processing error: {}", e),
         })??;
         
         Ok(document)
    }

    /// Process LaTeX input and render to HTML (async)
    pub async fn to_html_async(&self, input: &str) -> Result<String, LaTeXError> {
        // Check HTML cache first
        if let Some(cached_html) = self.cache.get_html(input) {
            return Ok(cached_html);
        }
        
        let document = self.process_async(input).await?;
        let cache = self.cache.clone();
        let input = input.to_string();
        
        let html = tokio::task::spawn_blocking(move || {
            let renderer = HtmlRenderer::new();
            let html = renderer.render(&document);
            
            // Cache the HTML result
            cache.put_html(&input, html.clone());
            
            html
        }).await.map_err(|e| LaTeXError::RendererError {
            position: Position::start(),
            message: format!("Async HTML rendering error: {}", e),
        })?;
        
        Ok(html)
    }
    
    /// Process multiple LaTeX documents concurrently
    pub async fn process_batch_async(&self, inputs: Vec<String>) -> Vec<Result<Document, LaTeXError>> {
        let tasks = self.create_processing_tasks(inputs);
        self.collect_task_results(tasks).await
    }
    
    /// Create async processing tasks for batch processing
    fn create_processing_tasks(&self, inputs: Vec<String>) -> Vec<tokio::task::JoinHandle<Result<Document, LaTeXError>>> {
        inputs.into_iter().map(|input| {
            let lexer = self.lexer.clone();
            let parser = self.parser.clone();
            tokio::spawn(async move {
                Self::process_single_document_blocking(lexer, parser, input).await
            })
        }).collect()
    }
    
    /// Process a single document in a blocking task
    async fn process_single_document_blocking(
        lexer: Lexer, 
        parser: Parser, 
        input: String
    ) -> Result<Document, LaTeXError> {
        tokio::task::spawn_blocking(move || {
            let mut lexer = lexer;
            let mut parser = parser;
            
            let tokens = lexer.tokenize(&input)?;
            let document = parser.parse(tokens)?;
            Ok(document)
        }).await.map_err(|e| LaTeXError::ParserError {
            position: Position::start(),
            message: format!("Async processing error: {}", e),
        })?
    }
    
    /// Collect results from async processing tasks
    async fn collect_task_results(
        &self, 
        tasks: Vec<tokio::task::JoinHandle<Result<Document, LaTeXError>>>
    ) -> Vec<Result<Document, LaTeXError>> {
        let mut results = Vec::new();
        for task in tasks {
            let result = self.handle_task_completion(task).await;
            results.push(result);
        }
        results
    }
    
    /// Handle completion of a single async task
    async fn handle_task_completion(
        &self, 
        task: tokio::task::JoinHandle<Result<Document, LaTeXError>>
    ) -> Result<Document, LaTeXError> {
        match task.await {
            Ok(result) => result,
            Err(e) => Err(LaTeXError::ParserError {
                position: Position::start(),
                message: format!("Batch processing error: {}", e),
            }),
        }
    }
}