//! LaTeX AST → [pdfrs](https://crates.io/crates/pdfrs) element conversion and PDF rendering.

use std::path::PathBuf;

use pdfrs::elements::{Element, TableAlignment, TextSegment};
use pdfrs::optimization::{OptimizationProfile, OptimizedPdfGenerator};
use pdfrs::pdf_generator::{AccessibilityOptions, PageLayout as PdfrsLayout};

use crate::error::LatexError;
use crate::parser::TexElement;
use crate::table::{Align, Table};

/// Rasterize an SVG file to a PNG temp file and return the temp path.
/// Returns the original path unchanged if it is not an SVG.
fn rasterize_svg_if_needed(path: &str, base_dir: Option<&std::path::Path>) -> String {
    let resolved = if std::path::Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else if let Some(base) = base_dir {
        base.join(path)
    } else {
        PathBuf::from(path)
    };

    let is_svg = resolved
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));

    if !is_svg {
        return path.to_string();
    }

    let svg_data = match std::fs::read(&resolved) {
        Ok(d) => d,
        Err(_) => return path.to_string(),
    };

    let opt = usvg::Options::default();
    let tree = match usvg::Tree::from_data(&svg_data, &opt) {
        Ok(t) => t,
        Err(_) => return path.to_string(),
    };
    let size = tree.size();
    let width = size.width().ceil().max(1.0) as u32;
    let height = size.height().ceil().max(1.0) as u32;

    let mut pixmap = match resvg::tiny_skia::Pixmap::new(width, height) {
        Some(p) => p,
        None => return path.to_string(),
    };
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let rgba = pixmap.data();
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for chunk in rgba.chunks_exact(4) {
        let a = chunk[3] as f32 / 255.0;
        rgb.push((chunk[0] as f32 * a + 255.0 * (1.0 - a)) as u8);
        rgb.push((chunk[1] as f32 * a + 255.0 * (1.0 - a)) as u8);
        rgb.push((chunk[2] as f32 * a + 255.0 * (1.0 - a)) as u8);
    }

    let mut png_buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_buf, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&rgb).unwrap();
    }

    let tmp_path = std::env::temp_dir().join(format!(
        "rtex_svg_{}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    if std::fs::write(&tmp_path, &png_buf).is_err() {
        return path.to_string();
    }
    tmp_path.to_string_lossy().into_owned()
}

/// Maximum mapped element count allowed by the pdfrs renderer.
pub const PDFRS_MAX_ELEMENTS: usize = 5_000;

/// Identifies the pdfrs PDF engine as the only PDF backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfBackend {
    /// Vendored [pdfrs](https://crates.io/crates/pdfrs) layout engine.
    Pdfrs,
}

impl PdfBackend {
    pub fn producer_label(self) -> &'static str {
        "rtex/pdfrs"
    }
}

/// Document metadata extracted from `\title`, `\author`, `\date`.
#[derive(Debug, Default, Clone)]
pub struct DocumentPdfMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}

/// Extract title/author/date from the element stream.
pub fn extract_document_meta(elements: &[TexElement]) -> DocumentPdfMeta {
    let mut meta = DocumentPdfMeta::default();
    for elem in elements {
        if let TexElement::Command { name, args } = elem {
            match name.as_str() {
                "title" if !args.is_empty() => meta.title = Some(args[0].clone()),
                "author" if !args.is_empty() => meta.author = Some(args[0].clone()),
                "date" if !args.is_empty() => meta.date = Some(args[0].clone()),
                _ => {}
            }
        }
    }
    meta
}

/// Map a LaTeX element tree to pdfrs document elements.
pub fn tex_to_pdfrs_elements(
    elements: &[TexElement],
    image_base_dir: Option<&std::path::Path>,
) -> Vec<Element> {
    let mut writer = ElementWriter::new();
    for element in elements {
        push_tex_element(element, &mut writer, 0, image_base_dir);
    }
    writer.finish()
}

/// Map elements and prepend title-page metadata for pdfrs.
#[cfg_attr(not(test), allow(dead_code))]
pub fn tex_to_pdfrs_document(elements: &[TexElement]) -> Vec<Element> {
    tex_to_pdfrs_document_with_base(elements, None)
}

