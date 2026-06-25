//! PDF document builder — converts parsed LaTeX elements into a PDF file.
//!
//! Uses the custom `pdf_core` module for low-level PDF generation,
//! DejaVu Sans for Unicode math support, and handles page layout,
//! text wrapping, and automatic page breaks.

use crate::parser::TexElement;
use crate::math_formatter::MathFormatter;
use crate::pdf_text_renderer::PdfTextRenderer;
use crate::pdf_core::{PdfGenerator, DictBuilder, ContentStream};
use std::path::Path;

/// Converts a sequence of `TexElement`s into a PDF file.
pub struct PdfBuilder {
    title: Option<String>,
    author: Option<String>,
    date: Option<String>,
}

impl PdfBuilder {
    /// Create a new builder with no metadata set.
    pub fn new() -> Self {
        Self {
            title: None,
            author: None,
            date: None,
        }
    }

    /// Build a PDF from `elements` and write it to `output_path`.
    pub fn build(&mut self, elements: Vec<TexElement>, output_path: &Path) -> Result<(), String> {
        let mut generator = PdfGenerator::new();

        // Load font
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts/DejaVuSans.ttf");
        let font_data = std::fs::read(&font_path).map_err(|e| format!("Failed to load font: {}", e))?;

        // Create font objects
        let (font_id, _font_descriptor_id, _cid_font_id, _to_unicode_id) =
            self.create_font_objects(&mut generator, &font_data)?;

        // Extract metadata from elements
        for elem in &elements {
            if let TexElement::Command { name, args } = elem {
                match name.as_str() {
                    "title" if !args.is_empty() => self.title = Some(args[0].clone()),
                    "author" if !args.is_empty() => self.author = Some(args[0].clone()),
                    "date" if !args.is_empty() => self.date = Some(args[0].clone()),
                    _ => {}
                }
            }
        }

        // Collect and embed images
        use std::collections::HashMap;
        let mut image_xobjects: HashMap<usize, (String, u32)> = HashMap::new();
        for (idx, elem) in elements.iter().enumerate() {
            if let TexElement::Image { path, .. } = elem {
                let img_path = std::path::Path::new(path);
                match crate::image::ImageInfo::from_path(img_path) {
                    Ok(info) => {
                        let dict = info.xobject_dict(info.data.len());
                        let img_id = generator.add_stream_object(dict, info.data);
                        image_xobjects.insert(idx, (format!("Im{}", idx), img_id));
                    }
                    Err(e) => eprintln!("Warning: could not load image '{}': {}", path, e),
                }
            }
        }

        // Build content stream (pass image xobjects so it can reference them)
        let content_data = self.build_content_stream(&elements, font_id, &image_xobjects)?;

        // Add content stream object
        let mut content_dict = DictBuilder::new();
        content_dict.add("Length", &content_data.len().to_string());
        let content_id = generator.add_stream_object(content_dict.build(), content_data);

        // Build XObject resource entries
        let mut xobj_entries = String::new();
        for (name, id) in image_xobjects.values() {
            xobj_entries.push_str(&format!("/{} {} 0 R ", name, id));
        }

        // Create resources dictionary
        let resources = if xobj_entries.is_empty() {
            format!(
                "<<\n/Font << /F1 {} 0 R >>\n>>\n",
                font_id
            )
        } else {
            format!(
                "<<\n/Font << /F1 {} 0 R >>\n/XObject << {}>>\n>>\n",
                font_id, xobj_entries
            )
        };

        // Create page object
        let page_content = format!(
            "<<\n/Type /Page\n/MediaBox [0 0 595 842]\n/Contents {} 0 R\n/Resources {}\n>>\n",
            content_id, resources
        );
        let page_id = generator.add_object(page_content);

        // Create pages object
        let pages_content = format!(
            "<<\n/Type /Pages\n/Kids [{} 0 R]\n/Count 1\n>>\n",
            page_id
        );
        let pages_id = generator.add_object(pages_content);

        // Update page to reference parent
        let page_with_parent = format!(
            "<<\n/Type /Page\n/Parent {} 0 R\n/MediaBox [0 0 595 842]\n/Contents {} 0 R\n/Resources {}\n>>\n",
            pages_id, content_id, resources
        );
        generator.objects[page_id as usize - 1].content = page_with_parent;

        // Create catalog
        let catalog_content = format!(
            "<<\n/Type /Catalog\n/Pages {} 0 R\n>>\n",
            pages_id
        );
        let _catalog_id = generator.add_object(catalog_content);

        // Write PDF to file
        generator.write_to_file(output_path)
            .map_err(|e| format!("Failed to write PDF: {}", e))?;

        Ok(())
    }
    
