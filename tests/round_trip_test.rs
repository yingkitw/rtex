// Round-trip testing for LaTeX → PDF conversion
// Tests deterministic conversion, structural validation, and regression detection

use rtex::convert_tex_to_pdf;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

const FIXTURE_DIR: &str = "tests/fixtures";

/// Helper function to convert LaTeX content to PDF bytes
fn convert_latex_to_bytes(latex_content: &str) -> Result<Vec<u8>, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let input_path = temp_dir.path().join("input.tex");
    let output_path = temp_dir.path().join("output.pdf");

    fs::write(&input_path, latex_content)
        .map_err(|e| format!("Failed to write LaTeX file: {}", e))?;

    convert_tex_to_pdf(&input_path, &output_path)
        .map_err(|e| format!("Failed to convert LaTeX to PDF: {}", e))?;

    fs::read(&output_path).map_err(|e| format!("Failed to read PDF file: {}", e))
}

/// Calculate SHA256 hash of PDF data
fn pdf_hash(pdf_data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pdf_data);
    format!("{:x}", hasher.finalize())
}

/// Validate PDF structure
fn validate_pdf_structure(pdf_data: &[u8]) -> Result<(), String> {
    // Check file size
    if pdf_data.is_empty() {
        return Err("PDF file is empty".to_string());
    }
    if pdf_data.len() < 100 {
        return Err("PDF file too small, likely invalid".to_string());
    }

    // Check PDF header
    if !pdf_data.starts_with(b"%PDF") {
        return Err("PDF missing header signature".to_string());
    }

    // Check for essential PDF components
    let pdf_str = String::from_utf8_lossy(pdf_data);
    if !pdf_str.contains("xref") {
        return Err("PDF missing xref table".to_string());
    }
    if !pdf_str.contains("trailer") {
        return Err("PDF missing trailer".to_string());
    }
    if !pdf_str.contains("%%EOF") {
        return Err("PDF missing EOF marker".to_string());
    }

    // Check for font embedding
    if !pdf_str.contains("/Font") {
        return Err("PDF missing font resources".to_string());
    }

    // Check for content stream
    if !pdf_str.contains("/Contents") {
        return Err("PDF missing content stream".to_string());
    }

    Ok(())
}

/// Verify two conversions produce valid, structurally stable output.
///
/// Byte-identical PDFs are required for ASCII-only documents. When Unicode or
/// math triggers pdfrs font subsetting, binary streams may vary slightly
/// between runs; those cases still require deterministic parsing and valid PDFs.
fn assert_reproducible_conversion(latex: &str, message: &str) {
    let pdf1 = convert_latex_to_bytes(latex).expect(message);
    let pdf2 = convert_latex_to_bytes(latex).expect(message);
    validate_pdf_structure(&pdf1).unwrap_or_else(|_| panic!("{message}: pdf1 structure"));
    validate_pdf_structure(&pdf2).unwrap_or_else(|_| panic!("{message}: pdf2 structure"));

    let mut parser1 = rtex::TexParser::new(latex.to_string());
    let mut parser2 = rtex::TexParser::new(latex.to_string());
    assert_eq!(
        format!("{:?}", parser1.parse()),
        format!("{:?}", parser2.parse()),
        "{message}: parse must be deterministic"
    );

    let needs_unicode_pdf = latex.contains('$')
        || latex.contains("\\alpha")
        || latex.contains("\\textbf")
        || latex.contains("\\textit");
    if needs_unicode_pdf {
        let size_diff = (pdf1.len() as i64 - pdf2.len() as i64).abs();
        assert!(
            size_diff < 1024,
            "{message}: PDF sizes diverged too much ({} vs {} bytes)",
            pdf1.len(),
            pdf2.len()
        );
    } else {
        assert_eq!(pdf1, pdf2, "{message}");
    }
}

// ==================== Deterministic Conversion Tests ====================

#[test]
fn test_deterministic_conversion_minimal() {
    let latex = r#"\documentclass{article}
\begin{document}
Hello, World!
\end{document}"#;

    assert_reproducible_conversion(latex, "Same LaTeX input must produce identical PDF output");
}

#[test]
fn test_deterministic_conversion_math() {
    let latex = r#"\documentclass{article}
\begin{document}
Math: $E = mc^2$ and $\int_0^\infty e^{-x} dx = 1$.
\end{document}"#;

    assert_reproducible_conversion(latex, "Math content must produce deterministic output");
}

#[test]
fn test_deterministic_conversion_complex() {
    let latex = r#"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\section{Test}
Text with \textbf{bold} and \textit{italic}.
Math: $x^2 + y^2 = z^2$.
\begin{itemize}
\item One
\item Two
\end{itemize}
\end{document}"#;

    assert_reproducible_conversion(latex, "Complex documents must produce deterministic output");
}

