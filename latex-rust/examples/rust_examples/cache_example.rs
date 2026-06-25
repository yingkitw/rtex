use latex_rust::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
        \documentclass{article}
        \begin{document}
        \title{Performance Test}
        \author{LaTeX Processor}
        \maketitle
        
        \section{Introduction}
        This is a test document to demonstrate caching performance.
        
        \subsection{Mathematical Expressions}
        Here are some equations:
        \begin{equation}
            E = mc^2
        \end{equation}
        
        \begin{equation}
            \sum_{i=1}^{n} i = \frac{n(n+1)}{2}
        \end{equation}
        
        \section{Conclusion}
        Caching significantly improves performance for repeated processing.
        \end{document}
    "#;
    
    println!("=== LaTeX Caching Performance Demo ===");
    println!();
    
    // First processing (cache miss)
    println!("First processing (cache miss):");
    let start = Instant::now();
    let html1 = processor.to_html(latex_input)?;
    let duration1 = start.elapsed();
    println!("Time taken: {:?}", duration1);
    println!("HTML length: {} characters", html1.len());
    
    // Get cache stats after first processing
    let stats = processor.cache_stats();
    println!("Cache stats: AST hits={}, AST misses={}, HTML hits={}, HTML misses={}, evictions={}", 
             stats.ast_hits, stats.ast_misses, stats.html_hits, stats.html_misses, stats.evictions);
    println!();
    
    // Second processing (cache hit)
    println!("Second processing (cache hit):");
    let start = Instant::now();
    let html2 = processor.to_html(latex_input)?;
    let duration2 = start.elapsed();
    println!("Time taken: {:?}", duration2);
    println!("HTML length: {} characters", html2.len());
    
    // Verify results are identical
    println!("Results identical: {}", html1 == html2);
    
    // Get cache stats after second processing
    let stats = processor.cache_stats();
    println!("Cache stats: AST hits={}, AST misses={}, HTML hits={}, HTML misses={}, evictions={}", 
             stats.ast_hits, stats.ast_misses, stats.html_hits, stats.html_misses, stats.evictions);
    
    // Calculate performance improvement
    if duration1.as_nanos() > 0 {
        let speedup = duration1.as_nanos() as f64 / duration2.as_nanos() as f64;
        println!("Performance improvement: {:.2}x faster", speedup);
    }
    println!();
    
    // Demonstrate cache configuration
    println!("=== Cache Configuration Demo ===");
    let cache_config = CacheConfig {
        max_ast_entries: 50,
        max_html_entries: 50,
        ttl: std::time::Duration::from_secs(300), // 5 minutes
        enable_lru: true,
    };
    processor.update_cache_config(cache_config);
    println!("Updated cache config: max_ast_entries=50, max_html_entries=50, ttl=300s");
    
    // Process multiple different inputs to show cache behavior
    println!("\nProcessing multiple documents:");
    for i in 1..=5 {
        let input = format!(r#"
            \documentclass{{article}}
            \begin{{document}}
            \title{{Document {}}}
            \section{{Content}}
            This is document number {}.
            \end{{document}}
        "#, i, i);
        
        let start = Instant::now();
        let _html = processor.to_html(&input)?;
        let duration = start.elapsed();
        println!("Document {}: {:?}", i, duration);
    }
    
    // Final cache stats
    let final_stats = processor.cache_stats();
    println!("\nFinal cache stats: AST hits={}, AST misses={}, HTML hits={}, HTML misses={}, evictions={}", 
             final_stats.ast_hits, final_stats.ast_misses, final_stats.html_hits, final_stats.html_misses, final_stats.evictions);
    
    // Clear cache demonstration
    println!("\nClearing cache...");
    processor.clear_cache();
    let cleared_stats = processor.cache_stats();
    println!("Cache stats after clear: AST hits={}, AST misses={}, HTML hits={}, HTML misses={}, evictions={}", 
             cleared_stats.ast_hits, cleared_stats.ast_misses, cleared_stats.html_hits, cleared_stats.html_misses, cleared_stats.evictions);
    
    Ok(())
}