/// Like [`tex_to_pdfrs_document`] but resolves/rasterizes SVG images using `image_base_dir`.
pub fn tex_to_pdfrs_document_with_base(
    elements: &[TexElement],
    image_base_dir: Option<&std::path::Path>,
) -> Vec<Element> {
    let meta = extract_document_meta(elements);
    let mut out = tex_to_pdfrs_elements(elements, image_base_dir);
    prepend_document_meta(&mut out, &meta);
    out
}

fn prepend_document_meta(out: &mut Vec<Element>, meta: &DocumentPdfMeta) {
    let mut header = Vec::new();
    if let Some(title) = &meta.title {
        header.push(Element::Heading {
            level: 1,
            text: title.clone(),
        });
    }
    if let Some(author) = &meta.author {
        header.push(Element::Paragraph {
            text: author.clone(),
        });
    }
    if let Some(date) = &meta.date {
        let date_text = if date == "\\today" {
            chrono::Local::now().format("%B %d, %Y").to_string()
        } else {
            date.clone()
        };
        header.push(Element::Paragraph { text: date_text });
    }
    if !header.is_empty() {
        header.push(Element::EmptyLine);
        header.append(out);
        *out = header;
    }
}

/// Options passed to the pdfrs PDF backend.
#[derive(Debug, Clone)]
pub struct PdfRenderOptions {
    pub layout: PdfrsLayout,
    pub font: String,
    pub base_font_size: f32,
    pub image_base_dir: Option<PathBuf>,
    pub accessibility: Option<AccessibilityOptions>,
}

impl Default for PdfRenderOptions {
    fn default() -> Self {
        Self {
            layout: PdfrsLayout::portrait(),
            font: "Helvetica".to_string(),
            base_font_size: 11.0,
            image_base_dir: None,
            accessibility: None,
        }
    }
}

/// Render LaTeX elements to PDF bytes using pdfrs.
pub fn render_pdf_bytes(
    elements: &[TexElement],
    mut options: PdfRenderOptions,
) -> Result<Vec<u8>, LatexError> {
    let meta = extract_document_meta(elements);
    if let Some(title) = meta.title {
        options.accessibility = Some(options.accessibility.unwrap_or_default().with_title(title));
    }
    let pdfrs_elements =
        tex_to_pdfrs_document_with_base(elements, options.image_base_dir.as_deref());
    if pdfrs_elements.len() > PDFRS_MAX_ELEMENTS {
        return Err(LatexError::PdfError {
            message: format!(
                "pdfrs element count {} exceeds limit {PDFRS_MAX_ELEMENTS}",
                pdfrs_elements.len()
            ),
            context: None,
        });
    }
    render_pdfrs_element_bytes(&pdfrs_elements, options)
}

/// Render pre-built pdfrs elements to PDF bytes.
pub fn render_pdfrs_element_bytes(
    elements: &[Element],
    options: PdfRenderOptions,
) -> Result<Vec<u8>, LatexError> {
    let profile = OptimizationProfile::Archive;
    let profile = if let Some(accessibility) = options.accessibility {
        let mut settings = profile.settings();
        settings.tagged_pdf = accessibility.tagged_pdf;
        OptimizationProfile::Custom(settings)
    } else {
        profile
    };
    let mut generator = OptimizedPdfGenerator::new(profile)
        .with_layout(options.layout)
        .with_font(&options.font)
        .with_font_size(options.base_font_size);
    if let Some(base) = options.image_base_dir {
        generator = generator.with_image_base_dir(base);
    }
    generator
        .generate_bytes(elements)
        .map_err(|e| LatexError::PdfError {
            message: e.to_string(),
            context: None,
        })
}

struct ElementWriter {
    out: Vec<Element>,
    segments: Vec<TextSegment>,
}

impl ElementWriter {
    fn new() -> Self {
        Self {
            out: Vec::new(),
            segments: Vec::new(),
        }
    }

    fn finish(mut self) -> Vec<Element> {
        self.flush_rich();
        self.out
    }

    fn flush_rich(&mut self) {
        if self.segments.is_empty() {
            return;
        }
        let segments = merge_plain_segments(std::mem::take(&mut self.segments));
        if segments.len() == 1 {
            match segments[0].clone() {
                TextSegment::Plain(text) => self.out.push(Element::Paragraph { text }),
                segment => self.out.push(Element::RichParagraph {
                    segments: vec![segment],
                }),
            }
        } else {
            self.out.push(Element::RichParagraph { segments });
        }
    }