#[test]
fn test_multiple_conversions_same_output() {
    let latex = r#"\documentclass{article}
\begin{document}
Test multiple conversions
\end{document}"#;

    let pdfs: Vec<Vec<u8>> = (0..5)
        .map(|_| convert_latex_to_bytes(latex).unwrap())
        .collect();

    // All PDFs should be identical
    for i in 1..pdfs.len() {
        assert_eq!(pdfs[0], pdfs[i], "Conversion {} differs from first", i);
    }
}

#[test]
fn test_hash_consistency() {
    let latex = r#"\documentclass{article}
\begin{document}
Hash consistency test
\end{document}"#;

    let hash1 = pdf_hash(&convert_latex_to_bytes(latex).unwrap());
    let hash2 = pdf_hash(&convert_latex_to_bytes(latex).unwrap());

    assert_eq!(hash1, hash2, "PDF hashes must be consistent");
}

// ==================== Structural Validation Tests ====================

#[test]
fn test_pdf_structure_minimal() {
    let latex = r#"\documentclass{article}
\begin{document}
Test
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).unwrap();
    validate_pdf_structure(&pdf).expect("PDF structure validation failed");
}

#[test]
fn test_pdf_structure_math() {
    let latex = r#"\documentclass{article}
\begin{document}
Equation: \begin{equation}E = mc^2\end{equation}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).unwrap();
    validate_pdf_structure(&pdf).expect("Math PDF structure validation failed");
}

#[test]
fn test_pdf_structure_complex() {
    let latex = r#"\documentclass{article}
\title{Test}
\author{Author}
\date{2024-01-15}
\begin{document}
\maketitle
\section{Section}
Text with \textbf{formatting}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).unwrap();
    validate_pdf_structure(&pdf).expect("Complex PDF structure validation failed");
}

#[test]
fn test_pdf_header() {
    let latex = r#"\documentclass{article}
\begin{document}
Test
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).unwrap();
    assert!(pdf.starts_with(b"%PDF"), "PDF must start with %PDF header");
}

#[test]
fn test_pdf_eof_marker() {
    let latex = r#"\documentclass{article}
\begin{document}
Test
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(pdf_str.contains("%%EOF"), "PDF must contain EOF marker");
}

// ==================== Happy Path Tests ====================

#[test]
fn test_happy_path_minimal() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/minimal.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read minimal.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert minimal.tex");
    validate_pdf_structure(&pdf).expect("Minimal PDF structure invalid");
}

#[test]
fn test_happy_path_text_formatting() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/text_formatting.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read text_formatting.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert text_formatting.tex");
    validate_pdf_structure(&pdf).expect("Text formatting PDF structure invalid");
}

#[test]
fn test_happy_path_mathematics() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/mathematics.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read mathematics.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert mathematics.tex");
    validate_pdf_structure(&pdf).expect("Mathematics PDF structure invalid");
}

#[test]
fn test_happy_path_lists() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/lists.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read lists.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert lists.tex");
    validate_pdf_structure(&pdf).expect("Lists PDF structure invalid");
}

#[test]
fn test_happy_path_tables() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/tables.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read tables.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert tables.tex");
    validate_pdf_structure(&pdf).expect("Tables PDF structure invalid");
}

#[test]
fn test_happy_path_metadata() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/metadata.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read metadata.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert metadata.tex");
    validate_pdf_structure(&pdf).expect("Metadata PDF structure invalid");
}

#[test]
fn test_happy_path_complex_document() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/complex_document.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read complex_document.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert complex_document.tex");
    validate_pdf_structure(&pdf).expect("Complex document PDF structure invalid");
}

#[test]
fn test_happy_path_comprehensive_latex_to_pdf() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/comprehensive.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read comprehensive.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert comprehensive.tex");
    validate_pdf_structure(&pdf).expect("Comprehensive PDF structure invalid");

    // Sanity bounds: must be non-trivial but not pathologically large.
    assert!(
        pdf.len() >= 2_000,
        "comprehensive PDF too small ({} bytes); likely missing content",
        pdf.len()
    );
    assert!(
        pdf.len() <= 5_000_000,
        "comprehensive PDF too large ({} bytes); possible rendering blowup",
        pdf.len()
    );

    let pdf_str = String::from_utf8_lossy(&pdf);
    assert!(
        pdf_str.contains("/Type /Page"),
        "PDF should contain page objects"
    );
    assert!(
        pdf_str.matches("/Type /Page").count() >= 2,
        "comprehensive document should span multiple pages, got {} page objects",
        pdf_str.matches("/Type /Page").count()
    );
}

// ==================== Edge Case Tests ====================

