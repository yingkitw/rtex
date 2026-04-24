use latex_rs::convert_tex_to_pdf;
use std::fs;
use std::path::PathBuf;

#[test]
fn generate_example_pdfs() {
    let output_dir = PathBuf::from("output");
    fs::create_dir_all(&output_dir).unwrap();

    let examples = vec![
        ("examples/minimal.tex", "output/minimal.pdf"),
        ("examples/sample.tex", "output/sample.pdf"),
        ("examples/math.tex", "output/math.pdf"),
        ("examples/table.tex", "output/table.pdf"),
        ("examples/lists.tex", "output/lists.pdf"),
        ("examples/code.tex", "output/code.pdf"),
    ];

    for (input, output) in examples {
        let input_path = PathBuf::from(input);
        let output_path = PathBuf::from(output);

        if input_path.exists() {
            match convert_tex_to_pdf(&input_path, &output_path) {
                Ok(_) => println!("Generated: {}", output),
                Err(e) => eprintln!("Failed to generate {}: {}", output, e),
            }
        }
    }

    println!("\nPDFs generated in the output/ directory");
}
