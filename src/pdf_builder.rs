use crate::parser::TexElement;
use crate::math_formatter::MathFormatter;
use lopdf::{Document, Object, Stream, Dictionary, StringFormat};
use lopdf::content::{Content, Operation};
use std::path::Path;

pub struct PdfBuilder {
    title: Option<String>,
    author: Option<String>,
    date: Option<String>,
}

impl PdfBuilder {
    pub fn new() -> Self {
        Self {
            title: None,
            author: None,
            date: None,
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
        
        // Create font file stream
        let mut font_stream = Stream::new(Dictionary::new(), font_data);
        font_stream.dict.set("Length1", font_stream.content.len() as i64);
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
        
        // Create page content
        let mut content = Content { operations: vec![] };
        let mut y_position = 750.0; // Start from top
        let left_margin = 50.0;
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
        if let Some(title) = &self.title {
            content.operations.push(Operation::new("BT", vec![]));
            content.operations.push(Operation::new("Tf", vec!["F1".into(), 24.into()]));
            content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
            
            let title_bytes = Self::encode_utf16_be(title);
            content.operations.push(Operation::new("Tj", vec![Object::String(title_bytes, StringFormat::Hexadecimal)]));
            content.operations.push(Operation::new("ET", vec![]));
            y_position -= 30.0;
        }
        
        // Add author
        if let Some(author) = &self.author {
            content.operations.push(Operation::new("BT", vec![]));
            content.operations.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
            content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
            
            let author_bytes = Self::encode_utf16_be(author);
            content.operations.push(Operation::new("Tj", vec![Object::String(author_bytes, StringFormat::Hexadecimal)]));
            content.operations.push(Operation::new("ET", vec![]));
            y_position -= 20.0;
        }
        
        // Add date
        if let Some(date) = &self.date {
            let date_text = if date == "\\today" {
                chrono::Local::now().format("%B %d, %Y").to_string()
            } else {
                date.clone()
            };
            
            content.operations.push(Operation::new("BT", vec![]));
            content.operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));
            content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
            
            let date_bytes = Self::encode_utf16_be(&date_text);
            content.operations.push(Operation::new("Tj", vec![Object::String(date_bytes, StringFormat::Hexadecimal)]));
            content.operations.push(Operation::new("ET", vec![]));
            y_position -= 25.0;
        }
        
