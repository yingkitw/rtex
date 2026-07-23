use rtex::TexParser;

fn main() {
    let content = r"This is a sample LaTeX document to demonstrate the \texttt{latex-rs} TeX to PDF converter.";

    let mut parser = TexParser::new(content.to_string());
    let elements = parser.parse();

    println!("Parsed elements:");
    for (i, elem) in elements.iter().enumerate() {
        println!("  {}: {:?}", i, elem);
    }

    // Test texttt parsing specifically
    let test_content = r"\texttt{latex-rs}";
    let mut parser2 = TexParser::new(test_content.to_string());
    let elements2 = parser2.parse();

    println!("\nTexttt test:");
    for elem in elements2 {
        println!("  {:?}", elem);
    }
}
