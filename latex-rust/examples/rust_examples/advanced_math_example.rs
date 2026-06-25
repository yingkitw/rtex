//! Example demonstrating advanced mathematical features
//! 
//! This example shows:
//! - Equation numbering
//! - Advanced math commands (fractions, roots, operators)
//! - Math environments (equation, align, matrix)
//! - Greek letters and symbols
//! - Cross-references to equations

use latex_rust::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LaTeXProcessor::new();
    
    println!("=== Advanced Math Processing Example ===");
    
    run_basic_equation_example(&mut processor)?;
    run_advanced_commands_example(&mut processor)?;
    run_matrix_example(&mut processor)?;
    run_greek_letters_example(&mut processor)?;
    run_complex_document_example(&mut processor)?;
    run_equation_numbering_example(&mut processor)?;
    run_command_detection_example(&mut processor)?;
    run_supported_environments_example();
    
    println!("\n=== Advanced Math Processing Complete ===");
    
    Ok(())
}

/// Run basic equation with numbering example
fn run_basic_equation_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    let equation_example = r#"
\begin{equation}
\label{eq:quadratic}
x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\end{equation}
"#;
    
    println!("\n1. Processing equation with numbering:");
    println!("Input: {}", equation_example.trim());
    
    match processor.process_math_advanced(equation_example) {
        Ok(html) => {
            println!("Output HTML (excerpt):");
            extract_and_display_math_part(&html);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

/// Run advanced math commands example
fn run_advanced_commands_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    let advanced_commands = r#"
\begin{align}
\sum_{i=1}^{n} i &= \frac{n(n+1)}{2} \\
\int_0^\infty e^{-x} dx &= 1 \\
\lim_{x \to 0} \frac{\sin x}{x} &= 1
\end{align}
"#;
    
    println!("\n2. Processing advanced math commands:");
    println!("Input: {}", advanced_commands.trim());
    
    match processor.process_math_advanced(advanced_commands) {
        Ok(html) => {
            println!("✓ Successfully processed advanced math commands");
            let math_count = html.matches("$$").count() / 2;
            println!("  Found {} math expressions", math_count);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

/// Run matrix operations example
fn run_matrix_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    let matrix_example = r#"
\begin{equation}
\mathbf{A} = \begin{pmatrix}
a & b \\
c & d
\end{pmatrix}, \quad
\det(\mathbf{A}) = ad - bc
\end{equation}
"#;
    
    println!("\n3. Processing matrix example:");
    println!("Input: {}", matrix_example.trim());
    
    match processor.process_math_advanced(matrix_example) {
        Ok(html) => {
            println!("✓ Successfully processed matrix");
            if html.contains("pmatrix") {
                println!("  Matrix environment detected");
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

/// Run Greek letters and symbols example
fn run_greek_letters_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    let greek_example = r#"
\begin{equation}
\alpha + \beta = \gamma, \quad
\Delta = b^2 - 4ac, \quad
\pi \approx 3.14159
\end{equation}
"#;
    
    println!("\n4. Processing Greek letters:");
    println!("Input: {}", greek_example.trim());
    
    match processor.process_math_advanced(greek_example) {
        Ok(html) => {
            println!("✓ Successfully processed Greek letters");
            let greek_count = count_greek_letters(&html);
            println!("  Found {} Greek letters", greek_count);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

/// Run complex document example
fn run_complex_document_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    let complex_document = create_complex_document();
    
    println!("\n5. Processing complex document:");
    println!("Input: Complex document with {} lines", complex_document.lines().count());
    
    match processor.process_math_advanced(&complex_document) {
        Ok(html) => {
            println!("✓ Successfully processed complex document");
            display_math_statistics(&html);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

/// Run equation numbering and references example
fn run_equation_numbering_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n6. Testing equation numbering:");
    
    processor.reset_equation_counter();
    
    let eq1 = r#"\begin{equation}\label{eq:first}E = mc^2\end{equation}"#;
    let eq2 = r#"\begin{equation}\label{eq:second}F = ma\end{equation}"#;
    
    let _ = processor.process_math_advanced(eq1);
    let _ = processor.process_math_advanced(eq2);
    
    display_equation_numbers(processor);
    
    Ok(())
}

/// Run math command detection example
fn run_command_detection_example(processor: &mut LaTeXProcessor) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n7. Testing math command detection:");
    
    let math_commands = ["frac", "sqrt", "sum", "int", "alpha", "beta", "sin", "cos", "matrix"];
    
    for cmd in &math_commands {
        let status = if processor.is_math_command(cmd) { "✓" } else { "✗" };
        let recognition = if processor.is_math_command(cmd) { "recognized" } else { "not recognized" };
        println!("  {} '{}' is {} as a math command", status, cmd, recognition);
    }
    
    Ok(())
}

/// Run supported math environments example
fn run_supported_environments_example() {
    println!("\n8. Supported math environments:");
    let supported_envs = LaTeXProcessor::get_supported_math_environments();
    println!("  Total: {} environments", supported_envs.len());
    println!("  Environments: {}", supported_envs.join(", "));
}

/// Extract and display the math part from HTML
fn extract_and_display_math_part(html: &str) {
    if let Some(start) = html.find("<div class=\"math-display\">$$") {
        if let Some(end) = html[start..].find("$$</div>") {
            let math_part = &html[start..start + end + 7];
            println!("{}", math_part);
        }
    }
}

/// Count Greek letters in HTML output
fn count_greek_letters(html: &str) -> usize {
    ["alpha", "beta", "gamma", "Delta", "pi"]
        .iter()
        .filter(|&letter| html.contains(letter))
        .count()
}

/// Create a complex document for testing
fn create_complex_document() -> String {
    r#"
\documentclass{article}
\begin{document}

\section{Mathematical Analysis}

Consider the quadratic equation:
\begin{equation}
\label{eq:quad}
ax^2 + bx + c = 0
\end{equation}

The solutions are given by:
\begin{equation}
\label{eq:solution}
x_{1,2} = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
\end{equation}

For the special case where $a = 1$, $b = -3$, and $c = 2$:
\begin{align}
x^2 - 3x + 2 &= 0 \\
(x - 1)(x - 2) &= 0 \\
x &= 1 \text{ or } x = 2
\end{align}

\end{document}
"#.to_string()
}

/// Display math statistics from HTML
fn display_math_statistics(html: &str) {
    let equation_count = html.matches("equation").count();
    let align_count = html.matches("align").count();
    let inline_math_count = html.matches("$").count() / 2;
    
    println!("  Equation environments: {}", equation_count);
    println!("  Align environments: {}", align_count);
    println!("  Inline math expressions: {}", inline_math_count);
}

/// Display equation numbers for labeled equations
fn display_equation_numbers(processor: &LaTeXProcessor) {
    if let Some(num1) = processor.get_equation_number("eq:first") {
        println!("  Equation 'eq:first' has number: {}", num1);
    }
    
    if let Some(num2) = processor.get_equation_number("eq:second") {
        println!("  Equation 'eq:second' has number: {}", num2);
    }
}