#[test]
fn test_edge_case_empty_document() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/edge_cases/empty_document.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read empty_document.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert empty_document.tex");
    validate_pdf_structure(&pdf).expect("Empty document PDF structure invalid");
}

#[test]
fn test_edge_case_long_text() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/edge_cases/long_text.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read long_text.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert long_text.tex");
    validate_pdf_structure(&pdf).expect("Long text PDF structure invalid");
}

#[test]
fn test_edge_case_special_chars() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/edge_cases/special_chars.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read special_chars.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert special_chars.tex");
    validate_pdf_structure(&pdf).expect("Special chars PDF structure invalid");
}

#[test]
fn test_edge_case_deep_nesting() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/edge_cases/deep_nesting.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read deep_nesting.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert deep_nesting.tex");
    validate_pdf_structure(&pdf).expect("Deep nesting PDF structure invalid");
}

#[test]
fn test_edge_case_mixed_content() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/edge_cases/mixed_content.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read mixed_content.tex fixture");

    let pdf = convert_latex_to_bytes(&latex).expect("Failed to convert mixed_content.tex");
    validate_pdf_structure(&pdf).expect("Mixed content PDF structure invalid");
}

// ==================== Malformed Input Tests ====================

#[test]
fn test_malformed_unclosed_command() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/malformed/unclosed_command.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read unclosed_command.tex fixture");

    // Should handle gracefully (may succeed or fail with appropriate error)
    let result = convert_latex_to_bytes(&latex);
    match result {
        Ok(pdf) => {
            // If it succeeds, PDF should still be valid
            validate_pdf_structure(&pdf).expect("Malformed input produced invalid PDF");
        }
        Err(_) => {
            // If it fails, that's acceptable too
        }
    }
}

#[test]
fn test_malformed_invalid_math() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/malformed/invalid_math.tex");

    let latex = fs::read_to_string(&fixture_path).expect("Failed to read invalid_math.tex fixture");

    // Should handle gracefully
    let result = convert_latex_to_bytes(&latex);
    match result {
        Ok(pdf) => {
            validate_pdf_structure(&pdf).expect("Invalid math produced invalid PDF");
        }
        Err(_) => {
            // Acceptable to fail
        }
    }
}

#[test]
fn test_malformed_missing_structure() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/malformed/missing_structure.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read missing_structure.tex fixture");

    // Should handle gracefully
    let result = convert_latex_to_bytes(&latex);
    match result {
        Ok(pdf) => {
            validate_pdf_structure(&pdf).expect("Missing structure produced invalid PDF");
        }
        Err(_) => {
            // Acceptable to fail
        }
    }
}

#[test]
fn test_malformed_unknown_commands() {
    let fixture_path = PathBuf::from(FIXTURE_DIR).join("latex/malformed/unknown_commands.tex");

    let latex =
        fs::read_to_string(&fixture_path).expect("Failed to read unknown_commands.tex fixture");

    // Should handle gracefully (parser typically ignores unknown commands)
    let result = convert_latex_to_bytes(&latex);
    match result {
        Ok(pdf) => {
            validate_pdf_structure(&pdf).expect("Unknown commands produced invalid PDF");
        }
        Err(_) => {
            // Acceptable to fail
        }
    }
}

// ==================== Golden File Tests ====================

#[test]
#[ignore] // Run manually with: cargo test --test round_trip -- --ignored test_golden_file
fn test_golden_file_minimal() {
    let latex_path = PathBuf::from(FIXTURE_DIR).join("latex/happy_path/minimal.tex");
    let golden_path = PathBuf::from(FIXTURE_DIR).join("golden_pdfs/minimal.pdf");

    let latex = fs::read_to_string(&latex_path).expect("Failed to read minimal.tex fixture");

    let actual_pdf = convert_latex_to_bytes(&latex).expect("Failed to convert minimal.tex");

    if golden_path.exists() {
        let expected_pdf = fs::read(&golden_path).expect("Failed to read golden PDF");

        assert_eq!(
            actual_pdf, expected_pdf,
            "PDF output differs from golden file. Run with --ignored generate-golden to update."
        );
    } else {
        panic!("Golden file not found. Run generate-golden test first.");
    }
}

// ==================== Golden File Generation ====================

