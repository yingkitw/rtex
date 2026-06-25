// Test to see how fractions are formatted
fn main() {
    // Simulate the fraction formatting
    let test_cases = vec![
        r"\frac{1}{2}",
        r"\frac{a}{b}",
        r"\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}",
    ];

    for test in test_cases {
        println!("LaTeX: {}", test);

        // What the format_fractions function does
        let result = format_fraction_simple(test);
        println!("Formatted: {}", result);
        println!();
    }

    // Test symbol replacement
    println!("Symbol replacements:");
    println!("\\pm -> {}", replace_pm("\\pm"));
    println!("\\sqrt -> {}", replace_sqrt("\\sqrt{x}"));
}

fn format_fraction_simple(latex: &str) -> String {
    // Simple version of what format_fractions does
    
    latex.replace(r"\frac{", "(")
        .replace("}{", ")/(")
        .replace("}", ")")
}

fn replace_pm(s: &str) -> String {
    s.replace("\\pm", "±")
}

fn replace_sqrt(s: &str) -> String {
    s.replace("\\sqrt", "√")
}
