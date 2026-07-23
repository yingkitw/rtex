// Quick test to check PDF text rendering
use std::fs;

fn main() {
    // Read a generated PDF
    let pdf_data = fs::read("test_output/text_element/text_element.pdf").unwrap();

    // Find content stream
    let pdf_str = String::from_utf8_lossy(&pdf_data);

    // Look for text operations
    if let Some(start) = pdf_str.find("BT\n") {
        let end = pdf_str[start..].find("ET\n").unwrap_or(200);
        let content = &pdf_str[start..start + end + 3];
        println!("Content stream sample:\n{}", content);
    }

    // Check for hex strings
    let hex_count = pdf_str.matches("<").count();
    println!("\nHex strings found: {}", hex_count);

    // Sample hex string
    if let Some(start) = pdf_str.find("<") {
        let end = pdf_str[start..].find(">").unwrap_or(100);
        let hex = &pdf_str[start..start + end.min(100) + 1];
        println!("Sample hex string: {}", hex);
    }
}
