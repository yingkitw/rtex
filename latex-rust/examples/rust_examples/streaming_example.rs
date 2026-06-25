//! Example demonstrating the streaming parser for large LaTeX documents

use latex_rust::{LaTeXProcessor, StreamingParser};
use latex_rust::streaming_parser::StreamingConfig;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of a large LaTeX document
    let large_latex_content = r#"
\documentclass{article}
\usepackage{amsmath}
\title{Large Document Example}
\author{Streaming Parser}
\date{\today}

\begin{document}
\maketitle

\section{Introduction}
This is a demonstration of the streaming parser capability.
The streaming parser is designed to handle large LaTeX documents
more efficiently by processing content in chunks.

\section{Mathematical Content}
Here's some inline math: $E = mc^2$.

And here's display math:
$$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$

\subsection{More Content}
\textbf{Bold text} and \textit{italic text} are supported.

\section{Lists and Structure}
The streaming parser can handle:
\begin{itemize}
\item Complex document structures
\item Mathematical expressions
\item Various LaTeX commands
\end{itemize}

\section{Conclusion}
The streaming parser provides memory-efficient processing
for large LaTeX documents.

\end{document}
"#;

    println!("=== Streaming Parser Example ===");
    
    // Method 1: Using LaTeXProcessor with streaming
    println!("\n1. Using LaTeXProcessor with streaming:");
    let mut processor = LaTeXProcessor::new();
    let cursor = Cursor::new(large_latex_content.as_bytes());
    
    match processor.process_stream(cursor) {
        Ok(document) => {
            println!("   ✓ Successfully parsed document with streaming parser");
            println!("   - Preamble nodes: {}", document.preamble.len());
            println!("   - Body nodes: {}", document.body.len());
            if let Some(title) = &document.metadata.title {
                println!("   - Title: {}", title);
            }
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
        }
    }
    
    // Method 2: Using StreamingParser directly with custom config
    println!("\n2. Using StreamingParser with custom configuration:");
    let config = StreamingConfig {
        buffer_size: 500,  // Smaller buffer for demonstration
        chunk_size: 1024,  // 1KB chunks
        preserve_whitespace: false,
    };
    
    let mut streaming_parser = StreamingParser::with_config(config);
    let cursor2 = Cursor::new(large_latex_content.as_bytes());
    
    match streaming_parser.parse_stream(cursor2) {
        Ok(document) => {
            println!("   ✓ Successfully parsed with custom streaming config");
            println!("   - Total body nodes: {}", document.body.len());
            
            // Show buffer statistics
            let (buffer_len, input_len, eof) = streaming_parser.buffer_stats();
            println!("   - Final buffer state: {} tokens, {} input chars, EOF: {}", 
                    buffer_len, input_len, eof);
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
        }
    }
    
    // Method 3: Stream to HTML
    println!("\n3. Streaming directly to HTML:");
    let mut processor2 = LaTeXProcessor::new();
    let cursor3 = Cursor::new(large_latex_content.as_bytes());
    
    match processor2.stream_to_html(cursor3) {
        Ok(html) => {
            println!("   ✓ Successfully converted to HTML via streaming");
            println!("   - HTML length: {} characters", html.len());
            // Show first 200 characters of HTML
            let preview = if html.len() > 200 {
                format!("{}...", &html[..200])
            } else {
                html
            };
            println!("   - HTML preview: {}", preview);
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
        }
    }
    
    println!("\n=== Memory Efficiency Benefits ===");
    println!("The streaming parser provides several advantages:");
    println!("• Processes large documents without loading everything into memory");
    println!("• Configurable buffer sizes for different memory constraints");
    println!("• Suitable for processing very large LaTeX files (>100MB)");
    println!("• Maintains parsing accuracy while reducing memory footprint");
    
    Ok(())
}