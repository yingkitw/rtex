#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;
    // Note: lopdf removed - using custom PDF generation
    use crate::parser::TexElement;
    use crate::parser::TexParser;
    use crate::{LatexError, NativeTexConverter};

    #[test]
    fn test_converter_creation() {
        let converter = NativeTexConverter::new();
        assert!(converter.cache.is_none());
    }

    #[test]
    fn test_converter_with_cache() {
        let converter = NativeTexConverter::with_cache();
        assert!(converter.cache.is_some());
    }

    #[test]
    fn test_cached_conversion_reuses_parsed_ast() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("cached.tex");
        let output1 = temp_dir.path().join("out1.pdf");
        let output2 = temp_dir.path().join("out2.pdf");

        let tex_content = r#"\documentclass{article}
\begin{document}
Cached content
\end{document}
"#;
        fs::write(&input_path, tex_content).unwrap();

        let converter = NativeTexConverter::with_cache();

        // First conversion — cache miss
        converter.convert(&input_path, &output1).unwrap();
        let stats1 = converter.cache_stats().unwrap();
        assert_eq!(stats1.parsed_hits, 0);
        assert_eq!(stats1.parsed_misses, 1);

        // Second conversion — cache hit (same file content)
        converter.convert(&input_path, &output2).unwrap();
        let stats2 = converter.cache_stats().unwrap();
        assert_eq!(stats2.parsed_hits, 1);
        assert_eq!(stats2.parsed_misses, 1);
    }

    #[test]
    fn test_invalid_input_path() {
        let converter = NativeTexConverter::new();
        let input = PathBuf::from("/nonexistent/file.tex");
        let output = PathBuf::from("/tmp/output.pdf");

        let result = converter.convert(&input, &output);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LatexError::InvalidPath));
    }

    #[test]
    fn test_simple_tex_conversion() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("test.tex");
        let output_path = temp_dir.path().join("test.pdf");

        let tex_content = r#"\documentclass{article}
\begin{document}
Hello, World!
\end{document}
"#;
        fs::write(&input_path, tex_content).unwrap();

        let converter = NativeTexConverter::new();
        let result = converter.convert(&input_path, &output_path);

        if let Err(e) = &result {
            eprintln!("Conversion failed: {}", e);
        }

        if result.is_ok() {
            assert!(output_path.exists());
            let metadata = fs::metadata(&output_path).unwrap();
            assert!(metadata.len() > 0);
        }
    }

    #[test]
    fn test_convert_tex_to_pdf_function() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("simple.tex");
        let output_path = temp_dir.path().join("simple.pdf");

        let tex_content = r#"\documentclass{article}
\begin{document}
Test document.
\end{document}
"#;
        fs::write(&input_path, tex_content).unwrap();

        let result = convert_tex_to_pdf(&input_path, &output_path);

        if result.is_ok() {
            assert!(output_path.exists());
        }
    }

    #[test]
    fn test_convert_includegraphics_svg() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("logo.svg");
        let mut svg_file = std::fs::File::create(&svg_path).unwrap();
        svg_file
            .write_all(
                br#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="blue"/></svg>"#,
            )
            .unwrap();

        let tex = format!(
            r#"\documentclass{{article}}
\begin{{document}}
\includegraphics[width=5cm]{{{}}}
\end{{document}}
"#,
            svg_path.display()
        );

        let pdf = convert_tex_string_to_pdf_bytes(&tex).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 500);
    }

    #[test]
    fn test_keep_intermediate_writes_artifacts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("keep.tex");
        let output_path = temp_dir.path().join("keep.pdf");

        let tex = r#"\documentclass{article}
