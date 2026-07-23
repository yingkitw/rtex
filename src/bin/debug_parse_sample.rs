use std::fs;

fn main() {
    let content = fs::read_to_string("examples/sample.tex").unwrap();

    // Simple parsing to find title/author/date
    println!("=== Checking LaTeX source ===");
    for line in content.lines() {
        if line.contains("\\title")
            || line.contains("\\author")
            || line.contains("\\date")
            || line.contains("\\maketitle")
        {
            println!("{}", line);
        }
    }
}
