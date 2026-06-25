use latex_rust::parser::Parser;
use latex_rust::lexer::Lexer;
use latex_rust::renderer::{HtmlRenderer, PdfRenderer};
use latex_rust::ast::{CodeStyle, Node};

fn create_tokens(input: &str) -> Vec<latex_rust::lexer::Token> {
    let lexer = Lexer::new();
    lexer.tokenize(input).unwrap()
}

#[test]
fn test_verbatim_environment_parsing() {
    let mut parser = Parser::new();
    let input = r#"\begin{document}\begin{verbatim}
fn main() {
    println!("Hello, world!");
}
\end{verbatim}\end{document}"#;
    let tokens = create_tokens(input);
    
    let document = parser.parse(tokens).expect("Failed to parse verbatim environment");
    
    // Check that we have a CodeBlock node in the document body
    assert!(document.body.len() > 0);
    match &document.body[0] {
        Node::CodeBlock { language, code, style, line_numbers, caption, label } => {
            assert_eq!(language, &None);
            assert_eq!(code, "\nfn main() {\n    println!(\"Hello, world!\");\n}\n");
            assert_eq!(style, &CodeStyle::Verbatim);
            assert_eq!(line_numbers, &false);
            assert_eq!(caption, &None);
            assert_eq!(label, &None);
        }
        _ => panic!("Expected CodeBlock node, got {:?}", document.body[0]),
    }
}

#[test]
fn test_lstlisting_environment_parsing() {
    let mut parser = Parser::new();
    let input = r#"\begin{document}\begin{lstlisting}[language=Python,numbers=left,caption=Example Code,label=lst:example]
def hello():
    print("Hello, Python!")
\end{lstlisting}\end{document}"#;
    let tokens = create_tokens(input);
    
    let document = parser.parse(tokens).expect("Failed to parse lstlisting environment");
    
    // Check that we have a CodeBlock node
    assert!(document.body.len() > 0);
    match &document.body[0] {
        Node::CodeBlock { language, code, style, line_numbers, caption, label } => {
            assert_eq!(language, &Some("Python".to_string())); // Language should be parsed from lstlisting parameters
            assert_eq!(code, "\ndef hello():\n    print(\"Hello, Python!\")\n");
            assert_eq!(style, &CodeStyle::Listings);
            assert_eq!(line_numbers, &true); // numbers=left should enable line numbers
            assert_eq!(caption, &Some("Example Code".to_string())); // Caption should be parsed
            assert_eq!(label, &Some("lst:example".to_string())); // Label should be parsed
        }
        _ => panic!("Expected CodeBlock node, got {:?}", document.body[0]),
    }
}

#[test]
fn test_minted_environment_parsing() {
    let mut parser = Parser::new();
    let input = r#"\begin{document}\begin{minted}{javascript}
function greet(name) {
    console.log(`Hello, ${name}!`);
}
\end{minted}\end{document}"#;
    let tokens = create_tokens(input);
    
    let document = parser.parse(tokens).expect("Failed to parse minted environment");
    
    // Check that we have a CodeBlock node
    assert!(document.body.len() > 0);
    match &document.body[0] {
        Node::CodeBlock { language, code, style, line_numbers, caption, label } => {
            assert_eq!(language, &Some("javascript".to_string())); // Language should be parsed from minted argument
            assert_eq!(code, "\nfunction greet(name) {\n    console.log(`Hello, {name}!`);\n}\n");
            assert_eq!(style, &CodeStyle::Minted);
            assert_eq!(line_numbers, &false);
            assert_eq!(caption, &None);
            assert_eq!(label, &None);
        }
        _ => panic!("Expected CodeBlock node, got {:?}", document.body[0]),
    }
}

#[test]
fn test_code_block_html_rendering() {
    let renderer = HtmlRenderer::new();
    
    // Test basic verbatim rendering
    let html = renderer.render_code_block(
        Some("rust"),
        "fn main() {\n    println!(\"Hello!\");\n}",
        &CodeStyle::Verbatim,
        false,
        None,
        None,
    );
    
    // Test basic verbatim rendering
    assert!(html.contains("<div class=\"code-block-container\">"));
    assert!(html.contains("fn main()"));
    assert!(html.contains("println!"));
}

#[test]
fn test_code_block_html_rendering_with_line_numbers() {
    let renderer = HtmlRenderer::new();
    
    // Test with line numbers
    let html = renderer.render_code_block(
        Some("python"),
        "def hello():\n    print('Hello!')",
        &CodeStyle::Listings,
        true,
        Some("Example Function"),
        Some("lst:hello"),
    );
    
    // Test with line numbers and caption
    assert!(html.contains("class=\"code-block-container\""));
    assert!(html.contains("<div class=\"code-caption\">Example Function</div>"));
    assert!(html.contains("def hello()"));
    assert!(html.contains("print(&#x27;Hello!&#x27;)"));
    assert!(html.contains("line-numbers"));
}

#[test]
fn test_inline_code_rendering() {
    let renderer = HtmlRenderer::new();
    
    let html = renderer.render_code_block(
        None,
        "print('hello')",
        &CodeStyle::Inline,
        false,
        None,
        None,
    );
    
    assert_eq!(html, "<div class=\"code-block-container\">\n<code class=\"code-block inline-code\">print(&#x27;hello&#x27;)</code></div>\n");
}

#[test]
fn test_code_block_pdf_rendering() {
    // This test verifies that PDF rendering doesn't crash
    // Full PDF output testing would require more complex setup
    let mut renderer = PdfRenderer::new();
    
    // Create a simple document with a code block
    let mut parser = Parser::new();
    let input = r#"\begin{document}\begin{verbatim}
fn main() {
    println!("Hello, world!");
}
\end{verbatim}\end{document}"#;
    let tokens = create_tokens(input);
    
    let document = parser.parse(tokens).expect("Failed to parse document");
    
    // This should not panic
    let result = renderer.render_to_file(&document, "/tmp/test_code_output.pdf");
    assert!(result.is_ok(), "PDF rendering failed: {:?}", result.err());
}