    fn create_font_objects(
        &self,
        generator: &mut PdfGenerator,
        font_data: &[u8],
    ) -> Result<(u32, u32, u32, u32), String> {
        // Compress font data
        let compressed_font = self.compress_font_data(font_data);
        
        // Create font file stream
        let font_file_dict = format!(
            "<<\n/Length {}\n/Length1 {}\n/Filter /FlateDecode\n>>\n",
            compressed_font.len(),
            font_data.len()
        );
        let font_file_id = generator.add_stream_object(font_file_dict, compressed_font);
        
        // Create font descriptor
        let font_descriptor = format!(
            "<<\n/Type /FontDescriptor\n/FontName /DejaVuSans\n/Flags 32\n\
            /FontBBox [-1069 -415 1975 2174]\n/ItalicAngle 0\n/Ascent 928\n\
            /Descent -236\n/CapHeight 729\n/StemV 80\n/FontFile2 {} 0 R\n>>\n",
            font_file_id
        );
        let font_descriptor_id = generator.add_object(font_descriptor);
        
        // Create CIDFont
        let cid_font = format!(
            "<<\n/Type /Font\n/Subtype /CIDFontType2\n/BaseFont /DejaVuSans\n\
            /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >>\n\
            /FontDescriptor {} 0 R\n/DW 600\n/CIDToGIDMap /Identity\n>>\n",
            font_descriptor_id
        );
        let cid_font_id = generator.add_object(cid_font);
        
        // Create ToUnicode CMap - maps character IDs to Unicode values
        let cmap_content = b"/CIDInit /ProcSet findresource begin\n\
12 dict begin\n\
begincmap\n\
/CIDSystemInfo\n\
<< /Registry (Adobe)\n\
/Ordering (UCS)\n\
/Supplement 0\n\
>> def\n\
/CMapName /Adobe-Identity-UCS def\n\
/CMapType 2 def\n\
1 begincodespacerange\n\
<0000> <FFFF>\n\
endcodespacerange\n\
100 beginbfchar\n\
<0020> <0020>\n\
<0021> <0021>\n\
<0028> <0028>\n\
<0029> <0029>\n\
<002B> <002B>\n\
<005B> <005B>\n\
<005D> <005D>\n\
<002D> <002D>\n\
<002F> <002F>\n\
<0030> <0030>\n\
<0031> <0031>\n\
<0032> <0032>\n\
<0033> <0033>\n\
<0034> <0034>\n\
<0035> <0035>\n\
<0036> <0036>\n\
<0037> <0037>\n\
<0038> <0038>\n\
<0039> <0039>\n\
<003D> <003D>\n\
<0041> <0041>\n\
<0042> <0042>\n\
<0043> <0043>\n\
<0044> <0044>\n\
<0045> <0045>\n\
<0046> <0046>\n\
<0047> <0047>\n\
<0048> <0048>\n\
<0049> <0049>\n\
<004A> <004A>\n\
<004B> <004B>\n\
<004C> <004C>\n\
<004D> <004D>\n\
<004E> <004E>\n\
<004F> <004F>\n\
<0050> <0050>\n\
<0051> <0051>\n\
<0052> <0052>\n\
<0053> <0053>\n\
<0054> <0054>\n\
<0055> <0055>\n\
<0056> <0056>\n\
<0057> <0057>\n\
<0058> <0058>\n\
<0059> <0059>\n\
<005A> <005A>\n\
<0061> <0061>\n\
<0062> <0062>\n\
<0063> <0063>\n\
<0064> <0064>\n\
<0065> <0065>\n\
<0066> <0066>\n\
<0067> <0067>\n\
<0068> <0068>\n\
<0069> <0069>\n\
<006A> <006A>\n\
<006B> <006B>\n\
<006C> <006C>\n\
<006D> <006D>\n\
<006E> <006E>\n\
<006F> <006F>\n\
<0070> <0070>\n\
<0071> <0071>\n\
<0072> <0072>\n\
<0073> <0073>\n\
<0074> <0074>\n\
<0075> <0075>\n\
<0076> <0076>\n\
<0077> <0077>\n\
<0078> <0078>\n\
<0079> <0079>\n\
<007A> <007A>\n\
<00B1> <00B1>\n\
<00B2> <00B2>\n\
<00B3> <00B3>\n\
<00BD> <00BD>\n\
<00BC> <00BC>\n\
<00BE> <00BE>\n\
<2153> <2153>\n\
<2154> <2154>\n\
<2155> <2155>\n\
<2156> <2156>\n\
<2157> <2157>\n\
<2158> <2158>\n\
<2159> <2159>\n\
<215A> <215A>\n\
<215B> <215B>\n\
<215C> <215C>\n\
<215D> <215D>\n\
<215E> <215E>\n\
<215F> <215F>\n\
<03B1> <03B1>\n\
<03B2> <03B2>\n\
<03B3> <03B3>\n\
<03B4> <03B4>\n\
<03B5> <03B5>\n\
<03C0> <03C0>\n\
<03C3> <03C3>\n\
<03C9> <03C9>\n\
<207A> <207A>\n\
<207B> <207B>\n\
<2080> <2080>\n\
<2081> <2081>\n\
<2082> <2082>\n\
<2083> <2083>\n\
<2084> <2084>\n\
<2085> <2085>\n\
<2086> <2086>\n\
<2087> <2087>\n\
<2088> <2088>\n\
<2089> <2089>\n\
<221A> <221A>\n\
<221E> <221E>\n\
<222B> <222B>\n\
<2211> <2211>\n\
<00F7> <00F7>\n\
endbfchar\n\
endcmap\n\
CMapName currentdict /CMap defineresource pop\n\
end\n\
end";
        
        let cmap_dict = format!("<<\n/Length {}\n>>\n", cmap_content.len());
        let to_unicode_id = generator.add_stream_object(cmap_dict, cmap_content.to_vec());
        
        // Create Type0 font
        let type0_font = format!(
            "<<\n/Type /Font\n/Subtype /Type0\n/BaseFont /DejaVuSans\n\
            /Encoding /Identity-H\n/DescendantFonts [{} 0 R]\n/ToUnicode {} 0 R\n>>\n",
            cid_font_id, to_unicode_id
        );
        let font_id = generator.add_object(type0_font);
        
        Ok((font_id, font_descriptor_id, cid_font_id, to_unicode_id))
    }
    