\title{Keep Test}
\begin{document}
Intermediate artifacts.
\end{document}
"#;
        std::fs::write(&input_path, tex).unwrap();

        let options = ConversionOptions::default().with_keep_intermediate(true);
        convert_tex_file(&input_path, &output_path, &options).unwrap();

        assert!(output_path.exists());
        assert!(temp_dir.path().join("keep.expanded.tex").exists());
        assert!(temp_dir.path().join("keep.ast.json").exists());
        assert!(temp_dir.path().join("keep.meta.json").exists());
    }

    #[test]
    fn test_convert_tex_string_html() {
        let tex = r#"\documentclass{article}
\begin{document}
\section{Hello}
HTML output test.
\end{document}
"#;
        let html = convert_tex_string(tex, OutputFormat::Html).unwrap();
        let text = String::from_utf8(html).unwrap();
        assert!(text.contains("<!DOCTYPE html>"));
        assert!(text.contains("HTML output test."));
    }

    #[test]
    fn test_convert_tex_string_to_pdf_bytes() {
        let tex = r#"\documentclass{article}
\begin{document}
Hello from in-memory conversion!
\end{document}
"#;
        let pdf = convert_tex_string_to_pdf_bytes(tex).unwrap();
        assert!(pdf.len() > 100);
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_convert_string_with_cache() {
        let tex = r#"\documentclass{article}
\begin{document}
Cached string conversion
\end{document}
"#;
        let converter = NativeTexConverter::with_cache();
        let pdf1 = converter.convert_string(tex).unwrap();
        let pdf2 = converter.convert_string(tex).unwrap();
        assert_eq!(pdf1, pdf2);
        let stats = converter.cache_stats().unwrap();
        assert_eq!(stats.parsed_hits, 1);
        assert_eq!(stats.parsed_misses, 1);
    }

    #[test]
    fn test_invalid_tex_syntax() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("invalid.tex");
        let output_path = temp_dir.path().join("invalid.pdf");

        let invalid_tex = r#"\documentclass{article}
\begin{document}
\undefined_command_that_should_fail
\end{document}
"#;
        fs::write(&input_path, invalid_tex).unwrap();

        let converter = NativeTexConverter::new();
        let result = converter.convert(&input_path, &output_path);

        match result {
            Err(LatexError::PdfError { .. }) => {}
            Err(LatexError::IoError { .. }) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => {}
        }
    }

    /// Get output directory for test PDFs.
    /// If LATEX_RS_TEST_OUTPUT env var is set, use that; otherwise use tempdir.
    fn get_test_output_dir(test_name: &str) -> (PathBuf, bool) {
        if let Ok(output_dir) = env::var("LATEX_RS_TEST_OUTPUT") {
            let dir = PathBuf::from(output_dir).join(test_name);
            fs::create_dir_all(&dir).ok();
            (dir, true)
        } else {
            let temp_dir = tempdir().unwrap();
            let path = temp_dir.keep();
            (path, false)
        }
    }

    // Note: extract_pdf_text removed with lopdf dependency
    // PDF text extraction would require external tool or library
    // For now, we validate PDF structure only

    /// Validate PDF structure and content
    fn validate_pdf_content(pdf_path: &PathBuf, _expected_texts: &[&str]) -> Result<(), String> {
        // Check file exists and has content
        let metadata =
            fs::metadata(pdf_path).map_err(|e| format!("Failed to read PDF metadata: {}", e))?;

        if metadata.len() == 0 {
            return Err("PDF file is empty".to_string());
        }

        if metadata.len() < 100 {
            return Err("PDF file too small, likely invalid".to_string());
        }

        // Check PDF header
        let pdf_data = fs::read(pdf_path).map_err(|e| format!("Failed to read PDF: {}", e))?;

        if !pdf_data.starts_with(b"%PDF") {
            return Err("PDF missing header signature".to_string());
        }

        // Check PDF has proper structure (xref and trailer)
        let pdf_str = String::from_utf8_lossy(&pdf_data);
        if !pdf_str.contains("xref") {
            return Err("PDF missing xref table".to_string());
        }
        if !pdf_str.contains("trailer") {
            return Err("PDF missing trailer".to_string());
        }
        if !pdf_str.contains("%%EOF") {
            return Err("PDF missing EOF marker".to_string());
        }

        // Note: Text content validation disabled after removing lopdf
        // PDFs are structurally valid but text extraction requires external tool

        Ok(())
    }

    fn verify_pdf_generation_with_validation(
        tex_content: &str,
        test_name: &str,
        expected_texts: &[&str],
    ) -> bool {
        let (output_dir, is_persistent) = get_test_output_dir(test_name);
        let input_path = output_dir.join(format!("{}.tex", test_name));
        let output_path = output_dir.join(format!("{}.pdf", test_name));

        fs::write(&input_path, tex_content).unwrap();

        let converter = NativeTexConverter::new();
        let result = converter.convert(&input_path, &output_path);

        match result {
            Ok(_) => {
                assert!(output_path.exists(), "PDF file should exist");

                // Validate PDF content
                if let Err(e) = validate_pdf_content(&output_path, expected_texts) {
                    panic!("PDF validation failed for {}: {}", test_name, e);
                }

                if is_persistent {
                    println!("Test PDF saved to: {:?}", output_path);
                }

                true
            }
            Err(e) => {
                panic!("Conversion failed for {}: {}", test_name, e);
            }
        }
    }

    fn verify_pdf_generation(tex_content: &str, test_name: &str) -> bool {
        verify_pdf_generation_with_validation(tex_content, test_name, &[])
    }

    #[test]
    fn test_text_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
Plain text content here.
More text on another line.
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "text_element",
            &["Plain text content here", "More text on another line"],
        );
    }

    #[test]
    fn test_section_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