    fn push_plain(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        if let Some(TextSegment::Plain(last)) = self.segments.last_mut() {
            last.push(' ');
            last.push_str(trimmed);
        } else {
            self.segments.push(TextSegment::Plain(trimmed.to_string()));
        }
    }

    fn push_segment(&mut self, segment: TextSegment) {
        self.segments.push(segment);
    }

    fn empty_line(&mut self) {
        self.flush_rich();
        self.out.push(Element::EmptyLine);
    }

    fn push_block(&mut self, element: Element) {
        self.flush_rich();
        self.out.push(element);
    }
}

fn merge_plain_segments(segments: Vec<TextSegment>) -> Vec<TextSegment> {
    let mut merged = Vec::new();
    for segment in segments {
        match segment {
            TextSegment::Plain(text) => {
                if let Some(TextSegment::Plain(last)) = merged.last_mut() {
                    last.push(' ');
                    last.push_str(&text);
                } else {
                    merged.push(TextSegment::Plain(text));
                }
            }
            other => merged.push(other),
        }
    }
    merged
}

fn push_tex_element(
    element: &TexElement,
    writer: &mut ElementWriter,
    list_depth: u8,
    image_base_dir: Option<&std::path::Path>,
) {
    match element {
        TexElement::Text(text) => push_text_with_inline_math(writer, text),
        TexElement::Paragraph => writer.empty_line(),
        TexElement::Section { level, title } => {
            writer.push_block(Element::Heading {
                level: section_heading_level(*level),
                text: title.clone(),
            });
        }
        TexElement::MathInline(expr) => {
            push_inline_math_segment(writer, expr);
        }
        TexElement::MathDisplay(expr) => {
            writer.push_block(Element::MathBlock {
                expression: math_block_expression(expr),
            });
        }
        TexElement::MathLines { lines, .. } => {
            writer.push_block(Element::MathBlock {
                expression: math_block_expression(&lines.join(" \\\\ ")),
            });
        }
        TexElement::ItemList {
            ordered,
            labels,
            items,
        } => {
            writer.flush_rich();
            for (idx, item) in items.iter().enumerate() {
                let inline: Vec<TexElement> = item
                    .iter()
                    .filter(|el| !matches!(el, TexElement::ItemList { .. }))
                    .cloned()
                    .collect();
                let nested: Vec<&TexElement> = item
                    .iter()
                    .filter(|el| matches!(el, TexElement::ItemList { .. }))
                    .collect();

                let (text, display_math) = split_item_content(&inline);
                if !inline.is_empty() {
                    if *ordered {
                        writer.out.push(Element::OrderedListItem {
                            number: (idx + 1) as u32,
                            text: text.clone(),
                            depth: list_depth,
                        });
                    } else {
                        let label = labels.get(idx).and_then(|l| l.as_ref());
                        let display = if let Some(lbl) = label {
                            if text.is_empty() {
                                lbl.clone()
                            } else {
                                format!("{lbl} {text}")
                            }
                        } else {
                            text
                        };
                        writer.out.push(Element::UnorderedListItem {
                            text: display,
                            depth: list_depth,
                        });
                    }
                }

                for expr in display_math {
                    writer.out.push(Element::MathBlock {
                        expression: math_block_expression(&expr),
                    });
                }

                for nested_list in nested {
                    push_tex_element(nested_list, writer, list_depth + 1, image_base_dir);
                }
            }
        }
        TexElement::DescriptionList { items } => {
            writer.flush_rich();
            for item in items {
                writer.out.push(Element::DefinitionItem {
                    term: item.term.clone(),
                    definition: flatten_item_text(&item.body),
                });
            }
        }
        TexElement::Theorem { kind, title, body } => {
            let mut heading = kind[..1].to_uppercase() + &kind[1..];
            if let Some(t) = title {
                heading.push_str(&format!(" ({t})"));
            }
            writer.push_block(Element::Heading {
                level: 4,
                text: heading,
            });
            push_children(body, writer, list_depth, image_base_dir);
        }
        TexElement::CodeBlock(code) => {
            writer.push_block(Element::CodeBlock {
                language: "latex".to_string(),
                code: code.clone(),
            });
        }
        TexElement::Image { path, .. } => {
            let resolved_path = rasterize_svg_if_needed(path, image_base_dir);
            writer.push_block(Element::Image {
                alt: String::new(),
                path: resolved_path,
            });
        }
        TexElement::Table(table) => {
            writer.flush_rich();
            push_table(table, &mut writer.out);
        }
        TexElement::ColoredText { text, .. } => {
            writer.push_plain(text);
        }
        TexElement::Citation { keys } => {
            writer.push_plain(&format!("[{}]", keys.join(", ")));
        }
        TexElement::Bibliography { entries } => {
            writer.flush_rich();
            for entry in entries {
                writer.out.push(Element::CitationDef {
                    key: entry.key.clone(),
                    text: entry.text.clone(),
                });
            }
            writer.out.push(Element::Bibliography);
        }
        TexElement::Label { .. } => {}
        TexElement::Ref { key } | TexElement::PageRef { key } => {
            writer.push_plain(&format!("?? ({key})"));
        }
        TexElement::Center(inner) | TexElement::Quote(inner) | TexElement::Abstract(inner) => {
            if matches!(element, TexElement::Abstract(_)) {
                writer.push_block(Element::Heading {
                    level: 3,
                    text: "Abstract".to_string(),
                });
            }
            for child in inner {
                if matches!(child, TexElement::Text(t) if !t.trim().is_empty()) {
                    if let TexElement::Text(t) = child {
                        writer.push_block(Element::BlockQuote {
                            text: t.trim().to_string(),
                            depth: 0,
                        });
                    }
                } else {
                    push_tex_element(child, writer, list_depth, image_base_dir);
                }
            }
        }
        TexElement::Footnote { text } => {
            writer.push_block(Element::Footnote {
                label: String::new(),
                text: text.clone(),
            });
        }
        TexElement::Caption { text } => {
            writer.push_plain(text);
        }
        TexElement::TableOfContents => writer.push_block(Element::Toc),
        TexElement::ListOfFigures | TexElement::ListOfTables => {
            writer.push_block(Element::Heading {
                level: 2,
                text: if matches!(element, TexElement::ListOfFigures) {
                    "List of Figures".to_string()
                } else {
                    "List of Tables".to_string()
                },
            });
        }
        TexElement::Command { name, args } => match name.as_str() {
            "maketitle" => push_maketitle(writer),
            "title" | "author" | "date" | "newpage" | "clearpage" | "pagebreak" => {}
            "textbf" if !args.is_empty() => {
                writer.push_segment(TextSegment::Bold(args[0].clone()));
            }
            "textit" | "emph" if !args.is_empty() => {
                writer.push_segment(TextSegment::Italic(args[0].clone()));
            }
            "texttt" if !args.is_empty() => {
                writer.push_segment(TextSegment::Code(args[0].clone()));
            }
            "textcolor" if args.len() >= 2 => {
                writer.push_plain(&args[1]);
            }
            "href" if args.len() >= 2 => {
                writer.push_segment(TextSegment::Link {
                    text: args[1].clone(),
                    url: args[0].clone(),
                });
            }
            "url" if !args.is_empty() => {
                writer.push_segment(TextSegment::Link {
                    text: args[0].clone(),
                    url: args[0].clone(),
                });
            }
            "today" => {
                writer.push_plain(&chrono::Local::now().format("%B %d, %Y").to_string());
            }
            _ => {
                if let Some(text) = args.first() {
                    writer.push_plain(text);
                }
            }
        },
    }
}