    fn compress_font_data(&self, font_data: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(font_data).ok();
        encoder.finish().unwrap_or_else(|_| font_data.to_vec())
    }
    
    // Font compression no longer needed with standard fonts
    // fn compress_font_data removed
    
    fn build_content_stream(&mut self, elements: &[TexElement], _font_id: u32, image_xobjects: &std::collections::HashMap<usize, (String, u32)>) -> Result<Vec<u8>, String> {
        let mut stream = ContentStream::new();
        
        // Page setup
        let left_margin = 72.0;
        let line_height = 14.0;
        let mut y_position = 780.0;
        let page_width = 595.0;
        let right_margin = 72.0;
        let content_width = page_width - left_margin - right_margin;
        let chars_per_line = ((content_width / 6.0) as usize).clamp(60, 85);
        
        // Render title, author, date
        if let Some(title) = &self.title {
            stream.begin_text();
            stream.set_font("F1", 24.0);
            stream.set_position(left_margin, y_position);
            stream.show_text(title);
            stream.end_text();
            y_position -= 30.0;
        }
        
        if let Some(author) = &self.author {
            stream.begin_text();
            stream.set_font("F1", 12.0);
            stream.set_position(left_margin, y_position);
            stream.show_text(author);
            stream.end_text();
            y_position -= 20.0;
        }
        
        if let Some(date) = &self.date {
            let date_text = if date == "\\today" {
                chrono::Local::now().format("%B %d, %Y").to_string()
            } else {
                date.clone()
            };
            
            stream.begin_text();
            stream.set_font("F1", 10.0);
            stream.set_position(left_margin, y_position);
            stream.show_text(&date_text);
            stream.end_text();
            y_position -= 25.0;
        }
        
        // Process elements
        let mut accumulated_text = String::new();
        let mut current_y = y_position;
        
        for (elem_idx, elem) in elements.iter().enumerate() {
            match elem {
                TexElement::Section { level, title } => {
                    // Flush accumulated text
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }
                    
                    current_y -= 10.0;
                    let font_size = if *level == 1 { 18.0 } else { 14.0 };
                    
                    stream.begin_text();
                    stream.set_font("F1", font_size);
                    stream.set_position(left_margin, current_y);
                    stream.show_text(title);
                    stream.end_text();
                    current_y -= line_height + 5.0;
                }
                TexElement::Text(text) => {
                    accumulated_text.push_str(text);
                    accumulated_text.push(' ');
                }
                TexElement::Table(table) => {
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }
                    current_y = crate::table::render_table(
                        table,
                        &mut stream,
                        left_margin,
                        current_y,
                        line_height,
                        content_width,
                    );
                }
                TexElement::MathInline(math) => {
                    let formatted = MathFormatter::format(math);
                    accumulated_text.push_str(&formatted);
                }
                TexElement::Paragraph => {
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }
                    current_y -= line_height;
                }
                TexElement::MathDisplay(math) => {
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }
                    
                    // Add spacing before equation
                    current_y -= 10.0;
                    let formatted = MathFormatter::format(math);
                    
                    // Add spacing around operators for better readability
                    let formatted = self.add_math_spacing(&formatted);
                    
