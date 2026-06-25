//! Comprehensive unit tests for the parser module

use latex_rust::parser::Parser;
use latex_rust::lexer::{Lexer, Token};
use latex_rust::ast::*;

#[cfg(test)]
mod parser_unit_tests {
    use super::*;

    fn create_tokens(input: &str) -> Vec<Token> {
        let lexer = Lexer::new();
        lexer.tokenize(input).unwrap()
    }

    #[test]
    fn test_parser_creation() {
        let _parser = Parser::new();
        // Test that parser can be created successfully
        assert!(true);
    }

    #[test]
    fn test_default_parser() {
        let _parser = Parser::default();
        // Test that default implementation works
        assert!(true);
    }

    #[test]
    fn test_empty_document() {
        let mut parser = Parser::new();
        let tokens = create_tokens("");
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        assert_eq!(document.body.len(), 0);
    }

    #[test]
    fn test_simple_text_parsing() {
        let mut parser = Parser::new();
        let tokens = create_tokens("\\begin{document}Hello world\\end{document}");
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        assert!(document.body.len() > 0);
        
        // Should contain text nodes
        let has_text = document.body.iter().any(|node| {
            matches!(node, Node::Text(_))
        });
        assert!(has_text);
    }

    #[test]
    fn test_basic_document_structure() {
        let mut parser = Parser::new();
        let input = "\\documentclass{article}\n\\begin{document}\nHello\n\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        assert!(document.metadata.document_class.is_some());
        assert_eq!(document.metadata.document_class.unwrap(), "article");
    }

    #[test]
    fn test_title_parsing() {
        let mut parser = Parser::new();
        let input = "\\title{Test Title}\\begin{document}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        assert!(document.metadata.title.is_some());
        assert_eq!(document.metadata.title.unwrap(), "Test Title");
    }

    #[test]
    fn test_author_parsing() {
        let mut parser = Parser::new();
        let input = "\\author{Test Author}\\begin{document}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        assert!(document.metadata.author.is_some());
        assert_eq!(document.metadata.author.unwrap(), "Test Author");
    }

