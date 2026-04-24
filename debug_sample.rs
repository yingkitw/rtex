use std::path::Path;
use latex_rs::NativeTexConverter;

fn main() {
    let input = Path::new("examples/sample.tex");
    let output = Path::new("debug_sample.pdf");
    
    let result = NativeTexConverter::convert_file(input, output);
    match result {
        Ok(_) => println!("Converted successfully"),
        Err(e) => println!("Error: {}", e),
    }
}