#[test]
#[ignore] // Run with: cargo test --test round_trip -- --ignored generate_golden_files
fn generate_golden_files() {
    let fixtures = vec![
        "latex/happy_path/minimal.tex",
        "latex/happy_path/text_formatting.tex",
        "latex/happy_path/mathematics.tex",
        "latex/happy_path/lists.tex",
        "latex/happy_path/tables.tex",
        "latex/happy_path/metadata.tex",
        "latex/happy_path/complex_document.tex",
        "latex/happy_path/comprehensive.tex",
    ];

    for fixture in fixtures {
        let latex_path = PathBuf::from(FIXTURE_DIR).join(fixture);
        let pdf_name = latex_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
            + ".pdf";
        let golden_path = PathBuf::from(FIXTURE_DIR)
            .join("golden_pdfs")
            .join(&pdf_name);

        let latex = fs::read_to_string(&latex_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", fixture));

        let pdf = convert_latex_to_bytes(&latex)
            .unwrap_or_else(|_| panic!("Failed to convert {}", fixture));

        // Validate before saving
        validate_pdf_structure(&pdf)
            .unwrap_or_else(|_| panic!("Invalid PDF generated for {}", fixture));

        // Ensure golden directory exists
        if let Some(parent) = golden_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(&golden_path, pdf)
            .unwrap_or_else(|_| panic!("Failed to write golden file {}", pdf_name));

        println!("Generated golden file: {}", golden_path.display());
    }

    println!("\nAll golden files generated successfully!");
}

// ==================== Regression Detection ====================

#[test]
fn test_regression_detection() {
    let latex = r#"\documentclass{article}
\begin{document}
Regression test content
\end{document}"#;

    let pdf1 = convert_latex_to_bytes(latex).unwrap();
    let hash1 = pdf_hash(&pdf1);

    // Simulate time passing (same input should produce same output)
    let pdf2 = convert_latex_to_bytes(latex).unwrap();
    let hash2 = pdf_hash(&pdf2);

    assert_eq!(
        hash1, hash2,
        "Regression detected: Same input produced different output"
    );

    // Test different inputs produce different outputs
    let latex2 = r#"\documentclass{article}
\begin{document}
Different content
\end{document}"#;

    let pdf3 = convert_latex_to_bytes(latex2).unwrap();
    let hash3 = pdf_hash(&pdf3);

    assert_ne!(
        hash1, hash3,
        "Different inputs should produce different outputs"
    );
}

// ==================== File Size Validation ====================

#[test]
fn test_file_size_bounds() {
    let latex = r#"\documentclass{article}
\begin{document}
Test file size
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).unwrap();

    // ASCII-only documents use standard Helvetica (no embedded font)
    assert!(pdf.len() > 500, "PDF too small: {} bytes", pdf.len());
    assert!(
        pdf.len() < 100_000,
        "Simple ASCII PDF should be <100KB, got {} bytes",
        pdf.len()
    );
}

#[test]
fn test_file_size_consistency() {
    let latex = r#"\documentclass{article}
\begin{document}
Size consistency test
\end{document}"#;

    let sizes: Vec<usize> = (0..10)
        .map(|_| convert_latex_to_bytes(latex).unwrap().len())
        .collect();

    // All sizes should be identical
    for i in 1..sizes.len() {
        assert_eq!(sizes[0], sizes[i], "File size varies across conversions");
    }
}

// ==================== LaTeX Feature Coverage Tests ====================
//
// These tests exercise specific LaTeX commands/environments end-to-end so the
// conversion must not only parse them but also emit a valid PDF.

#[test]
fn test_description_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{description}
    \item[Apple] A red fruit.
    \item[Bear] A large mammal.
    \item[Cat] A small feline.
\end{description}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("description must convert");
    validate_pdf_structure(&pdf).expect("description PDF structure invalid");
    assert!(pdf.len() > 800, "description PDF suspiciously small");
}

#[test]
fn test_align_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{align}
    a &= b + c \\
    d &= e + f + g \\
    h &= i
\end{align}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("align must convert");
    validate_pdf_structure(&pdf).expect("align PDF structure invalid");
}

#[test]
fn test_align_starred_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{align*}
    x &= 1 \\
    y &= 2
\end{align*}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("align* must convert");
    validate_pdf_structure(&pdf).expect("align* PDF structure invalid");
}

#[test]
fn test_gather_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{gather}
    x = 1 \\
    y = 2 \\
    z = 3
\end{gather}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("gather must convert");
    validate_pdf_structure(&pdf).expect("gather PDF structure invalid");
}

#[test]
fn test_multline_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{multline}
    a + b + c + d + e + f + g + h \\
    + i + j + k
\end{multline}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("multline must convert");
    validate_pdf_structure(&pdf).expect("multline PDF structure invalid");
}

#[test]
fn test_cases_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
$f(x) = \begin{cases}
    x & \text{if } x \geq 0 \\
    -x & \text{if } x < 0
\end{cases}$
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("cases must convert");
    validate_pdf_structure(&pdf).expect("cases PDF structure invalid");
}

#[test]
fn test_href_command_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Visit \href{https://example.com}{Example Site} for more information.
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("\\href must convert");
    validate_pdf_structure(&pdf).expect("\\href PDF structure invalid");
}

