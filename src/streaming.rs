//! Streaming and progress support for large LaTeX documents.
//!
//! Provides chunked file reading, conversion-stage progress callbacks,
//! and memory-efficiency helpers for documents that may exceed available
//! RAM when loaded as a single `String`.

use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::ConversionOptions;
use crate::incremental::IncrementalCompiler;
use crate::output::{OutputFormat, render_elements};
use crate::packages::PackageFetcher;

/// Callback trait for reporting conversion progress.
pub trait ProgressReporter: Send {
    /// Called when a stage begins.  `pct` is an estimated overall
    /// completion percentage (0.0–100.0).
    fn stage_started(&mut self, name: &str, pct: f64);
    /// Called when a stage finishes.
    fn stage_finished(&mut self, name: &str);
}

/// No-op reporter — useful when progress is not required.
pub struct NoOpReporter;

impl ProgressReporter for NoOpReporter {
    fn stage_started(&mut self, _name: &str, _pct: f64) {}
    fn stage_finished(&mut self, _name: &str) {}
}

/// Simple console reporter that prints stage names to stderr.
pub struct ConsoleReporter;

impl ProgressReporter for ConsoleReporter {
    fn stage_started(&mut self, name: &str, pct: f64) {
        eprintln!("[{:>5.1}%] {}", pct, name);
    }
    fn stage_finished(&mut self, name: &str) {
        eprintln!("[ done ] {}", name);
    }
}

/// Converts a LaTeX file to the configured output format with progress reporting and
/// memory-conscious chunked reading.
pub struct StreamingConverter<R: ProgressReporter> {
    reporter: R,
    chunk_size: usize,
    incremental: IncrementalCompiler,
    force_rebuild: bool,
    keep_intermediate: bool,
    options: ConversionOptions,
}

impl StreamingConverter<NoOpReporter> {
    pub fn new() -> Self {
        Self {
            reporter: NoOpReporter,
            chunk_size: 1024 * 1024, // 1 MiB
            incremental: IncrementalCompiler::new(),
            force_rebuild: false,
            keep_intermediate: false,
            options: ConversionOptions::default(),
        }
    }
}

fn package_cache_dir(options: &ConversionOptions) -> std::path::PathBuf {
    options
        .package_cache
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from(".rtex/cache"))
}

impl<R: ProgressReporter> StreamingConverter<R> {
    pub fn with_reporter(reporter: R) -> Self {
        Self {
            reporter,
            chunk_size: 1024 * 1024,
            incremental: IncrementalCompiler::new(),
            force_rebuild: false,
            keep_intermediate: false,
            options: ConversionOptions::default(),
        }
    }

