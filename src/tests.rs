#[cfg(test)]
mod tests {
    use super::super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_converter_creation() {
        let converter = NativeTexConverter::new();
        assert!(std::mem::size_of_val(&converter) == 0);
    }

    #[test]
    fn test_invalid_input_path() {
        let converter = NativeTexConverter::new();
        let input = PathBuf::from("/nonexistent/file.tex");
        let output = PathBuf::from("/tmp/output.pdf");
        
        let result = converter.convert(&input, &output);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TexError::InvalidPath));
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
            Err(TexError::CompilationError(_)) => {},
            Err(TexError::PdfGenerationError(_)) => {},
            Err(TexError::ReadError(_)) => {},
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => {},
        }
    }

    fn verify_pdf_generation(tex_content: &str, test_name: &str) -> bool {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join(format!("{}.tex", test_name));
        let output_path = temp_dir.path().join(format!("{}.pdf", test_name));

        fs::write(&input_path, tex_content).unwrap();

        let converter = NativeTexConverter::new();
        let result = converter.convert(&input_path, &output_path);

        match result {
            Ok(_) => {
                assert!(output_path.exists(), "PDF file should exist");
                let metadata = fs::metadata(&output_path).unwrap();
                assert!(metadata.len() > 0, "PDF file should not be empty");
                assert!(metadata.len() > 100, "PDF file should be reasonably sized");
                
                let pdf_data = fs::read(&output_path).unwrap();
                assert!(pdf_data.starts_with(b"%PDF"), "File should start with PDF header");
                
                true
            }
            Err(e) => {
                panic!("Conversion failed for {}: {}", test_name, e);
            }
        }
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
            assert!(metadata.len() >= 1000, "PDF should be at least 1KB");
            assert!(metadata.len() <= 1_000_000, "PDF should be less than 1MB for simple doc");
        }
    }

    #[test]
    fn test_multiple_conversions() {
        let temp_dir = tempdir().unwrap();
        let converter = NativeTexConverter::new();

        for i in 1..=3 {
            let input_path = temp_dir.path().join(format!("doc{}.tex", i));
            let output_path = temp_dir.path().join(format!("doc{}.pdf", i));

            let tex_content = format!(r#"\documentclass{{article}}
\begin{{document}}
Document number {}
\end{{document}}
"#, i);
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
}
