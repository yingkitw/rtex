//! # rtex — Native TeX to multi-format converter
//!
//! A self-contained Rust library for converting LaTeX documents to PDF, HTML,
//! DOCX, or EPUB without an external TeX installation.
//!
//! ## Quick example
//!
//! ```no_run
//! use rtex::{convert_tex_to_pdf, convert_tex_string, OutputFormat};
//! use std::path::Path;
//!
//! // File → PDF
//! convert_tex_to_pdf(Path::new("doc.tex"), Path::new("doc.pdf")).unwrap();
//!
//! // String → HTML
//! let html = convert_tex_string("\\documentclass{article}\\begin{document}Hi\\end{document}", OutputFormat::Html).unwrap();
//! ```
//!
//! ## Modules
//!
//! - [`NativeTexConverter`] — main conversion entry point
//! - [`OutputFormat`] — PDF, HTML, DOCX, EPUB selection
//! - [`PackageFetcher`] — on-demand CTAN package download
//! - [`TexParser`] / [`TexElement`] — LaTeX parsing AST
//! - [`PdfBuilder`] — native PDF generation
//!
//! ## Feature flags
//!
//! - `wasm` — enables `wasm-bindgen` exports for browser use
//! - `lsp` — builds the `rtex-lsp` language server binary

use std::fs;
use std::path::Path;
use std::sync::Mutex;

mod bibliography;
mod cache;
mod color;
mod common;
pub mod config;
pub mod error;
mod fonts;
mod image;
mod incremental;
mod intermediate;
mod layout;
pub mod lsp;
mod macros;
mod math;
mod math_formatter;
mod math_processor;
mod output;
mod packages;
pub mod page_layout;
mod parallel;
mod parser;
mod pdf;
mod plugins;
mod references;
mod streaming;
mod table;
pub mod template;
mod tex;
pub mod traits;
mod typography;
pub(crate) mod utils;
mod watch;

pub use cache::{CacheConfig, CacheStats, DocumentCache};
pub use error::{LatexError, Position};
pub use incremental::IncrementalCompiler;
pub use intermediate::{IntermediateMeta, sibling_artifact, write_intermediates};
pub use lsp::{
    CompletionKind, Severity, SymbolKind, TexCompletion, TexDiagnostic, TexPosition, TexRange,
    TexSymbol, analyze_diagnostics, command_completions, completions_at, document_symbols,
    hover_at,
};
pub use macros::expand_document;
pub use math_formatter::MathFormatter;
pub use math_processor::{MathCommandInfo, MathCommandType, MathProcessor};
pub use output::{DocumentMeta, OutputFormat, PdfBackend, render_elements, render_elements_in_dir};
pub use packages::{PackageFetcher, PackageRequest};
pub use parallel::{ParallelConverter, convert_dir};
pub use parser::{TexElement, TexParser};
pub use pdf::builder::PdfBuilder;
pub use plugins::{
    CustomFormatPlugin, FormatType, Plugin, PluginError, PluginRegistry, TodayPlugin, UrlPlugin,
};
pub use streaming::{ConsoleReporter, NoOpReporter, ProgressReporter, StreamingConverter};
pub use template::{
    ColorScheme, DocumentTemplate, HeadingScale, Margins, PaperSize, TitlePageConfig,
};
pub use tex::{
    BoxDirection, BrokenLine, CatCode, Dimension, Glue, InfiniteUnit, LineBreaker, LineItem,
    Stretch, TeXBox, TexLexer, Token, TokenizedParagraph, line_badness,
};
pub use typography::{KerningTable, TextSegment, TypographyEngine, TypographyOptions};
pub use watch::{watch_batch, watch_single};

/// Run the rtex language server over stdio (requires `lsp` feature).
#[cfg(feature = "lsp")]
pub fn run_lsp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    lsp::run_stdio_server()
}
pub use bibliography::{BibEntry, BibEntryType, BibliographyManager};
pub use common::{Clear, Stats};

/// Options controlling conversion format and package fetching.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    pub format: OutputFormat,
    pub fetch_packages: bool,
    pub package_cache: Option<std::path::PathBuf>,
    pub keep_intermediate: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Pdf,
            fetch_packages: false,
            package_cache: None,
            keep_intermediate: false,
        }
    }
}

impl ConversionOptions {
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_fetch_packages(mut self, fetch: bool) -> Self {
        self.fetch_packages = fetch;
        self
    }

    pub fn with_keep_intermediate(mut self, keep: bool) -> Self {
        self.keep_intermediate = keep;
        self
    }
}

