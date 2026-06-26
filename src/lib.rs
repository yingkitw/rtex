//! latex-rs - Native TeX to PDF converter
//!
//! A self-contained Rust library for converting LaTeX documents to PDF
//! without requiring an external LaTeX installation.

use std::path::Path;
use std::fs;
use std::sync::Mutex;

mod parser;
mod pdf;
mod math;
mod math_formatter;
mod image;
mod table;
mod color;
mod layout;
mod macros;
mod bibliography;
mod references;
mod streaming;
mod plugins;
mod typography;
mod tex;
mod cache;
mod incremental;
mod math_processor;
mod parallel;
mod watch;
mod common;
pub(crate) mod utils;
pub mod error;
pub mod config;
pub mod traits;
pub mod page_layout;
pub mod template;

pub use error::{LatexError, Position};
pub use parser::TexParser;
pub use pdf::builder::PdfBuilder;
pub use math_formatter::MathFormatter;
pub use streaming::{StreamingConverter, ProgressReporter, NoOpReporter, ConsoleReporter};
pub use plugins::{Plugin, PluginRegistry, TodayPlugin, UrlPlugin, PluginError, FormatType, CustomFormatPlugin};
pub use typography::{TypographyEngine, TypographyOptions, KerningTable, TextSegment};
pub use tex::{CatCode, Token, TexLexer, Dimension};
pub use cache::{DocumentCache, CacheConfig, CacheStats};
pub use template::{DocumentTemplate, PaperSize, Margins, HeadingScale, ColorScheme, TitlePageConfig};
pub use incremental::IncrementalCompiler;
pub use math_processor::{MathProcessor, MathCommandType, MathCommandInfo};
pub use parallel::{ParallelConverter, convert_dir};
pub use watch::{watch_single, watch_batch};
pub use common::{Clear, Stats};
pub use bibliography::{BibEntry, BibEntryType, BibliographyManager};

/// Trait for converting a LaTeX file to PDF.
pub trait TexConverter {
    /// Convert `input` (a `.tex` file) to `output` (a `.pdf` file).
    fn convert(&self, input: &Path, output: &Path) -> Result<(), LatexError>;
}

/// Pure-Rust TeX-to-PDF converter.
///
/// No external LaTeX installation is required.
pub struct NativeTexConverter {
    cache: Option<Mutex<DocumentCache>>,
}

impl NativeTexConverter {
    /// Create a new converter without caching.
    pub fn new() -> Self {
        Self { cache: None }
    }

    /// Create a converter with an in-memory document cache.
    ///
    /// Parsed ASTs are cached by content hash, skipping re-parsing
    /// when the same source is converted again.
    pub fn with_cache() -> Self {
        Self {
            cache: Some(Mutex::new(DocumentCache::new())),
        }
    }

    /// Convenience constructor that creates a converter and runs the conversion.
    pub fn convert_file(input: &Path, output: &Path) -> Result<(), LatexError> {
        let converter = Self::new();
        converter.convert(input, output)
    }

    /// Return a snapshot of cache statistics (test-only).
    #[cfg(test)]
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|c| c.lock().unwrap().stats().clone())
    }
}

impl Default for NativeTexConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TexConverter for NativeTexConverter {
    fn convert(&self, input: &Path, output: &Path) -> Result<(), LatexError> {
        if !input.exists() {
            return Err(LatexError::InvalidPath);
        }

        let content = fs::read_to_string(input)?;

        let elements = if let Some(ref cache) = self.cache {
            let mut c = cache.lock().unwrap();
            if let Some(els) = c.get_parsed(&content) {
                els
            } else {
                drop(c); // release lock before parsing
                let mut parser = TexParser::new(content.clone());
                let els = parser.parse();
                cache.lock().unwrap().put_parsed(&content, els.clone());
                els
            }
        } else {
            let mut parser = TexParser::new(content);
            parser.parse()
        };

        let mut builder = PdfBuilder::new();
        builder.build(elements, output)?;

        Ok(())
    }
}

/// One-shot helper: convert a `.tex` file to a `.pdf`.
pub fn convert_tex_to_pdf(input: &Path, output: &Path) -> Result<(), LatexError> {
    let converter = NativeTexConverter::new();
    converter.convert(input, output)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod example_tests;