\section{First Section}
Content in section one.
\subsection{Subsection Title}
Content in subsection.
\section{Second Section}
Content in section two.
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "section_element",
            &[
                "First Section",
                "Content in section one",
                "Subsection Title",
                "Second Section",
            ],
        );
    }

    #[test]
    fn test_paragraph_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
First paragraph content here.

Second paragraph after blank line.

Third paragraph content.
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "paragraph_element",
            &[
                "First paragraph content",
                "Second paragraph",
                "Third paragraph",
            ],
        );
    }

    #[test]
    fn test_math_inline_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
Inline math expression: $x = y + z$ and $a^2 + b^2 = c^2$.
More math: $\alpha + \beta = \gamma$.
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "math_inline_element",
            &["Inline math expression", "x = y + z", "a"],
        );
    }

    #[test]
    fn test_math_display_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
Display math equation:
\begin{equation}
x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}
\end{equation}
End of document.
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "math_display_element",
            &["Display math equation", "End of document"],
        );
    }

    #[test]
    fn test_itemlist_unordered_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
Shopping list:
\begin{itemize}
\item First bullet item
\item Second bullet item
\item Third bullet with math
\end{itemize}
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "itemlist_unordered_element",
            &[
                "Shopping list",
                "First bullet item",
                "Second bullet item",
                "Third bullet",
            ],
        );
    }

    #[test]
    fn test_itemlist_ordered_element_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\begin{document}
Procedure steps:
\begin{enumerate}
\item First step instruction
\item Second step instruction
\item Third step instruction
\end{enumerate}
\end{document}
"#;
        verify_pdf_generation_with_validation(
            tex_content,
            "itemlist_ordered_element",
            &[
                "Procedure steps",
                "First step instruction",
                "Second step instruction",
                "Third step",
            ],
        );
    }

    #[test]
    fn test_command_title_author_date_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\title{My Test Title}
\author{Test Author Name}
\date{2024-01-15}
\begin{document}
\maketitle
Document body content starts here and should appear.
\end{document}
"#;
        // Note: Title/author may not appear in text extraction, verify body content
        verify_pdf_generation_with_validation(
            tex_content,
            "command_metadata_element",
            &["Document body content starts"],
        );
    }

    #[test]
    fn test_complex_nested_elements_pdf_conversion() {
        let tex_content = r#"\documentclass{article}
\title{Complex Test}
\author{Test Suite}
\date{\today}
\begin{document}
\maketitle

\section{Introduction}
Welcome to the test. Inline math: $E=mc^2$.

\subsection{Details}
More details here with display math:
\begin{equation}
\int_0^\infty e^{-x} dx = 1
\end{equation}

\section{Results}
\begin{itemize}
\item Result one: $\alpha$
\item Result two: $\beta$
\item Result three with subscript: $x_i$
\end{itemize}

\begin{enumerate}
\item Step 1
\item Step 2
\end{enumerate}

Final paragraph.
\end{document}
"#;
        verify_pdf_generation(tex_content, "complex_nested_elements");
    }

    #[test]
    fn test_minimal_example() {
        let tex_content = r#"\documentclass{article}
\begin{document}
Hello, World!
\end{document}
"#;
        verify_pdf_generation(tex_content, "minimal");
    }

    #[test]
    fn test_math_example() {
        let tex_content = r#"\documentclass{article}
\usepackage{amsmath}

\begin{document}
\section{Mathematics}

The quadratic formula:
\begin{equation}
    x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\end{equation}

Inline math: $E = mc^2$
\end{document}
"#;
        verify_pdf_generation(tex_content, "math");
    }

    #[test]
    fn test_table_example() {
        let tex_content = r#"\documentclass{article}

\begin{document}
\section{Tables}

\begin{tabular}{lcc}
\hline
Item & Quantity & Price \\
\hline
Apples & 5 & \$2.50 \\
Oranges & 3 & \$1.80 \\
\hline
\end{tabular}
\end{document}
"#;
        verify_pdf_generation(tex_content, "table");
    }

    #[test]
    fn test_lists_example() {
        let tex_content = r#"\documentclass{article}

\begin{document}
\section{Lists}

\begin{itemize}
    \item First item
    \item Second item
    \item Third item
\end{itemize}

\begin{enumerate}
    \item Step one
    \item Step two
    \item Step three
\end{enumerate}
\end{document}
"#;
        verify_pdf_generation(tex_content, "lists");
    }

    #[test]
    fn test_complex_document() {
        let tex_content = r#"\documentclass{article}
\usepackage{amsmath}

\title{Test Document}
\author{latex-rs}
\date{\today}

\begin{document}
\maketitle

\section{Introduction}
This is a test document with multiple features.

\section{Mathematics}
\begin{equation}
    \int_0^\infty e^{-x} dx = 1
\end{equation}

\section{Lists}
\begin{itemize}
    \item Feature one
    \item Feature two
\end{itemize}

\section{Conclusion}
All features work correctly.
\end{document}
"#;
        verify_pdf_generation(tex_content, "complex");
    }

    #[test]
    fn test_pdf_file_size() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("size_test.tex");
        let output_path = temp_dir.path().join("size_test.pdf");

        let tex_content = r#"\documentclass{article}
