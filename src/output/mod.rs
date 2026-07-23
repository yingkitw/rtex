//! Multi-format output renderers.
//!
//! Converts parsed [`TexElement`] trees into HTML, DOCX, EPUB, or PDF bytes.

pub mod common;
mod docx;
mod epub;
mod html;
pub mod pdfrs_pdf;

use crate::error::LatexError;
use crate::parser::TexElement;
use crate::pdf::builder::PdfBuilder;
pub use pdfrs_pdf::{PDFRS_MAX_BYTES, PdfBackend, PdfRenderOptions, render_pdf_bytes};

pub use common::DocumentMeta;

/// Supported output formats for LaTeX conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Pdf,
    Html,
    Docx,
    Epub,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "html" | "htm" => Some(Self::Html),
            "docx" | "word" => Some(Self::Docx),
            "epub" => Some(Self::Epub),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Html => "html",
            Self::Docx => "docx",
            Self::Epub => "epub",
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Html => "text/html",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Epub => "application/epub+zip",
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = LatexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| LatexError::ConfigError {
            message: format!("Unknown output format '{s}'. Use pdf, html, docx, or epub."),
        })
    }
}

/// Render parsed elements to bytes in the requested format.
pub fn render_elements(
    elements: Vec<TexElement>,
    format: OutputFormat,
) -> Result<Vec<u8>, LatexError> {
    render_elements_in_dir(elements, format, None)
}

/// Like [`render_elements`] with an optional base directory for relative images (PDF).
pub fn render_elements_in_dir(
    elements: Vec<TexElement>,
    format: OutputFormat,
    image_base: Option<&std::path::Path>,
) -> Result<Vec<u8>, LatexError> {
    match format {
        OutputFormat::Pdf => render_pdf_with_fallback(elements, image_base),
        OutputFormat::Html => html::render(&elements),
        OutputFormat::Docx => docx::render(&elements),
        OutputFormat::Epub => epub::render(&elements),
    }
}

/// Render PDF via pdfrs; fall back to the native engine only on failure or oversize output.
///
/// Override with `RTEX_PDF_BACKEND=pdfrs|native` (fail if forced backend cannot run).
fn render_pdf_with_fallback(
    elements: Vec<TexElement>,
    image_base: Option<&std::path::Path>,
) -> Result<Vec<u8>, LatexError> {
    let forced = PdfBackend::from_env();
    let options = PdfRenderOptions {
        image_base_dir: image_base.map(std::path::Path::to_path_buf),
        ..Default::default()
    };

    if forced != Some(PdfBackend::Native) {
        match render_pdf_bytes(&elements, options) {
            Ok(bytes) if bytes.len() <= PDFRS_MAX_BYTES => {
                return Ok(stamp_pdf_producer(bytes, PdfBackend::Pdfrs));
            }
            Ok(_) if forced == Some(PdfBackend::Pdfrs) => {
                return Err(LatexError::PdfError {
                    message: "RTEX_PDF_BACKEND=pdfrs but output exceeds PDFRS_MAX_BYTES".into(),
                    context: None,
                });
            }
            Err(e) if forced == Some(PdfBackend::Pdfrs) => return Err(e),
            _ => {}
        }
    }

    if forced == Some(PdfBackend::Pdfrs) {
        return Err(LatexError::PdfError {
            message: "RTEX_PDF_BACKEND=pdfrs but pdfrs rendering failed".into(),
            context: None,
        });
    }

    let mut builder = PdfBuilder::new();
    let bytes = builder
        .emit_native_pdf(elements)
        .map_err(|message| LatexError::PdfError {
            message,
            context: None,
        })?;
    Ok(stamp_pdf_producer(bytes, PdfBackend::Native))
}

/// Stamp `/Producer` so PDF metadata identifies which backend ran.
fn stamp_pdf_producer(mut pdf: Vec<u8>, backend: PdfBackend) -> Vec<u8> {
    let label = backend.producer_label();
    let replacements = [
        (
            b"/Producer (pdfrs)".as_slice(),
            format!("/Producer ({label})").into_bytes(),
        ),
        (
            b"/Producer (rtex)".as_slice(),
            format!("/Producer ({label})").into_bytes(),
        ),
        (
            b"/Producer (pdf-cli)".as_slice(),
            format!("/Producer ({label})").into_bytes(),
        ),
    ];
    for (from, to) in replacements {
        if let Some(pos) = find_bytes(&pdf, from) {
            pdf.splice(pos..pos + from.len(), to);
            break;
        }
    }
    pdf
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::TexParser;

    fn sample_elements() -> Vec<TexElement> {
        let tex = r#"\documentclass{article}
\title{Sample}
\author{rtex}
\begin{document}
\section{Intro}
Hello \textbf{world} and $x^2$.
\end{document}
"#;
        let mut parser = TexParser::new(tex.to_string());
        parser.parse()
    }

    #[test]
    fn output_format_parse_and_extension() {
        assert_eq!(OutputFormat::parse("HTML"), Some(OutputFormat::Html));
        assert_eq!(OutputFormat::Html.extension(), "html");
        assert_eq!(
            OutputFormat::Docx.mime_type(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
    }

    #[test]
    fn render_html_contains_structure() {
        let bytes = render_elements(sample_elements(), OutputFormat::Html).unwrap();
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<h2>Intro</h2>"));
        assert!(html.contains("<strong>world</strong>"));
    }

    #[test]
    fn render_docx_is_zip_archive() {
        let bytes = render_elements(sample_elements(), OutputFormat::Docx).unwrap();
        assert!(bytes.starts_with(b"PK"));
        assert!(bytes.windows(4).any(|w| w == b"word"));
    }

    #[test]
    fn render_epub_has_mimetype() {
        let bytes = render_elements(sample_elements(), OutputFormat::Epub).unwrap();
        assert!(bytes.starts_with(b"PK"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("application/epub+zip"));
        assert!(text.contains("chapter.xhtml"));
    }

    #[test]
    fn pdf_stamps_pdfrs_producer() {
        let bytes = render_elements(sample_elements(), OutputFormat::Pdf).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("rtex/pdfrs"),
            "expected /Producer rtex/pdfrs on primary path"
        );
    }

    #[test]
    fn pdf_backend_env_native_stamps_native_producer() {
        // SAFETY: test-only; serial test process, restored immediately after.
        unsafe {
            std::env::set_var("RTEX_PDF_BACKEND", "native");
        }
        let bytes = render_elements(sample_elements(), OutputFormat::Pdf).unwrap();
        unsafe {
            std::env::remove_var("RTEX_PDF_BACKEND");
        }
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("rtex/native"),
            "expected /Producer rtex/native when forced"
        );
    }
}
