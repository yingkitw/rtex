//! Async LaTeX processing example
//!
//! This example demonstrates how to use the async functionality
//! for non-blocking LaTeX processing operations.

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use latex_rust::LaTeXProcessor;
    
    println!("=== Async LaTeX Processing Example ===");
    
    let processor = LaTeXProcessor::new();
    
    // Single document async processing
    println!("\n1. Single Document Async Processing:");
    let latex_input = r#"
        \documentclass{article}
        \begin{document}
        \title{Async Processing Test}
        \author{LaTeX-Rust}
        \maketitle
        
        This document was processed \textbf{asynchronously}!
        
        \section{Features}
        \begin{itemize}
        \item Non-blocking processing
        \item Concurrent document handling
        \item Improved performance for large documents
        \end{itemize}
        
        \end{document}
    "#;
    
    let start = std::time::Instant::now();
    let html_result = processor.to_html_async(latex_input).await?;
    let duration = start.elapsed();
    
    println!("Processed document in {:?}", duration);
    println!("HTML output length: {} characters", html_result.len());
    
    // Batch processing example
    println!("\n2. Batch Processing (Concurrent):");
    let documents = vec![
        r#"\documentclass{article}\begin{document}\section{Doc 1}Hello from document 1!\end{document}"#.to_string(),
        r#"\documentclass{article}\begin{document}\section{Doc 2}Hello from document 2!\end{document}"#.to_string(),
        r#"\documentclass{article}\begin{document}\section{Doc 3}Hello from document 3!\end{document}"#.to_string(),
        r#"\documentclass{article}\begin{document}\section{Doc 4}Hello from document 4!\end{document}"#.to_string(),
        r#"\documentclass{article}\begin{document}\section{Doc 5}Hello from document 5!\end{document}"#.to_string(),
    ];
    
    let start = std::time::Instant::now();
    let batch_results = processor.process_batch_async(documents).await;
    let duration = start.elapsed();
    
    println!("Processed {} documents concurrently in {:?}", batch_results.len(), duration);
    
    let successful = batch_results.iter().filter(|r| r.is_ok()).count();
    let failed = batch_results.len() - successful;
    
    println!("Results: {} successful, {} failed", successful, failed);
    
    // Performance comparison (simulated)
    println!("\n3. Performance Benefits:");
    println!("✓ Non-blocking operations allow other tasks to run");
    println!("✓ Concurrent batch processing improves throughput");
    println!("✓ Better resource utilization for I/O-bound operations");
    
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --example async_example --features async");
}