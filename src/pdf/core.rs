//! Custom PDF generation core — learned from pdfrs.
//!
//! Replaces the `lopdf` dependency with simple, maintainable PDF generation.

use std::io::Write;

static HEX_TABLE: &[u8; 16] = b"0123456789ABCDEF";

/// PDF object representation
#[derive(Debug)]
pub struct PdfObject {
    pub id: u32,
    pub generation: u32,
    pub content: String,
    pub is_stream: bool,
    pub stream_data: Option<Vec<u8>>,
}

/// PDF generator - manages objects and generates final PDF
pub struct PdfGenerator {
    pub objects: Vec<PdfObject>,
    pub next_id: u32,
    info_id: Option<u32>,
}

impl PdfGenerator {
    pub fn new() -> Self {
        PdfGenerator {
            objects: Vec::new(),
            next_id: 1,
            info_id: None,
        }
    }

    /// Set the Info dictionary object ID for the trailer.
    pub fn set_info(&mut self, id: u32) {
        self.info_id = Some(id);
    }

    /// Add a dictionary object and return its ID
    pub fn add_object(&mut self, content: String) -> u32 {
        let id = self.next_id;
        self.objects.push(PdfObject {
            id,
            generation: 0,
            content,
            is_stream: false,
            stream_data: None,
        });
        self.next_id += 1;
        id
    }

    /// Add a stream object (dictionary + data) and return its ID
    pub fn add_stream_object(&mut self, dictionary: String, data: Vec<u8>) -> u32 {
        let id = self.next_id;
        self.objects.push(PdfObject {
            id,
            generation: 0,
            content: dictionary,
            is_stream: true,
            stream_data: Some(data),
        });
        self.next_id += 1;
        id
    }

    /// Generate the complete PDF as bytes
    pub fn generate(&self) -> Vec<u8> {
        let mut pdf = Vec::new();

        // PDF header (version 1.4)
        pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

        // Calculate offsets for xref table
        let mut offsets = Vec::with_capacity(self.objects.len());
        let mut current_offset = pdf.len() as u32;

        // Write objects and collect offsets
        for obj in &self.objects {
            offsets.push(current_offset);
            let _ = writeln!(&mut pdf, "{} {} obj", obj.id, obj.generation);
            pdf.extend_from_slice(obj.content.as_bytes());

            // Stream data if present
            if obj.is_stream
                && let Some(data) = &obj.stream_data {
                    pdf.extend_from_slice(b"stream\n");
                    pdf.extend_from_slice(data);
                    pdf.extend_from_slice(b"\nendstream\n");
                }

            pdf.extend_from_slice(b"endobj\n");
            current_offset = pdf.len() as u32;
        }

        // xref table
        let xref_offset = pdf.len() as u32;
        let _ = writeln!(&mut pdf, "xref\n0 {}", self.objects.len() + 1);
        pdf.extend_from_slice(b"0000000000 65535 f \n");

        for offset in offsets {
            let _ = writeln!(&mut pdf, "{:010} 00000 n ", offset);
        }

        // trailer
        pdf.extend_from_slice(b"trailer\n");
        pdf.extend_from_slice(b"<<\n");
        let _ = writeln!(&mut pdf, "/Size {}", self.objects.len() + 1);
        if !self.objects.is_empty() {
            // Root is the last object (catalog)
            let _ = writeln!(&mut pdf, "/Root {} 0 R", self.objects.len());
        }
        if let Some(info_id) = self.info_id {
            let _ = writeln!(&mut pdf, "/Info {} 0 R", info_id);
        }
        pdf.extend_from_slice(b">>\n");
        pdf.extend_from_slice(b"startxref\n");
        let _ = writeln!(&mut pdf, "{}", xref_offset);
        pdf.extend_from_slice(b"%%EOF\n");

        pdf
    }

    /// Write PDF to file
    pub fn write_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let pdf_bytes = self.generate();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&pdf_bytes)?;
        Ok(())
    }
}

impl Default for PdfGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create PDF dictionary strings
pub struct DictBuilder {
    entries: Vec<(String, String)>,
}

impl DictBuilder {
    pub fn new() -> Self {
        DictBuilder {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: &str) -> &mut Self {
        self.entries.push((key.to_string(), value.to_string()));
        self
    }

    #[allow(dead_code)]
    pub fn add_ref(&mut self, key: &str, obj_id: u32) -> &mut Self {
        self.entries.push((key.to_string(), format!("{} 0 R", obj_id)));
        self
    }

    #[allow(dead_code)]
    pub fn add_array(&mut self, key: &str, values: &[String]) -> &mut Self {
        let array = format!("[{}]", values.join(" "));
        self.entries.push((key.to_string(), array));
        self
    }

    pub fn build(&self) -> String {
        let mut dict = String::with_capacity(self.entries.len() * 32 + 8);
        dict.push_str("<<\n");
        for (key, value) in &self.entries {
            dict.push('/');
            dict.push_str(key);
            dict.push(' ');
            dict.push_str(value);
            dict.push('\n');
        }
        dict.push_str(">>\n");
        dict
    }
}

impl Default for DictBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Content stream builder for PDF pages
pub struct ContentStream {
    operations: Vec<u8>,
}

impl ContentStream {
    pub fn new() -> Self {
        ContentStream {
            operations: Vec::new(),
        }
    }

