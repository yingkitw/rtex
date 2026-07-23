//! EPUB renderer for parsed LaTeX documents.

use std::io::Write;

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::error::LatexError;
use crate::parser::TexElement;

use super::common::{escape_html, extract_metadata};
use super::html;

pub fn render(elements: &[TexElement]) -> Result<Vec<u8>, LatexError> {
    let meta = extract_metadata(elements);
    let title = meta.title.as_deref().unwrap_or("Document");
    let xhtml = html::render(elements)?;
    let xhtml_str = String::from_utf8_lossy(&xhtml);

    let opf = build_opf(title, &meta.author, &meta.date);
    let nav = build_nav(title);

    let buffer = Vec::new();
    let cursor = std::io::Cursor::new(buffer);
    let mut zip = ZipWriter::new(cursor);

    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflated =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)
        .map_err(|e| epub_error(e.to_string()))?;
    zip.write_all(b"application/epub+zip")
        .map_err(|e| epub_error(e.to_string()))?;

    zip.start_file("META-INF/container.xml", deflated)
        .map_err(|e| epub_error(e.to_string()))?;
    zip.write_all(CONTAINER_XML.as_bytes())
        .map_err(|e| epub_error(e.to_string()))?;

    zip.start_file("OEBPS/content.opf", deflated)
        .map_err(|e| epub_error(e.to_string()))?;
    zip.write_all(opf.as_bytes())
        .map_err(|e| epub_error(e.to_string()))?;

    zip.start_file("OEBPS/nav.xhtml", deflated)
        .map_err(|e| epub_error(e.to_string()))?;
    zip.write_all(nav.as_bytes())
        .map_err(|e| epub_error(e.to_string()))?;

    zip.start_file("OEBPS/chapter.xhtml", deflated)
        .map_err(|e| epub_error(e.to_string()))?;
    zip.write_all(xhtml_str.as_bytes())
        .map_err(|e| epub_error(e.to_string()))?;

    let cursor = zip.finish().map_err(|e| epub_error(e.to_string()))?;
    Ok(cursor.into_inner())
}

fn epub_error(message: String) -> LatexError {
    LatexError::PdfError {
        message: format!("EPUB generation error: {message}"),
        context: None,
    }
}

fn build_opf(title: &str, author: &Option<String>, date: &Option<String>) -> String {
    let creator = author
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| "rtex".to_string());
    let modified = date.as_deref().unwrap_or("2026-01-01");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">urn:uuid:rtex-doc</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{creator}</dc:creator>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">{modified}T00:00:00Z</meta>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter"/>
  </spine>
</package>"#,
        title = escape_html(title),
        creator = creator,
        modified = escape_html(modified),
    )
}

fn build_nav(title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Navigation</title></head>
<body>
  <nav epub:type="toc" id="toc"><ol><li><a href="chapter.xhtml">{title}</a></li></ol></nav>
</body>
</html>"#,
        title = escape_html(title),
    )
}

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
