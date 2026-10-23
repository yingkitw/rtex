use rtex::{OutputFormat, convert_tex_string, convert_tex_to_pdf};
use std::fs;
use std::path::PathBuf;

#[test]
fn generate_example_pdfs() {
    let output_dir = PathBuf::from("output");
    fs::create_dir_all(&output_dir).unwrap();

    let examples = [
        ("examples/minimal.tex", "output/minimal.pdf"),
        ("examples/sample.tex", "output/sample.pdf"),
        ("examples/math.tex", "output/math.pdf"),
        ("examples/table.tex", "output/table.pdf"),
        ("examples/lists.tex", "output/lists.pdf"),
        ("examples/code.tex", "output/code.pdf"),
        ("examples/advanced_math.tex", "output/advanced_math.pdf"),
        ("examples/simple_test.tex", "output/simple_test.pdf"),
        ("examples/features.tex", "output/features.pdf"),
        ("examples/quadratic.tex", "output/quadratic.pdf"),
    ];

    for (input, output) in examples {
        let input_path = PathBuf::from(input);
        let output_path = PathBuf::from(output);
        assert!(
            input_path.exists(),
            "missing example fixture: {}",
            input_path.display()
        );
        convert_tex_to_pdf(&input_path, &output_path).unwrap_or_else(|e| {
            panic!("Failed to generate {}: {}", output_path.display(), e);
        });
        let bytes = fs::read(&output_path).expect("read generated PDF");
        assert!(
            bytes.starts_with(b"%PDF"),
            "{} missing PDF header",
            output_path.display()
        );
        assert!(
            bytes.len() > 100,
            "{} too small ({} bytes)",
            output_path.display(),
            bytes.len()
        );
    }
}

#[test]
fn multi_format_emits_all_backends() {
    let source = r"\documentclass{article}\begin{document}Hello.\end{document}";
    let html = convert_tex_string(source, OutputFormat::Html).expect("html");
    let docx = convert_tex_string(source, OutputFormat::Docx).expect("docx");
    let epub = convert_tex_string(source, OutputFormat::Epub).expect("epub");
    let pdf = convert_tex_string(source, OutputFormat::Pdf).expect("pdf");

    assert!(html.starts_with(b"<!DOCTYPE") || html.starts_with(b"<html"));
    assert!(docx.starts_with(b"PK")); // zip
    assert!(epub.starts_with(b"PK")); // zip
    assert!(pdf.starts_with(b"%PDF"));

    assert!(String::from_utf8_lossy(&html).contains("Hello"));
    assert!(docx.len() > 200);
    assert!(epub.len() > 200);
}