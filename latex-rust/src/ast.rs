//! Abstract Syntax Tree for LaTeX documents

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete LaTeX document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub preamble: Vec<Node>,
    pub body: Vec<Node>,
    pub metadata: DocumentMetadata,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub document_class: Option<String>,
    pub packages: Vec<String>,
}

/// AST node representing different LaTeX elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    /// Plain text content
    Text(String),
    
    /// LaTeX command with optional arguments
    Command {
        name: String,
        args: Vec<Argument>,
        content: Option<Box<Node>>,
    },
    
    /// Environment (e.g., \begin{itemize}...\end{itemize})
    Environment {
        name: String,
        args: Vec<Argument>,
        content: Vec<Node>,
    },
    
    /// Mathematical expression
    Math {
        display: bool, // true for display math ($$), false for inline ($)
        content: String,
    },
    
    /// Mathematical environment (equation, align, gather, etc.)
    MathEnvironment {
        env_type: MathEnvironmentType,
        content: Vec<Node>,
        label: Option<String>,
        numbered: bool,
        equation_number: Option<usize>,
    },
    
    /// Group of nodes (enclosed in braces)
    Group(Vec<Node>),
    
    /// List of items
    List {
        list_type: ListType,
        items: Vec<ListItem>,
    },
    
    /// Table
    Table {
        table_type: TableType,
        alignment: Vec<ColumnAlignment>,
        rows: Vec<TableRow>,
        caption: Option<String>,
        label: Option<String>,
        position: Option<String>, // [h], [t], [b], [p], etc.
    },
    
    /// Figure with optional caption
    Figure {
        path: String,
        caption: Option<String>,
        label: Option<String>,
        width: Option<String>,
        height: Option<String>,
    },
    
    /// Section heading
    Section {
        level: SectionLevel,
        title: String,
        content: Vec<Node>,
    },
    
    /// Paragraph break
    Paragraph,
    
    /// Line break
    LineBreak,
    
    /// Whitespace
    Whitespace(String),
    
    /// Label for cross-referencing
    Label(String),
    
    /// Reference to a label
    Reference {
        ref_type: ReferenceType,
        label: String,
    },
    
    /// Footnote
    Footnote {
        content: Vec<Node>,
    },
    
    /// Spacing command
    Spacing {
        space_type: SpacingType,
        amount: Option<String>,
    },
    
    /// Custom command definition
    CommandDefinition {
        name: String,
        num_args: Option<usize>,
        default_args: Vec<String>,
        definition: Vec<Node>,
    },
    
    /// Citation reference
    Citation {
        cite_type: CitationType,
        keys: Vec<String>,
        prenote: Option<String>,
        postnote: Option<String>,
    },
    
    /// Bibliography entry
    BibliographyEntry {
        key: String,
        entry_type: String,
        fields: HashMap<String, String>,
    },
    
    /// Bibliography environment
    Bibliography {
        style: Option<String>,
        entries: Vec<Node>,
    },
    
    /// Package declaration
    Package {
        name: String,
        options: Vec<String>,
    },
    
    /// Code block with syntax highlighting
    CodeBlock {
        language: Option<String>,
        code: String,
        style: CodeStyle,
        line_numbers: bool,
        caption: Option<String>,
        label: Option<String>,
    },
}

/// Command or environment argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    /// Required argument {content}
    Required(Vec<Node>),
    /// Optional argument [content]
    Optional(Vec<Node>),
}

/// List types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ListType {
    Itemize,
    Enumerate,
    Description,
}

/// List item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub label: Option<String>,
    pub content: Vec<Node>,
}

/// Table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub rules_before: Vec<TableRule>, // Rules that appear before this row
    pub rules_after: Vec<TableRule>,  // Rules that appear after this row
}

/// Table types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableType {
    Tabular,
    Longtable,
    Array,
    Tabularx,
}

/// Column alignment for tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnAlignment {
    Left,
    Center,
    Right,
    Paragraph(String), // p{width}
    FixedWidth(String), // m{width}, b{width}
    ArrayColumn(String), // For array environment: c, l, r with math mode
    XColumn, // X column for tabularx
    SColumn(String), // S column for siunitx numbers
}

