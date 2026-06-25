//! Integration tests for LaTeX-Rust processor

use latex_rust::*;

#[test]
fn test_basic_document_processing() {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
\documentclass{article}
\title{Test Document}
\author{Test Author}
\begin{document}
\maketitle
\section{Introduction}
This is a test.
\end{document}
"#;
    
    let result = processor.to_html(latex_input);
    assert!(result.is_ok());
    
    let html = result.unwrap();
    assert!(html.contains("<title>Test Document</title>"));
    assert!(html.contains("<h1 class=\"document-title\">Test Document</h1>"));
    assert!(html.contains("<p class=\"document-author\">Test Author</p>"));
    assert!(html.contains("<h3 class=\"section-2\">Introduction</h3>"));
}

#[test]
fn test_text_formatting() {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
\documentclass{article}
\begin{document}
This is \textbf{bold}, \textit{italic}, and \texttt{monospace} text.
\end{document}
"#;
    
    let html = processor.to_html(latex_input).unwrap();
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<code>monospace</code>"));
}

#[test]
fn test_mathematical_expressions() {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
\documentclass{article}
\begin{document}
Inline math: $x^2 + y^2 = z^2$

Display math:
$$\int_0^\infty e^{-x} dx = 1$$
\end{document}
"#;
    
    let html = processor.to_html(latex_input).unwrap();
    assert!(html.contains("<span class=\"math-inline\">$x^2 + y^2 = z^2$</span>"));
    assert!(html.contains("<div class=\"math-display\">$$\\int_0^\\infty e^{-x} dx = 1$$</div>"));
}

#[test]
fn test_lists() {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
\documentclass{article}
\begin{document}
\begin{itemize}
\item First item
\item Second item
\end{itemize}

\begin{enumerate}
\item Numbered first
\item Numbered second
\end{enumerate}
\end{document}
"#;
    
    let html = processor.to_html(latex_input).unwrap();
    assert!(html.contains("<ul>"));
    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>First item</li>"));
    assert!(html.contains("<li>Numbered first</li>"));
}

#[test]
fn test_environments() {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
\documentclass{article}
\begin{document}
\begin{abstract}
This is an abstract.
\end{abstract}

\begin{quote}
This is a quote.
\end{quote}

\begin{center}
Centered text.
\end{center}
\end{document}
"#;
    
    let html = processor.to_html(latex_input).unwrap();
    assert!(html.contains("<div class=\"abstract\">"));
    assert!(html.contains("<h2>Abstract</h2>"));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<div class=\"center\">"));
}

#[test]
fn test_section_hierarchy() {
    let mut processor = LaTeXProcessor::new();
    
    let latex_input = r#"
\documentclass{article}
\begin{document}
\section{Section}
\subsection{Subsection}
\subsubsection{Subsubsection}
\end{document}
"#;
    
    let html = processor.to_html(latex_input).unwrap();
    assert!(html.contains("<h3 class=\"section-2\">Section</h3>"));
    assert!(html.contains("<h4 class=\"section-3\">Subsection</h4>"));
    assert!(html.contains("<h5 class=\"section-4\">Subsubsection</h5>"));
}

#[test]
fn test_lexer_tokenization() {
    let lexer = Lexer::new();
    
    let tokens = lexer.tokenize("\\section{Hello World}").unwrap();
    
    assert_eq!(tokens.len(), 7); // command, {, text, whitespace, text, }, EOF
    
    match &tokens[0].token_type {
        TokenType::Command(cmd) => assert_eq!(cmd, "section"),
        _ => panic!("Expected command token"),
    }
    
    match &tokens[1].token_type {
        TokenType::LeftBrace => {},
        _ => panic!("Expected left brace"),
    }
    
    match &tokens[6].token_type {
        TokenType::Eof => {},
        _ => panic!("Expected EOF token"),
    }
}

#[test]
fn test_math_tokenization() {
    let lexer = Lexer::new();
    
    let tokens = lexer.tokenize("$x^2$ and $$y^3$$").unwrap();
    
    // Find dollar tokens
    let dollar_tokens: Vec<_> = tokens.iter()
        .filter(|t| matches!(t.token_type, TokenType::Dollar | TokenType::DoubleDollar))
        .collect();
    
    assert_eq!(dollar_tokens.len(), 4); // $, $, $$, $$
    assert!(matches!(dollar_tokens[0].token_type, TokenType::Dollar));
    assert!(matches!(dollar_tokens[1].token_type, TokenType::Dollar));
    assert!(matches!(dollar_tokens[2].token_type, TokenType::DoubleDollar));
    assert!(matches!(dollar_tokens[3].token_type, TokenType::DoubleDollar));
}