fn push_children(
    children: &[TexElement],
    writer: &mut ElementWriter,
    depth: u8,
    image_base_dir: Option<&std::path::Path>,
) {
    for child in children {
        push_tex_element(child, writer, depth, image_base_dir);
    }
}

/// Split plain text that embeds `$...$` into rich inline segments.
fn push_text_with_inline_math(writer: &mut ElementWriter, text: &str) {
    let mut plain = String::new();
    let mut in_math = false;
    let mut math_buf = String::new();

    for ch in text.chars() {
        if ch == '$' {
            if in_math {
                flush_plain_buffer(writer, &mut plain);
                push_inline_math_segment(writer, &math_buf);
                math_buf.clear();
                in_math = false;
            } else {
                flush_plain_buffer(writer, &mut plain);
                in_math = true;
            }
        } else if in_math {
            math_buf.push(ch);
        } else {
            plain.push(ch);
        }
    }

    if in_math {
        plain.push('$');
        plain.push_str(&math_buf);
    }
    flush_plain_buffer(writer, &mut plain);
}

fn flush_plain_buffer(writer: &mut ElementWriter, plain: &mut String) {
    let trimmed = plain.trim();
    if !trimmed.is_empty() {
        writer.push_plain(trimmed);
    }
    plain.clear();
}