fn default_package_cache() -> std::path::PathBuf {
    std::path::PathBuf::from(".rtex/cache")
}

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
    options: ConversionOptions,
}

impl NativeTexConverter {
    /// Create a new converter without caching.
    pub fn new() -> Self {
        Self {
            cache: None,
            options: ConversionOptions::default(),
        }
    }

    /// Create a converter with custom options.
    pub fn with_options(options: ConversionOptions) -> Self {
        Self {
            cache: None,
            options,
        }
    }

    /// Create a converter with an in-memory document cache.
    ///
    /// Parsed ASTs are cached by content hash, skipping re-parsing
    /// when the same source is converted again.
    pub fn with_cache() -> Self {
        Self {
            cache: Some(Mutex::new(DocumentCache::new())),
            options: ConversionOptions::default(),
        }
    }

    /// Parse TeX source into structured elements, using the cache when enabled.
    fn parse_content(&self, content: &str, base_dir: Option<&Path>) -> Vec<parser::TexElement> {
        if let Some(ref cache) = self.cache {
            let mut c = cache.lock().unwrap();
            if let Some(els) = c.get_parsed(content) {
                return els;
            }
            drop(c);
            let els = self.parse_fresh(content, base_dir);
            cache.lock().unwrap().put_parsed(content, els.clone());
            els
        } else {
            self.parse_fresh(content, base_dir)
        }
    }

    fn parse_fresh(&self, content: &str, base_dir: Option<&Path>) -> Vec<parser::TexElement> {
        let mut search_paths = Vec::new();
        if self.options.fetch_packages {
            let cache_dir = self
                .options
                .package_cache
                .clone()
                .unwrap_or_else(default_package_cache);
            let fetcher = PackageFetcher::new(cache_dir);
            let _ = fetcher.prepare_source(content);
            search_paths = fetcher.search_paths();
        }

        let mut parser = TexParser::new(content.to_string());
        if let Some(dir) = base_dir {
            parser = parser.with_base_dir(dir);
        }
        if !search_paths.is_empty() {
            parser = parser.with_search_paths(search_paths);
        }
        parser.parse()
    }

    /// Convert TeX source to output bytes in the configured format.
    pub fn convert_string(&self, tex: &str) -> Result<Vec<u8>, LatexError> {
        self.convert_string_in_dir(tex, None, None)
    }

    /// Convert TeX source with an optional base directory for `\input` and graphics.
    pub fn convert_string_in_dir(
        &self,
        tex: &str,
        base_dir: Option<&Path>,
        output: Option<&Path>,
    ) -> Result<Vec<u8>, LatexError> {
        let elements = self.parse_content(tex, base_dir);
        let bytes = render_elements_in_dir(elements.clone(), self.options.format, base_dir)?;

        if self.options.keep_intermediate {
            if let Some(out) = output {
                let expanded = expand_document(tex);
                write_intermediates(out, &expanded, &elements, self.options.format)?;
            }
        }

        Ok(bytes)
    }

    /// Convenience constructor that creates a converter and runs the conversion.
    pub fn convert_file(input: &Path, output: &Path) -> Result<(), LatexError> {
        let converter = Self::new();
        converter.convert(input, output)
    }

    /// Return a snapshot of cache statistics (test-only).
    #[cfg(test)]
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache
            .as_ref()
            .map(|c| c.lock().unwrap().stats().clone())
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
        let base_dir = input.parent();
        let bytes = self.convert_string_in_dir(&content, base_dir, Some(output))?;
        fs::write(output, bytes).map_err(|source| LatexError::IoError {
            path: output.to_path_buf(),
            source,
        })?;

        Ok(())
    }
}

/// Convert a TeX source string directly to PDF bytes (no filesystem access).
pub fn convert_tex_string_to_pdf_bytes(tex: &str) -> Result<Vec<u8>, LatexError> {
    NativeTexConverter::new().convert_string(tex)
}

/// Convert TeX source to bytes in the requested format.
pub fn convert_tex_string(tex: &str, format: OutputFormat) -> Result<Vec<u8>, LatexError> {
    NativeTexConverter::with_options(ConversionOptions::default().with_format(format))
        .convert_string(tex)
}

/// Convert a `.tex` file to an output file using the given options.
pub fn convert_tex_file(
    input: &Path,
    output: &Path,
    options: &ConversionOptions,
) -> Result<(), LatexError> {
    NativeTexConverter::with_options(options.clone()).convert(input, output)
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
