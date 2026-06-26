use rtex::{TexParser, PdfBuilder};
use std::path::Path;

fn main() {
    // Create a simple test with special characters
    let content = r"
\begin{document}
Special characters test:
• Bullet point
∞ Infinity symbol  
∫ Integral symbol
² Superscript 2
₀ Subscript 0
⁻ Superscript minus
ˣ Modifier letter x
\end{document}
";
    
    let mut parser = TexParser::new(content.to_string());
    let elements = parser.parse();
    
    let mut builder = PdfBuilder::new();
    builder.build(elements, Path::new("output/font_test.pdf")).unwrap();
    
    println!("Font test PDF generated: output/font_test.pdf");
}