#[test]
fn test_parser_document_structure() {
    let mut parser = Parser::new();
    let lexer = Lexer::new();
    
    let latex_input = r#"
\documentclass{article}
\title{Test Title}
\author{Test Author}
\begin{document}
\maketitle
\section{Introduction}
Content here.
\end{document}
"#;
    
    let tokens = lexer.tokenize(latex_input).unwrap();
    let document = parser.parse(tokens).unwrap();
    
    assert_eq!(document.metadata.title, Some("Test Title".to_string()));
    assert_eq!(document.metadata.author, Some("Test Author".to_string()));
    assert_eq!(document.metadata.document_class, Some("article".to_string()));
    assert!(!document.body.is_empty());
}

#[test]
fn test_ast_node_types() {
    // Test text node
    let text_node = Node::Text("Hello".to_string());
    assert!(text_node.is_text());
    assert_eq!(text_node.as_text(), Some("Hello"));
    
    // Test command node
    let cmd_node = Node::Command {
        name: "textbf".to_string(),
        args: vec![],
        content: None,
    };
    assert!(cmd_node.is_command());
    assert_eq!(cmd_node.as_command(), Some("textbf"));
    
    // Test section levels
    assert_eq!(SectionLevel::Section.level(), 2);
    assert_eq!(SectionLevel::Subsection.level(), 3);
    assert_eq!(SectionLevel::from_command("section"), Some(SectionLevel::Section));
}

#[test]
fn test_html_renderer() {
    let renderer = HtmlRenderer::new();
    
    // Test text rendering
    let text_node = Node::Text("Hello <world>".to_string());
    let html = renderer.render_node(&text_node);
    assert_eq!(html, "Hello &lt;world&gt;");
    
    // Test command rendering
    let cmd_node = Node::Command {
        name: "textbf".to_string(),
        args: vec![Argument::Required(vec![Node::Text("bold".to_string())])],
        content: None,
    };
    let html = renderer.render_node(&cmd_node);
    assert_eq!(html, "<strong>bold</strong>");
    
    // Test math rendering
    let math_node = Node::Math {
        display: false,
        content: "x^2".to_string(),
    };
    let html = renderer.render_node(&math_node);
    assert_eq!(html, "<span class=\"math-inline\">$x^2$</span>");
}

#[test]
fn test_error_handling() {
    let mut processor = LaTeXProcessor::new();
    
    // Test with malformed LaTeX
    let malformed_input = "\\section{Unclosed brace";
    let result = processor.to_html(malformed_input);
    assert!(result.is_err());
    
    // Test empty input
    let empty_result = processor.to_html("");
    assert!(empty_result.is_ok());
}

#[test]
fn test_complex_document() {
    let mut processor = LaTeXProcessor::new();
    
    let complex_latex = r#"
\documentclass{article}
\usepackage{amsmath}
\title{Complex Document}
\author{Test Author}
\date{2024}

\begin{document}
\maketitle

\begin{abstract}
This is a complex document with multiple features.
\end{abstract}

\section{Introduction}
This document contains \textbf{bold}, \textit{italic}, and \texttt{monospace} text.

\subsection{Mathematics}
Inline math: $E = mc^2$

Display math:
$$\sum_{i=1}^n i = \frac{n(n+1)}{2}$$

\subsection{Lists}
\begin{itemize}
\item First item with \emph{emphasis}
\item Second item
\begin{enumerate}
\item Nested numbered item
\item Another nested item
\end{enumerate}
\end{itemize}

\section{Environments}
\begin{quote}
This is a quoted passage that demonstrates the quote environment.
\end{quote}

\begin{center}
Centered text for emphasis.
\end{center}

\section{Conclusion}
This concludes our complex document example.

\end{document}
"#;
    
    let result = processor.to_html(complex_latex);
    assert!(result.is_ok());
    
    let html = result.unwrap();
    
    // Verify document structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<title>Complex Document</title>"));
    assert!(html.contains("MathJax"));
    
    // Verify content
    assert!(html.contains("<h1 class=\"document-title\">Complex Document</h1>"));
    assert!(html.contains("<div class=\"abstract\">"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<code>monospace</code>"));
    assert!(html.contains("$E = mc^2$"));
    assert!(html.contains("$$\\sum_{i=1}^n i = \\frac{n(n+1)}{2}$$"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<ol>"));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<div class=\"center\">"));
}