/// Table rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableRule {
    Hline,           // \hline
    Cline(String),   // \cline{i-j}
    Toprule,         // \toprule (booktabs)
    Midrule,         // \midrule (booktabs)
    Bottomrule,      // \bottomrule (booktabs)
}

/// Multicolumn specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiColumn {
    pub span: usize,
    pub alignment: ColumnAlignment,
    pub content: Vec<Node>,
}

/// Multirow specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRow {
    pub span: usize,
    pub width: Option<String>,
    pub content: Vec<Node>,
}

/// Table cell types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableCell {
    Normal(Vec<Node>),
    MultiColumn(MultiColumn),
    MultiRow(MultiRow),
    MultiColumnRow {
        multicolumn: MultiColumn,
        multirow: MultiRow,
    },
}

/// Mathematical environment types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathEnvironmentType {
    Equation,
    Align,
    Gather,
    Multline,
    Array,
    Matrix,
    Pmatrix,
    Bmatrix,
    Vmatrix,
    Smallmatrix,
    Cases,
    Split,
    Aligned,
    Eqnarray,
}

/// Reference types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    Ref,      // \ref{}
    Pageref,  // \pageref{}
    Eqref,    // \eqref{}
}

/// Spacing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpacingType {
    Vspace,   // \vspace{}
    Hspace,   // \hspace{}
    Vfill,    // \vfill
    Hfill,    // \hfill
    Newpage,  // \newpage
    Clearpage, // \clearpage
    Pagebreak, // \pagebreak
    Linebreak, // \\
    Par,      // \par
}

/// Citation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CitationType {
    Cite,      // \cite{}
    Citep,     // \citep{} (parenthetical)
    Citet,     // \citet{} (textual)
    Citeauthor, // \citeauthor{}
    Citeyear,  // \citeyear{}
    Nocite,    // \nocite{}
}

/// Section levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SectionLevel {
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    Subparagraph,
}

/// Code block styles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodeStyle {
    Verbatim,    // Basic verbatim environment
    Listings,    // lstlisting package style
    Minted,      // minted package style
    Inline,      // Inline code (\verb or \mint)
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            preamble: Vec::new(),
            body: Vec::new(),
            metadata: DocumentMetadata::default(),
        }
    }
    
    /// Add a node to the document body
    pub fn add_node(&mut self, node: Node) {
        self.body.push(node);
    }
    
    /// Add a node to the preamble
    pub fn add_preamble_node(&mut self, node: Node) {
        self.preamble.push(node);
    }
    
    /// Set document title
    pub fn set_title(&mut self, title: String) {
        self.metadata.title = Some(title);
    }
    
    /// Set document author
    pub fn set_author(&mut self, author: String) {
        self.metadata.author = Some(author);
    }
    
    /// Add a package to the document
    pub fn add_package(&mut self, package: String) {
        self.metadata.packages.push(package);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    /// Check if this node is a text node
    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }
    
    /// Check if this node is a command
    pub fn is_command(&self) -> bool {
        matches!(self, Node::Command { .. })
    }
    
    /// Get the text content if this is a text node
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Node::Text(text) => Some(text),
            _ => None,
        }
    }
    
    /// Get command name if this is a command node
    pub fn as_command(&self) -> Option<&str> {
        match self {
            Node::Command { name, .. } => Some(name),
            _ => None,
        }
    }
}

impl SectionLevel {
    /// Get the numeric level (lower is higher in hierarchy)
    pub fn level(&self) -> u8 {
        match self {
            SectionLevel::Part => 0,
            SectionLevel::Chapter => 1,
            SectionLevel::Section => 2,
            SectionLevel::Subsection => 3,
            SectionLevel::Subsubsection => 4,
            SectionLevel::Paragraph => 5,
            SectionLevel::Subparagraph => 6,
        }
    }
    
    /// Create from command name
    pub fn from_command(cmd: &str) -> Option<Self> {
        match cmd {
            "part" => Some(SectionLevel::Part),
            "chapter" => Some(SectionLevel::Chapter),
            "section" => Some(SectionLevel::Section),
            "subsection" => Some(SectionLevel::Subsection),
            "subsubsection" => Some(SectionLevel::Subsubsection),
            "paragraph" => Some(SectionLevel::Paragraph),
            "subparagraph" => Some(SectionLevel::Subparagraph),
            _ => None,
        }
    }
}