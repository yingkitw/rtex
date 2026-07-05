//! DOCX renderer for parsed LaTeX documents.

use std::io::Write;

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::LatexError;
use crate::parser::TexElement;

use super::common::{escape_xml, extract_metadata, plain_text_from_elements, render_elements_html};

pub fn render(elements: &[TexElement]) -> Result<Vec<u8>, LatexError> {
    let meta = extract_metadata(elements);
    let mut html_body = String::new();
    render_elements_html(elements, &mut html_body);
    let plain = plain_text_from_elements(elements);

    let title = meta.title.as_deref().unwrap_or("Document");
    let document_xml = build_document_xml(title, &plain, &html_body);
    let content_types = CONTENT_TYPES;
    let rels = PACKAGE_RELS;
    let doc_rels = DOCUMENT_RELS;

    let buffer = Vec::new();
    let cursor = std::io::Cursor::new(buffer);
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", options)
        .map_err(|e| docx_error(e.to_string()))?;
    zip.write_all(content_types.as_bytes())
        .map_err(|e| docx_error(e.to_string()))?;

    zip.start_file("_rels/.rels", options)
        .map_err(|e| docx_error(e.to_string()))?;
    zip.write_all(rels.as_bytes())
        .map_err(|e| docx_error(e.to_string()))?;

    zip.start_file("word/_rels/document.xml.rels", options)
        .map_err(|e| docx_error(e.to_string()))?;
    zip.write_all(doc_rels.as_bytes())
        .map_err(|e| docx_error(e.to_string()))?;

    zip.start_file("word/document.xml", options)
        .map_err(|e| docx_error(e.to_string()))?;
    zip.write_all(document_xml.as_bytes())
        .map_err(|e| docx_error(e.to_string()))?;

    let cursor = zip.finish().map_err(|e| docx_error(e.to_string()))?;
    Ok(cursor.into_inner())
}

fn docx_error(message: String) -> LatexError {
    LatexError::PdfError {
        message: format!("DOCX generation error: {message}"),
        context: None,
    }
}

fn build_document_xml(title: &str, plain: &str, html_fallback: &str) -> String {
    let body_source = if plain.trim().is_empty() {
        html_fallback
    } else {
        plain
    };

    let mut xml = String::from(DOCUMENT_HEADER);
    xml.push_str(&paragraph_xml(title, true));
    for paragraph in body_source.split("\n\n") {
        let line = paragraph.trim();
        if line.is_empty() {
            continue;
        }
        xml.push_str(&paragraph_xml(line, false));
    }
    xml.push_str(DOCUMENT_FOOTER);
    xml
}

fn paragraph_xml(text: &str, heading: bool) -> String {
    let escaped = escape_xml(text);
    if heading {
        format!(
            "<w:p><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>",
            escaped
        )
    } else {
        format!("<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>", escaped)
    }
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;

const PACKAGE_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#;

const DOCUMENT_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>"#;

const DOCUMENT_FOOTER: &str = r#"
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>
  </w:body>
</w:document>"#;