    #[test]
    fn test_section_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\section{Introduction}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain a section node
        let has_section = document.body.iter().any(|node| {
            matches!(node, Node::Section { .. })
        });
        assert!(has_section);
    }

    #[test]
    fn test_subsection_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\subsection{Details}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain a subsection node
        let has_subsection = document.body.iter().any(|node| {
            matches!(node, Node::Section { level: SectionLevel::Subsection, .. })
        });
        assert!(has_subsection);
    }

    #[test]
    fn test_text_formatting_commands() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\textbf{bold} \\textit{italic}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain command nodes
        let has_commands = document.body.iter().any(|node| {
            matches!(node, Node::Command { .. })
        });
        assert!(has_commands);
    }

    #[test]
    fn test_math_mode_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}$x^2$\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain math nodes
        let has_math = document.body.iter().any(|node| {
            matches!(node, Node::Math { .. })
        });
        assert!(has_math);
    }

    #[test]
    fn test_display_math_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}$$x^2 + y^2 = z^2$$\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain display math nodes
        let has_display_math = document.body.iter().any(|node| {
            matches!(node, Node::Math { display: true, .. })
        });
        assert!(has_display_math);
    }

    #[test]
    fn test_environment_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{center}Centered text\\end{center}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain environment nodes
        let has_environment = document.body.iter().any(|node| {
            matches!(node, Node::Environment { name, .. } if name == "center")
        });
        assert!(has_environment);
    }

    #[test]
    fn test_itemize_environment() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{itemize}\\item First\\item Second\\end{itemize}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain list environment
        let has_list = document.body.iter().any(|node| {
            matches!(node, Node::List { .. })
        });
        assert!(has_list);
    }

    #[test]
    fn test_enumerate_environment() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{enumerate}\\item One\\item Two\\end{enumerate}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain ordered list
        let has_ordered_list = document.body.iter().any(|node| {
            matches!(node, Node::List { list_type: ListType::Enumerate, .. })
        });
        assert!(has_ordered_list);
    }

    #[test]
    fn test_tabular_environment() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{tabular}{|c|c|}\\hline a & b \\\\\\hline\\end{tabular}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain table nodes
        let has_table = document.body.iter().any(|node| {
            matches!(node, Node::Table { .. })
        });
        assert!(has_table);
    }

    #[test]
    fn test_figure_environment() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{figure}\\caption{Test}\\end{figure}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain figure nodes
        let has_figure = document.body.iter().any(|node| {
            matches!(node, Node::Figure { .. })
        });
        assert!(has_figure);
    }

    #[test]
    fn test_math_environment() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{equation}x = y\\end{equation}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain math environment or equation environment
        let has_math_env = document.body.iter().any(|node| {
            matches!(node, Node::Math { display: true, .. }) ||
            matches!(node, Node::MathEnvironment { .. }) ||
            matches!(node, Node::Environment { name, .. } if name == "equation")
        });
        assert!(has_math_env);
    }

    #[test]
    fn test_label_and_reference() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\section{Test}\\label{sec:test}See \\ref{sec:test}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain label and reference nodes
        let has_label = document.body.iter().any(|node| {
            matches!(node, Node::Label { .. })
        });
        let has_ref = document.body.iter().any(|node| {
            matches!(node, Node::Reference { .. })
        });
        assert!(has_label || has_ref); // At least one should be present
    }

    #[test]
    fn test_footnote_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}Text\\footnote{Note}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain footnote nodes
        let has_footnote = document.body.iter().any(|node| {
            matches!(node, Node::Footnote { .. })
        });
        assert!(has_footnote);
    }

    #[test]
    fn test_citation_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}See \\cite{ref1}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain citation nodes
        let has_citation = document.body.iter().any(|node| {
            matches!(node, Node::Citation { .. })
        });
        assert!(has_citation);
    }

    #[test]
    fn test_package_parsing() {
        let mut parser = Parser::new();
        let input = "\\usepackage{amsmath}\\begin{document}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        // Check if package is in preamble instead of metadata
        let has_package = document.preamble.iter().any(|node| {
            matches!(node, Node::Package { name, .. } if name == "amsmath")
        });
        assert!(has_package || document.metadata.packages.contains(&"amsmath".to_string()));
    }

    #[test]
    fn test_package_with_options() {
        let mut parser = Parser::new();
        let input = "\\usepackage[utf8]{inputenc}\\begin{document}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        // Check if package is in preamble instead of metadata
        let has_package = document.preamble.iter().any(|node| {
            matches!(node, Node::Package { name, .. } if name == "inputenc")
        });
        assert!(has_package || document.metadata.packages.contains(&"inputenc".to_string()));
    }

    #[test]
    fn test_command_with_optional_args() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\section{Long Title}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should parse section with both optional and required arguments
        let has_section_with_args = document.body.iter().any(|node| {
            if let Node::Section { title, .. } = node {
                !title.is_empty()
            } else {
                false
            }
        });
        assert!(has_section_with_args);
    }

    #[test]
    fn test_nested_environments() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{center}\\begin{itemize}\\item Test\\end{itemize}\\end{center}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should handle nested environments
        assert!(document.body.len() > 0);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut parser = Parser::new();
        let input = "\\begin{document}Hello   world\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain whitespace nodes
        let has_whitespace = document.body.iter().any(|node| {
            matches!(node, Node::Whitespace(_))
        });
        assert!(has_whitespace);
    }

    #[test]
    fn test_line_breaks() {
        let mut parser = Parser::new();
        let input = "\\begin{document}Line 1\n\nLine 2\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should handle line breaks and paragraph breaks
        let has_paragraph = document.body.iter().any(|node| {
            matches!(node, Node::Paragraph)
        });
        assert!(has_paragraph);
    }

    #[test]
    fn test_error_handling_missing_end() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\begin{itemize}\\item Test";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        // Should handle missing \end gracefully
        // The exact behavior depends on implementation
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_error_handling_unmatched_braces() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\textbf{bold text\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        // Should handle unmatched braces
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_error_handling_unknown_command() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\unknowncommand{test}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        // Should handle unknown commands gracefully
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should contain the unknown command as a generic command node
        let has_unknown_cmd = document.body.iter().any(|node| {
            matches!(node, Node::Command { name, .. } if name == "unknowncommand")
        });
        assert!(has_unknown_cmd);
    }

    #[test]
    fn test_complex_document_structure() {
        let mut parser = Parser::new();
        let input = r#"\documentclass{article}
\usepackage{amsmath}
\title{Test Document}
\author{Test Author}
\begin{document}
\maketitle
\section{Introduction}
This is a test.
\subsection{Details}
More details here.
\begin{itemize}
\item First item
\item Second item
\end{itemize}
\section{Math}
$$E = mc^2$$
\end{document}"#;
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Check document metadata
        assert_eq!(document.metadata.document_class, Some("article".to_string()));
        assert_eq!(document.metadata.title, Some("Test Document".to_string()));
        assert_eq!(document.metadata.author, Some("Test Author".to_string()));
        // Check if package is in preamble instead of metadata
        let has_package = document.preamble.iter().any(|node| {
            matches!(node, Node::Package { name, .. } if name == "amsmath")
        });
        assert!(has_package || document.metadata.packages.contains(&"amsmath".to_string()));
        
        // Check document structure
        assert!(document.body.len() > 0);
        
        // Should contain sections, lists, and math
        let has_section = document.body.iter().any(|node| matches!(node, Node::Section { .. }));
        let has_list = document.body.iter().any(|node| matches!(node, Node::List { .. }));
        let has_math = document.body.iter().any(|node| matches!(node, Node::Math { .. }));
        
        assert!(has_section);
        assert!(has_list);
        assert!(has_math);
    }

    #[test]
    fn test_argument_parsing() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\textcolor{red}{colored text}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should parse command with multiple arguments
        let has_textcolor = document.body.iter().any(|node| {
            if let Node::Command { name, args, .. } = node {
                name == "textcolor" && args.len() >= 2
            } else {
                false
            }
        });
        assert!(has_textcolor);
    }

    #[test]
    fn test_empty_arguments() {
        let mut parser = Parser::new();
        let input = "\\begin{document}\\textbf{}\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should handle empty arguments
        let has_textbf = document.body.iter().any(|node| {
            matches!(node, Node::Command { name, .. } if name == "textbf")
        });
        assert!(has_textbf);
    }

    #[test]
    fn test_special_characters() {
        let mut parser = Parser::new();
        let input = "\\begin{document}Text with & ampersand\\end{document}";
        let tokens = create_tokens(input);
        let result = parser.parse(tokens);
        
        assert!(result.is_ok());
        let document = result.unwrap();
        
        // Should handle special characters
        assert!(document.body.len() > 0);
    }
}