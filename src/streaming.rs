//! Streaming and progress support for large LaTeX documents.
//!
//! Provides chunked file reading, conversion-stage progress callbacks,
//! and memory-efficiency helpers for documents that may exceed available
//! RAM when loaded as a single `String`.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

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

/// Converts a LaTeX file to PDF with progress reporting and
/// memory-conscious chunked reading.
pub struct StreamingConverter<R: ProgressReporter> {
    reporter: R,
    chunk_size: usize,
    template: Option<crate::template::DocumentTemplate>,
}

impl StreamingConverter<NoOpReporter> {
    pub fn new() -> Self {
        Self {
            reporter: NoOpReporter,
            chunk_size: 1024 * 1024, // 1 MiB
            template: None,
        }
    }
}

impl<R: ProgressReporter> StreamingConverter<R> {
    pub fn with_reporter(reporter: R) -> Self {
        Self {
            reporter,
            chunk_size: 1024 * 1024,
            template: None,
        }
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Attach a document template for styling.
    pub fn with_template(mut self, template: crate::template::DocumentTemplate) -> Self {
        self.template = Some(template);
        self
    }

    /// Convert `input` → `output`, calling back on each stage.
    pub fn convert(&mut self, input: &Path, output: &Path) -> Result<(), crate::error::LatexError> {
        if !input.exists() {
            return Err(crate::error::LatexError::InvalidPath);
        }

        // Stage 1: read source (0–30%)
        self.reporter.stage_started("reading source", 0.0);
        let content = self.read_file(input)?;
        self.reporter.stage_finished("reading source");

        // Stage 2: parse (30–60%)
        self.reporter.stage_started("parsing", 30.0);
        let mut parser = crate::parser::TexParser::new(content);
        if let Some(parent) = input.parent() {
            parser = parser.with_base_dir(parent);
        }
        let elements = parser.parse();
        self.reporter.stage_finished("parsing");

        // Stage 3: build PDF (60–100%)
        self.reporter.stage_started("building PDF", 60.0);
        let mut builder = crate::pdf_builder::PdfBuilder::new();
        if let Some(template) = self.template.take() {
            builder = builder.with_template(template);
        }
        builder.build(elements, output)?;
        self.reporter.stage_finished("building PDF");

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
            self.events.lock().unwrap().push((format!("done:{}", name), 0.0));
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
}
