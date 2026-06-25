use latex_rust::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LaTeXProcessor::new();
    
    // Enable incremental parsing
    processor.enable_incremental_parsing();
    
    println!("=== LaTeX Incremental Parsing Demo ===");
    println!();
    
    // Original document
    let original_doc = r#"
\documentclass{article}
\usepackage{amsmath}
\begin{document}
\title{Incremental Parsing Demo}
\author{LaTeX Processor}
\maketitle

\section{Introduction}
This document demonstrates incremental parsing capabilities.
The processor can detect changes and only reprocess modified sections.

\section{Mathematical Content}
Here are some equations that won't change:
\begin{equation}
    E = mc^2
\end{equation}

\begin{equation}
    \sum_{i=1}^{n} i = \frac{n(n+1)}{2}
\end{equation}

\section{Dynamic Content}
This section will be modified to show incremental parsing benefits.
Original content here.

\section{Conclusion}
Incremental parsing significantly improves performance for large documents.
\end{document}
    "#;
    
    // First processing (full parse)
    println!("=== Initial Processing (Full Parse) ===");
    let start = Instant::now();
    let _doc1 = processor.process_incremental(original_doc)?;
    let duration1 = start.elapsed();
    println!("Time taken: {:?}", duration1);
    
    if let Some(stats) = processor.incremental_stats() {
        println!("Incremental parsing stats:");
        println!("  Total sections: {}", stats.total_sections);
        println!("  Cached sections: {}", stats.cached_sections);
        println!("  Document hash: {:?}", stats.document_hash);
    }
    println!();
    
    // Modified document (only one section changed)
    let modified_doc = r#"
\documentclass{article}
\usepackage{amsmath}
\begin{document}
\title{Incremental Parsing Demo}
\author{LaTeX Processor}
\maketitle

\section{Introduction}
This document demonstrates incremental parsing capabilities.
The processor can detect changes and only reprocess modified sections.

\section{Mathematical Content}
Here are some equations that won't change:
\begin{equation}
    E = mc^2
\end{equation}

\begin{equation}
    \sum_{i=1}^{n} i = \frac{n(n+1)}{2}
\end{equation}

\section{Dynamic Content}
This section has been MODIFIED to show incremental parsing benefits.
New content added here with additional information.
More changes to demonstrate the incremental parsing system.

\section{Conclusion}
Incremental parsing significantly improves performance for large documents.
\end{document}
    "#;
    
    // Second processing (incremental parse)
    println!("=== Modified Document Processing (Incremental Parse) ===");
    let start = Instant::now();
    let _doc2 = processor.process_incremental(modified_doc)?;
    let duration2 = start.elapsed();
    println!("Time taken: {:?}", duration2);
    
    if let Some(stats) = processor.incremental_stats() {
        println!("Incremental stats: {} total sections, {} cached sections", 
                 stats.total_sections, stats.cached_sections);
    }
    
    // Calculate performance improvement
    if duration1.as_nanos() > 0 && duration2.as_nanos() > 0 {
        let speedup = duration1.as_nanos() as f64 / duration2.as_nanos() as f64;
        println!("Performance improvement: {:.2}x faster", speedup);
    }
    println!();
    
    // Demonstrate configuration
    println!("=== Custom Configuration Demo ===");
    let incremental_config = IncrementalConfig {
        min_section_size: 50,
        max_sections: 20,
        paragraph_level: true,
    };
    
    processor.enable_incremental_parsing_with_config(incremental_config);
    println!("Enabled incremental parsing with custom config:");
    println!("- Min section size: 50 characters");
    println!("- Max sections: 500");
    println!("- Paragraph-level tracking: enabled");
    println!();
    
    // Test with paragraph-level changes
    let paragraph_modified_doc = r#"
\documentclass{article}
\usepackage{amsmath}
\begin{document}
\title{Incremental Parsing Demo}
\author{LaTeX Processor}
\maketitle

\section{Introduction}
This document demonstrates incremental parsing capabilities.
The processor can detect changes and only reprocess modified sections.

This is a new paragraph added to test paragraph-level incremental parsing.
Only this paragraph should need to be reparsed.

\section{Mathematical Content}
Here are some equations that won't change:
\begin{equation}
    E = mc^2
\end{equation}

\begin{equation}
    \sum_{i=1}^{n} i = \frac{n(n+1)}{2}
\end{equation}

\section{Dynamic Content}
This section has been MODIFIED to show incremental parsing benefits.
New content added here with additional information.
More changes to demonstrate the incremental parsing system.

\section{Conclusion}
Incremental parsing significantly improves performance for large documents.
\end{document}
    "#;
    
    println!("=== Paragraph-Level Change Processing ===");
    let start = Instant::now();
    let _doc3 = processor.process_incremental(paragraph_modified_doc)?;
    let duration3 = start.elapsed();
    println!("Time taken: {:?}", duration3);
    
    if let Some(stats) = processor.incremental_stats() {
        println!("Incremental stats: {} total sections, {} cached sections", 
                 stats.total_sections, stats.cached_sections);
    }
    println!();
    
    // Compare with regular processing
    println!("=== Comparison with Regular Processing ===");
    processor.disable_incremental_parsing();
    
    let start = Instant::now();
    let _doc4 = processor.process(paragraph_modified_doc)?;
    let duration4 = start.elapsed();
    println!("Regular processing time: {:?}", duration4);
    
    if duration3.as_nanos() > 0 && duration4.as_nanos() > 0 {
        let speedup = duration4.as_nanos() as f64 / duration3.as_nanos() as f64;
        println!("Incremental vs Regular: {:.2}x faster", speedup);
    }
    println!();
    
    // Re-enable incremental parsing and clear cache
    processor.enable_incremental_parsing();
    println!("=== Cache Management Demo ===");
    
    // Process document to populate cache
    let _doc5 = processor.process_incremental(original_doc)?;
    if let Some(stats) = processor.incremental_stats() {
        println!("Before cache clear: {} total sections, {} cached sections", 
                 stats.total_sections, stats.cached_sections);
    }
    
    // Clear incremental cache
    processor.clear_incremental_cache();
    if let Some(stats) = processor.incremental_stats() {
        println!("After cache clear: {} total sections, {} cached sections", 
                 stats.total_sections, stats.cached_sections);
    }
    
    println!("\n=== Summary ===");
    println!("Incremental parsing provides significant performance benefits for:");
    println!("- Large documents with small changes");
    println!("- Iterative editing workflows");
    println!("- Real-time preview applications");
    println!("- Documents with stable sections (headers, math, etc.)");
    
    Ok(())
}