use rtex::{TexParser, convert_tex_string_to_pdf_bytes};
use std::fs;

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
    let _elements = parser.parse();

    let pdf = convert_tex_string_to_pdf_bytes(content).unwrap();
    fs::write("output/font_test.pdf", pdf).unwrap();

    println!("Font test PDF generated: output/font_test.pdf");
}
