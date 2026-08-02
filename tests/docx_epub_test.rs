// Structural validation tests for DOCX and EPUB output formats.
// Verifies that the output is a valid zip archive with the expected
// internal file structure.

use rtex::{OutputFormat, convert_tex_string};
use std::io::Cursor;
use zip::ZipArchive;

fn render(tex: &str, format: OutputFormat) -> Vec<u8> {
    convert_tex_string(tex, format).expect("conversion should succeed")
}

fn doc(body: &str) -> String {
    format!(
        r#"\documentclass{{article}}
\begin{{document}}
{}
\end{{document}}"#,
        body
    )
}

fn assert_zip_contains(zip_bytes: &[u8], expected_file: &str, label: &str) {
    let cursor = Cursor::new(zip_bytes.to_vec());
    let archive = ZipArchive::new(cursor).unwrap_or_else(|_| panic!("{} should be a valid zip", label));
    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.file_names().nth(i).map(|n| n.to_string()))
        .collect();
    assert!(
        names.iter().any(|n| n == expected_file),
        "{} should contain file {:?}, found: {:?}",
        label,
        expected_file,
        names
    );
}

// ==================== DOCX Tests ====================

#[test]
fn docx_produces_valid_zip() {
    let bytes = render(&doc("Hello DOCX."), OutputFormat::Docx);
    assert!(!bytes.is_empty(), "DOCX output should not be empty");
    let cursor = Cursor::new(bytes);
    let archive = ZipArchive::new(cursor).expect("DOCX should be a valid zip");
    assert!(archive.len() >= 3, "DOCX should contain at least 3 files");
}

#[test]
fn docx_contains_content_types() {
    let bytes = render(&doc("Content types test."), OutputFormat::Docx);
    assert_zip_contains(&bytes, "[Content_Types].xml", "DOCX");
}

#[test]
fn docx_contains_document_xml() {
    let bytes = render(&doc("Document XML test."), OutputFormat::Docx);
    assert_zip_contains(&bytes, "word/document.xml", "DOCX");
}

#[test]
fn docx_contains_rels() {
    let bytes = render(&doc("Rels test."), OutputFormat::Docx);
    assert_zip_contains(&bytes, "_rels/.rels", "DOCX");
}

#[test]
fn docx_renders_text_content() {
    let bytes = render(&doc("Visible DOCX text."), OutputFormat::Docx);
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("DOCX should be valid zip");
    let mut doc_xml = archive
        .by_name("word/document.xml")
        .expect("document.xml should exist");
    let mut content = String::new();
    std::io::Read::read_to_string(&mut doc_xml, &mut content).unwrap();
    assert!(content.contains("Visible DOCX text."), "DOCX should contain body text");
}

// ==================== EPUB Tests ====================

#[test]
fn epub_produces_valid_zip() {
    let bytes = render(&doc("Hello EPUB."), OutputFormat::Epub);
    assert!(!bytes.is_empty(), "EPUB output should not be empty");
    let cursor = Cursor::new(bytes);
    let archive = ZipArchive::new(cursor).expect("EPUB should be a valid zip");
    assert!(archive.len() >= 3, "EPUB should contain at least 3 files");
}

#[test]
fn epub_contains_mimetype() {
    let bytes = render(&doc("Mimetype test."), OutputFormat::Epub);
    assert_zip_contains(&bytes, "mimetype", "EPUB");
}

#[test]
fn epub_contains_opf() {
    let bytes = render(&doc("OPF test."), OutputFormat::Epub);
    let cursor = Cursor::new(bytes);
    let archive = ZipArchive::new(cursor).expect("EPUB should be valid zip");
    let has_opf = (0..archive.len())
        .filter_map(|i| archive.file_names().nth(i).map(|n| n.to_string()))
        .any(|n| n.ends_with(".opf"));
    assert!(has_opf, "EPUB should contain an .opf file");
}

#[test]
fn epub_contains_xhtml() {
    let bytes = render(&doc("XHTML content test."), OutputFormat::Epub);
    let cursor = Cursor::new(bytes);
    let archive = ZipArchive::new(cursor).expect("EPUB should be valid zip");
    let has_xhtml = (0..archive.len())
        .filter_map(|i| archive.file_names().nth(i).map(|n| n.to_string()))
        .any(|n| n.ends_with(".xhtml") || n.ends_with(".html"));
    assert!(has_xhtml, "EPUB should contain an XHTML/HTML file");
}

#[test]
fn epub_renders_text_content() {
    let bytes = render(&doc("Visible EPUB text."), OutputFormat::Epub);
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).expect("EPUB should be valid zip");
    // Search all XHTML/HTML files for the body text
    for i in 0..archive.len() {
        let name = archive.file_names().nth(i).map(|n| n.to_string()).unwrap_or_default();
        if name.ends_with(".xhtml") || name.ends_with(".html") {
            let mut file = archive.by_index(i).unwrap();
            let mut content = String::new();
            std::io::Read::read_to_string(&mut file, &mut content).unwrap();
            if content.contains("Visible EPUB text.") {
                return;
            }
        }
    }
    panic!("No XHTML/HTML file in EPUB contained the expected body text");
}
