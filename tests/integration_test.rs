use rtex::convert_tex_to_pdf;
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
