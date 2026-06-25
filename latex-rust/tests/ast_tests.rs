//! Unit tests for the AST module

use latex_rust::ast::*;
use std::collections::HashMap;
use serde_json;

#[cfg(test)]
mod ast_unit_tests {
    use super::*;

    // Document tests
    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert!(doc.preamble.is_empty());
        assert!(doc.body.is_empty());
        assert!(doc.metadata.title.is_none());
        assert!(doc.metadata.author.is_none());
        assert!(doc.metadata.date.is_none());
        assert!(doc.metadata.document_class.is_none());
        assert!(doc.metadata.packages.is_empty());
    }

    #[test]
    fn test_document_default() {
        let doc = Document::default();
        assert!(doc.preamble.is_empty());
        assert!(doc.body.is_empty());
        assert!(doc.metadata.title.is_none());
    }

    #[test]
    fn test_document_add_node() {
        let mut doc = Document::new();
        let text_node = Node::Text("Hello World".to_string());
        doc.add_node(text_node.clone());
        assert_eq!(doc.body.len(), 1);
        assert!(matches!(doc.body[0], Node::Text(_)));
    }

    #[test]
    fn test_document_add_preamble_node() {
        let mut doc = Document::new();
        let package_node = Node::Package {
            name: "amsmath".to_string(),
            options: vec![],
        };
        doc.add_preamble_node(package_node.clone());
        assert_eq!(doc.preamble.len(), 1);
        assert!(matches!(doc.preamble[0], Node::Package { .. }));
    }

    #[test]
    fn test_document_set_metadata() {
        let mut doc = Document::new();
        doc.set_title("Test Title".to_string());
        doc.set_author("Test Author".to_string());
        doc.add_package("amsmath".to_string());
        
        assert_eq!(doc.metadata.title, Some("Test Title".to_string()));
        assert_eq!(doc.metadata.author, Some("Test Author".to_string()));
        assert_eq!(doc.metadata.packages, vec!["amsmath".to_string()]);
    }

    // Node tests
    #[test]
    fn test_node_text() {
        let node = Node::Text("Hello".to_string());
        assert!(node.is_text());
        assert!(!node.is_command());
        assert_eq!(node.as_text(), Some("Hello"));
        assert_eq!(node.as_command(), None);
    }

    #[test]
    fn test_node_command() {
        let node = Node::Command {
            name: "textbf".to_string(),
            args: vec![Argument::Required(vec![Node::Text("bold".to_string())])],
            content: None,
        };
        assert!(!node.is_text());
        assert!(node.is_command());
        assert_eq!(node.as_text(), None);
        assert_eq!(node.as_command(), Some("textbf"));
    }

    #[test]
    fn test_node_environment() {
        let node = Node::Environment {
            name: "itemize".to_string(),
            args: vec![],
            content: vec![Node::Text("Item 1".to_string())],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_math() {
        let inline_math = Node::Math {
            display: false,
            content: "x + y".to_string(),
        };
        let display_math = Node::Math {
            display: true,
            content: "E = mc^2".to_string(),
        };
        assert!(!inline_math.is_text());
        assert!(!display_math.is_text());
    }

    #[test]
    fn test_node_math_environment() {
        let node = Node::MathEnvironment {
            env_type: MathEnvironmentType::Equation,
            content: vec![Node::Text("x = 1".to_string())],
            label: Some("eq:test".to_string()),
            numbered: true,
            equation_number: None,
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_group() {
        let node = Node::Group(vec![
            Node::Text("Hello ".to_string()),
            Node::Text("World".to_string()),
        ]);
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_list() {
        let node = Node::List {
            list_type: ListType::Itemize,
            items: vec![ListItem {
                label: None,
                content: vec![Node::Text("Item 1".to_string())],
            }],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_table() {
        let node = Node::Table {
            table_type: TableType::Tabular,
            alignment: vec![ColumnAlignment::Left, ColumnAlignment::Center],
            rows: vec![TableRow {
                cells: vec![
                    TableCell::Normal(vec![Node::Text("Cell 1".to_string())]),
                    TableCell::Normal(vec![Node::Text("Cell 2".to_string())]),
                ],
                rules_before: vec![],
                rules_after: vec![],
            }],
            caption: None,
            label: None,
            position: None,
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_figure() {
        let node = Node::Figure {
            path: "image.png".to_string(),
            caption: Some("Test figure".to_string()),
            label: Some("fig:test".to_string()),
            width: Some("50%".to_string()),
            height: None,
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_section() {
        let node = Node::Section {
            level: SectionLevel::Section,
            title: "Test Section".to_string(),
            content: vec![Node::Text("Section content".to_string())],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_paragraph() {
        let node = Node::Paragraph;
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_line_break() {
        let node = Node::LineBreak;
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_whitespace() {
        let node = Node::Whitespace("   ".to_string());
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_label() {
        let node = Node::Label("test:label".to_string());
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_reference() {
        let node = Node::Reference {
            ref_type: ReferenceType::Ref,
            label: "test:label".to_string(),
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_footnote() {
        let node = Node::Footnote {
            content: vec![Node::Text("Footnote text".to_string())],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_spacing() {
        let node = Node::Spacing {
            space_type: SpacingType::Vspace,
            amount: Some("10pt".to_string()),
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_command_definition() {
        let node = Node::CommandDefinition {
            name: "mycommand".to_string(),
            num_args: Some(2),
            default_args: vec!["default".to_string()],
            definition: vec![Node::Text("#1 and #2".to_string())],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_citation() {
        let node = Node::Citation {
            cite_type: CitationType::Cite,
            keys: vec!["key1".to_string(), "key2".to_string()],
            prenote: Some("see".to_string()),
            postnote: Some("p. 10".to_string()),
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_bibliography_entry() {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Title".to_string());
        fields.insert("author".to_string(), "Test Author".to_string());
        
        let node = Node::BibliographyEntry {
            key: "test2023".to_string(),
            entry_type: "article".to_string(),
            fields,
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_bibliography() {
        let node = Node::Bibliography {
            style: Some("plain".to_string()),
            entries: vec![],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    #[test]
    fn test_node_package() {
        let node = Node::Package {
            name: "amsmath".to_string(),
            options: vec!["fleqn".to_string()],
        };
        assert!(!node.is_text());
        assert!(!node.is_command());
    }

    // Argument tests
    #[test]
    fn test_argument_required() {
        let arg = Argument::Required(vec![Node::Text("content".to_string())]);
        assert!(matches!(arg, Argument::Required(_)));
    }

    #[test]
    fn test_argument_optional() {
        let arg = Argument::Optional(vec![Node::Text("content".to_string())]);
        assert!(matches!(arg, Argument::Optional(_)));
    }

    // ListType tests
    #[test]
    fn test_list_type_equality() {
        assert_eq!(ListType::Itemize, ListType::Itemize);
        assert_eq!(ListType::Enumerate, ListType::Enumerate);
        assert_eq!(ListType::Description, ListType::Description);
        assert_ne!(ListType::Itemize, ListType::Enumerate);
    }

    // ListItem tests
    #[test]
    fn test_list_item() {
        let item = ListItem {
            label: Some("Item 1".to_string()),
            content: vec![Node::Text("Content".to_string())],
        };
        assert_eq!(item.label, Some("Item 1".to_string()));
        assert_eq!(item.content.len(), 1);
    }

    // TableRow tests
    #[test]
    fn test_table_row() {
        let row = TableRow {
            cells: vec![TableCell::Normal(vec![Node::Text("Cell".to_string())])],
            rules_before: vec![TableRule::Hline],
            rules_after: vec![TableRule::Midrule],
        };
        assert_eq!(row.cells.len(), 1);
        assert_eq!(row.rules_before.len(), 1);
        assert_eq!(row.rules_after.len(), 1);
    }

    // ColumnAlignment tests
    #[test]
    fn test_column_alignment() {
        let left = ColumnAlignment::Left;
        let center = ColumnAlignment::Center;
        let right = ColumnAlignment::Right;
        let paragraph = ColumnAlignment::Paragraph("5cm".to_string());
        let fixed = ColumnAlignment::FixedWidth("3cm".to_string());
        
        assert!(matches!(left, ColumnAlignment::Left));
        assert!(matches!(center, ColumnAlignment::Center));
        assert!(matches!(right, ColumnAlignment::Right));
        assert!(matches!(paragraph, ColumnAlignment::Paragraph(_)));
        assert!(matches!(fixed, ColumnAlignment::FixedWidth(_)));
    }

    // TableRule tests
    #[test]
    fn test_table_rule() {
        let hline = TableRule::Hline;
        let cline = TableRule::Cline("1-3".to_string());
        let toprule = TableRule::Toprule;
        let midrule = TableRule::Midrule;
        let bottomrule = TableRule::Bottomrule;
        
        assert!(matches!(hline, TableRule::Hline));
        assert!(matches!(cline, TableRule::Cline(_)));
        assert!(matches!(toprule, TableRule::Toprule));
        assert!(matches!(midrule, TableRule::Midrule));
        assert!(matches!(bottomrule, TableRule::Bottomrule));
    }

    // MultiColumn tests
    #[test]
    fn test_multi_column() {
        let multi_col = MultiColumn {
            span: 3,
            alignment: ColumnAlignment::Center,
            content: vec![Node::Text("Merged cell".to_string())],
        };
        assert_eq!(multi_col.span, 3);
        assert!(matches!(multi_col.alignment, ColumnAlignment::Center));
        assert_eq!(multi_col.content.len(), 1);
    }

    // TableCell tests
    #[test]
    fn test_table_cell() {
        let normal_cell = TableCell::Normal(vec![Node::Text("Normal".to_string())]);
        let multi_cell = TableCell::MultiColumn(MultiColumn {
            span: 2,
            alignment: ColumnAlignment::Left,
            content: vec![Node::Text("Multi".to_string())],
        });
        
        assert!(matches!(normal_cell, TableCell::Normal(_)));
        assert!(matches!(multi_cell, TableCell::MultiColumn(_)));
    }

    // MathEnvironmentType tests
    #[test]
    fn test_math_environment_type() {
        let equation = MathEnvironmentType::Equation;
        let align = MathEnvironmentType::Align;
        let gather = MathEnvironmentType::Gather;
        let multline = MathEnvironmentType::Multline;
        let array = MathEnvironmentType::Array;
        let matrix = MathEnvironmentType::Matrix;
        let pmatrix = MathEnvironmentType::Pmatrix;
        let bmatrix = MathEnvironmentType::Bmatrix;
        let vmatrix = MathEnvironmentType::Vmatrix;
        
        assert!(matches!(equation, MathEnvironmentType::Equation));
        assert!(matches!(align, MathEnvironmentType::Align));
        assert!(matches!(gather, MathEnvironmentType::Gather));
        assert!(matches!(multline, MathEnvironmentType::Multline));
        assert!(matches!(array, MathEnvironmentType::Array));
        assert!(matches!(matrix, MathEnvironmentType::Matrix));
        assert!(matches!(pmatrix, MathEnvironmentType::Pmatrix));
        assert!(matches!(bmatrix, MathEnvironmentType::Bmatrix));
        assert!(matches!(vmatrix, MathEnvironmentType::Vmatrix));
    }

    // ReferenceType tests
    #[test]
    fn test_reference_type() {
        let ref_type = ReferenceType::Ref;
        let pageref = ReferenceType::Pageref;
        let eqref = ReferenceType::Eqref;
        
        assert!(matches!(ref_type, ReferenceType::Ref));
        assert!(matches!(pageref, ReferenceType::Pageref));
        assert!(matches!(eqref, ReferenceType::Eqref));
    }

    // SpacingType tests
    #[test]
    fn test_spacing_type() {
        let vspace = SpacingType::Vspace;
        let hspace = SpacingType::Hspace;
        let vfill = SpacingType::Vfill;
        let hfill = SpacingType::Hfill;
        let newpage = SpacingType::Newpage;
        let clearpage = SpacingType::Clearpage;
        let pagebreak = SpacingType::Pagebreak;
        let linebreak = SpacingType::Linebreak;
        let par = SpacingType::Par;
        
        assert!(matches!(vspace, SpacingType::Vspace));
        assert!(matches!(hspace, SpacingType::Hspace));
        assert!(matches!(vfill, SpacingType::Vfill));
        assert!(matches!(hfill, SpacingType::Hfill));
        assert!(matches!(newpage, SpacingType::Newpage));
        assert!(matches!(clearpage, SpacingType::Clearpage));
        assert!(matches!(pagebreak, SpacingType::Pagebreak));
        assert!(matches!(linebreak, SpacingType::Linebreak));
        assert!(matches!(par, SpacingType::Par));
    }

    // CitationType tests
    #[test]
    fn test_citation_type() {
        let cite = CitationType::Cite;
        let citep = CitationType::Citep;
        let citet = CitationType::Citet;
        let citeauthor = CitationType::Citeauthor;
        let citeyear = CitationType::Citeyear;
        let nocite = CitationType::Nocite;
        
        assert!(matches!(cite, CitationType::Cite));
        assert!(matches!(citep, CitationType::Citep));
        assert!(matches!(citet, CitationType::Citet));
        assert!(matches!(citeauthor, CitationType::Citeauthor));
        assert!(matches!(citeyear, CitationType::Citeyear));
        assert!(matches!(nocite, CitationType::Nocite));
    }

    // SectionLevel tests
    #[test]
    fn test_section_level_equality() {
        assert_eq!(SectionLevel::Section, SectionLevel::Section);
        assert_ne!(SectionLevel::Section, SectionLevel::Subsection);
    }

    #[test]
    fn test_section_level_level() {
        assert_eq!(SectionLevel::Part.level(), 0);
        assert_eq!(SectionLevel::Chapter.level(), 1);
        assert_eq!(SectionLevel::Section.level(), 2);
        assert_eq!(SectionLevel::Subsection.level(), 3);
        assert_eq!(SectionLevel::Subsubsection.level(), 4);
        assert_eq!(SectionLevel::Paragraph.level(), 5);
        assert_eq!(SectionLevel::Subparagraph.level(), 6);
    }

    #[test]
    fn test_section_level_from_command() {
        assert_eq!(SectionLevel::from_command("part"), Some(SectionLevel::Part));
        assert_eq!(SectionLevel::from_command("chapter"), Some(SectionLevel::Chapter));
        assert_eq!(SectionLevel::from_command("section"), Some(SectionLevel::Section));
        assert_eq!(SectionLevel::from_command("subsection"), Some(SectionLevel::Subsection));
        assert_eq!(SectionLevel::from_command("subsubsection"), Some(SectionLevel::Subsubsection));
        assert_eq!(SectionLevel::from_command("paragraph"), Some(SectionLevel::Paragraph));
        assert_eq!(SectionLevel::from_command("subparagraph"), Some(SectionLevel::Subparagraph));
        assert_eq!(SectionLevel::from_command("unknown"), None);
    }

    // Serialization tests
    #[test]
    fn test_document_serialization() {
        let mut doc = Document::new();
        doc.set_title("Test Document".to_string());
        doc.add_node(Node::Text("Hello World".to_string()));
        
        let json = serde_json::to_string(&doc).expect("Failed to serialize document");
        let deserialized: Document = serde_json::from_str(&json).expect("Failed to deserialize document");
        
        assert_eq!(doc.metadata.title, deserialized.metadata.title);
        assert_eq!(doc.body.len(), deserialized.body.len());
    }

    #[test]
    fn test_node_serialization() {
        let node = Node::Command {
            name: "textbf".to_string(),
            args: vec![Argument::Required(vec![Node::Text("bold".to_string())])],
            content: None,
        };
        
        let json = serde_json::to_string(&node).expect("Failed to serialize node");
        let deserialized: Node = serde_json::from_str(&json).expect("Failed to deserialize node");
        
        assert!(matches!(deserialized, Node::Command { .. }));
        if let Node::Command { name, .. } = deserialized {
            assert_eq!(name, "textbf");
        }
    }

    #[test]
    fn test_complex_node_serialization() {
        let node = Node::Table {
            table_type: TableType::Tabular,
            alignment: vec![ColumnAlignment::Left, ColumnAlignment::Center],
            rows: vec![TableRow {
                cells: vec![
                    TableCell::Normal(vec![Node::Text("Cell 1".to_string())]),
                    TableCell::MultiColumn(MultiColumn {
                        span: 2,
                        alignment: ColumnAlignment::Right,
                        content: vec![Node::Text("Merged".to_string())],
                    }),
                ],
                rules_before: vec![TableRule::Toprule],
                rules_after: vec![TableRule::Bottomrule],
            }],
            caption: None,
            label: None,
            position: None,
        };
        
        let json = serde_json::to_string(&node).expect("Failed to serialize complex node");
        let deserialized: Node = serde_json::from_str(&json).expect("Failed to deserialize complex node");
        
        assert!(matches!(deserialized, Node::Table { .. }));
    }

    // DocumentMetadata tests
    #[test]
    fn test_document_metadata_default() {
        let metadata = DocumentMetadata::default();
        assert!(metadata.title.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.date.is_none());
        assert!(metadata.document_class.is_none());
        assert!(metadata.packages.is_empty());
    }

    #[test]
    fn test_document_metadata_serialization() {
        let mut metadata = DocumentMetadata::default();
        metadata.title = Some("Test Title".to_string());
        metadata.author = Some("Test Author".to_string());
        metadata.packages.push("amsmath".to_string());
        
        let json = serde_json::to_string(&metadata).expect("Failed to serialize metadata");
        let deserialized: DocumentMetadata = serde_json::from_str(&json).expect("Failed to deserialize metadata");
        
        assert_eq!(metadata.title, deserialized.title);
        assert_eq!(metadata.author, deserialized.author);
        assert_eq!(metadata.packages, deserialized.packages);
    }
}