        // Process elements
        for elem in elements {
            match elem {
                TexElement::Section { level, title } => {
                    y_position -= 10.0;
                    let font_size = if level == 1 { 18.0 } else { 14.0 };
                    
                    content.operations.push(Operation::new("BT", vec![]));
                    content.operations.push(Operation::new("Tf", vec!["F1".into(), font_size.into()]));
                    content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
                    
                    let title_bytes = Self::encode_utf16_be(&title);
                    content.operations.push(Operation::new("Tj", vec![Object::String(title_bytes, StringFormat::Hexadecimal)]));
                    content.operations.push(Operation::new("ET", vec![]));
                    y_position -= line_height + 5.0;
                }
                TexElement::Text(text) => {
                    let formatted_text = self.format_inline_math(&text);
                    let Some(validated_text) = self.normalize_text_element(&formatted_text) else {
                        continue;
                    };

                    let lines = self.wrap_text(&validated_text, 80);
                    
                    for line in lines {
                        if y_position < 50.0 {
                            y_position = 750.0; // New page would go here
                        }
                        
                        content.operations.push(Operation::new("BT", vec![]));
                        content.operations.push(Operation::new("Tf", vec!["F1".into(), 11.into()]));
                        content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
                        
                        let line_bytes = Self::encode_utf16_be(&line);
                        content.operations.push(Operation::new("Tj", vec![Object::String(line_bytes, StringFormat::Hexadecimal)]));
                        content.operations.push(Operation::new("ET", vec![]));
                        y_position -= line_height;
                    }
                }
                TexElement::MathInline(math) => {
                    let formatted = MathFormatter::format(&math);
                    
                    content.operations.push(Operation::new("BT", vec![]));
                    content.operations.push(Operation::new("Tf", vec!["F1".into(), 11.into()]));
                    content.operations.push(Operation::new("Td", vec![left_margin.into(), y_position.into()]));
                    
                    let math_bytes = Self::encode_utf16_be(&formatted);
                    content.operations.push(Operation::new("Tj", vec![Object::String(math_bytes, StringFormat::Hexadecimal)]));
                    content.operations.push(Operation::new("ET", vec![]));
                    y_position -= line_height;
                }
                TexElement::MathDisplay(math) => {
                    y_position -= 5.0;
                    let formatted = MathFormatter::format(&math);
                    
                    content.operations.push(Operation::new("BT", vec![]));
                    content.operations.push(Operation::new("Tf", vec!["F1".into(), 13.into()]));
                    content.operations.push(Operation::new("Td", vec![(left_margin + 30.0).into(), y_position.into()]));
                    
                    let math_bytes = Self::encode_utf16_be(&formatted);
                    content.operations.push(Operation::new("Tj", vec![Object::String(math_bytes, StringFormat::Hexadecimal)]));
                    content.operations.push(Operation::new("ET", vec![]));
                    y_position -= line_height + 5.0;
                }
                TexElement::ItemList { ordered, items } => {
                    for (idx, item) in items.iter().enumerate() {
                        let bullet = if ordered {
                            format!("{}.", idx + 1)
                        } else {
                            "•".to_string()
                        };
                        
                        content.operations.push(Operation::new("BT", vec![]));
                        content.operations.push(Operation::new("Tf", vec!["F1".into(), 11.into()]));
                        content.operations.push(Operation::new("Td", vec![(left_margin + 10.0).into(), y_position.into()]));
                        
                        let bullet_bytes = Self::encode_utf16_be(&bullet);
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
                        content.operations.push(Operation::new("Td", vec![(left_margin + 25.0).into(), y_position.into()]));
                        
                        let item_bytes = Self::encode_utf16_be(item_text.trim());
                        content.operations.push(Operation::new("Tj", vec![Object::String(item_bytes, StringFormat::Hexadecimal)]));
                        content.operations.push(Operation::new("ET", vec![]));
                        y_position -= line_height;
                    }
                }
                TexElement::Paragraph => {
                    y_position -= line_height;
                }
                _ => {}
            }
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
    
    fn encode_utf16_be(text: &str) -> Vec<u8> {
        let mut result = vec![0xFE, 0xFF]; // BOM for UTF-16BE
        for ch in text.chars() {
            let code = ch as u32;
            if code <= 0xFFFF {
                result.push((code >> 8) as u8);
                result.push((code & 0xFF) as u8);
            } else {
                // Surrogate pair for characters > U+FFFF
                let code = code - 0x10000;
                let high = 0xD800 + (code >> 10);
                let low = 0xDC00 + (code & 0x3FF);
                result.push((high >> 8) as u8);
                result.push((high & 0xFF) as u8);
                result.push((low >> 8) as u8);
                result.push((low & 0xFF) as u8);
            }
        }
        result
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

    fn normalize_text_element(&self, text: &str) -> Option<String> {
        let mut normalized = String::with_capacity(text.len());
        let mut previous_was_whitespace = false;

        for ch in text.chars() {
            if ch.is_control() && !ch.is_whitespace() {
                continue;
            }

            if ch.is_whitespace() {
                if !previous_was_whitespace {
                    normalized.push(' ');
                    previous_was_whitespace = true;
                }
            } else {
                normalized.push(ch);
                previous_was_whitespace = false;
            }
        }

        let trimmed = normalized.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn char_count(text: &str) -> usize {
        text.chars().count()
    }

    fn split_by_char_count(text: &str, max_chars: usize) -> Vec<String> {
        if max_chars == 0 {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        let mut current_len = 0;

        for ch in text.chars() {
            if current_len == max_chars {
                chunks.push(current);
                current = String::new();
                current_len = 0;
            }
            current.push(ch);
            current_len += 1;
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }
    
    fn wrap_text(&self, text: &str, max_chars: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if Self::char_count(word) > max_chars {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }

                for chunk in Self::split_by_char_count(word, max_chars) {
                    lines.push(chunk);
                }
                continue;
            }

            let additional = if current_line.is_empty() {
                Self::char_count(word)
            } else {
                Self::char_count(word) + 1
            };

            if Self::char_count(&current_line) + additional > max_chars {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        if lines.is_empty() {
            lines.push(String::new());
        }
        
        lines
    }
}

impl Default for PdfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::PdfBuilder;

    #[test]
    fn format_inline_math_preserves_unmatched_dollar() {
        let builder = PdfBuilder::new();
        let input = "Price starts at $99";

        let output = builder.format_inline_math(input);

        assert_eq!(output, "Price starts at $99");
    }

    #[test]
    fn normalize_text_element_removes_controls_and_collapses_whitespace() {
        let builder = PdfBuilder::new();
        let input = "  Hello\u{0007}\tworld\n\n from\r\n latex-rs  ";

        let output = builder.normalize_text_element(input);

        assert_eq!(output, Some("Hello world from latex-rs".to_string()));
    }

    #[test]
    fn wrap_text_splits_long_words_by_char_count() {
        let builder = PdfBuilder::new();
        let input = "abcdefghijk";

        let lines = builder.wrap_text(input, 4);

        assert_eq!(lines, vec!["abcd", "efgh", "ijk"]);
    }
}
