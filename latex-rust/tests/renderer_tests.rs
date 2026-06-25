//! Unit tests for the renderer module

use latex_rust::ast::*;
use latex_rust::renderer::{HtmlRenderer, MathRenderer};
use std::collections::HashMap;

#[cfg(test)]
mod renderer_unit_tests {
    use super::*;

    // Helper function to create a simple document
    fn create_simple_document() -> Document {
        Document {
            metadata: DocumentMetadata {
                document_class: Some("article".to_string()),
                title: Some("Test Document".to_string()),
                author: Some("Test Author".to_string()),
                date: Some("2024".to_string()),
                packages: vec!["amsmath".to_string()],
            },
            preamble: vec![],
            body: vec![Node::Text("Hello world".to_string())],
        }
    }

    // Helper function to create a document with packages in preamble
    fn create_document_with_packages() -> Document {
        Document {
            metadata: DocumentMetadata {
                document_class: Some("article".to_string()),
                title: Some("Test Document".to_string()),
                author: Some("Test Author".to_string()),
                date: None,
                packages: vec![],
            },
            preamble: vec![
                Node::Package {
                    name: "amsmath".to_string(),
                    options: vec![],
                },
                Node::Package {
                    name: "geometry".to_string(),
                    options: vec!["margin=1in".to_string()],
                },
            ],
            body: vec![Node::Text("Content".to_string())],
        }
    }

    #[test]
    fn test_html_renderer_creation() {
        let renderer = HtmlRenderer::new();
        // Should create without panicking
        assert!(true);
    }

    #[test]
    fn test_html_renderer_default() {
        let renderer = HtmlRenderer::default();
        // Should create without panicking
        assert!(true);
    }

    #[test]
    fn test_render_simple_document() {
        let renderer = HtmlRenderer::new();
        let document = create_simple_document();
        let html = renderer.render(&document);
        
        // Check basic HTML structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("</html>"));
        
        // Check title
        assert!(html.contains("<title>Test Document</title>"));
        
        // Check document content
        assert!(html.contains("Hello world"));
    }

    #[test]
    fn test_render_document_with_metadata() {
        let renderer = HtmlRenderer::new();
        let document = create_simple_document();
        let html = renderer.render(&document);
        
        // Check metadata rendering
        assert!(html.contains("<h1 class=\"document-title\">Test Document</h1>"));
        assert!(html.contains("<p class=\"document-author\">Test Author</p>"));
        assert!(html.contains("<p class=\"document-date\">2024</p>"));
    }

    #[test]
    fn test_render_document_with_packages() {
        let renderer = HtmlRenderer::new();
        let document = create_document_with_packages();
        let html = renderer.render(&document);
        
        // Check package comments
        assert!(html.contains("<!-- Package: amsmath -->"));
        assert!(html.contains("<!-- Package: geometry [margin=1in] -->"));
    }

    #[test]
    fn test_render_text_node() {
        let renderer = HtmlRenderer::new();
        let node = Node::Text("Simple text".to_string());
        let html = renderer.render_node(&node);
        
        assert_eq!(html, "Simple text");
    }