/// Push inline math using display layout for fractions, otherwise formatted Unicode.
fn push_inline_math_segment(writer: &mut ElementWriter, expr: &str) {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return;
    }
    if inline_uses_display_math(trimmed) {
        writer.push_block(Element::MathBlock {
            expression: math_block_expression(trimmed),
        });
        return;
    }
    writer.push_segment(TextSegment::Plain(format_math_for_text(trimmed)));
}

fn inline_uses_display_math(expr: &str) -> bool {
    expr.contains("\\frac")
        || expr.contains("\\dfrac")
        || expr.contains("\\tfrac")
        || expr.contains("\\sqrt")
}

fn math_block_expression(expr: &str) -> String {
    expr.to_string()
}

fn push_maketitle(_writer: &mut ElementWriter) {
    // Title block is usually emitted via preceding \title/\author/\date commands;
    // maketitle itself is a no-op marker in the pdfrs stream.
    _writer.empty_line();
}

fn push_table(table: &Table, out: &mut Vec<Element>) {
    let alignments: Vec<TableAlignment> = table
        .columns
        .iter()
        .map(|a| match a {
            Align::Left => TableAlignment::Left,
            Align::Center => TableAlignment::Center,
            Align::Right => TableAlignment::Right,
        })
        .collect();

    for row in &table.rows {
        if row.is_separator {
            out.push(Element::HorizontalRule);
            continue;
        }
        out.push(Element::TableRow {
            cells: row.cells.clone(),
            is_separator: false,
            alignments: alignments.clone(),
        });
    }
}

fn flatten_item_text(items: &[TexElement]) -> String {
    split_item_content(items).0
}

/// Split list-item content into plain text and display-math expressions.
///
/// `\frac` / `\sqrt` go to display layout (vinculum / stacked fractions);
/// simple symbols stay as Unicode in the list item text.
fn split_item_content(items: &[TexElement]) -> (String, Vec<String>) {
    let mut parts = Vec::new();
    let mut display = Vec::new();

    for item in items {
        match item {
            TexElement::Text(t) => {
                collect_text_with_math(t, &mut parts, &mut display);
            }
            TexElement::MathInline(m) => {
                if inline_uses_display_math(m) {
                    display.push(m.clone());
                } else {
                    parts.push(format_math_for_text(m));
                }
            }
            TexElement::Command { name, args }
                if matches!(name.as_str(), "textbf" | "textit" | "emph" | "texttt") =>
            {
                if let Some(a) = args.first() {
                    parts.push(a.clone());
                }
            }
            TexElement::ColoredText { text, .. } => parts.push(text.clone()),
            _ => {}
        }
    }
    (parts.join(" "), display)
}

fn collect_text_with_math(text: &str, parts: &mut Vec<String>, display: &mut Vec<String>) {
    let mut plain = String::new();
    let mut in_math = false;
    let mut math_buf = String::new();

    for ch in text.chars() {
        if ch == '$' {
            if in_math {
                let trimmed = math_buf.trim().to_string();
                if !trimmed.is_empty() {
                    if inline_uses_display_math(&trimmed) {
                        flush_part(parts, &mut plain);
                        display.push(trimmed);
                    } else {
                        plain.push_str(&format_math_for_text(&trimmed));
                    }
                }
                math_buf.clear();
                in_math = false;
            } else {
                in_math = true;
            }
        } else if in_math {
            math_buf.push(ch);
        } else {
            plain.push(ch);
        }
    }
    if in_math {
        plain.push('$');
        plain.push_str(&math_buf);
    }
    flush_part(parts, &mut plain);
}

fn flush_part(parts: &mut Vec<String>, plain: &mut String) {
    let trimmed = plain.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    plain.clear();
}

fn section_heading_level(level: usize) -> u8 {
    match level {
        0 => 1,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        _ => 6,
    }
}