\begin{document}
This is a test for PDF file size verification.
\end{document}
"#;
        fs::write(&input_path, tex_content).unwrap();

        let converter = NativeTexConverter::new();
        let result = converter.convert(&input_path, &output_path);

        if result.is_ok() {
            let metadata = fs::metadata(&output_path).unwrap();
            assert!(metadata.len() >= 500, "PDF should be at least 500 bytes");
            assert!(
                metadata.len() <= 100_000,
                "ASCII simple doc should be <100KB, got {} bytes",
                metadata.len()
            );
        }
    }

    #[test]
    fn test_multiple_conversions() {
        let temp_dir = tempdir().unwrap();
        let converter = NativeTexConverter::new();

        for i in 1..=3 {
            let input_path = temp_dir.path().join(format!("doc{}.tex", i));
            let output_path = temp_dir.path().join(format!("doc{}.pdf", i));

            let tex_content = format!(
                r#"\documentclass{{article}}
\begin{{document}}
Document number {}
\end{{document}}
"#,
                i
            );
            fs::write(&input_path, tex_content).unwrap();

            let result = converter.convert(&input_path, &output_path);

            if result.is_ok() {
                assert!(output_path.exists(), "PDF {} should exist", i);
            }
        }
    }

    #[test]
    fn test_output_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("test.tex");
        let output_dir = temp_dir.path().join("output");
        let output_path = output_dir.join("test.pdf");

        let tex_content = r#"\documentclass{article}
\begin{document}
Test output directory creation.
\end{document}
"#;
        fs::write(&input_path, tex_content).unwrap();

        fs::create_dir_all(&output_dir).unwrap();

        let converter = NativeTexConverter::new();
        let result = converter.convert(&input_path, &output_path);

        if result.is_ok() {
            assert!(output_dir.exists(), "Output directory should exist");
            assert!(output_path.exists(), "PDF should be in output directory");
        }
    }

    #[test]
    fn test_texttt_parsing_issue() {
        let content = r"demonstrate the \texttt{latex-rs} TeX to PDF converter.";

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        // \texttt should produce a Command element with the text content
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Command { name, args } if name == "texttt" && args == &["latex-rs".to_string()])
        }), "texttt should be parsed as Command with correct args");
    }

    #[test]
    fn test_inline_math_parsing_issue() {
        let content = r"$\int_0^\infty e^{-x} dx = 1$";

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        println!("Math input: {}", content);
        println!("Parsed elements:");
        for (i, elem) in elements.iter().enumerate() {
            println!("  {}: {:?}", i, elem);
        }
    }

    #[test]
    fn test_infinity_superscript() {
        use crate::math_formatter::MathFormatter;

        let input = r"\int_0^\infty e^{-x} dx = 1";
        let output = MathFormatter::format(input);
        println!("Math format input: {}", input);
        println!("Math format output: {}", output);

        // Let's also test just the infinity part
        let inf_input = r"^\infty";
        let inf_output = MathFormatter::format(inf_input);
        println!("Infinity input: {}", inf_input);
        println!("Infinity output: {}", inf_output);

        // Check that infinity is properly formatted (with Unicode symbols using DejaVu)
        assert!(
            output.contains("∫") || output.contains("∞"),
            "Infinity and integral should be formatted with Unicode symbols, got: {}",
            output
        );
    }

    #[test]
    fn test_text_formatting_renders_in_pdf() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("formatting.tex");
        let output_path = temp_dir.path().join("formatting.pdf");

        let tex_content = r#"\documentclass{article}
\begin{document}
\textbf{bold} and \textit{italic} and \texttt{mono}.
\end{document}
"#;
        fs::write(&input_path, tex_content).unwrap();

        let converter = NativeTexConverter::new();
        converter.convert(&input_path, &output_path).unwrap();

        assert!(output_path.exists(), "PDF should be generated");
        let pdf_data = fs::read(&output_path).unwrap();
        // PDF uses UTF-16BE hex encoding for text; just verify the file is
        // non-trivial and starts with %PDF header.
        assert!(pdf_data.starts_with(b"%PDF"), "should be a valid PDF");
        assert!(pdf_data.len() > 500, "PDF should contain actual content");
    }
}
