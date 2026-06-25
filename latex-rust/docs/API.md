# LaTeX-Rust API Documentation

This document describes the public API of the LaTeX-Rust library.

## Core API

### Document Processing

```rust
use latex_rust::{Document, Renderer, HtmlRenderer, PdfRenderer};

// Parse a LaTeX document
let document = Document::from_str(latex_content)?;

// Render to HTML
let html_renderer = HtmlRenderer::new();
let html_output = html_renderer.render(&document)?;

// Render to PDF
let pdf_renderer = PdfRenderer::new();
let pdf_output = pdf_renderer.render(&document)?;
```

### Lexer API

```rust
use latex_rust::lexer::{Lexer, Token};

// Create a lexer
let mut lexer = Lexer::new(input);

// Tokenize input
let tokens: Vec<Token> = lexer.tokenize()?;

// Process tokens one by one
while let Some(token) = lexer.next_token()? {
    match token {
        Token::Text(content) => println!("Text: {}", content),
        Token::Command(name) => println!("Command: {}", name),
        // ... handle other token types
    }
}
```

### Parser API

```rust
use latex_rust::parser::{Parser, ParseError};
use latex_rust::ast::{Document, Node};

// Create a parser
let mut parser = Parser::new(tokens);

// Parse document
let document: Document = parser.parse_document()?;

// Access document structure
for node in &document.content {
    match node {
        Node::Section { level, title, content } => {
            println!("Section: {}", title);
        },
        Node::Text(text) => {
            println!("Text: {}", text);
        },
        // ... handle other node types
    }
}
```

### AST Types

#### Document

```rust
pub struct Document {
    pub metadata: DocumentMetadata,
    pub content: Vec<Node>,
}

pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub document_class: Option<String>,
    pub packages: Vec<String>,
}
```

#### Node Types

```rust
pub enum Node {
    Text(String),
    Command {
        name: String,
        args: Vec<Argument>,
    },
    Environment {
        name: String,
        args: Vec<Argument>,
        content: Vec<Node>,
    },
    Math {
        content: String,
        display: bool,
    },
    Section {
        level: SectionLevel,
        title: String,
        content: Vec<Node>,
    },
    List {
        list_type: ListType,
        items: Vec<ListItem>,
    },
    Table {
        rows: Vec<TableRow>,
        column_spec: Vec<ColumnAlignment>,
    },
    Figure {
        content: Vec<Node>,
        caption: Option<String>,
        label: Option<String>,
    },
    Citation {
        keys: Vec<String>,
        citation_type: CitationType,
    },
    Reference {
        key: String,
        ref_type: ReferenceType,
    },
    // ... other node types
}
```

### Renderer API

#### HTML Renderer

```rust
use latex_rust::renderer::{HtmlRenderer, RenderOptions};

// Create renderer with options
let options = RenderOptions {
    include_css: true,
    math_renderer: MathRenderer::MathJax,
    custom_css: Some("body { font-family: serif; }".to_string()),
};

let renderer = HtmlRenderer::with_options(options);
let html = renderer.render(&document)?;
```

#### PDF Renderer

```rust
use latex_rust::renderer::{PdfRenderer, PdfOptions};

// Create PDF renderer with options
let options = PdfOptions {
    page_size: PageSize::A4,
    margin: Margin::default(),
    font_family: "Times New Roman".to_string(),
};

let renderer = PdfRenderer::with_options(options);
let pdf_bytes = renderer.render(&document)?;
```

## Error Handling

```rust
use latex_rust::error::{LaTeXError, ErrorKind};

match result {
    Ok(document) => {
        // Process document
    },
    Err(LaTeXError { kind, message, position }) => {
        match kind {
            ErrorKind::LexError => println!("Lexing error: {}", message),
            ErrorKind::ParseError => println!("Parsing error: {}", message),
            ErrorKind::RenderError => println!("Rendering error: {}", message),
            ErrorKind::IoError => println!("IO error: {}", message),
        }
        
        if let Some(pos) = position {
            println!("At line {}, column {}", pos.line, pos.column);
        }
    }
}
```

## Configuration

### Lexer Configuration

```rust
use latex_rust::lexer::{Lexer, LexerConfig};

let config = LexerConfig {
    preserve_whitespace: true,
    handle_comments: true,
    math_delimiters: vec![("$", "$"), ("$$", "$$")],
};

let lexer = Lexer::with_config(input, config);
```

### Parser Configuration

```rust
use latex_rust::parser::{Parser, ParserConfig};

let config = ParserConfig {
    strict_mode: false,
    max_nesting_depth: 100,
    custom_commands: HashMap::new(),
};

let parser = Parser::with_config(tokens, config);
```

## Utility Functions

```rust
use latex_rust::utils;

// Escape HTML entities
let escaped = utils::escape_html("<script>alert('xss')</script>");

// Normalize whitespace
let normalized = utils::normalize_whitespace("  multiple   spaces  ");

// Extract text content from nodes
let text = utils::extract_text(&nodes);
```

## Examples

### Basic Usage

```rust
use latex_rust::*;

fn main() -> Result<(), LaTeXError> {
    let latex = r#"
        \documentclass{article}
        \begin{document}
        \section{Hello World}
        This is a \textbf{bold} statement.
        \end{document}
    "#;
    
    let document = Document::from_str(latex)?;
    let renderer = HtmlRenderer::new();
    let html = renderer.render(&document)?;
    
    println!("{}", html);
    Ok(())
}
```

### Advanced Usage

```rust
use latex_rust::*;
use std::fs;

fn process_latex_file(input_path: &str, output_path: &str) -> Result<(), LaTeXError> {
    // Read input file
    let latex_content = fs::read_to_string(input_path)
        .map_err(|e| LaTeXError::io_error(e.to_string()))?;
    
    // Parse document
    let document = Document::from_str(&latex_content)?;
    
    // Configure renderer
    let options = RenderOptions {
        include_css: true,
        math_renderer: MathRenderer::MathJax,
        custom_css: Some(fs::read_to_string("custom.css").ok()),
    };
    
    let renderer = HtmlRenderer::with_options(options);
    let html = renderer.render(&document)?;
    
    // Write output
    fs::write(output_path, html)
        .map_err(|e| LaTeXError::io_error(e.to_string()))?;
    
    Ok(())
}
```