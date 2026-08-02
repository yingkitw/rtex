//! Integration test for sqrt rendering audit

use rtex::convert_tex_string;
use rtex::OutputFormat;

#[test]
fn audit_sqrt_html_rendering() {
    let test_cases = vec![
        // Basic cases
        ("\\sqrt{x}", "Simple sqrt"),
        ("\\sqrt[3]{8}", "Cube root"),
        ("\\sqrt[4]{16}", "Fourth root"),
        
        // Edge cases for parentheses
        ("\\sqrt{x+y}", "Sum needs parens"),
        ("\\sqrt{x-y}", "Difference needs parens"),
        ("\\sqrt{x, y}", "Comma needs parens"),
        ("\\sqrt{x y}", "Space needs parens"),
        ("\\sqrt{a÷b}", "Division needs parens"),
        ("\\sqrt{[n]}", "Brackets need parens"),
        
        // Cases that shouldn't have parens
        ("\\sqrt{x}", "Single variable no parens"),
        ("\\sqrt{2x}", "Product no parens"),
        ("\\sqrt{x^2}", "Exponent no parens"),
        ("\\sqrt{xy}", "Two variables no parens"),
        
        // Nested sqrt
        ("\\sqrt{\\sqrt{x}}", "Nested sqrt"),
        
        // Complex expressions
        ("\\sqrt{b^2-4ac}", "Quadratic formula radicand"),
        ("\\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}", "Full quadratic"),
        
        // Multiple sqrt in one expression
        ("\\sqrt{x} + \\sqrt{y}", "Two sqrt terms"),
    ];
    
    println!("\n=== SQRT HTML Rendering Audit ===\n");
    
    for (latex, description) in test_cases {
        let tex = format!(
            "\\documentclass{{article}}\\begin{{document}}${}$\\end{{document}}",
            latex
        );
        
        match convert_tex_string(&tex, OutputFormat::Html) {
            Ok(html_bytes) => {
                let html = String::from_utf8_lossy(&html_bytes);
                // Extract the math content from HTML
                if let Some(start) = html.find("<body>") {
                    let body = &html[start + 6..];
                    if let Some(end) = body.find("</body>") {
                        let content = &body[..end];
                        println!("{}:", description);
                        println!("  LaTeX:  {}", latex);
                        // Show just the math content
                        println!("  HTML:   {}", content.trim());
                        println!();
                    }
                }
            }
            Err(e) => {
                println!("{}: FAILED - {}", description, e);
            }
        }
    }
}

#[test]
fn audit_sqrt_pdf_rendering() {
    let test_cases = vec![
        ("\\sqrt{x}", "Simple sqrt PDF"),
        ("\\sqrt[3]{x}", "Cube root PDF"),
        ("\\sqrt{x+y}", "Sum in sqrt PDF"),
    ];
    
    println!("\n=== SQRT PDF Rendering Audit ===\n");
    
    for (latex, description) in test_cases {
        let tex = format!(
            "\\documentclass{{article}}\\begin{{document}}${}$\\end{{document}}",
            latex
        );
        
        match convert_tex_string(&tex, OutputFormat::Pdf) {
            Ok(_pdf) => {
                println!("{}: OK", description);
                println!("  LaTeX: {}", latex);
                println!();
            }
            Err(e) => {
                println!("{}: FAILED - {}", description, e);
            }
        }
    }
}