                    // Render equation with larger font and centered indentation
                    stream.begin_text();
                    stream.set_font("F1", 14.0);
                    stream.set_position(left_margin + 40.0, current_y);
                    stream.show_text(&formatted);
                    stream.end_text();
                    
                    // Add spacing after equation
                    current_y -= line_height + 10.0;
                }
                TexElement::ItemList { ordered, items } => {
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }
                    
                    for (idx, item) in items.iter().enumerate() {
                        let bullet = if *ordered {
                            format!("{}.", idx + 1)
                        } else {
                            "•".to_string()
                        };
                        
                        stream.begin_text();
                        stream.set_font("F1", 11.0);
                        stream.set_position(left_margin + 10.0, current_y);
                        stream.show_text(&bullet);
                        stream.end_text();
                        
                        let mut item_text = String::new();
                        for elem in item {
                            if let TexElement::Text(t) = elem {
                                item_text.push_str(t);
                                item_text.push(' ');
                            }
                        }
                        
                        stream.begin_text();
                        stream.set_font("F1", 11.0);
                        stream.set_position(left_margin + 25.0, current_y);
                        stream.show_text(item_text.trim());
                        stream.end_text();
                        current_y -= line_height;
                    }
                }
                TexElement::CodeBlock(code) => {
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }

                    current_y -= 5.0;

                    stream.begin_text();
                    stream.set_font("F1", 10.0);
                    stream.set_position(left_margin + 10.0, current_y);
                    stream.show_text(code);
                    stream.end_text();
                    current_y -= line_height * (code.lines().count() as f32) + 5.0;
                }
                TexElement::Image { path: _, width, height } => {
                    if !accumulated_text.is_empty() {
                        current_y = self.render_text_block(
                            &mut stream,
                            &accumulated_text,
                            left_margin,
                            current_y,
                            line_height,
                            chars_per_line,
                        );
                        accumulated_text.clear();
                    }

                    // Look up the XObject by element index
                    if let Some((img_name, _)) = image_xobjects.get(&elem_idx) {
                        let img_width = width.as_ref()
                            .and_then(|w| crate::image::parse_dimension(w))
                            .unwrap_or(200.0);
                        let img_height = height.as_ref()
                            .and_then(|h| crate::image::parse_dimension(h))
                            .unwrap_or(img_width);

                        current_y -= 10.0;
                        stream.draw_image(img_name, left_margin, current_y - img_height, img_width, img_height);
                        current_y -= img_height + 10.0;
                    }
                }
                _ => {}
            }
        }
        
        // Flush remaining text
        if !accumulated_text.is_empty() {
            self.render_text_block(
                &mut stream,
                &accumulated_text,
                left_margin,
                current_y,
                line_height,
                chars_per_line,
            );
        }
        
        Ok(stream.data())
    }
    
    fn render_text_block(
        &mut self,
        stream: &mut ContentStream,
        text: &str,
        left_margin: f32,
        y_position: f32,
        line_height: f32,
        chars_per_line: usize,
    ) -> f32 {
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
            
            stream.begin_text();
            stream.set_font("F1", 11.0);
            stream.set_position(left_margin, current_y);
            stream.show_text(&line);
            stream.end_text();
            current_y -= line_height;
        }
        
        current_y
    }
    
    fn add_math_spacing(&self, math: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = math.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i];
            
            // Check for multi-character operators
            if i + 2 < chars.len() && chars[i..i+3] == ['+', '/', '-'] {
                result.push_str(" +/- ");
                i += 3;
                continue;
            }
            if i + 2 < chars.len() && chars[i..i+3] == ['-', '/', '+'] {
                result.push_str(" -/+ ");
                i += 3;
                continue;
            }
            
            // Add spaces around single operators
            match ch {
                '=' | '+' | '*' => {
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push(ch);
                    result.push(' ');
                }
                '-' => {
                    // Only add space if not part of a negative number
                    if i > 0 && !matches!(chars.get(i-1), Some(&'[') | Some(&'(') | Some(&' ') | Some(&'=')) {
                        if !result.ends_with(' ') {
                            result.push(' ');
                        }
                        result.push(ch);
                        result.push(' ');
                    } else {
                        result.push(ch);
                    }
                }
                ' ' => {
                    // Only add space if not already there
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
                _ => result.push(ch),
            }
            i += 1;
        }
        
        // Clean up multiple spaces
        let mut cleaned = result.trim().to_string();
        while cleaned.contains("  ") {
            cleaned = cleaned.replace("  ", " ");
        }
        
        cleaned
    }
    
    fn format_inline_math(&self, text: &str) -> String {
        let mut result = String::new();
        let chars = text.chars();
        let mut in_math = false;
        let mut math_buffer = String::new();
        
        for ch in chars {
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