    /// Begin text block
    pub fn begin_text(&mut self) {
        self.operations.extend_from_slice(b"BT\n");
    }

    /// End text block
    pub fn end_text(&mut self) {
        self.operations.extend_from_slice(b"ET\n");
    }

    /// Set font and size
    pub fn set_font(&mut self, font_name: &str, size: f32) {
        let _ = writeln!(&mut self.operations, "/{} {} Tf", font_name, size);
    }

    /// Set text position
    pub fn set_position(&mut self, x: f32, y: f32) {
        let _ = writeln!(&mut self.operations, "{} {} Td", x, y);
    }

    /// Set text rise (baseline offset) in points — positive = superscript, negative = subscript.
    pub fn set_text_rise(&mut self, rise: f32) {
        let _ = writeln!(&mut self.operations, "{} Ts", rise);
    }

    /// Set text rendering mode: 0=fill, 1=stroke, 2=fill+stroke, 3=invisible, 4=fill+clip, etc.
    pub fn set_text_rendering_mode(&mut self, mode: i32) {
        let _ = writeln!(&mut self.operations, "{} Tr", mode);
    }

    /// Save the current graphics state (`q`).
    pub fn save_state(&mut self) {
        self.operations.push(b'q');
        self.operations.push(b'\n');
    }

    /// Restore the previous graphics state (`Q`).
    pub fn restore_state(&mut self) {
        self.operations.push(b'Q');
        self.operations.push(b'\n');
    }

    /// Concatenate a transformation matrix to the current CTM (`a b c d e f cm`).
    pub fn concat_matrix(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let _ = writeln!(&mut self.operations, "{} {} {} {} {} {} cm", a, b, c, d, e, f);
    }

    /// Show text using UTF-16BE hex encoding for Unicode support
    pub fn show_text(&mut self, text: &str) {
        // Encode text as UTF-16BE
        let mut utf16_bytes = Vec::new();
        for ch in text.chars() {
            let code = ch as u32;
            if code <= 0xFFFF {
                utf16_bytes.push((code >> 8) as u8);
                utf16_bytes.push((code & 0xFF) as u8);
            } else {
                // Surrogate pair for characters above U+FFFF
                let code = code - 0x10000;
                let high = 0xD800 + (code >> 10);
                let low = 0xDC00 + (code & 0x3FF);
                utf16_bytes.push((high >> 8) as u8);
                utf16_bytes.push((high & 0xFF) as u8);
                utf16_bytes.push((low >> 8) as u8);
                utf16_bytes.push((low & 0xFF) as u8);
            }
        }
        
        // Write as hex string
        self.operations.push(b'<');
        for byte in utf16_bytes {
            self.operations.push(HEX_TABLE[(byte >> 4) as usize]);
            self.operations.push(HEX_TABLE[(byte & 0x0F) as usize]);
        }
        self.operations.extend_from_slice(b"> Tj\n");
    }

    /// Show text with per-segment kerning adjustments using the PDF `TJ`
    /// operator.  `segments` is a slice of `(text, adjustment)` pairs where
    /// `adjustment` is in thousandths of an em (negative = tighter).
    pub fn show_text_with_kerning(&mut self, segments: &[(String, i16)]) {
        self.operations.push(b'[');
        for (text, adj) in segments {
            let mut utf16_bytes = Vec::new();
            for ch in text.chars() {
                let code = ch as u32;
                if code <= 0xFFFF {
                    utf16_bytes.push((code >> 8) as u8);
                    utf16_bytes.push((code & 0xFF) as u8);
                } else {
                    let code = code - 0x10000;
                    let high = 0xD800 + (code >> 10);
                    let low = 0xDC00 + (code & 0x3FF);
                    utf16_bytes.push((high >> 8) as u8);
                    utf16_bytes.push((high & 0xFF) as u8);
                    utf16_bytes.push((low >> 8) as u8);
                    utf16_bytes.push((low & 0xFF) as u8);
                }
            }
            self.operations.push(b'<');
            for byte in utf16_bytes {
                self.operations.push(HEX_TABLE[(byte >> 4) as usize]);
                self.operations.push(HEX_TABLE[(byte & 0x0F) as usize]);
            }
            self.operations.push(b'>');
            let _ = write!(&mut self.operations, " {}", adj);
        }
        self.operations.extend_from_slice(b"] TJ\n");
    }

    /// Move to coordinate (x, y) for path construction.
    pub fn move_to(&mut self, x: f32, y: f32) {
        let _ = writeln!(&mut self.operations, "{} {} m", x, y);
    }

    /// Draw a line to coordinate (x, y).
    pub fn line_to(&mut self, x: f32, y: f32) {
        let _ = writeln!(&mut self.operations, "{} {} l", x, y);
    }

