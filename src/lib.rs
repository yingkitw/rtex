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
mod fonts;
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

    /// Parse TeX source into structured elements, using the cache when enabled.
    fn parse_content(&self, content: &str) -> Vec<parser::TexElement> {
        if let Some(ref cache) = self.cache {
            let mut c = cache.lock().unwrap();
            if let Some(els) = c.get_parsed(content) {
                return els;
            }
            drop(c);
            let mut parser = TexParser::new(content.to_string());
            let els = parser.parse();
            cache.lock().unwrap().put_parsed(content, els.clone());
            els
        } else {
            let mut parser = TexParser::new(content.to_string());
            parser.parse()
        }
    }

    /// Convert TeX source directly to PDF bytes without filesystem access.
    ///
    /// This is the primary API for WASM and other embedded environments.
    pub fn convert_string(&self, tex: &str) -> Result<Vec<u8>, LatexError> {
        let elements = self.parse_content(tex);
        let mut builder = PdfBuilder::new();
        builder
            .build_to_bytes(elements)
            .map_err(|msg| LatexError::PdfError {
                message: msg,
                context: None,
            })
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
        let pdf = self.convert_string(&content)?;
        fs::write(output, pdf).map_err(|source| LatexError::IoError {
            path: output.to_path_buf(),
            source,
        })?;

        Ok(())
    }
}

/// Convert a TeX source string directly to PDF bytes (no filesystem access).
///
/// Ideal for WASM targets and in-process embedding where reading/writing
/// files is unavailable or undesirable.
pub fn convert_tex_string_to_pdf_bytes(tex: &str) -> Result<Vec<u8>, LatexError> {
    NativeTexConverter::new().convert_string(tex)
}

/// One-shot helper: convert a `.tex` file to a `.pdf`.
pub fn convert_tex_to_pdf(input: &Path, output: &Path) -> Result<(), LatexError> {
    let converter = NativeTexConverter::new();
    converter.convert(input, output)
}

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod example_tests;