    #[test]
    fn test_render_text_with_html_escaping() {
        let renderer = HtmlRenderer::new();
        let node = Node::Text("<script>alert('xss')</script>".to_string());
        let html = renderer.render_node(&node);
        
        // Should escape HTML characters
        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_render_text_formatting_commands() {
        let renderer = HtmlRenderer::new();
        
        // Test bold
        let bold_node = Node::Command {
            name: "textbf".to_string(),
            args: vec![Argument::Required(vec![Node::Text("bold text".to_string())])],
            content: None,
        };
        let html = renderer.render_node(&bold_node);
        assert!(html.contains("<strong>bold text</strong>"));
        
        // Test italic
        let italic_node = Node::Command {
            name: "textit".to_string(),
            args: vec![Argument::Required(vec![Node::Text("italic text".to_string())])],
            content: None,
        };
        let html = renderer.render_node(&italic_node);
        assert!(html.contains("<em>italic text</em>"));
        
        // Test typewriter
        let tt_node = Node::Command {
            name: "texttt".to_string(),
            args: vec![Argument::Required(vec![Node::Text("code text".to_string())])],
            content: None,
        };
        let html = renderer.render_node(&tt_node);
        assert!(html.contains("<code>code text</code>"));
    }

    #[test]
    fn test_render_font_size_commands() {
        let renderer = HtmlRenderer::new();
        
        let large_node = Node::Command {
            name: "large".to_string(),
            args: vec![Argument::Required(vec![Node::Text("large text".to_string())])],
            content: None,
        };
        let html = renderer.render_node(&large_node);
        assert!(html.contains("font-large"));
    }

    #[test]
    fn test_render_math_inline() {
        let renderer = HtmlRenderer::new();
        let math_node = Node::Math {
            display: false,
            content: "x = y + z".to_string(),
        };
        let html = renderer.render_node(&math_node);
        
        assert!(html.contains("$x = y + z$"));
    }

    #[test]
    fn test_render_math_display() {
        let renderer = HtmlRenderer::new();
        let math_node = Node::Math {
            display: true,
            content: "E = mc^2".to_string(),
        };
        let html = renderer.render_node(&math_node);
        
        assert!(html.contains("$$E = mc^2$$"));
    }

    #[test]
    fn test_render_list_itemize() {
        let renderer = HtmlRenderer::new();
        let list_node = Node::List {
            list_type: ListType::Itemize,
            items: vec![
                ListItem {
                    label: None,
                    content: vec![Node::Text("First item".to_string())],
                },
                ListItem {
                    label: None,
                    content: vec![Node::Text("Second item".to_string())],
                },
            ],
        };
        let html = renderer.render_node(&list_node);
        
        assert!(html.contains("<ul>"));
        assert!(html.contains("</ul>"));
        assert!(html.contains("<li>First item</li>"));
        assert!(html.contains("<li>Second item</li>"));
    }

    #[test]
    fn test_render_list_enumerate() {
        let renderer = HtmlRenderer::new();
        let list_node = Node::List {
            list_type: ListType::Enumerate,
            items: vec![
                ListItem {
                    label: None,
                    content: vec![Node::Text("First item".to_string())],
                },
            ],
        };
        let html = renderer.render_node(&list_node);
        
        assert!(html.contains("<ol>"));
        assert!(html.contains("</ol>"));
        assert!(html.contains("<li>First item</li>"));
    }

    #[test]
    fn test_render_table() {
        let renderer = HtmlRenderer::new();
        let table_node = Node::Table {
            table_type: TableType::Tabular,
            alignment: vec![ColumnAlignment::Left, ColumnAlignment::Center],
            rows: vec![
                TableRow {
                    cells: vec![
                        TableCell::Normal(vec![Node::Text("Cell 1".to_string())]),
                        TableCell::Normal(vec![Node::Text("Cell 2".to_string())]),
                    ],
                    rules_before: vec![],
                    rules_after: vec![],
                },
            ],
            caption: None,
            label: None,
            position: None,
        };
        let html = renderer.render_node(&table_node);
        
        assert!(html.contains("<table class=\"tabular\">"));
        assert!(html.contains("</table>"));
        assert!(html.contains("<tr>"));
        assert!(html.contains("<td style=\"text-align: left;\">Cell 1</td>"));
        assert!(html.contains("<td style=\"text-align: center;\">Cell 2</td>"));
    }

    #[test]
    fn test_render_section() {
        let renderer = HtmlRenderer::new();
        let section_node = Node::Section {
            level: SectionLevel::Section,
            title: "Test Section".to_string(),
            content: vec![Node::Text("Section content".to_string())],
        };
        let html = renderer.render_node(&section_node);
        assert!(html.contains("<h3 class=\"section-2\">Test Section</h3>"));
        assert!(html.contains("Section content"));
    }

    #[test]
    fn test_render_figure() {
        let renderer = HtmlRenderer::new();
        let figure_node = Node::Figure {
            path: "image.png".to_string(),
            caption: Some("Test caption".to_string()),
            label: Some("fig:test".to_string()),
            width: Some("50%".to_string()),
            height: None,
        };
        let html = renderer.render_node(&figure_node);
        
        assert!(html.contains("<figure"));
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"image.png\""));
        assert!(html.contains("Test caption"));
    }

    #[test]
    fn test_render_reference() {
        let renderer = HtmlRenderer::new();
        let ref_node = Node::Reference {
            ref_type: ReferenceType::Ref,
            label: "fig:test".to_string(),
        };
        let html = renderer.render_node(&ref_node);
        
        assert!(html.contains("<a href=\"#fig:test\""));
    }

    #[test]
    fn test_render_footnote() {
        let renderer = HtmlRenderer::new();
        let footnote_node = Node::Footnote {
            content: vec![Node::Text("Footnote text".to_string())],
        };
        let html = renderer.render_node(&footnote_node);
        
        assert!(html.contains("<sup class=\"footnote\""));
        assert!(html.contains("Footnote text"));
    }

    #[test]
    fn test_render_environment_itemize() {
        let renderer = HtmlRenderer::new();
        let env_node = Node::Environment {
            name: "itemize".to_string(),
            args: vec![],
            content: vec![Node::Text("Item content".to_string())],
        };
        let html = renderer.render_node(&env_node);
        
        assert!(html.contains("<ul>"));
        assert!(html.contains("Item content"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_render_environment_quote() {
        let renderer = HtmlRenderer::new();
        let env_node = Node::Environment {
            name: "quote".to_string(),
            args: vec![],
            content: vec![Node::Text("Quoted text".to_string())],
        };
        let html = renderer.render_node(&env_node);
        
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("Quoted text"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn test_render_environment_verbatim() {
        let renderer = HtmlRenderer::new();
        let env_node = Node::Environment {
            name: "verbatim".to_string(),
            args: vec![],
            content: vec![Node::Text("<code>example</code>".to_string())],
        };
        let html = renderer.render_node(&env_node);
        
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("example"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_render_group() {
        let renderer = HtmlRenderer::new();
        let group_node = Node::Group(vec![
            Node::Text("First ".to_string()),
            Node::Text("Second".to_string()),
        ]);
        let html = renderer.render_node(&group_node);
        
        assert_eq!(html, "First Second");
    }

    #[test]
    fn test_render_line_break() {
        let renderer = HtmlRenderer::new();
        let br_node = Node::LineBreak;
        let html = renderer.render_node(&br_node);
        
        assert_eq!(html, "<br>\n");
    }

    #[test]
    fn test_render_paragraph() {
        let renderer = HtmlRenderer::new();
        let p_node = Node::Paragraph;
        let html = renderer.render_node(&p_node);
        
        assert_eq!(html, "<p></p>\n");
    }

    #[test]
    fn test_render_whitespace() {
        let renderer = HtmlRenderer::new();
        let ws_node = Node::Whitespace("   ".to_string());
        let html = renderer.render_node(&ws_node);
        
        assert_eq!(html, "   ");
    }

    #[test]
    fn test_render_label() {
        let renderer = HtmlRenderer::new();
        let label_node = Node::Label("test-label".to_string());
        let html = renderer.render_node(&label_node);
        
        assert!(html.contains("<span id=\"test-label\""));
        assert!(html.contains("class=\"label\""));
    }

    #[test]
    fn test_render_unknown_command() {
        let renderer = HtmlRenderer::new();
        let unknown_node = Node::Command {
            name: "unknowncommand".to_string(),
            args: vec![],
            content: None,
        };
        let html = renderer.render_node(&unknown_node);
        
        assert!(html.contains("<!-- Unknown command: unknowncommand -->"));
    }

    #[test]
    fn test_render_spacing() {
        let renderer = HtmlRenderer::new();
        let spacing_node = Node::Spacing {
            space_type: SpacingType::Vspace,
            amount: Some("10pt".to_string()),
        };
        let html = renderer.render_node(&spacing_node);
        
        assert!(html.contains("<div style=\"height: 10pt;\"></div>"));
    }

    #[test]
    fn test_render_citation() {
        let renderer = HtmlRenderer::new();
        let citation_node = Node::Citation {
            cite_type: CitationType::Cite,
            keys: vec!["key1".to_string()],
            prenote: None,
            postnote: None,
        };
        let html = renderer.render_node(&citation_node);
        
        assert!(html.contains("<a href=\"#ref-key1\" class=\"citation\">[key1]</a>"));
    }

    #[test]
    fn test_render_complex_nested_structure() {
        let renderer = HtmlRenderer::new();
        let complex_node = Node::Section {
            level: SectionLevel::Section,
            title: "Complex Section".to_string(),
            content: vec![
                Node::Text("Introduction text. ".to_string()),
                Node::Command {
                    name: "textbf".to_string(),
                    args: vec![Argument::Required(vec![Node::Text("Bold text".to_string())])],
                    content: None,
                },
                Node::Text(" and ".to_string()),
                Node::Math {
                    display: false,
                    content: "x^2".to_string(),
                },
                Node::List {
                    list_type: ListType::Itemize,
                    items: vec![
                        ListItem {
                            label: None,
                            content: vec![Node::Text("Item 1".to_string())],
                        },
                    ],
                },
            ],
        };
        let html = renderer.render_node(&complex_node);
        
        assert!(html.contains("<h3 class=\"section-2\">"));
        assert!(html.contains("Complex Section"));
        assert!(html.contains("Introduction text"));
        assert!(html.contains("<strong>Bold text</strong>"));
        assert!(html.contains("$x^2$"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("Item 1"));
    }

    // Math Renderer Tests
    #[test]
    fn test_math_renderer_creation() {
        let math_renderer = MathRenderer::new();
        // Should create without panicking
        assert!(true);
    }

    #[test]
    fn test_math_renderer_default() {
        let math_renderer = MathRenderer::default();
        // Should create without panicking
        assert!(true);
    }

    #[test]
    fn test_math_renderer_inline() {
        let math_renderer = MathRenderer::new();
        let result = math_renderer.render("x + y", false);
        
        assert_eq!(result, "<span class=\"math-inline\">$x + y$</span>");
    }

    #[test]
    fn test_math_renderer_display() {
        let math_renderer = MathRenderer::new();
        let result = math_renderer.render("\\sum_{i=1}^n x_i", true);
        
        assert_eq!(result, "<div class=\"math-display\">$$\\sum_{i=1}^n x_i$$</div>\n");
    }

    #[test]
    fn test_math_renderer_empty_content() {
        let math_renderer = MathRenderer::new();
        let result = math_renderer.render("", false);
        
        assert_eq!(result, "<span class=\"math-inline\">$$</span>");
    }

    #[test]
    fn test_render_document_without_metadata() {
        let renderer = HtmlRenderer::new();
        let document = Document {
            metadata: DocumentMetadata {
                document_class: None,
                title: None,
                author: None,
                date: None,
                packages: vec![],
            },
            preamble: vec![],
            body: vec![Node::Text("Just content".to_string())],
        };
        let html = renderer.render(&document);
        
        // Should have default title
        assert!(html.contains("<title>LaTeX Document</title>"));
        // Should not have document header
        assert!(!html.contains("<header class=\"document-header\">"));
        // Should have content
        assert!(html.contains("Just content"));
    }

    #[test]
    fn test_render_empty_document() {
        let renderer = HtmlRenderer::new();
        let document = Document {
            metadata: DocumentMetadata {
                document_class: None,
                title: None,
                author: None,
                date: None,
                packages: vec![],
            },
            preamble: vec![],
            body: vec![],
        };
        let html = renderer.render(&document);
        
        // Should still have valid HTML structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<body>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_css_inclusion() {
        let renderer = HtmlRenderer::new();
        let document = create_simple_document();
        let html = renderer.render(&document);
        
        // Should include CSS
        assert!(html.contains("<style>"));
        assert!(html.contains("</style>"));
        // Should include MathJax
        assert!(html.contains("MathJax"));
    }

    #[test]
    fn test_math_environment_rendering() {
        let renderer = HtmlRenderer::new();
        let document = Document {
            metadata: DocumentMetadata {
                document_class: None,
                title: None,
                author: None,
                date: None,
                packages: vec![],
            },
            preamble: vec![],
            body: vec![
                Node::Environment {
                    name: "equation".to_string(),
                    args: vec![],
                    content: vec![Node::Text("x = y + z".to_string())],
                },
            ],
        };
        
        let html = renderer.render(&document);
        assert!(html.contains("<div class=\"environment-equation\">"));
        assert!(html.contains("x = y + z"));
    }

    #[test]
    fn test_citation_rendering() {
        let renderer = HtmlRenderer::new();
        let document = Document {
            metadata: DocumentMetadata {
                document_class: None,
                title: None,
                author: None,
                date: None,
                packages: vec![],
            },
            preamble: vec![],
            body: vec![
                Node::Citation {
                    cite_type: CitationType::Cite,
                    keys: vec!["key1".to_string(), "key2".to_string()],
                    prenote: None,
                    postnote: None,
                },
            ],
        };
        
        let html = renderer.render(&document);
        assert!(html.contains("<a href=\"#ref-key1\" class=\"citation\">[key1]</a>"));
        assert!(html.contains("<a href=\"#ref-key2\" class=\"citation\">[key2]</a>"));
    }

    #[test]
    fn test_figure_rendering() {
        let renderer = HtmlRenderer::new();
        let document = Document {
            metadata: DocumentMetadata {
                document_class: None,
                title: None,
                author: None,
                date: None,
                packages: vec![],
            },
            preamble: vec![],
            body: vec![
                Node::Figure {
                    path: "image.png".to_string(),
                    caption: Some("Test figure".to_string()),
                    label: Some("fig:test".to_string()),
                    width: Some("50%".to_string()),
                    height: None,
                },
            ],
        };
        
        let html = renderer.render(&document);
        assert!(html.contains("<figure class=\"latex-figure\" data-label=\"fig:test\">"));
        assert!(html.contains("<img src=\"image.png\" alt=\"Figure\" width=\"50%\"/>"));
        assert!(html.contains("<figcaption>Test figure</figcaption>"));
    }

    #[test]
    fn test_verbatim_environment() {
        let renderer = HtmlRenderer::new();
        let verbatim_node = Node::Environment {
            name: "verbatim".to_string(),
            args: vec![],
            content: vec![Node::Text("code here".to_string())],
        };
        
        let html = renderer.render_node(&verbatim_node);
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("code here"));
        assert!(html.contains("</code></pre>"));
    }
}