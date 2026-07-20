//! PDF document builder — converts parsed LaTeX elements into a PDF file.
//!
//! Uses the custom `pdf_core` module for low-level PDF generation,
//! DejaVu Sans for Unicode math support, and handles page layout,
//! text wrapping, and automatic page breaks.

use crate::parser::{MathLineKind, TexElement};
use crate::math_formatter::MathFormatter;
use crate::pdf::text_renderer::PdfTextRenderer;
use crate::pdf::core::{PdfGenerator, DictBuilder};
use std::path::Path;

/// Flatten a slice of inline elements into a single string suitable for the
/// PDF text accumulator. Math delimiters (`$…$`) are inserted around inline
/// math so the downstream `format_inline_math` pass can format them, and
/// nested list / quote / center / abstract blocks are flattened recursively
/// so their text reaches the page instead of being dropped.
fn flatten_inline(elements: &[TexElement]) -> String {
    let mut out = String::new();
    for elem in elements {
        match elem {
            TexElement::Text(t) => {
                out.push_str(t);
                out.push(' ');
            }
            TexElement::MathInline(math) => {
                out.push('$');
                out.push_str(math);
                out.push('$');
                out.push(' ');
            }
            TexElement::MathDisplay(math) => {
                out.push_str(math);
                out.push(' ');
            }
            TexElement::ColoredText { text, .. } => {
                out.push_str(text);
                out.push(' ');
            }
            TexElement::Command { name, args } => {
                let inline_cmds = [
                    "textbf", "textit", "texttt", "emph", "textsuperscript",
                    "textsubscript", "text", "ensuremath", "textsc", "textrm",
                    "textsf", "textsl", "textup", "textmd", "underline",
                    "url",
                ];
                if inline_cmds.contains(&name.as_str()) {
                    if let Some(text) = args.first() {
                        out.push_str(text);
                        out.push(' ');
                    }
                } else if name == "href" && args.len() >= 2 {
                    out.push_str(&args[1]);
                    out.push(' ');
                }
            }
            TexElement::ItemList { items, .. } => {
                out.push_str("• ");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        out.push_str("; ");
                    }
                    out.push_str(&flatten_inline(item));
                }
            }
            TexElement::DescriptionList { items } => {
                for item in items {
                    if !item.term.is_empty() {
                        out.push_str(&item.term);
                        out.push_str(": ");
                    }
                    out.push_str(&flatten_inline(&item.body));
                }
            }
            TexElement::Theorem { kind, title, body } => {
                let mut heading = kind[..1].to_uppercase() + &kind[1..];
                if let Some(t) = title {
                    heading.push_str(&format!(" ({t})"));
                }
                heading.push_str(". ");
                out.push_str(&heading);
                out.push_str(&flatten_inline(body));
            }
            TexElement::Center(inner)
            | TexElement::Quote(inner)
            | TexElement::Abstract(inner) => {
                out.push_str(&flatten_inline(inner));
            }
            TexElement::Footnote { text } | TexElement::Caption { text } => {
                out.push_str(text);
                out.push(' ');
            }
            TexElement::Ref { key } | TexElement::PageRef { key } => {
                out.push_str(key);
                out.push(' ');
            }
            TexElement::Citation { keys } => {
                out.push('[');
                out.push_str(&keys.join(", "));
                out.push(']');
                out.push(' ');
            }
            TexElement::CodeBlock(code) => {
                out.push_str(code);
                out.push(' ');
            }
            _ => {}
        }
    }
    out
}

/// Converts a sequence of `TexElement`s into a PDF file.
pub struct PdfBuilder {
    title: Option<String>,
    author: Option<String>,
    date: Option<String>,
    plugins: Option<crate::plugins::PluginRegistry>,
    typography: Option<crate::typography::TypographyEngine>,
    template: Option<crate::template::DocumentTemplate>,
}

impl PdfBuilder {
    /// Create a new builder with no metadata set.
    pub fn new() -> Self {
        Self {
            title: None,
            author: None,
            date: None,
            plugins: None,
            typography: None,
            template: None,
        }
    }

    /// Attach a plugin registry for element transformation.
    pub fn with_plugins(mut self, plugins: crate::plugins::PluginRegistry) -> Self {
        self.plugins = Some(plugins);
        self
    }

    /// Attach a typography engine for ligatures and kerning.
    pub fn with_typography(mut self, engine: crate::typography::TypographyEngine) -> Self {
        self.typography = Some(engine);
        self
    }

    /// Attach a document template for page layout, fonts and colours.
    pub fn with_template(mut self, template: crate::template::DocumentTemplate) -> Self {
        self.template = Some(template);
        self
    }

