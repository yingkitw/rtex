//! LaTeX-Rust CLI application

use clap::{Arg, Command};
use latex_rust::LaTeXProcessor;
use log::{error, info};
use std::fs;
use std::path::Path;

fn main() {
    env_logger::init();

    let matches = Command::new("latex-rust")
        .version("0.1.0")
        .author("LaTeX-Rust Team")
        .about("A LaTeX processor written in Rust")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input LaTeX file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file (default: input.html)"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format")
                .value_parser(["html", "pdf"])
                .default_value("html"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let verbose = matches.get_flag("verbose");

    // Determine output file
    let output_file = match matches.get_one::<String>("output") {
        Some(output) => output.clone(),
        None => {
            let input_path = Path::new(input_file);
            let stem = input_path.file_stem().unwrap_or_default();
            let extension = match format.as_str() {
                "html" => "html",
                "pdf" => "pdf",
                _ => "html",
            };
            format!("{}.{}", stem.to_string_lossy(), extension)
        }
    };

    if verbose {
        info!("Processing {input_file} -> {output_file}");
    }

    // Process the file
    if let Err(e) = process_file(input_file, &output_file, format, verbose) {
        error!("Error processing file: {e}");
        std::process::exit(1);
    }

    if verbose {
        info!("Successfully processed {input_file} to {output_file}");
    }
}

fn process_file(
    input_file: &str,
    output_file: &str,
    format: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read input file
    let input_content = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read input file '{input_file}': {e}"))?;

    if verbose {
        info!("Read {} characters from {}", input_content.len(), input_file);
    }

    // Create processor
    let mut processor = LaTeXProcessor::new();

    // Process based on format
    match format {
        "html" => {
            let output_content = processor.to_html(&input_content)?;
            if verbose {
                info!("Generated {} characters of {} output", output_content.len(), format);
            }
            // Write output file
            fs::write(output_file, output_content)
                .map_err(|e| format!("Failed to write output file '{output_file}': {e}"))?;
        },
        "pdf" => {
            processor.to_pdf(&input_content, output_file)?;
            if verbose {
                info!("Generated PDF output to {output_file}");
            }
        },
        _ => return Err(format!("Unsupported format: {format}").into()),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_basic_processing() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.tex");
        let output_path = dir.path().join("test.html");

        // Create test LaTeX file
        let latex_content = r#"
\documentclass{article}
\title{Test Document}
\author{Test Author}
\begin{document}
\maketitle
\section{Introduction}
This is a test document with \textbf{bold text} and \textit{italic text}.

\subsection{Math}
Here's some inline math: $x^2 + y^2 = z^2$.

And display math:
$$\int_0^\infty e^{-x} dx = 1$$

\end{document}
"#;

        fs::write(&input_path, latex_content).unwrap();

        // Process the file
        let result = process_file(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            "html",
            false,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());

        let output_content = fs::read_to_string(&output_path).unwrap();
        
        // Check for basic HTML structure and content
        assert!(output_content.contains("<!DOCTYPE html>"));
        assert!(output_content.contains("<body>"));
        assert!(output_content.contains("</body>"));
        assert!(output_content.contains("</html>"));
        assert!(output_content.contains("Test Document"));
        assert!(output_content.contains("Test Author"));
    }

    #[test]
    fn test_error_handling() {
        let result = process_file("nonexistent.tex", "output.html", "html", false);
        assert!(result.is_err());
    }
}