//! Custom PDF generation core — learned from pdfrs.
//!
//! Replaces the `lopdf` dependency with simple, maintainable PDF generation.

use std::io::Write;

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
}

impl PdfGenerator {
    pub fn new() -> Self {
        PdfGenerator {
            objects: Vec::new(),
            next_id: 1,
        }
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
        let mut offsets = Vec::new();
        let mut current_offset = pdf.len() as u32;

        // Write objects and collect offsets
        for obj in &self.objects {
            offsets.push(current_offset);
            
            // Object header
            let obj_header = format!("{} {} obj\n", obj.id, obj.generation);
            pdf.extend_from_slice(obj_header.as_bytes());
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
        pdf.extend_from_slice(format!("xref\n0 {}\n", self.objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");

        for offset in offsets {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        // trailer
        pdf.extend_from_slice(b"trailer\n");
        pdf.extend_from_slice(b"<<\n");
        pdf.extend_from_slice(format!("/Size {}\n", self.objects.len() + 1).as_bytes());
        if !self.objects.is_empty() {
            // Root is the last object (catalog)
            pdf.extend_from_slice(format!("/Root {} 0 R\n", self.objects.len()).as_bytes());
        }
        pdf.extend_from_slice(b">>\n");
        pdf.extend_from_slice(b"startxref\n");
        pdf.extend_from_slice(format!("{}\n", xref_offset).as_bytes());
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

    pub fn add_ref(&mut self, key: &str, obj_id: u32) -> &mut Self {
        self.entries.push((key.to_string(), format!("{} 0 R", obj_id)));
        self
    }

    pub fn add_array(&mut self, key: &str, values: &[String]) -> &mut Self {
        let array = format!("[{}]", values.join(" "));
        self.entries.push((key.to_string(), array));
        self
    }

    pub fn build(&self) -> String {
        let mut dict = String::from("<<\n");
        for (key, value) in &self.entries {
            dict.push_str(&format!("/{} {}\n", key, value));
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
        self.operations.extend_from_slice(
            format!("/{} {} Tf\n", font_name, size).as_bytes()
        );
    }

    /// Set text position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.operations.extend_from_slice(
            format!("{} {} Td\n", x, y).as_bytes()
        );
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
            self.operations.extend_from_slice(format!("{:02X}", byte).as_bytes());
        }
        self.operations.extend_from_slice(b"> Tj\n");
    }

    /// Move to coordinate (x, y) for path construction.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.operations.extend_from_slice(
            format!("{} {} m\n", x, y).as_bytes()
        );
    }

    /// Draw a line to coordinate (x, y).
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.operations.extend_from_slice(
            format!("{} {} l\n", x, y).as_bytes()
        );
    }

    /// Stroke the current path.
    pub fn stroke(&mut self) {
        self.operations.extend_from_slice(b"S\n");
    }

    /// Draw an image XObject at the given position and size.
    ///
    /// `name` is the resource name (e.g. "Im1").
    pub fn draw_image(&mut self, name: &str, x: f32, y: f32, width: f32, height: f32) {
        self.operations.extend_from_slice(b"q\n");
        self.operations.extend_from_slice(
            format!("{} 0 0 {} {} {} cm\n", width, height, x, y).as_bytes()
        );
        self.operations.extend_from_slice(format!("/{} Do\n", name).as_bytes());
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
}