    /// Build a PDF from `elements` and return the raw bytes.
    pub fn build_to_bytes(&mut self, elements: Vec<TexElement>) -> Result<Vec<u8>, String> {
        let mut elements = elements;
        if let Some(plugins) = self.plugins.as_mut() {
            elements = plugins.transform(elements);
        }
        let mut generator = PdfGenerator::new();

        // Subset and embed DejaVu only when Unicode/math characters are present
        let used_chars = crate::pdf::font_subset::collect_used_chars(&elements);
        let embedded_unicode = crate::pdf::font_subset::requires_embedded_font(&used_chars);
        let font_id = if embedded_unicode {
            let full_font_data = crate::fonts::DEJAVU_SANS.to_vec();
            let font_data = crate::pdf::font_subset::subset_font(&full_font_data, &used_chars)
                .unwrap_or(full_font_data);
            self.create_embedded_font_objects(&mut generator, &font_data)?.0
        } else {
            self.create_standard_font_objects(&mut generator)?
        };

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

        // Resolve page layout from template or default to A4 portrait
        let page_layout = self.template.as_ref().map(|t| t.page_layout()).unwrap_or_else(crate::page_layout::PageLayout::a4_portrait);
        let media_box = format!("[0 0 {} {}]", page_layout.width, page_layout.height);

        // Build content streams (one per page)
        let mut layout_state =
            self.build_content_stream(&elements, font_id, &image_xobjects, &page_layout, embedded_unicode)?;

        // Render footnotes at the bottom of each page
        let footnotes_per_page = std::mem::take(&mut layout_state.all_footnotes);
        for (page_idx, footnotes) in footnotes_per_page.iter().enumerate() {
            if !footnotes.is_empty() {
                self.render_page_footnotes(
                    &mut layout_state.pages[page_idx],
                    footnotes,
                    &page_layout,
                );
            }
        }

        // Build XObject resource entries
        let mut xobj_entries = String::new();
        for (name, id) in image_xobjects.values() {
            xobj_entries.push_str(&format!("/{} {} 0 R ", name, id));
        }

        // Create resources dictionary (shared across all pages)
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

        // Create a content stream and page object for each page
        let mut page_ids = Vec::new();
        for stream in &layout_state.pages {
            let content_data = stream.data();
            let compressed = compress_content(&content_data);
            let mut content_dict = DictBuilder::new();
            content_dict.add("Length", &compressed.len().to_string());
            content_dict.add("Filter", "/FlateDecode");
            let content_id = generator.add_stream_object(content_dict.build(), compressed);

            let page_content = format!(
                "<<\n/Type /Page\n/MediaBox {}\n/Contents {} 0 R\n/Resources {}\n>>\n",
                media_box, content_id, resources
            );
            let page_id = generator.add_object(page_content);
            page_ids.push((page_id, content_id));
        }

        // Create pages object
        let kids = page_ids.iter()
            .map(|(pid, _)| format!("{} 0 R", pid))
            .collect::<Vec<_>>()
            .join(" ");
        let pages_content = format!(
            "<<\n/Type /Pages\n/Kids [{}]\n/Count {}\n>>\n",
            kids, page_ids.len()
        );
        let pages_id = generator.add_object(pages_content);

        // Update each page to reference parent
        for (page_id, content_id) in &page_ids {
            let page_with_parent = format!(
                "<<\n/Type /Page\n/Parent {} 0 R\n/MediaBox [0 0 595 842]\n/Contents {} 0 R\n/Resources {}\n>>\n",
                pages_id, content_id, resources
            );
            generator.objects[*page_id as usize - 1].content = page_with_parent;
        }

        // Create Info dictionary
        let mut info_entries = Vec::new();
        if let Some(title) = &self.title {
            info_entries.push(format!("/Title {}", Self::pdf_string_literal(title)));
        }
        if let Some(author) = &self.author {
            info_entries.push(format!("/Author {}", Self::pdf_string_literal(author)));
        }
        if let Some(date) = &self.date {
            let date_text = if date == "\\today" {
                chrono::Local::now().format("%B %d, %Y").to_string()
            } else {
                date.clone()
            };
            info_entries.push(format!("/CreationDate {}", Self::pdf_string_literal(&date_text)));
        }
        info_entries.push(format!(
            "/Producer {} /Creator {}",
            Self::pdf_string_literal("latex-rs"),
            Self::pdf_string_literal("latex-rs")
        ));
        let info_content = format!(
            "<<\n{}\n>>\n",
            info_entries.join("\n")
        );
        let info_id = generator.add_object(info_content);
        generator.set_info(info_id);

        // Create catalog
        let catalog_content = format!(
            "<<\n/Type /Catalog\n/Pages {} 0 R\n>>\n",
            pages_id
        );
        let _catalog_id = generator.add_object(catalog_content);

        Ok(generator.generate())
    }

    /// Build a PDF from `elements` and write it to `output_path`.
    pub fn build(&mut self, elements: Vec<TexElement>, output_path: &Path) -> Result<(), String> {
        let pdf = self.build_to_bytes(elements)?;
        std::fs::write(output_path, &pdf)
            .map_err(|e| format!("Failed to write PDF: {}", e))
    }
    
    fn create_standard_font_objects(&self, generator: &mut PdfGenerator) -> Result<u32, String> {
        let font = "<<\n/Type /Font\n/Subtype /Type1\n/BaseFont /Helvetica\n/Encoding /WinAnsiEncoding\n>>\n";
        Ok(generator.add_object(font.to_string()))
    }