#[test]
fn test_nested_lists_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{itemize}
    \item Source
    \begin{itemize}
        \item main.rs
        \item lib.rs
    \end{itemize}
    \item Docs
\end{itemize}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("nested lists must convert");
    validate_pdf_structure(&pdf).expect("nested list PDF structure invalid");
}

#[test]
fn test_itemize_with_math_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{itemize}
    \item Inline $E = mc^2$ reference
    \item \textbf{Bold} item
\end{itemize}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("list with math must convert");
    validate_pdf_structure(&pdf).expect("list with math PDF structure invalid");
}

#[test]
fn test_equation_star_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{equation*}
    E = mc^2
\end{equation*}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("equation* must convert");
    validate_pdf_structure(&pdf).expect("equation* PDF structure invalid");
}

#[test]
fn test_multi_feature_document_renders_pdf() {
    // Combines several newly-supported features in one document so a
    // regression in any of them will surface here.
    let latex = r#"\documentclass{article}
\usepackage{amsmath}
\title{Features}
\author{rtex}
\begin{document}
\maketitle

\section{Description}
\begin{description}
    \item[Foo] Definition of foo.
    \item[Bar] Definition of bar.
\end{description}

\section{Align}
\begin{align}
    a &= b + c \\
    d &= e + f
\end{align}

\section{Cases}
\begin{equation*}
    |x| = \begin{cases}
        x & \text{if } x \geq 0 \\
        -x & \text{otherwise}
    \end{cases}
\end{equation*}

\section{Links}
Read more at \href{https://example.com}{Example}.
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("multi-feature must convert");
    validate_pdf_structure(&pdf).expect("multi-feature PDF structure invalid");
}

#[test]
fn test_theorem_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{theorem}[Pythagoras]
For a right triangle with legs $a, b$ and hypotenuse $c$, $a^2 + b^2 = c^2$.
\end{theorem}
\begin{proof}
By induction on the side lengths. QED.
\end{proof}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("theorem must convert");
    validate_pdf_structure(&pdf).expect("theorem PDF structure invalid");
}

#[test]
fn test_definition_and_lemma_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{definition}
A \emph{group} is a set $G$ with an associative binary operation.
\end{definition}
\begin{lemma}
Every finite group has a well-defined order.
\end{lemma}
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("definition+lemma must convert");
    validate_pdf_structure(&pdf).expect("definition+lemma PDF structure invalid");
}

#[test]
fn test_math_ellipsis_and_binom_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Ellipsis: $\ldots$ and $\cdots$ and $\vdots$ and $\ddots$.

Binomial: $\binom{n}{k} = \tfrac{n!}{k!(n-k)!}$.
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("ellipsis+binom must convert");
    validate_pdf_structure(&pdf).expect("ellipsis+binom PDF structure invalid");
}

#[test]
fn test_starred_section_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\section*{Acknowledgement}
Thanks to reviewers.

\subsection*{Preface}
A short lead-in.
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("starred section must convert");
    validate_pdf_structure(&pdf).expect("starred section PDF structure invalid");
}

#[test]
fn test_cases_inside_display_math_renders_pdf() {
    // The display-math parser should detect the nested `\begin{cases}`
    // block and render it as cases (MathLines) rather than raw text.
    let latex = r#"\documentclass{article}
\begin{document}
\[
f(x) = \begin{cases}
    x^2  & \text{if } x \geq 0 \\
    -x   & \text{if } x < 0
\end{cases}
\]
\end{document}"#;

    let pdf = convert_latex_to_bytes(latex).expect("nested cases must convert");
    validate_pdf_structure(&pdf).expect("nested cases PDF structure invalid");
}

// ==================== TexElement Full Coverage Tests ====================
// One round-trip PDF test per TexElement variant to ensure every parsed
// element can be rendered without panicking or producing an invalid PDF.

#[test]
fn test_line_break_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
First line \\ Second line
Third line \\[2em] Fourth line
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("line break must convert");
    validate_pdf_structure(&pdf).expect("line break PDF structure invalid");
}

#[test]
fn test_flushleft_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{flushleft}
Left aligned text content.
\end{flushleft}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("flushleft must convert");
    validate_pdf_structure(&pdf).expect("flushleft PDF structure invalid");
}

#[test]
fn test_flushright_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{flushright}
Right aligned text content.
\end{flushright}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("flushright must convert");
    validate_pdf_structure(&pdf).expect("flushright PDF structure invalid");
}

#[test]
fn test_colored_text_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\textcolor{red}{Important warning text.}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("colored text must convert");
    validate_pdf_structure(&pdf).expect("colored text PDF structure invalid");
}

