use rtex::TexParser;

fn main() {
    let content = r"demonstrate the \texttt{latex-rs} TeX to PDF converter.";
    
    let mut parser = TexParser::new(content.to_string());
    let elements = parser.parse();
    
    println!("Input: {}", content);
    println!("\nParsed elements:");
    for (i, elem) in elements.iter().enumerate() {
        println!("  {}: {:?}", i, elem);
    }
}