    /// Configure output format, package fetching, and intermediate artifacts.
    pub fn with_options(mut self, options: ConversionOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Enable or disable incremental compilation (enabled by default).
    pub fn with_incremental(mut self, enabled: bool) -> Self {
        self.incremental = if enabled {
            IncrementalCompiler::new()
        } else {
            IncrementalCompiler::disabled()
        };
        self
    }

    /// Force a full rebuild even when the source and dependencies are unchanged.
    pub fn with_force_rebuild(mut self, force: bool) -> Self {
        self.force_rebuild = force;
        self
    }

    /// Keep intermediate artifacts (`.expanded.tex`, `.ast.json`, `.meta.json`).
    pub fn with_keep_intermediate(mut self, keep: bool) -> Self {
        self.keep_intermediate = keep;
        self
    }

    /// Snapshot of incremental build statistics: `(skips, builds)`.
    pub fn incremental_stats(&self) -> (u64, u64) {
        self.incremental.stats()
    }

    /// Convert `input` → `output`, calling back on each stage.
    pub fn convert(&mut self, input: &Path, output: &Path) -> Result<(), crate::error::LatexError> {
        if !input.exists() {
            return Err(crate::error::LatexError::InvalidPath);
        }

        if !self.force_rebuild && output.exists() {
            let deps = crate::watch::discover_inputs(input);
            if !self.incremental.needs_rebuild(input)
                || IncrementalCompiler::outputs_up_to_date(input, output, &deps)
            {
                self.reporter.stage_started("skipped (up to date)", 100.0);
                self.reporter.stage_finished("skipped (up to date)");
                self.incremental
                    .mark_built(input, vec![output.to_path_buf()], deps);
                return Ok(());
            }
        }

        // Stage 1: read source (0–30%)
        self.reporter.stage_started("reading source", 0.0);
        let content = self.read_file(input)?;
        self.reporter.stage_finished("reading source");

        let expanded = crate::macros::expand_document(&content);

        let mut search_paths = Vec::new();
        if self.options.fetch_packages {
            let fetcher = PackageFetcher::new(package_cache_dir(&self.options));
            let _ = fetcher.prepare_source(&content);
            search_paths = fetcher.search_paths();
        }

        // Stage 2: parse (30–60%)
        self.reporter.stage_started("parsing", 30.0);
        let mut parser = crate::parser::TexParser::new(content);
        if let Some(parent) = input.parent() {
            parser = parser.with_base_dir(parent);
        }
        if !search_paths.is_empty() {
            parser = parser.with_search_paths(search_paths);
        }
        let elements = parser.parse();
        self.reporter.stage_finished("parsing");

        // Stage 3: build output (60–100%)
        let build_stage = match self.options.format {
            OutputFormat::Pdf => "building PDF",
            OutputFormat::Html => "building HTML",
            OutputFormat::Docx => "building DOCX",
            OutputFormat::Epub => "building EPUB",
        };
        self.reporter.stage_started(build_stage, 60.0);
        match self.options.format {
            OutputFormat::Pdf | OutputFormat::Html | OutputFormat::Docx | OutputFormat::Epub => {
                let bytes = render_elements(elements.clone(), self.options.format)?;
                fs::write(output, bytes).map_err(|source| crate::error::LatexError::IoError {
                    path: output.to_path_buf(),
                    source,
                })?;
            }
        }
        self.reporter.stage_finished(build_stage);

        if self.keep_intermediate {
            crate::intermediate::write_intermediates(
                output,
                &expanded,
                &elements,
                self.options.format,
            )?;
        }

        let deps = crate::watch::discover_inputs(input);
        self.incremental
            .mark_built(input, vec![output.to_path_buf()], deps);

        Ok(())
    }

    /// Read a potentially large file, allocating only what is needed.
    fn read_file(&mut self, path: &Path) -> Result<String, crate::error::LatexError> {
        let file = File::open(path)?;
        let meta = file.metadata()?;
        let len = meta.len() as usize;

        // For small files just read everything at once.
        if len <= self.chunk_size {
            let mut buf = String::with_capacity(len);
            let mut reader = BufReader::new(file);
            reader.read_to_string(&mut buf)?;
            return Ok(buf);
        }

        // For large files read in chunks and accumulate.
        let mut reader = BufReader::with_capacity(self.chunk_size, file);
        let mut buf = String::with_capacity(len);
        let mut chunk = vec![0u8; self.chunk_size];
        let mut total_read = 0usize;

        loop {
            let n = reader.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            // SAFETY: the parser expects valid UTF-8; invalid bytes are replaced.
            let text = String::from_utf8_lossy(&chunk[..n]);
            buf.push_str(&text);
            total_read += n;

            // Report intermediate progress for very large files.
            if len > 0 {
                let pct = 30.0 * (total_read as f64 / len as f64).min(1.0);
                self.reporter.stage_started("reading source", pct);
            }
        }

        Ok(buf)
    }
}

impl Default for StreamingConverter<NoOpReporter> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience one-shot conversion with progress reporting.
#[allow(dead_code)]
pub fn convert_with_progress<P: ProgressReporter>(
    input: &Path,
    output: &Path,
    reporter: P,
) -> Result<(), crate::error::LatexError> {
    let mut converter = StreamingConverter::with_reporter(reporter);
    converter.convert(input, output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct CollectingReporter {
        events: Arc<Mutex<Vec<(String, f64)>>>,
    }

    impl ProgressReporter for CollectingReporter {
        fn stage_started(&mut self, name: &str, pct: f64) {
            self.events.lock().unwrap().push((name.to_string(), pct));
        }
        fn stage_finished(&mut self, name: &str) {
            self.events
                .lock()
                .unwrap()
                .push((format!("done:{}", name), 0.0));
        }
    }

    #[test]
    fn test_converter_reports_stages() {
        let reporter = CollectingReporter::default();
        let events = reporter.events.clone();

        let mut converter = StreamingConverter::with_reporter(reporter);
        // Use a non-existent path — the method returns InvalidPath before
        // any stage callback, so the log should be empty.
        let input = Path::new("/nonexistent/streaming_test.tex");
        let output = Path::new("/nonexistent/streaming_test.pdf");
        let result = converter.convert(input, output);
        assert!(result.is_err());

        let log = events.lock().unwrap();
        assert!(log.is_empty());
    }

    #[test]
    fn test_noop_reporter_does_not_panic() {
        let mut r = NoOpReporter;
        r.stage_started("x", 50.0);
        r.stage_finished("x");
    }

    #[test]
    fn test_console_reporter_does_not_panic() {
        let mut r = ConsoleReporter;
        r.stage_started("x", 50.0);
        r.stage_finished("x");
    }

    #[test]
    fn incremental_skips_unchanged_conversion() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let dst = tmp.path().join("doc.pdf");
        std::fs::write(
            &src,
            "\\documentclass{article}\\begin{document}Hi\\end{document}",
        )
        .unwrap();

        let mut first = StreamingConverter::new();
        first.convert(&src, &dst).expect("first conversion");
        assert!(dst.exists());

        let reporter = CollectingReporter::default();
        let events = reporter.events.clone();
        let mut second = StreamingConverter::with_reporter(reporter);
        second
            .convert(&src, &dst)
            .expect("second conversion should skip");

        let log = events.lock().unwrap();
        assert!(log.iter().any(|(name, _)| name == "skipped (up to date)"));
    }

    #[test]
    fn force_rebuild_runs_even_when_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let dst = tmp.path().join("doc.pdf");
        std::fs::write(
            &src,
            "\\documentclass{article}\\begin{document}Hi\\end{document}",
        )
        .unwrap();

        let mut converter = StreamingConverter::new();
        converter.convert(&src, &dst).expect("first conversion");

        let reporter = CollectingReporter::default();
        let events = reporter.events.clone();
        let mut converter = StreamingConverter::with_reporter(reporter).with_force_rebuild(true);
        converter.convert(&src, &dst).expect("forced conversion");

        let log = events.lock().unwrap();
        assert!(!log.iter().any(|(name, _)| name == "skipped (up to date)"));
        assert!(log.iter().any(|(name, _)| name == "reading source"));
    }

    #[test]
    fn streaming_converts_html() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let dst = tmp.path().join("doc.html");
        std::fs::write(
            &src,
            "\\documentclass{article}\\begin{document}Hello\\end{document}",
        )
        .unwrap();

        let mut converter = StreamingConverter::new()
            .with_options(ConversionOptions::default().with_format(OutputFormat::Html));
        converter.convert(&src, &dst).expect("html conversion");

        let html = std::fs::read_to_string(&dst).unwrap();
        assert!(html.contains("Hello"));
        assert!(html.contains("<html"));
    }

    #[test]
    fn streaming_writes_intermediate_artifacts() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let dst = tmp.path().join("doc.pdf");
        std::fs::write(
            &src,
            "\\documentclass{article}\\title{T}\\begin{document}Hi\\end{document}",
        )
        .unwrap();

        let mut converter = StreamingConverter::new().with_keep_intermediate(true);
        converter.convert(&src, &dst).expect("conversion");

        assert!(dst.exists());
        assert!(tmp.path().join("doc.expanded.tex").exists());
        assert!(tmp.path().join("doc.ast.json").exists());
        assert!(tmp.path().join("doc.meta.json").exists());
    }
}