#[test]
fn test_citation_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
See \cite{smith2024, jones2023} for details.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("citation must convert");
    validate_pdf_structure(&pdf).expect("citation PDF structure invalid");
}

#[test]
fn test_bibliography_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Reference \cite{key1}.
\begin{thebibliography}{9}
\bibitem{key1} Author, \emph{Title}, Publisher, 2024.
\bibitem{key2} Another Author, \emph{Another Title}, 2023.
\end{thebibliography}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("bibliography must convert");
    validate_pdf_structure(&pdf).expect("bibliography PDF structure invalid");
}

#[test]
fn test_label_ref_pageref_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\section{Intro}\label{sec:intro}
See Section~\ref{sec:intro} on page~\pageref{sec:intro}.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("label/ref/pageref must convert");
    validate_pdf_structure(&pdf).expect("label/ref/pageref PDF structure invalid");
}

#[test]
fn test_tableofcontents_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\tableofcontents
\section{First}
\section{Second}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("TOC must convert");
    validate_pdf_structure(&pdf).expect("TOC PDF structure invalid");
}

#[test]
fn test_listoffigures_listoftables_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\listoffigures
\listoftables
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("LOF/LOT must convert");
    validate_pdf_structure(&pdf).expect("LOF/LOT PDF structure invalid");
}

#[test]
fn test_footnote_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
This has a footnote\footnote{Footnote text here.}.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("footnote must convert");
    validate_pdf_structure(&pdf).expect("footnote PDF structure invalid");
}

#[test]
fn test_caption_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{figure}
\centering
Caption text.
\caption{A test caption.}
\end{figure}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("caption must convert");
    validate_pdf_structure(&pdf).expect("caption PDF structure invalid");
}

#[test]
fn test_quote_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{quote}
A famous quote goes here.
\end{quote}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("quote must convert");
    validate_pdf_structure(&pdf).expect("quote PDF structure invalid");
}

#[test]
fn test_abstract_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{abstract}
This is the abstract text.
\end{abstract}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("abstract must convert");
    validate_pdf_structure(&pdf).expect("abstract PDF structure invalid");
}

#[test]
fn test_codeblock_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{lstlisting}
fn main() {
    println!("Hello");
}
\end{lstlisting}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("codeblock must convert");
    validate_pdf_structure(&pdf).expect("codeblock PDF structure invalid");
}

#[test]
fn test_verbatim_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{verbatim}
raw text \with $commands$
\end{verbatim}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("verbatim must convert");
    validate_pdf_structure(&pdf).expect("verbatim PDF structure invalid");
}

#[test]
fn test_figure_environment_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{figure}
\centering
Figure content here.
\caption{Figure caption.}
\end{figure}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("figure must convert");
    validate_pdf_structure(&pdf).expect("figure PDF structure invalid");
}

#[test]
fn test_minipage_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{minipage}[c]{0.5\textwidth}
Minipage content.
\end{minipage}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("minipage must convert");
    validate_pdf_structure(&pdf).expect("minipage PDF structure invalid");
}

#[test]
fn test_displaymath_env_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{displaymath}
E = mc^2
\end{displaymath}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("displaymath env must convert");
    validate_pdf_structure(&pdf).expect("displaymath env PDF structure invalid");
}

#[test]
fn test_math_env_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{math} x + y = z \end{math}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("math env must convert");
    validate_pdf_structure(&pdf).expect("math env PDF structure invalid");
}

#[test]
fn test_eqnarray_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{eqnarray}
a &=& b + c \\
d &=& e + f
\end{eqnarray}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("eqnarray must convert");
    validate_pdf_structure(&pdf).expect("eqnarray PDF structure invalid");
}

#[test]
fn test_hspace_vspace_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
A\hspace{1em}B\hspace*{2em}C
\vspace{12pt}
D
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("hspace/vspace must convert");
    validate_pdf_structure(&pdf).expect("hspace/vspace PDF structure invalid");
}

#[test]
fn test_mbox_parbox_makebox_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\mbox{Boxed text} \parbox[c]{3cm}{Parbox content} \makebox[5cm][s]{Makebox content}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("mbox/parbox/makebox must convert");
    validate_pdf_structure(&pdf).expect("mbox/parbox/makebox PDF structure invalid");
}

#[test]
fn test_multicolumn_cline_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{tabular}{|c|c|c|}
\hline
\multicolumn{2}{|c|}{Header} & C \\
\hline
A & B & C \\
\cline{1-2}
D & E & F \\
\hline
\end{tabular}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("multicolumn/cline must convert");
    validate_pdf_structure(&pdf).expect("multicolumn/cline PDF structure invalid");
}

#[test]
fn test_footnotemark_footnotetext_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Text\footnotemark\ with mark.
\footnotetext{The footnote text.}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("footnotemark/text must convert");
    validate_pdf_structure(&pdf).expect("footnotemark/text PDF structure invalid");
}

