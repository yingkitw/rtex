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
pub use pdfrs_pdf::{PdfBackend, PdfRenderOptions, render_pdf_bytes};

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
        OutputFormat::Pdf => render_pdf(elements, image_base),
        OutputFormat::Html => html::render(&elements),
        OutputFormat::Docx => docx::render(&elements),
        OutputFormat::Epub => epub::render(&elements),
    }
}

/// Render PDF using the pdfrs backend.
fn render_pdf(
    elements: Vec<TexElement>,
    image_base: Option<&std::path::Path>,
) -> Result<Vec<u8>, LatexError> {
    let options = PdfRenderOptions {
        image_base_dir: image_base.map(std::path::Path::to_path_buf),
        ..Default::default()
    };
    let bytes = render_pdf_bytes(&elements, options)?;
    Ok(bytes)
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
    fn pdf_uses_pdfrs_backend() {
        let bytes = render_elements(sample_elements(), OutputFormat::Pdf).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }

    #[test]
    fn pdf_backend_env_native_is_ignored_and_uses_pdfrs() {
        // SAFETY: This is test-only code that runs in a controlled environment.
        // The environment variable is set and restored within the same test,
        // ensuring no interference with other code.
        unsafe {
            std::env::set_var("RTEX_PDF_BACKEND", "native");
        }
        let bytes = render_elements(sample_elements(), OutputFormat::Pdf).unwrap();
        unsafe {
            std::env::remove_var("RTEX_PDF_BACKEND");
        }
        assert!(bytes.starts_with(b"%PDF"));
        assert!(!bytes.is_empty());
    }
}
