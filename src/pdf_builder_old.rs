use crate::parser::TexElement;
use crate::math_formatter::MathFormatter;
use crate::pdf_text_renderer::PdfTextRenderer;
use lopdf::{Document, Object, Stream, Dictionary, StringFormat};
use lopdf::content::{Content, Operation};
use std::collections::HashSet;
use std::path::Path;

pub struct PdfBuilder {
    title: Option<String>,
    author: Option<String>,
    date: Option<String>,
    used_chars: HashSet<char>,
}

impl PdfBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            author: None,
            date: None,
            used_chars: HashSet::new(),
        }
    }

    pub fn build(&mut self, elements: Vec<TexElement>, output_path: &Path) -> Result<(), String> {
        let mut doc = Document::with_version("1.7");
        
        // Load font
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts/DejaVuSans.ttf");
        let font_data = std::fs::read(&font_path).map_err(|e| format!("Failed to load font: {}", e))?;
        
        // Add font to document
        let font_id = doc.new_object_id();
        let font_descriptor_id = doc.new_object_id();
        let font_file_id = doc.new_object_id();
        
        // Create font file stream with compression
        let compressed_font = self.create_subset_font_data(&font_data);
        let mut font_stream = Stream::new(Dictionary::new(), compressed_font);
        font_stream.dict.set("Length1", font_data.len() as i64); // Original length
        font_stream.dict.set("Filter", "FlateDecode");
        doc.objects.insert(font_file_id, Object::Stream(font_stream));
        
        // Create font descriptor
        let mut font_descriptor = Dictionary::new();
        font_descriptor.set("Type", "FontDescriptor");
        font_descriptor.set("FontName", "DejaVuSans");
        font_descriptor.set("Flags", 32); // Symbolic
        font_descriptor.set("FontBBox", Object::Array(vec![
            Object::Integer(-1069),
            Object::Integer(-415),
            Object::Integer(1975),
            Object::Integer(2174),
        ]));
        font_descriptor.set("ItalicAngle", 0);
        font_descriptor.set("Ascent", 928);
        font_descriptor.set("Descent", -236);
        font_descriptor.set("CapHeight", 729);
        font_descriptor.set("StemV", 80);
        font_descriptor.set("FontFile2", font_file_id);
        doc.objects.insert(font_descriptor_id, Object::Dictionary(font_descriptor));
        
        // Create CIDFont
        let cid_font_id = doc.new_object_id();
        let mut cid_font = Dictionary::new();
        cid_font.set("Type", "Font");
        cid_font.set("Subtype", "CIDFontType2");
        cid_font.set("BaseFont", "DejaVuSans");
        cid_font.set("CIDSystemInfo", Dictionary::from_iter(vec![
            ("Registry", Object::string_literal("Adobe")),
            ("Ordering", Object::string_literal("Identity")),
            ("Supplement", 0.into()),
        ]));
        cid_font.set("FontDescriptor", font_descriptor_id);
        cid_font.set("DW", 1000); // Default width
        
        // Add CIDToGIDMap for proper glyph mapping
        cid_font.set("CIDToGIDMap", Object::string_literal("Identity"));
        
        // Add specific widths for commonly used characters
        let mut widths = Vec::new();
        // Add widths for math symbols and special characters
        // These are approximate widths for DejaVuSans
        widths.push(Object::Array(vec![
            0x221E.into(), // ∞
            1000.into()
        ]));
        widths.push(Object::Array(vec![
            0x222B.into(), // ∫
            1000.into()
        ]));
        widths.push(Object::Array(vec![
            0x2022.into(), // •
            600.into()
        ]));
        cid_font.set("W", Object::Array(widths));
        
        doc.objects.insert(cid_font_id, Object::Dictionary(cid_font));
        
        // Create Type0 font
        let mut font_dict = Dictionary::new();
        font_dict.set("Type", "Font");
        font_dict.set("Subtype", "Type0");
        font_dict.set("BaseFont", "DejaVuSans");
        font_dict.set("Encoding", "Identity-H");
        font_dict.set("DescendantFonts", Object::Array(vec![Object::Reference(cid_font_id)]));
        
        // Create ToUnicode CMap
        let to_unicode_id = doc.new_object_id();
        let cmap_content = b"/CIDInit /ProcSet findresource begin\n\
12 dict begin\n\
begincmap\n\
/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
/CMapName /Adobe-Identity-UCS def\n\
/CMapType 2 def\n\
1 begincodespacerange\n\
<0000> <FFFF>\n\
endcodespacerange\n\
1 beginbfrange\n\
<0000> <FFFF> <0000>\n\
endbfrange\n\
endcmap\n\
CMapName currentdict /CMap defineresource pop\n\
end\n\
end";
        let cmap_stream = Stream::new(Dictionary::new(), cmap_content.to_vec());
        doc.objects.insert(to_unicode_id, Object::Stream(cmap_stream));
        font_dict.set("ToUnicode", to_unicode_id);
        
        doc.objects.insert(font_id, Object::Dictionary(font_dict));
        
        // Create page content with better margins
        let mut content = Content { operations: vec![] };
        let mut y_position = 780.0; // Start higher for better top margin
        let left_margin = 72.0;  // 1 inch left margin
        let right_margin = 72.0; // 1 inch right margin
        let page_width = 595.0;
        let content_width = page_width - left_margin - right_margin;
        // Estimate chars per line based on average char width (about 6pt for 11pt font)
        let chars_per_line = ((content_width / 6.0) as usize).max(60).min(85);
        let line_height = 14.0;
        
        // Extract metadata
        for elem in &elements {
            match elem {
                TexElement::Command { name, args } => {
                    match name.as_str() {
                        "title" if !args.is_empty() => self.title = Some(args[0].clone()),
                        "author" if !args.is_empty() => self.author = Some(args[0].clone()),
                        "date" if !args.is_empty() => self.date = Some(args[0].clone()),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Add title
        if let Some(title) = self.title.clone() {
            content.operations.push(Operation::new("BT", vec![]));
            content.operations.push(Operation::new("Tf", vec!["F1".into(), 24.into()]));
            content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
            
            let title_bytes = PdfTextRenderer::encode_utf16_be(&title);
            content.operations.push(Operation::new("Tj", vec![Object::String(title_bytes, StringFormat::Hexadecimal)]));
            content.operations.push(Operation::new("ET", vec![]));
            y_position -= 30.0;
        }
        
        // Add author
        if let Some(author) = self.author.clone() {
            content.operations.push(Operation::new("BT", vec![]));
            content.operations.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
            content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
            
            let author_bytes = PdfTextRenderer::encode_utf16_be(&author);
            content.operations.push(Operation::new("Tj", vec![Object::String(author_bytes, StringFormat::Hexadecimal)]));
            content.operations.push(Operation::new("ET", vec![]));
            y_position -= 20.0;
        }
        
        // Add date
        if let Some(date) = self.date.clone() {
            let date_text = if date == "\\today" {
                chrono::Local::now().format("%B %d, %Y").to_string()
            } else {
                date.clone()
            };
            
            content.operations.push(Operation::new("BT", vec![]));
            content.operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));
            content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
            
            let date_bytes = PdfTextRenderer::encode_utf16_be(&date_text);
            content.operations.push(Operation::new("Tj", vec![Object::String(date_bytes, StringFormat::Hexadecimal)]));
            content.operations.push(Operation::new("ET", vec![]));
            y_position -= 25.0;
        }
        
        // Process elements
        let mut accumulated_text = String::new();
        let mut current_y = y_position;
        
        for elem in elements {
            match elem {
                TexElement::Section { level, title } => {
                    // Flush any accumulated text before section
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(&accumulated_text, &mut content, left_margin, current_y, line_height, chars_per_line);
                        accumulated_text.clear();
                    }
                    
                    current_y -= 10.0;
                    let font_size = if level == 1 { 18.0 } else { 14.0 };
                    
                    content.operations.push(Operation::new("BT", vec![]));
                    content.operations.push(Operation::new("Tf", vec!["F1".into(), font_size.into()]));
                    content.operations.push(Operation::new("Td", vec![left_margin.into(), current_y.into()]));
                    
                    let title_bytes = PdfTextRenderer::encode_utf16_be(&title);
                    content.operations.push(Operation::new("Tj", vec![Object::String(title_bytes, StringFormat::Hexadecimal)]));
                    content.operations.push(Operation::new("ET", vec![]));
                    current_y -= line_height + 5.0;
                }
                TexElement::Text(text) => {
                    accumulated_text.push_str(&text);
                    accumulated_text.push(' ');
                }
                TexElement::MathInline(math) => {
                    // Add math to accumulated text
                    let formatted = MathFormatter::format(&math);
                    accumulated_text.push_str(&formatted);
                }
                TexElement::Paragraph => {
                    // Flush accumulated text and add paragraph break
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(&accumulated_text, &mut content, left_margin, current_y, line_height, chars_per_line);
                        accumulated_text.clear();
                    }
                    current_y -= line_height; // Extra space for paragraph
                }
                TexElement::MathDisplay(math) => {
                    // Flush accumulated text before display math
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(&accumulated_text, &mut content, left_margin, current_y, line_height, chars_per_line);
                        accumulated_text.clear();
                    }
                    
                    current_y -= 5.0;
                    let formatted = MathFormatter::format(&math);
                    
                    content.operations.push(Operation::new("BT", vec![]));
                    content.operations.push(Operation::new("Tf", vec!["F1".into(), 13.into()]));
                    content.operations.push(Operation::new("Td", vec![(left_margin + 30.0).into(), current_y.into()]));
                    
                    let math_bytes = PdfTextRenderer::encode_utf16_be(&formatted);
                    content.operations.push(Operation::new("Tj", vec![Object::String(math_bytes, StringFormat::Hexadecimal)]));
                    content.operations.push(Operation::new("ET", vec![]));
                    current_y -= line_height + 5.0;
                }
                TexElement::ItemList { ordered, items } => {
                    // Flush accumulated text before list
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(&accumulated_text, &mut content, left_margin, current_y, line_height, chars_per_line);
                        accumulated_text.clear();
                    }
                    
                    for (idx, item) in items.iter().enumerate() {
                        let bullet = if ordered {
                            format!("{}.", idx + 1)
                        } else {
                            "•".to_string()
                        };
                        
                        content.operations.push(Operation::new("BT", vec![]));
                        content.operations.push(Operation::new("Tf", vec!["F1".into(), 11.into()]));
                        content.operations.push(Operation::new("Td", vec![(left_margin + 10.0).into(), current_y.into()]));
                        
                        let bullet_bytes = PdfTextRenderer::encode_utf16_be(&bullet);
                        content.operations.push(Operation::new("Tj", vec![Object::String(bullet_bytes, StringFormat::Hexadecimal)]));
                        content.operations.push(Operation::new("ET", vec![]));
                        
                        let mut item_text = String::new();
                        for elem in item {
                            if let TexElement::Text(t) = elem {
                                item_text.push_str(t);
                                item_text.push(' ');
                            }
                        }
                        
                        content.operations.push(Operation::new("BT", vec![]));
                        content.operations.push(Operation::new("Tf", vec!["F1".into(), 11.into()]));
                        content.operations.push(Operation::new("Td", vec![(left_margin + 25.0).into(), current_y.into()]));
                        
                        let item_bytes = PdfTextRenderer::encode_utf16_be(item_text.trim());
                        content.operations.push(Operation::new("Tj", vec![Object::String(item_bytes, StringFormat::Hexadecimal)]));
                        content.operations.push(Operation::new("ET", vec![]));
                        current_y -= line_height;
                    }
                }
                TexElement::CodeBlock(code) => {
                    // Flush accumulated text before code block
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(&accumulated_text, &mut content, left_margin, current_y, line_height, chars_per_line);
                        accumulated_text.clear();
                    }
                    
                    current_y -= 5.0;
                    
                    content.operations.push(Operation::new("BT", vec![]));
                    content.operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));
                    content.operations.push(Operation::new("Td", vec![(left_margin + 10.0).into(), current_y.into()]));
                    
                    let code_bytes = PdfTextRenderer::encode_utf16_be(&code);
                    content.operations.push(Operation::new("Tj", vec![Object::String(code_bytes, StringFormat::Hexadecimal)]));
                    content.operations.push(Operation::new("ET", vec![]));
                    current_y -= line_height * (code.lines().count() as f32) + 5.0;
                }
                _ => {}
            }
        }
        
        // Flush any remaining accumulated text
        if !accumulated_text.is_empty() {
            current_y = self.render_text_block(&accumulated_text, &mut content, left_margin, current_y, line_height, chars_per_line);
        }
        
        // Create page
        let content_id = doc.add_object(Stream::new(Dictionary::new(), content.encode().map_err(|e| e.to_string())?));
        
        let mut page = Dictionary::new();
        page.set("Type", "Page");
        page.set("MediaBox", vec![0.into(), 0.into(), 595.into(), 842.into()]); // A4
        page.set("Contents", content_id);
        
        let mut resources = Dictionary::new();
        let mut fonts = Dictionary::new();
        fonts.set("F1", font_id);
        resources.set("Font", fonts);
        page.set("Resources", resources);
        
        let page_id = doc.add_object(page);
        
        // Create pages object
        let pages_id = doc.add_object(Dictionary::from_iter(vec![
            ("Type", "Pages".into()),
            ("Kids", vec![page_id.into()].into()),
            ("Count", 1.into()),
        ]));
        
        // Update page parent
        if let Some(Object::Dictionary(page_dict)) = doc.objects.get_mut(&page_id) {
            page_dict.set("Parent", pages_id);
        }
        
        // Create catalog
        let catalog_id = doc.add_object(Dictionary::from_iter(vec![
            ("Type", "Catalog".into()),
            ("Pages", pages_id.into()),
        ]));
        
        doc.trailer.set("Root", catalog_id);
        
        // Save document
        doc.save(output_path).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    fn render_text_block(&mut self, text: &str, content: &mut Content, left_margin: f32, y_position: f32, line_height: f32, chars_per_line: usize) -> f32 {
        let mut current_y = y_position;
        
        let formatted_text = self.format_inline_math(text);
        let Some(validated_text) = PdfTextRenderer::normalize_text(&formatted_text) else {
            return current_y;
        };

        let lines = PdfTextRenderer::wrap_text(&validated_text, chars_per_line);
        
        for line in lines {
            if current_y < 50.0 {
                current_y = 750.0; // New page would go here
            }
            
            let line_bytes = PdfTextRenderer::encode_utf16_be(&line);
            self.collect_chars(&line);
            PdfTextRenderer::render_text(content, &line, "F1", 11.0, left_margin, current_y, line_bytes);
            current_y -= line_height;
        }
        
        current_y
    }

    fn collect_chars(&mut self, text: &str) {
        for ch in text.chars() {
            self.used_chars.insert(ch);
        }
    }

    fn create_subset_font_data(&self, full_font_data: &[u8]) -> Vec<u8> {
        // Compress font data to reduce PDF size
        use std::io::Write;
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(full_font_data).ok();
        encoder.finish().unwrap_or_else(|_| full_font_data.to_vec())
    }
    
    fn format_inline_math(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars();
        let mut in_math = false;
        let mut math_buffer = String::new();
        
        while let Some(ch) = chars.next() {
            if ch == '$' {
                if in_math {
                    let formatted = MathFormatter::format(&math_buffer);
                    result.push_str(&formatted);
                    math_buffer.clear();
                    in_math = false;
                } else {
                    in_math = true;
                }
            } else if in_math {
                math_buffer.push(ch);
            } else {
                result.push(ch);
            }
        }
        
        if !math_buffer.is_empty() {
            if in_math {
                // Preserve unmatched '$' as literal text instead of silently dropping it.
                result.push('$');
            }
            result.push_str(&math_buffer);
        }

        result
    }
}

impl Default for PdfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf_text_renderer::PdfTextRenderer;

    #[test]
    fn format_inline_math_preserves_unmatched_dollar() {
        let builder = PdfBuilder::new();
        let input = "Price starts at $99";

        let output = builder.format_inline_math(input);

        assert_eq!(output, "Price starts at $99");
    }

    #[test]
    fn normalize_text_element_removes_controls_and_collapses_whitespace() {
        let input = "  Hello\u{0007}\tworld\n\n from\r\n latex-rs  ";

        let output = PdfTextRenderer::normalize_text(input);

        assert_eq!(output, Some("Hello world from latex-rs".to_string()));
    }

    #[test]
    fn wrap_text_splits_long_words_by_char_count() {
        let input = "abcdefghijk";

        let lines = PdfTextRenderer::wrap_text(input, 4);

        assert_eq!(lines, vec!["abcd", "efgh", "ijk"]);
    }
}