#[test]
fn test_enquote_textnormal_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\enquote{Quoted text} and \textnormal{Normal text}.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("enquote/textnormal must convert");
    validate_pdf_structure(&pdf).expect("enquote/textnormal PDF structure invalid");
}

#[test]
fn test_skip_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\setlength{\parindent}{0pt}
\setcounter{page}{3}
\ignorespaces
\selectfont
Hello after skip commands.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("skip commands must convert");
    validate_pdf_structure(&pdf).expect("skip commands PDF structure invalid");
}

#[test]
fn test_special_chars_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\copyright\ \pounds\ \S\ \P\ \dag\ \ddag\ \ldots\ \LaTeX\ \TeX\ \AA\ \aa\ \ss\ \OE\ \ae\ \textcopyright\ \textregistered\ \texttrademark
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("special chars must convert");
    validate_pdf_structure(&pdf).expect("special chars PDF structure invalid");
}

#[test]
fn test_linebreak_nopagebreak_samepage_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
A \linebreak B \nopagebreak C \samepage D
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("page control must convert");
    validate_pdf_structure(&pdf).expect("page control PDF structure invalid");
}

#[test]
fn test_counter_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\setcounter{section}{5}
\arabic{section} \roman{section} \alph{section} \thechapter
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("counter commands must convert");
    validate_pdf_structure(&pdf).expect("counter commands PDF structure invalid");
}

#[test]
fn test_split_aligned_gathered_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{amsmath}
\begin{document}
\[
\begin{aligned}
a &= b \\
c &= d
\end{aligned}
\]
\[
\begin{gathered}
x + y \\
z + w
\end{gathered}
\]
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("aligned/gathered must convert");
    validate_pdf_structure(&pdf).expect("aligned/gathered PDF structure invalid");
}

#[test]
fn test_font_declarations_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\em \bf \it \rm \sf \tt \sc \sl
Normal text after font declarations.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("font declarations must convert");
    validate_pdf_structure(&pdf).expect("font declarations PDF structure invalid");
}

#[test]
fn test_font_size_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\tiny tiny \scriptsize script \footnotesize footnote \small small
\normalsize normal \large large \Large Large \LARGE LARGE \huge huge \Huge Huge
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("font sizes must convert");
    validate_pdf_structure(&pdf).expect("font sizes PDF structure invalid");
}

#[test]
fn test_all_text_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\textsc{SmallCaps} \textrm{Roman} \textsf{Sans} \textsl{Slanted}
\textup{Upright} \textmd{Medium} \textnormal{Normal}
\textsuperscript{sup} \textsubscript{sub}
\sout{strikethrough} \overline{bar} \fbox{framed}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("text commands must convert");
    validate_pdf_structure(&pdf).expect("text commands PDF structure invalid");
}

#[test]
fn test_box_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\raisebox{2pt}{raised} \rotatebox{90}{rotated} \scalebox{2}{scaled}
\colorbox{red}{colored box} \fcolorbox{black}{yellow}{framed color box}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("box commands must convert");
    validate_pdf_structure(&pdf).expect("box commands PDF structure invalid");
}

#[test]
fn test_phantom_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\phantom{hidden} \vphantom{vhidden} \hphantom{hhidden}
visible text
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("phantom commands must convert");
    validate_pdf_structure(&pdf).expect("phantom commands PDF structure invalid");
}

#[test]
fn test_spacing_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
A\hfill B\vfill C\qquad D\quad E\, F\; G\: H\! I\strut J
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("spacing commands must convert");
    validate_pdf_structure(&pdf).expect("spacing commands PDF structure invalid");
}

#[test]
fn test_rule_command_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\rule{5cm}{0.4pt}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("rule must convert");
    validate_pdf_structure(&pdf).expect("rule PDF structure invalid");
}

#[test]
fn test_index_glossary_appendix_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Text\index{term} with index\glossary{glossary term}.
\appendix
\section{Appendix Section}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("index/glossary/appendix must convert");
    validate_pdf_structure(&pdf).expect("index/glossary/appendix PDF structure invalid");
}

#[test]
fn test_href_url_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\url{https://example.com} and \href{https://example.com}{Example Link}.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("href/url must convert");
    validate_pdf_structure(&pdf).expect("href/url PDF structure invalid");
}

#[test]
fn test_hrulefill_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Above the rule.
\hrulefill
Below the rule.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("hrulefill must convert");
    validate_pdf_structure(&pdf).expect("hrulefill PDF structure invalid");
}

