//! latex-rs - Native TeX to PDF converter
//!
//! A self-contained Rust library for converting LaTeX documents to PDF
//! without requiring an external LaTeX installation.

use std::path::Path;
use std::fs;

mod parser;
mod pdf_builder;
mod pdf_core;
mod pdf_text_renderer;
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
mod pdf;
mod font_subset;
pub(crate) mod utils;
pub mod error;
pub mod config;
pub mod traits;
pub mod page_layout;
pub mod template;

pub use error::{LatexError, Position};
pub use parser::TexParser;
pub use pdf_builder::PdfBuilder;
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
pub struct NativeTexConverter;

impl NativeTexConverter {
    /// Create a new converter.
    pub fn new() -> Self {
        Self
    }

    /// Convenience constructor that creates a converter and runs the conversion.
    pub fn convert_file(input: &Path, output: &Path) -> Result<(), LatexError> {
        let converter = Self::new();
        converter.convert(input, output)
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

        let mut parser = TexParser::new(content);
        let elements = parser.parse();

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