/// Format inline math for plain-text contexts (list items, etc.).
fn format_math_for_text(expr: &str) -> String {
    crate::math_formatter::MathFormatter::format(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_section_to_heading() {
        let elements = vec![TexElement::Section {
            level: 1,
            title: "Intro".to_string(),
        }];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert!(matches!(
            mapped.first(),
            Some(Element::Heading { level: 1, text }) if text == "Intro"
        ));
    }

    #[test]
    fn coalesces_consecutive_text_into_one_paragraph() {
        let elements = vec![
            TexElement::Text("Hello".to_string()),
            TexElement::Text("world".to_string()),
            TexElement::Paragraph,
            TexElement::Text("Next".to_string()),
            TexElement::Text("line".to_string()),
        ];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert_eq!(mapped.len(), 3);
        assert!(matches!(
            mapped[0],
            Element::Paragraph { ref text } if text == "Hello world"
        ));
        assert!(matches!(mapped[1], Element::EmptyLine));
        assert!(matches!(
            mapped[2],
            Element::Paragraph { ref text } if text == "Next line"
        ));
    }

    #[test]
    fn renders_nested_lists_with_pdfrs() {
        let content = std::fs::read_to_string("examples/lists.tex").expect("lists.tex");
        let mut parser = crate::TexParser::new(content);
        let elements = parser.parse();
        let pdf = render_pdf_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(
            pdf.len() < 100_000,
            "lists.tex should render via pdfrs (<100KB), got {} bytes",
            pdf.len()
        );
    }

    #[test]
    fn prepends_document_metadata() {
        let elements = vec![
            TexElement::Command {
                name: "title".to_string(),
                args: vec!["My Doc".to_string()],
            },
            TexElement::Command {
                name: "author".to_string(),
                args: vec!["Alice".to_string()],
            },
            TexElement::Text("Body".to_string()),
        ];
        let mapped = tex_to_pdfrs_document(&elements);
        assert!(matches!(
            mapped.first(),
            Some(Element::Heading { level: 1, text }) if text == "My Doc"
        ));
        assert!(mapped.iter().any(|e| matches!(
            e,
            Element::Paragraph { text } if text == "Alice"
        )));
    }

    #[test]
    fn renders_minimal_document_to_pdf() {
        let elements = vec![
            TexElement::Command {
                name: "title".to_string(),
                args: vec!["Test".to_string()],
            },
            TexElement::Text("Hello world".to_string()),
        ];
        let pdf = render_pdf_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn math_example_renders_via_pdfrs() {
        let content = std::fs::read_to_string("examples/math.tex").expect("math.tex");
        let mut parser = crate::TexParser::new(content);
        let elements = parser.parse();
        let pdf = render_pdf_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(
            pdf.len() < 500_000,
            "math.tex should use pdfrs (<500KB), got {} bytes",
            pdf.len()
        );
    }

    #[test]
    fn simple_test_embedded_math_renders_without_dollar_delimiters() {
        let content = std::fs::read_to_string("examples/simple_test.tex").expect("simple_test.tex");
        let mut parser = crate::TexParser::new(content);
        let elements = parser.parse();
        let mapped = tex_to_pdfrs_document(&elements);
        let has_rendered_math = mapped.iter().any(|e| match e {
            Element::Paragraph { text } => text.contains('α') || text.contains('∑'),
            Element::RichParagraph { segments } => segments.iter().any(|s| {
                matches!(s, TextSegment::Plain(t) if t.contains('α') || t.contains('∑'))
                    || matches!(s, TextSegment::MathInline(_))
            }),
            _ => false,
        });
        assert!(
            has_rendered_math,
            "expected rendered inline math, got: {mapped:?}"
        );
        assert!(
            !mapped.iter().any(|e| match e {
                Element::Paragraph { text } => text.contains('$'),
                Element::RichParagraph { segments } => segments
                    .iter()
                    .any(|s| matches!(s, TextSegment::Plain(t) if t.contains('$'))),
                _ => false,
            }),
            "literal $ delimiters should not appear in output"
        );
    }

    #[test]
    fn advanced_math_example_renders_greek_symbols() {
        let content =
            std::fs::read_to_string("examples/advanced_math.tex").expect("advanced_math.tex");
        let mut parser = crate::TexParser::new(content);
        let elements = parser.parse();
        let pdf = render_pdf_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(
            pdf.len() < 200_000,
            "advanced_math PDF should be reasonable, got {} bytes",
            pdf.len()
        );
        let mapped = tex_to_pdfrs_document(&elements);
        assert!(
            mapped.iter().any(|e| match e {
                Element::Paragraph { text } => text.contains('α') || text.contains('β'),
                Element::RichParagraph { segments } => segments.iter().any(|s| {
                    matches!(s, TextSegment::Plain(t) if t.contains('α') || t.contains('β'))
                        || matches!(s, TextSegment::MathInline(m) if m.contains("\\alpha"))
                }),
                _ => false,
            }),
            "expected rendered Greek letters in mapped output"
        );
    }

    #[test]
    fn features_example_renders_with_pdfrs() {
        let content = std::fs::read_to_string("examples/features.tex").expect("features.tex");
        let mut parser = crate::TexParser::new(content);
        let elements = parser.parse();
        let mapped = tex_to_pdfrs_document(&elements);
        assert!(
            mapped
                .iter()
                .any(|e| matches!(e, Element::DefinitionItem { .. })),
            "expected description list items"
        );
        assert!(
            mapped
                .iter()
                .any(|e| matches!(e, Element::MathBlock { .. })),
            "expected display math blocks"
        );
        assert!(
            mapped
                .iter()
                .any(|e| matches!(e, Element::RichParagraph { .. })),
            "expected inline rich paragraphs for texttt/href"
        );
        let pdf = render_pdf_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(
            pdf.len() < 200_000,
            "features.tex PDF should be reasonable size, got {} bytes",
            pdf.len()
        );
    }

    #[test]
    fn inline_formatting_uses_rich_paragraph() {
        let elements = vec![
            TexElement::Text("Use ".to_string()),
            TexElement::Command {
                name: "texttt".to_string(),
                args: vec!["align".to_string()],
            },
            TexElement::Text(" for equations.".to_string()),
        ];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert!(matches!(
            mapped.first(),
            Some(Element::RichParagraph { segments })
            if segments.len() >= 3
        ));
    }

    #[test]
    fn unicode_math_pdf_stays_small_with_subset_fonts() {
        use pdfrs::elements::Element;
        let formatted = crate::math_formatter::MathFormatter::format(
            "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}",
        );
        let elements = vec![Element::Paragraph { text: formatted }];
        let pdf = render_pdfrs_element_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(
            pdf.len() < 500_000,
            "subset Unicode PDF should be <500KB, got {} bytes",
            pdf.len()
        );
    }

    #[test]
    fn display_math_paragraph_stays_small() {
        let elements = vec![TexElement::MathDisplay(
            "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}".to_string(),
        )];
        let pdf = render_pdf_bytes(&elements, PdfRenderOptions::default()).unwrap();
        assert!(pdf.len() < 500_000, "got {} bytes", pdf.len());
    }

    #[test]
    fn list_item_sqrt_emits_math_block() {
        let elements = vec![TexElement::ItemList {
            ordered: false,
            labels: vec![],
            items: vec![vec![TexElement::Text("Root $\\sqrt{x^2+y^2}$".to_string())]],
        }];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert!(
            mapped.iter().any(|e| matches!(
                e,
                Element::MathBlock { expression } if expression.contains("\\sqrt")
            )),
            "expected MathBlock for sqrt in list item, got: {mapped:?}"
        );
    }

    #[test]
    fn renders_math_inline() {
        let elements = vec![
            TexElement::Text("Angle ".to_string()),
            TexElement::MathInline("\\alpha".to_string()),
        ];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert!(matches!(
            mapped.first(),
            Some(Element::Paragraph { text }) if text == "Angle α"
        ));
    }

    #[test]
    fn sqrt_in_expression_uses_math_block_with_vinculum() {
        let elements = vec![TexElement::Text(
            "Square root: $\\sqrt{x^2 + y^2}$".to_string(),
        )];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert!(
            mapped.iter().any(|e| matches!(
                e,
                Element::MathBlock { expression } if expression.contains("\\sqrt")
            )),
            "expected MathBlock with raw sqrt for vinculum layout, got: {mapped:?}"
        );
    }

    #[test]
    fn frac_in_inline_math_uses_math_block() {
        let elements = vec![TexElement::Text("Formula: $x = \\frac{1}{2}$".to_string())];
        let mapped = tex_to_pdfrs_elements(&elements, None);
        assert!(
            mapped
                .iter()
                .any(|e| matches!(e, Element::MathBlock { .. })),
            "expected display math block for inline fraction, got: {mapped:?}"
        );
    }
}