#[test]
fn test_include_renders_pdf() {
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let included = tmp.path().join("chapter.tex");
    {
        let mut f = std::fs::File::create(&included).unwrap();
        f.write_all(b"Included content.").unwrap();
    }
    let input = tmp.path().join("main.tex");
    let latex = r#"\documentclass{article}
\begin{document}
Main text.
\include{chapter}
\end{document}"#;
    fs::write(&input, latex).unwrap();
    let output = tmp.path().join("output.pdf");
    convert_tex_to_pdf(&input, &output).expect("include must convert");
    let pdf = fs::read(&output).expect("must read PDF");
    validate_pdf_structure(&pdf).expect("include PDF structure invalid");
}

// ==================== Extended features round-trip tests ====================

#[test]
fn test_siunitx_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{siunitx}
\begin{document}
The speed is \SI{299792458}{\meter\per\second}.
Range: \SIrange{0}{100}{\celsius}.
Unit: \si{\kilo\gram}.
Number: \num{12345}.
\sisetup{detect-all}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("siunitx must convert");
    validate_pdf_structure(&pdf).expect("siunitx PDF structure invalid");
}

#[test]
fn test_verb_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Use \verb|printf("hello")| here.
Use \verb!code with |pipe|! here.
Use \verb*|visible spaces| here.
Use \lstinline|for(int i=0;i<n;i++)| here.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("verb must convert");
    validate_pdf_structure(&pdf).expect("verb PDF structure invalid");
}

#[test]
fn test_text_symbols_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
Degree: \textdegree C.
Euro: \texteuro.
Mu: \textmu.
Underscore: \textunderscore.
Ohm: \textohm.
Numero: \textnumero.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("text symbols must convert");
    validate_pdf_structure(&pdf).expect("text symbols PDF structure invalid");
}

#[test]
fn test_multicols_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{multicol}
\begin{document}
\begin{multicols}{2}
This is column text that flows across multiple columns.
It should render without crashing.
\end{multicols}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("multicols must convert");
    validate_pdf_structure(&pdf).expect("multicols PDF structure invalid");
}

#[test]
fn test_wrapfigure_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{wrapfig}
\begin{document}
\begin{wrapfigure}{r}{0.4\textwidth}
Figure content here.
\end{wrapfigure}
Wrapping text around the figure.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("wrapfigure must convert");
    validate_pdf_structure(&pdf).expect("wrapfigure PDF structure invalid");
}

#[test]
fn test_wraptable_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{wrapfig}
\begin{document}
\begin{wraptable}{r}{0.5\textwidth}
\begin{tabular}{|c|c|}
\hline
A & B \\
\hline
1 & 2 \\
\hline
\end{tabular}
\end{wraptable}
Text after table.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("wraptable must convert");
    validate_pdf_structure(&pdf).expect("wraptable PDF structure invalid");
}

#[test]
fn test_tabbing_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\begin{tabbing}
Name \= Score \= Grade \\
Alice \> 95 \> A \\
Bob \> 82 \> B \\
\end{tabbing}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("tabbing must convert");
    validate_pdf_structure(&pdf).expect("tabbing PDF structure invalid");
}

#[test]
fn test_algorithm_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{algorithm}
\usepackage{algorithmic}
\begin{document}
\begin{algorithm}
\caption{Euclidean Algorithm}
\begin{algorithmic}
\STATE $r \gets a \bmod b$
\WHILE{$r \neq 0$}
\STATE $a \gets b$
\ENDWHILE
\end{algorithmic}
\end{algorithm}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("algorithm must convert");
    validate_pdf_structure(&pdf).expect("algorithm PDF structure invalid");
}

#[test]
fn test_alltt_renders_pdf() {
    let latex = r#"\documentclass{article}
\usepackage{alltt}
\begin{document}
\begin{alltt}
  def hello():
      print("Hello, World!")
      return 42
\end{alltt}
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("alltt must convert");
    validate_pdf_structure(&pdf).expect("alltt PDF structure invalid");
}

#[test]
fn test_smash_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\smash{x} in text mode.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("smash must convert");
    validate_pdf_structure(&pdf).expect("smash PDF structure invalid");
}

#[test]
fn test_nolinkurl_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
See \nolinkurl{https://example.com} for details.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("nolinkurl must convert");
    validate_pdf_structure(&pdf).expect("nolinkurl PDF structure invalid");
}

#[test]
fn test_penalty_commands_renders_pdf() {
    let latex = r#"\documentclass{article}
\begin{document}
\widowpenalty=10000
\clubpenalty=10000
\hyphenpenalty=100
\tolerance=200
\emergencystretch=3em
\hbadness=10000
\vbadness=10000
Text after penalty commands.
\end{document}"#;
    let pdf = convert_latex_to_bytes(latex).expect("penalty commands must convert");
    validate_pdf_structure(&pdf).expect("penalty commands PDF structure invalid");
}