    fn create_embedded_font_objects(
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
}

/// Compress raw PDF content stream bytes with FlateDecode.
fn compress_content(data: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    if encoder.write_all(data).is_ok() {
        encoder.finish().unwrap_or_else(|_| data.to_vec())
    } else {
        data.to_vec()
    }
}

impl PdfBuilder {
    fn build_content_stream(
        &mut self,
        elements: &[TexElement],
        _font_id: u32,
        image_xobjects: &std::collections::HashMap<usize, (String, u32)>,
        page_layout: &crate::page_layout::PageLayout,
        embedded_unicode: bool,
    ) -> Result<crate::layout::LayoutState, String> {
        use crate::layout::LayoutState;

        let mut state = LayoutState::with_encoding(*page_layout, embedded_unicode);
        // Resolve style values from template or use defaults
        let tp = self.template.as_ref().map(|t| &t.title_page);
        let base_font_size = self.template.as_ref().map(|t| t.base_font_size).unwrap_or(11.0);
        let mut line_height = state.line_height(base_font_size);

        let title_font_size = tp.map(|tp| tp.title_font_size).unwrap_or(24.0);
        let title_spacing = tp.map(|tp| tp.spacing_after_title).unwrap_or(30.0);
        let author_font_size = tp.map(|tp| tp.author_font_size).unwrap_or(12.0);
        let author_spacing = tp.map(|tp| tp.spacing_after_author).unwrap_or(20.0);
        let date_font_size = tp.map(|tp| tp.date_font_size).unwrap_or(10.0);
        let date_spacing = tp.map(|tp| tp.spacing_after_date).unwrap_or(25.0);

        // Render title, author, date on the first page (if title_page is enabled)
        let title_page_enabled = tp.map(|tp| tp.enabled).unwrap_or(true);
        if title_page_enabled {
            if let Some(title) = &self.title {
                state.ensure_space(title_spacing);
                let x = state.left_margin();
                let y = state.current_y;
                let stream = state.current_stream();
                stream.begin_text();
                stream.set_font("F1", title_font_size);
                stream.set_position(x, y);
                stream.show_text(title);
                stream.end_text();
                state.advance(title_spacing);
            }

            if let Some(author) = &self.author {
                state.ensure_space(author_spacing);
                let x = state.left_margin();
                let y = state.current_y;
                let stream = state.current_stream();
                stream.begin_text();
                stream.set_font("F1", author_font_size);
                stream.set_position(x, y);
                stream.show_text(author);
                stream.end_text();
                state.advance(author_spacing);
            }

            if let Some(date) = &self.date {
                state.ensure_space(date_spacing);
                let date_text = if date == "\\today" {
                    chrono::Local::now().format("%B %d, %Y").to_string()
                } else {
                    date.clone()
                };
                let x = state.left_margin();
                let y = state.current_y;
                let stream = state.current_stream();
                stream.begin_text();
                stream.set_font("F1", date_font_size);
                stream.set_position(x, y);
                stream.show_text(&date_text);
                stream.end_text();
                state.advance(date_spacing);
            }
        }

        // Pre-scan for bibliography entries to build key→number map
        let mut bib_entries: Vec<(String, String)> = Vec::new();
        for elem in elements.iter() {
            if let TexElement::Bibliography { entries } = elem {
                for entry in entries {
                    bib_entries.push((entry.key.clone(), entry.text.clone()));
                }
            }
        }
        let citation_map = crate::bibliography::build_citation_map(&bib_entries);

        // Pre-scan for cross-references
        let mut ref_store = crate::references::RefStore::new();
        ref_store.scan(elements);

        // Process elements
        let mut accumulated_text = String::new();
        let mut footnote_counter = 0;
        let mut sections_seen: Vec<(usize, String, usize)> = Vec::new();

        for (elem_idx, elem) in elements.iter().enumerate() {
            match elem {
                TexElement::Section { level, title } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    sections_seen.push((*level, title.clone(), state.pages.len()));
                    let headings = self.template.as_ref().map(|t| t.headings.clone()).unwrap_or_default();
                    let font_size = match *level {
                        0 => 25.0,
                        1 => headings.h1,
                        2 => headings.h2,
                        3 => headings.h3,
                        4 => headings.h4,
                        5 => headings.h5,
                        6 => 18.0,
                        _ => headings.h6,
                    };
                    state.ensure_space(font_size + 6.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", font_size);
                    stream.set_position(x, y);
                    stream.show_text(title);
                    stream.end_text();
                    state.advance(font_size + 6.0);
                }
                TexElement::TableOfContents => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(30.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 16.0);
                    stream.set_position(x, y);
                    stream.show_text("Contents");
                    stream.end_text();
                    state.advance(24.0);

                    for (level, title, page) in &sections_seen {
                        let indent = (*level as f32 - 1.0) * 15.0;
                        let entry_font_size = 11.0;
                        state.ensure_space(entry_font_size + 2.0);
                        let x = state.left_margin() + indent;
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.begin_text();
                        stream.set_font("F1", entry_font_size);
                        stream.set_position(x, y);
                        let entry_text = format!("{} {}", title, page);
                        stream.show_text(&entry_text);
                        stream.end_text();
                        state.advance(entry_font_size + 2.0);
                    }
                    state.advance(10.0);
                }
                TexElement::Text(text) => {
                    accumulated_text.push_str(text);
                    accumulated_text.push(' ');
                }
                TexElement::Table(table) => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(50.0);
                    let left = state.left_margin();
                    let width = state.content_width();
                    let current_y = state.current_y;
                    state.current_y = crate::table::render_table(
                        table,
                        state.current_stream(),
                        left,
                        current_y,
                        line_height,
                        width,
                    );
                }
                TexElement::MathInline(math) => {
                    let formatted = MathFormatter::format(math);
                    accumulated_text.push_str(&formatted);
                }
                TexElement::ColoredText { color, text } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(20.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    if let Some(c) = crate::color::Color::parse(color) {
                        stream.set_color(c.r, c.g, c.b);
                    }
                    stream.begin_text();
                    stream.set_font("F1", 11.0);
                    stream.set_position(x, y);
                    stream.show_text(text);
                    stream.end_text();
                    stream.set_color(0.0, 0.0, 0.0);
                    state.advance(20.0);
                }
                TexElement::Paragraph => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.advance(line_height);
                }
                TexElement::MathDisplay(math) => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(40.0);
                    let formatted = MathFormatter::format(math);
                    let formatted = self.add_math_spacing(&formatted);
                    let x = state.left_margin() + 40.0;
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 14.0);
                    stream.set_position(x, y);
                    stream.show_text(&formatted);
                    stream.end_text();
                    state.advance(line_height + 10.0);
                }
                TexElement::MathLines { lines, kind } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let math_line_height = state.line_height(14.0);
                    state.ensure_space(lines.len() as f32 * (math_line_height + 4.0) + 10.0);
                    let total = lines.len();
                    let content_width = state.content_width();
                    for (idx, line) in lines.iter().enumerate() {
                        let formatted = MathFormatter::format(line);
                        let formatted = self.add_math_spacing(&formatted);
                        let text_width = formatted.len() as f32 * 14.0 * 0.55;
                        let x = match kind {
                            MathLineKind::Multline if idx == 0 => state.left_margin(),
                            MathLineKind::Multline if idx + 1 == total => {
                                (state.left_margin() + content_width - text_width).max(state.left_margin())
                            }
                            _ => state.left_margin() + (content_width - text_width).max(0.0) / 2.0,
                        };
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.begin_text();
                        stream.set_font("F1", 14.0);
                        stream.set_position(x, y);
                        stream.show_text(&formatted);
                        stream.end_text();
                        state.advance(math_line_height + 4.0);
                    }
                }
                TexElement::ItemList { ordered, labels, items } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(items.len() as f32 * line_height);
                    for (idx, item) in items.iter().enumerate() {
                        let bullet = if let Some(label) = labels.get(idx).and_then(|l| l.as_ref()) {
                            label.clone()
                        } else if *ordered {
                            format!("{}.", idx + 1)
                        } else {
                            "•".to_string()
                        };
                        let x1 = state.left_margin() + 10.0;
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.begin_text();
                        stream.set_font("F1", 11.0);
                        stream.set_position(x1, y);
                        stream.show_text(&bullet);
                        stream.end_text();

                        let item_text = flatten_inline(item);
                        let x2 = state.left_margin() + 25.0;
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.begin_text();
                        stream.set_font("F1", 11.0);
                        stream.set_position(x2, y);
                        stream.show_text(item_text.trim());
                        stream.end_text();
                        state.advance(line_height);
                    }
                }
                TexElement::DescriptionList { items } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(items.len() as f32 * line_height);
                    for item in items {
                        if !item.term.is_empty() {
                            let x1 = state.left_margin();
                            let y = state.current_y;
                            let stream = state.current_stream();
                            stream.begin_text();
                            stream.set_font("F1", 11.0);
                            stream.set_position(x1, y);
                            stream.show_text(&item.term);
                            stream.end_text();

                            let body_text = flatten_inline(&item.body);
                            let x2 = state.left_margin() + 80.0;
                            let stream = state.current_stream();
                            stream.begin_text();
                            stream.set_font("F1", 11.0);
                            stream.set_position(x2, y);
                            stream.show_text(&body_text);
                            stream.end_text();
                        } else {
                            let body_text = flatten_inline(&item.body);
                            let x = state.left_margin() + 20.0;
                            let y = state.current_y;
                            let stream = state.current_stream();
                            stream.begin_text();
                            stream.set_font("F1", 11.0);
                            stream.set_position(x, y);
                            stream.show_text(&body_text);
                            stream.end_text();
                        }
                        state.advance(line_height);
                    }
                }
                TexElement::Theorem { kind, title, body } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let mut heading = kind[..1].to_uppercase() + &kind[1..];
                    if let Some(t) = title {
                        heading.push_str(&format!(" ({t})"));
                    }
                    heading.push('.');
                    let indent = 16.0;
                    let saved_left = state.layout.margin_left;
                    let saved_right = state.layout.margin_right;
                    state.layout.margin_left += indent;
                    state.layout.margin_right += indent;
                    state.ensure_space(line_height * 2.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 11.0);
                    stream.set_position(x, y);
                    stream.show_text(&heading);
                    stream.end_text();
                    state.advance(line_height);
                    let body_text = flatten_inline(body);
                    if !body_text.trim().is_empty() {
                        self.render_text_block(&mut state, &body_text, line_height);
                    }
                    state.layout.margin_left = saved_left;
                    state.layout.margin_right = saved_right;
                    state.advance(line_height * 0.5);
                }
                TexElement::CodeBlock(code) => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let code_height = line_height * (code.lines().count() as f32) + 10.0;
                    state.ensure_space(code_height);
                    state.advance(5.0);
                    let x = state.left_margin() + 10.0;
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 10.0);
                    stream.set_position(x, y);
                    stream.show_text(code);
                    stream.end_text();
                    state.advance(code_height);
                }
                TexElement::Image { path: _, width, height } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    if let Some((img_name, _)) = image_xobjects.get(&elem_idx) {
                        let img_width = width.as_ref()
                            .and_then(|w| crate::image::parse_dimension(w))
                            .unwrap_or(200.0);
                        let img_height = height.as_ref()
                            .and_then(|h| crate::image::parse_dimension(h))
                            .unwrap_or(img_width);
                        let total_height = img_height + 20.0;
                        state.ensure_space(total_height);
                        state.advance(10.0);
                        let x = state.left_margin();
                        let y = state.current_y - img_height;
                        state.current_stream().draw_image(img_name, x, y, img_width, img_height);
                        state.advance(total_height - 10.0);
                    }
                }
                TexElement::Citation { keys } => {
                    let cite_text = crate::bibliography::format_citation(keys, &citation_map);
                    if !cite_text.is_empty() {
                        accumulated_text.push_str(&cite_text);
                        accumulated_text.push(' ');
                    }
                }
                TexElement::Bibliography { entries } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(30.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 16.0);
                    stream.set_position(x, y);
                    stream.show_text("References");
                    stream.end_text();
                    state.advance(line_height + 5.0);

                    for (idx, entry) in entries.iter().enumerate() {
                        let label = format!("[{}] ", idx + 1);
                        let full_text = format!("{}{}", label, entry.text);
                        self.render_text_block(&mut state, &full_text, line_height);
                    }
                }
                TexElement::Label { key } => {
                    let page = state.pages.len();
                    ref_store.set_page(key, page);
                }
                TexElement::Ref { key } => {
                    let text = ref_store.resolve_ref(key).unwrap_or("??");
                    accumulated_text.push_str(text);
                    accumulated_text.push(' ');
                }
                TexElement::PageRef { key } => {
                    let text = ref_store
                        .resolve_pageref(key)
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "??".to_string());
                    accumulated_text.push_str(&text);
                    accumulated_text.push(' ');
                }
                TexElement::Command { name, args: _ } if matches!(name.as_str(), "newpage" | "clearpage" | "pagebreak") => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.new_page();
                }
                TexElement::Command { name, args } if name == "vspace" && !args.is_empty() => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let space = crate::tex::Dimension::parse(&args[0]).map(|d| d.pt() as f32).unwrap_or(0.0);
                    state.advance(space.max(0.0));
                }
                TexElement::Command { name, args } if name == "underline" && !args.is_empty() => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let text = &args[0];
                    state.ensure_space(line_height);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let font_size = state.current_font_size;
                    let text_width = text.len() as f32 * font_size * 0.55;
                    {
                        let stream = state.current_stream();
                        stream.begin_text();
                        stream.set_font("F1", font_size);
                        stream.set_position(x, y);
                        stream.show_text(text);
                        stream.end_text();
                        // Draw underline beneath text
                        stream.move_to(x, y - 2.0);
                        stream.line_to(x + text_width, y - 2.0);
                        stream.stroke();
                    }
                    state.advance(line_height);
                }
                TexElement::Command { name, args } => {
                    if name == "centering" {
                        state.centering = true;
                        continue;
                    }
                    if name == "raggedright" || name == "flushleft" {
                        state.raggedleft = false;
                        continue;
                    }
                    if name == "raggedleft" || name == "flushright" {
                        state.raggedleft = true;
                        continue;
                    }
                    if name == "noindent" {
                        state.noindent = true;
                        continue;
                    }
                    // Text formatting — render the text even if we can't
                    // yet apply bold/italic/monospace styling.
                    if matches!(name.as_str(), "textbf" | "textit" | "texttt" | "emph" | "textsuperscript" | "textsubscript" | "text" | "ensuremath" | "textsc" | "textrm" | "textsf" | "textsl" | "textup" | "textmd") {
                        if let Some(text) = args.first() {
                            accumulated_text.push_str(text);
                            accumulated_text.push(' ');
                        }
                        continue;
                    }
                    if name == "overline" {
                        if let Some(text) = args.first() {
                            let with_overline: String = text.chars().map(|c| format!("{}̅", c)).collect();
                            accumulated_text.push_str(&with_overline);
                            accumulated_text.push(' ');
                        }
                        continue;
                    }
                    if name == "sout" {
                        if let Some(text) = args.first() {
                            let with_strike: String = text.chars().map(|c| format!("{}̶", c)).collect();
                            accumulated_text.push_str(&with_strike);
                            accumulated_text.push(' ');
                        }
                        continue;
                    }
                    if name == "today" {
                        let now = chrono::Local::now();
                        accumulated_text.push_str(&now.format("%B %e, %Y").to_string());
                        accumulated_text.push(' ');
                        continue;
                    }
                    if name == "url" {
                        if let Some(text) = args.first() {
                            accumulated_text.push_str(text);
                            accumulated_text.push(' ');
                        }
                        continue;
                    }
                    if name == "phantom" || name == "vphantom" || name == "hphantom" {
                        if let Some(text) = args.first()
                            && !text.is_empty() {
                                // Flush accumulated text first
                                if !accumulated_text.is_empty() {
                                    self.render_text_block(&mut state, &accumulated_text, line_height);
                                    accumulated_text.clear();
                                }
                                let font_size = state.current_font_size;
                                let x = state.left_margin();
                                let y = state.current_y;
                                let stream = state.current_stream();
                                stream.begin_text();
                                stream.set_font("F1", font_size);
                                stream.set_text_rendering_mode(3); // invisible
                                stream.set_position(x, y);
                                stream.show_text(text);
                                stream.set_text_rendering_mode(0); // back to visible
                                stream.end_text();
                                if name == "vphantom" || name == "phantom" {
                                    state.advance(line_height);
                                }
                                // For hphantom, we don't advance vertically; the width is consumed by the invisible text
                            }
                        continue;
                    }
                    if name == "raisebox" && args.len() >= 2 {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let distance = crate::tex::Dimension::parse(&args[0]).map(|d| d.pt() as f32).unwrap_or(0.0);
                        let text = &args[1];
                        let font_size = state.current_font_size;
                        state.ensure_space(font_size + distance.abs());
                        let x = state.left_margin();
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.begin_text();
                        stream.set_font("F1", font_size);
                        stream.set_text_rise(distance);
                        stream.set_position(x, y);
                        stream.show_text(text);
                        stream.set_text_rise(0.0);
                        stream.end_text();
                        state.advance(line_height);
                        continue;
                    }
                    if name == "rotatebox" && args.len() >= 2 {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let angle_deg: f32 = args[0].parse().unwrap_or(0.0);
                        let text = &args[1];
                        let font_size = state.current_font_size;
                        let theta = angle_deg * std::f32::consts::PI / 180.0;
                        let cos_t = theta.cos();
                        let sin_t = theta.sin();
                        let x = state.left_margin();
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.save_state();
                        // Translate to text position, rotate, then translate back
                        stream.concat_matrix(1.0, 0.0, 0.0, 1.0, x, y);
                        stream.concat_matrix(cos_t, sin_t, -sin_t, cos_t, 0.0, 0.0);
                        stream.concat_matrix(1.0, 0.0, 0.0, 1.0, -x, -y);
                        stream.begin_text();
                        stream.set_font("F1", font_size);
                        stream.set_position(x, y);
                        stream.show_text(text);
                        stream.end_text();
                        stream.restore_state();
                        state.advance(line_height);
                        continue;
                    }
                    if name == "scalebox" && args.len() >= 2 {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let scale: f32 = args[0].parse().unwrap_or(1.0);
                        let text = &args[1];
                        let font_size = state.current_font_size;
                        let x = state.left_margin();
                        let y = state.current_y;
                        let stream = state.current_stream();
                        stream.save_state();
                        // Translate to text position, scale, then translate back
                        stream.concat_matrix(1.0, 0.0, 0.0, 1.0, x, y);
                        stream.concat_matrix(scale, 0.0, 0.0, scale, 0.0, 0.0);
                        stream.concat_matrix(1.0, 0.0, 0.0, 1.0, -x, -y);
                        stream.begin_text();
                        stream.set_font("F1", font_size);
                        stream.set_position(x, y);
                        stream.show_text(text);
                        stream.end_text();
                        stream.restore_state();
                        state.advance(line_height);
                        continue;
                    }
                    if name == "fbox" && !args.is_empty() {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let text = &args[0];
                        state.ensure_space(line_height + 2.0);
                        let font_size = state.current_font_size;
                        let text_width = text.len() as f32 * font_size * 0.55;
                        let x = state.left_margin();
                        let y = state.current_y;
                        let padding = 2.0;
                        {
                            let stream = state.current_stream();
                            // Draw box around text
                            stream.move_to(x, y + font_size + padding);
                            stream.line_to(x + text_width + padding * 2.0, y + font_size + padding);
                            stream.line_to(x + text_width + padding * 2.0, y - padding);
                            stream.line_to(x, y - padding);
                            stream.line_to(x, y + font_size + padding);
                            stream.stroke();
                            // Draw text inside
                            stream.begin_text();
                            stream.set_font("F1", font_size);
                            stream.set_position(x + padding, y);
                            stream.show_text(text);
                            stream.end_text();
                        }
                        state.advance(line_height + 4.0);
                        continue;
                    }
                    if name == "colorbox" && args.len() >= 2 {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let color = crate::color::Color::parse(&args[0]);
                        let text = &args[1];
                        state.ensure_space(line_height + 2.0);
                        let font_size = state.current_font_size;
                        let text_width = text.len() as f32 * font_size * 0.55;
                        let x = state.left_margin();
                        let y = state.current_y;
                        let padding = 2.0;
                        let box_width = text_width + padding * 2.0;
                        let box_height = font_size + padding * 2.0;
                        {
                            let stream = state.current_stream();
                            // Fill background rectangle
                            if let Some(c) = color {
                                stream.set_color(c.r, c.g, c.b);
                            }
                            stream.fill_rect(x, y - padding, box_width, box_height);
                            // Reset to default black for text
                            stream.set_color(0.0, 0.0, 0.0);
                            // Draw text inside
                            stream.begin_text();
                            stream.set_font("F1", font_size);
                            stream.set_position(x + padding, y);
                            stream.show_text(text);
                            stream.end_text();
                        }
                        state.advance(line_height + 4.0);
                        continue;
                    }
                    if name == "fcolorbox" && args.len() >= 3 {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let frame_color = crate::color::Color::parse(&args[0]);
                        let back_color = crate::color::Color::parse(&args[1]);
                        let text = &args[2];
                        state.ensure_space(line_height + 2.0);
                        let font_size = state.current_font_size;
                        let text_width = text.len() as f32 * font_size * 0.55;
                        let x = state.left_margin();
                        let y = state.current_y;
                        let padding = 2.0;
                        let box_width = text_width + padding * 2.0;
                        let box_height = font_size + padding * 2.0;
                        {
                            let stream = state.current_stream();
                            // Fill background
                            if let Some(c) = back_color {
                                stream.set_color(c.r, c.g, c.b);
                            }
                            stream.fill_rect(x, y - padding, box_width, box_height);
                            // Stroke frame
                            if let Some(c) = frame_color {
                                stream.set_stroke_color(c.r, c.g, c.b);
                            }
                            stream.stroke_rect(x, y - padding, box_width, box_height);
                            // Reset to default black for text
                            stream.set_color(0.0, 0.0, 0.0);
                            stream.set_stroke_color(0.0, 0.0, 0.0);
                            // Draw text inside
                            stream.begin_text();
                            stream.set_font("F1", font_size);
                            stream.set_position(x + padding, y);
                            stream.show_text(text);
                            stream.end_text();
                        }
                        state.advance(line_height + 4.0);
                        continue;
                    }
                    if name == "rule" && args.len() >= 2 {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let width = crate::tex::Dimension::parse(&args[0]).map(|d| d.pt() as f32).unwrap_or(0.0);
                        let height = crate::tex::Dimension::parse(&args[1]).map(|d| d.pt() as f32).unwrap_or(0.0);
                        if width > 0.0 && height > 0.0 {
                            state.ensure_space(height.max(line_height));
                            let x = state.left_margin();
                            let y = state.current_y;
                            let stream = state.current_stream();
                            stream.move_to(x, y);
                            stream.line_to(x + width, y);
                            stream.stroke();
                            state.advance(height.max(line_height));
                        }
                        continue;
                    }
                    // Spacing commands
                    if matches!(name.as_str(), "medskip" | "bigskip" | "smallskip") {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        let advance = match name.as_str() {
                            "bigskip" => 24.0,
                            "medskip" => 12.0,
                            "smallskip" => 6.0,
                            _ => 0.0,
                        };
                        if advance > 0.0 {
                            state.ensure_space(advance);
                            state.advance(advance);
                        }
                        continue;
                    }
                    if name == "hrulefill" {
                        if !accumulated_text.is_empty() {
                            self.render_text_block(&mut state, &accumulated_text, line_height);
                            accumulated_text.clear();
                        }
                        state.ensure_space(1.0);
                        let x = state.left_margin();
                        let y = state.current_y;
                        let page_width = state.layout.content_width();
                        let stream = state.current_stream();
                        stream.move_to(x, y);
                        stream.line_to(x + page_width, y);
                        stream.stroke();
                        state.advance(1.0);
                        continue;
                    }
                    if matches!(name.as_str(), "hfill" | "vfill" | "dotfill" | "strut" | "mathstrut") {
                        // No-op in basic renderer
                        continue;
                    }
                    if matches!(name.as_str(), "qquad" | "quad" | "semicolon" | "comma" | "bang" | "colon" | "control_space") {
                        let space = match name.as_str() {
                            "qquad" => "  ",
                            "quad" => " ",
                            "semicolon" => " ",
                            "comma" => " ",
                            "bang" => "",
                            "colon" => " ",
                            "control_space" => " ",
                            _ => "",
                        };
                        accumulated_text.push_str(space);
                        continue;
                    }
                    // Font size commands
                    let size = match name.as_str() {
                        "tiny" => 6.0,
                        "scriptsize" => 7.0,
                        "footnotesize" => 8.0,
                        "small" => 9.0,
                        "normalsize" => base_font_size,
                        "large" => 12.0,
                        "Large" => 14.0,
                        "LARGE" => 17.0,
                        "huge" => 20.0,
                        "Huge" => 25.0,
                        _ => 0.0, // not a font size command
                    };
                    if size > 0.0 {
                        state.current_font_size = size;
                        line_height = state.line_height(size);
                    }
                }
                TexElement::Center(inner) => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                        state.centering = false;
                    }
                    let center_text = flatten_inline(inner);
                    if !center_text.trim().is_empty() {
                        state.centering = true;
                        self.render_text_block(&mut state, &center_text, line_height);
                        state.centering = false;
                    }
                }
                TexElement::Footnote { text } => {
                    footnote_counter += 1;
                    accumulated_text.push_str(&format!("[{}]", footnote_counter));
                    state.current_page_footnotes.push((footnote_counter, text.clone()));
                }
                TexElement::Caption { text } => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let caption_font_size = state.current_font_size * 0.9;
                    state.ensure_space(caption_font_size + 4.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", caption_font_size);
                    stream.set_position(x, y);
                    let caption_text = format!("Figure: {}", text);
                    stream.show_text(&caption_text);
                    stream.end_text();
                    state.advance(caption_font_size + 4.0);
                }
                TexElement::ListOfFigures => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(30.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 16.0);
                    stream.set_position(x, y);
                    stream.show_text("List of Figures");
                    stream.end_text();
                    state.advance(24.0);
                }
                TexElement::ListOfTables => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(30.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 16.0);
                    stream.set_position(x, y);
                    stream.show_text("List of Tables");
                    stream.end_text();
                    state.advance(24.0);
                }
                TexElement::Quote(inner) => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    let indent = 20.0;
                    let saved_left = state.layout.margin_left;
                    state.layout.margin_left += indent;
                    let body_text = flatten_inline(inner);
                    if !body_text.trim().is_empty() {
                        self.render_text_block(&mut state, &body_text, line_height);
                    }
                    state.layout.margin_left = saved_left;
                    state.advance(line_height);
                }
                TexElement::Abstract(inner) => {
                    if !accumulated_text.is_empty() {
                        self.render_text_block(&mut state, &accumulated_text, line_height);
                        accumulated_text.clear();
                    }
                    state.ensure_space(30.0);
                    let x = state.left_margin();
                    let y = state.current_y;
                    let stream = state.current_stream();
                    stream.begin_text();
                    stream.set_font("F1", 14.0);
                    stream.set_position(x, y);
                    stream.show_text("Abstract");
                    stream.end_text();
                    state.advance(20.0);
                    let saved_left = state.layout.margin_left;
                    let saved_right = state.layout.margin_right;
                    state.layout.margin_left += 20.0;
                    state.layout.margin_right += 20.0;
                    let body_text = flatten_inline(inner);
                    if !body_text.trim().is_empty() {
                        self.render_text_block(&mut state, &body_text, line_height);
                    }
                    state.layout.margin_left = saved_left;
                    state.layout.margin_right = saved_right;
                    state.advance(line_height);
                }
            }
        }

        // Flush remaining text
        if !accumulated_text.is_empty() {
            self.render_text_block(&mut state, &accumulated_text, line_height);
        }

        // Archive footnotes for the final page (empty vec if none)
        state.all_footnotes.push(std::mem::take(&mut state.current_page_footnotes));

        Ok(state)
    }

    fn render_text_block(
        &mut self,
        state: &mut crate::layout::LayoutState,
        text: &str,
        line_height: f32,
    ) {
        let formatted_text = self.format_inline_math(text);
        let Some(validated_text) = PdfTextRenderer::normalize_text(&formatted_text) else {
            return;
        };
        let content_width = state.content_width();
        let font_size = state.current_font_size;
        let lines = PdfTextRenderer::wrap_text_by_width(&validated_text, content_width, font_size);
        let left_margin = state.left_margin();
        let centering = state.centering;
        let raggedleft = state.raggedleft;

        for line in lines {
            if state.current_y - line_height < state.content_bottom() {
                state.new_page();
            }
            let y = state.current_y;
            let text_width = line.len() as f32 * font_size * 0.55;
            let x = if centering {
                left_margin + (content_width - text_width).max(0.0) / 2.0
            } else if raggedleft {
                (left_margin + content_width - text_width).max(left_margin)
            } else {
                left_margin
            };
            let stream = state.current_stream();
            stream.begin_text();
            stream.set_font("F1", font_size);
            stream.set_position(x, y);
            if let Some(engine) = self.typography.as_ref() {
                let segs = engine.process(&line);
                if segs.len() == 1 && segs[0].adjustment == 0 {
                    stream.show_text(&segs[0].text);
                } else {
                    let pairs: Vec<(String, i16)> = segs
                        .iter()
                        .map(|s| (s.text.clone(), s.adjustment))
                        .collect();
                    stream.show_text_with_kerning(&pairs);
                }
            } else {
                stream.show_text(&line);
            }
            stream.end_text();
            state.advance(line_height);
        }
        state.centering = false;
        state.raggedleft = false;
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

    fn render_page_footnotes(
        &self,
        stream: &mut crate::pdf::core::ContentStream,
        footnotes: &[(usize, String)],
        page_layout: &crate::page_layout::PageLayout,
    ) {
        let left = page_layout.margin_left;
        let bottom = page_layout.margin_bottom;
        let separator_y = bottom + 50.0;
        let font_size = 8.0;
        let line_height = font_size + 2.0;

        // Draw separator line
        stream.move_to(left, separator_y);
        stream.line_to(left + 100.0, separator_y);
        stream.stroke();

        // Render footnotes from bottom up
        for (idx, (num, text)) in footnotes.iter().enumerate() {
            let y = separator_y - 6.0 - (idx as f32 * line_height);
            stream.begin_text();
            stream.set_font("F1", font_size);
            stream.set_position(left, y);
            stream.show_text(&format!("{} {}", num, text));
            stream.end_text();
        }
    }

    /// Wrap a Rust string in PDF literal-string parentheses, escaping
    /// backslashes, parentheses, and non-ASCII bytes per PDF spec.
    fn pdf_string_literal(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('(');
        for b in s.bytes() {
            match b {
                b'\\' | b'(' | b')' => {
                    out.push('\\');
                    out.push(b as char);
                }
                0x20..=0x7E => out.push(b as char),
                _ => {
                    // Escape non-ASCII as octal \ddd
                    out.push_str(&format!("\\{:03o}", b));
                }
            }
        }
        out.push(')');
        out
    }
}

impl Default for PdfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::compress_content;

    #[test]
    fn compress_content_produces_valid_output() {
        let data = b"BT /F1 12 Tf 100 700 Td (Hello World) Tj ET";
        let compressed = compress_content(data);
        assert!(!compressed.is_empty());
        // Decompress to verify round-trip
        use std::io::Read;
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn compress_content_is_deterministic() {
        let data = b"repeated repeated repeated repeated text";
        let c1 = compress_content(data);
        let c2 = compress_content(data);
        assert_eq!(c1, c2);
    }

    #[test]
    fn compress_content_empty_roundtrips() {
        let data = b"";
        let compressed = compress_content(data);
        use std::io::Read;
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