    /// Stroke the current path.
    pub fn stroke(&mut self) {
        self.operations.extend_from_slice(b"S\n");
    }

    /// Set the non-stroking (fill) RGB color.
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        let _ = writeln!(&mut self.operations, "{} {} {} rg", r, g, b);
    }

    /// Set the stroking RGB color.
    pub fn set_stroke_color(&mut self, r: f32, g: f32, b: f32) {
        let _ = writeln!(&mut self.operations, "{} {} {} RG", r, g, b);
    }

    /// Fill a rectangle at (x, y) with the given width and height.
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let _ = writeln!(&mut self.operations, "{} {} {} {} re f", x, y, width, height);
    }

    /// Stroke a rectangle at (x, y) with the given width and height.
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let _ = writeln!(&mut self.operations, "{} {} {} {} re S", x, y, width, height);
    }

    #[allow(dead_code)]
    /// Fill and stroke a rectangle at (x, y) with the given width and height.
    pub fn fill_and_stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let _ = writeln!(&mut self.operations, "{} {} {} {} re B", x, y, width, height);
    }

    /// Draw an image XObject at the given position and size.
    ///
    /// `name` is the resource name (e.g. "Im1").
    pub fn draw_image(&mut self, name: &str, x: f32, y: f32, width: f32, height: f32) {
        self.operations.extend_from_slice(b"q\n");
        let _ = writeln!(&mut self.operations, "{} 0 0 {} {} {} cm", width, height, x, y);
        let _ = writeln!(&mut self.operations, "/{} Do", name);
        self.operations.extend_from_slice(b"Q\n");
    }

    /// Get the content stream data
    pub fn data(&self) -> Vec<u8> {
        self.operations.clone()
    }
}

impl Default for ContentStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_generator_creates_valid_header() {
        let generator = PdfGenerator::new();
        let pdf = generator.generate();
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn test_add_object() {
        let mut generator = PdfGenerator::new();
        let id = generator.add_object("<< /Type /Test >>".to_string());
        assert_eq!(id, 1);
        assert_eq!(generator.objects.len(), 1);
    }

    #[test]
    fn test_dict_builder() {
        let mut dict = DictBuilder::new();
        dict.add("Type", "/Page")
            .add("MediaBox", "[0 0 612 792]");
        let result = dict.build();
        assert!(result.contains("/Type /Page"));
        assert!(result.contains("/MediaBox [0 0 612 792]"));
    }

    #[test]
    fn test_content_stream() {
        let mut stream = ContentStream::new();
        stream.begin_text();
        stream.set_font("F1", 12.0);
        stream.set_position(72.0, 720.0);
        stream.end_text();
        
        let data = stream.data();
        assert!(data.starts_with(b"BT\n"));
        assert!(data.ends_with(b"ET\n"));
    }

    #[test]
    fn test_add_stream_object() {
        let mut generator = PdfGenerator::new();
        let id = generator.add_stream_object(
            "<< /Type /Stream >>".to_string(),
            vec![0x48, 0x65, 0x6C, 0x6C, 0x6F],
        );
        assert_eq!(id, 1);
        assert_eq!(generator.objects.len(), 1);
        assert!(generator.objects[0].is_stream);
        assert_eq!(generator.objects[0].stream_data.as_ref().unwrap().len(), 5);
    }

    #[test]
    fn test_generate_with_objects() {
        let mut generator = PdfGenerator::new();
        generator.add_object("<< /Type /Catalog >>".to_string());
        let pdf = generator.generate();
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("xref"));
        assert!(pdf_str.contains("trailer"));
        assert!(pdf_str.contains("/Root 1 0 R"));
    }

    #[test]
    fn test_dict_builder_ref_and_array() {
        let mut dict = DictBuilder::new();
        dict.add("Type", "/Page")
            .add_ref("Parent", 2)
            .add_array("MediaBox", &["0".to_string(), "0".to_string(), "612".to_string(), "792".to_string()]);
        let result = dict.build();
        assert!(result.contains("/Parent 2 0 R"));
        assert!(result.contains("/MediaBox [0 0 612 792]"));
    }

    #[test]
    fn test_content_stream_show_text() {
        let mut stream = ContentStream::new();
        stream.begin_text();
        stream.show_text("Hi");
        stream.end_text();
        
        let data = stream.data();
        let text = String::from_utf8_lossy(&data);
        assert!(text.contains("BT"));
        assert!(text.contains("ET"));
        assert!(text.contains("> Tj"));
    }

    #[test]
    fn test_object_ids_are_sequential() {
        let mut generator = PdfGenerator::new();
        let id1 = generator.add_object("obj1".to_string());
        let id2 = generator.add_stream_object("obj2".to_string(), vec![]);
        let id3 = generator.add_object("obj3".to_string());
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_trailer_includes_info_dictionary() {
        let mut generator = PdfGenerator::new();
        let info_id = generator.add_object("<< /Title (Hello) >>".to_string());
        generator.set_info(info_id);
        generator.add_object("<< /Type /Catalog >>".to_string());

        let pdf = generator.generate();
        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("/Info 1 0 R"));
    }
}
