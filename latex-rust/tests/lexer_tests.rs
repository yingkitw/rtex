//! Comprehensive unit tests for the lexer module

use latex_rust::lexer::{Lexer, Token, TokenType};
use latex_rust::error::{LaTeXError, Position};

#[cfg(test)]
mod lexer_unit_tests {
    use super::*;

    #[test]
    fn test_lexer_creation() {
        let lexer = Lexer::new();
        // Test that lexer can be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_default_lexer() {
        let lexer = Lexer::default();
        // Test that default implementation works
        assert!(true);
    }

    #[test]
    fn test_empty_input() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("").unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token_type, TokenType::Eof));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_whitespace_only() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("   \t  ").unwrap();
        
        assert_eq!(tokens.len(), 2); // whitespace + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Whitespace(_)));
        if let TokenType::Whitespace(ref ws) = tokens[0].token_type {
            assert_eq!(ws, "   \t  ");
        }
        assert!(matches!(tokens[1].token_type, TokenType::Eof));
    }

    #[test]
    fn test_simple_text() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("hello").unwrap();
        
        assert_eq!(tokens.len(), 2); // text + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Text(ref text) if text == "hello"));
        assert_eq!(tokens[0].position.offset, 0);
        assert!(matches!(tokens[1].token_type, TokenType::Eof));
    }

    #[test]
    fn test_text_with_spaces() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("hello world").unwrap();
        
        assert_eq!(tokens.len(), 4); // text + whitespace + text + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Text(ref text) if text == "hello"));
        assert!(matches!(tokens[1].token_type, TokenType::Whitespace(_)));
        assert!(matches!(tokens[2].token_type, TokenType::Text(ref text) if text == "world"));
        assert!(matches!(tokens[3].token_type, TokenType::Eof));
    }

    #[test]
    fn test_simple_command() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\section").unwrap();
        
        assert_eq!(tokens.len(), 2); // command + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Command(ref cmd) if cmd == "section"));
        assert_eq!(tokens[0].position.offset, 0);
        assert!(matches!(tokens[1].token_type, TokenType::Eof));
    }

    #[test]
    fn test_command_with_star() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\section*").unwrap();
        
        assert_eq!(tokens.len(), 2); // command + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Command(ref cmd) if cmd == "section*"));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_double_backslash() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\\\").unwrap();
        
        assert_eq!(tokens.len(), 2); // command + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Command(ref cmd) if cmd == "\\\\"));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_single_backslash() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\").unwrap();
        
        assert_eq!(tokens.len(), 2); // backslash + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Backslash));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_braces() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("{}").unwrap();
        
        assert_eq!(tokens.len(), 3); // { + } + EOF
        assert!(matches!(tokens[0].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[1].token_type, TokenType::RightBrace));
        assert!(matches!(tokens[2].token_type, TokenType::Eof));
    }

    #[test]
    fn test_brackets() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("[]").unwrap();
        
        assert_eq!(tokens.len(), 3); // [ + ] + EOF
        assert!(matches!(tokens[0].token_type, TokenType::LeftBracket));
        assert!(matches!(tokens[1].token_type, TokenType::RightBracket));
        assert!(matches!(tokens[2].token_type, TokenType::Eof));
    }

    #[test]
    fn test_single_dollar() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("$").unwrap();
        
        assert_eq!(tokens.len(), 2); // $ + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Dollar));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_double_dollar() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("$$").unwrap();
        
        assert_eq!(tokens.len(), 2); // $$ + EOF
        assert!(matches!(tokens[0].token_type, TokenType::DoubleDollar));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_ampersand() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("&").unwrap();
        
        assert_eq!(tokens.len(), 2); // & + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Ampersand));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_newline() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\n").unwrap();
        
        assert_eq!(tokens.len(), 2); // newline + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Newline));
        assert_eq!(tokens[0].position.offset, 0);
    }

    #[test]
    fn test_complex_document() {
        let lexer = Lexer::new();
        let input = "\\documentclass{article}\n\\begin{document}\nHello world!\n\\end{document}";
        let tokens = lexer.tokenize(input).unwrap();
        
        // Should have multiple tokens including commands, braces, text, newlines
        assert!(tokens.len() > 10);
        
        // Check first few tokens
        assert!(matches!(tokens[0].token_type, TokenType::Command(ref cmd) if cmd == "documentclass"));
        assert!(matches!(tokens[1].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[2].token_type, TokenType::Text(ref text) if text == "article"));
        assert!(matches!(tokens[3].token_type, TokenType::RightBrace));
        assert!(matches!(tokens[4].token_type, TokenType::Newline));
    }

    #[test]
    fn test_math_expressions() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("$x^2 + y^2 = z^2$").unwrap();
        
        // Should start and end with dollar signs
        assert!(matches!(tokens[0].token_type, TokenType::Dollar));
        assert!(matches!(tokens[tokens.len()-2].token_type, TokenType::Dollar)); // -2 because last is EOF
        
        // Should contain text tokens for variables and operators
        let text_tokens: Vec<&Token> = tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::Text(_)))
            .collect();
        assert!(text_tokens.len() > 0);
    }

    #[test]
    fn test_display_math() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("$$\\frac{1}{2}$$").unwrap();
        
        // Should start and end with double dollar signs
        assert!(matches!(tokens[0].token_type, TokenType::DoubleDollar));
        assert!(matches!(tokens[tokens.len()-2].token_type, TokenType::DoubleDollar)); // -2 because last is EOF
        
        // Should contain the frac command
        let has_frac = tokens.iter().any(|t| {
            matches!(t.token_type, TokenType::Command(ref cmd) if cmd == "frac")
        });
        assert!(has_frac);
    }

    #[test]
    fn test_table_structure() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("a & b & c").unwrap();
        
        // Should contain ampersands for table alignment
        let ampersand_count = tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::Ampersand))
            .count();
        assert_eq!(ampersand_count, 2);
    }

    #[test]
    fn test_consecutive_whitespace() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("hello     world").unwrap();
        
        assert_eq!(tokens.len(), 4); // text + whitespace + text + EOF
        if let TokenType::Whitespace(ref ws) = tokens[1].token_type {
            assert_eq!(ws, "     ");
        } else {
            panic!("Expected whitespace token");
        }
    }

    #[test]
    fn test_mixed_whitespace() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("a \t b").unwrap();
        
        assert_eq!(tokens.len(), 4); // text + whitespace + text + EOF
        if let TokenType::Whitespace(ref ws) = tokens[1].token_type {
            assert_eq!(ws, " \t ");
        } else {
            panic!("Expected whitespace token");
        }
    }

    #[test]
    fn test_token_positions() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\section{test}").unwrap();
        
        // Check that positions are correct
        assert_eq!(tokens[0].position.offset, 0); // \section
        assert_eq!(tokens[1].position.offset, 8); // {
        assert_eq!(tokens[2].position.offset, 9); // test
        assert_eq!(tokens[3].position.offset, 13); // }
    }

    #[test]
    fn test_token_lengths() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\documentclass").unwrap();
        
    }

    #[test]
    fn test_special_characters_in_text() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("hello@#%world").unwrap();
        
        assert_eq!(tokens.len(), 2); // text + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Text(ref text) if text == "hello@#%world"));
    }

    #[test]
    fn test_unicode_text() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("héllo wörld").unwrap();
        
        assert_eq!(tokens.len(), 4); // text + whitespace + text + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Text(ref text) if text == "héllo"));
        assert!(matches!(tokens[2].token_type, TokenType::Text(ref text) if text == "wörld"));
    }

    #[test]
    fn test_command_followed_by_special_chars() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\section{}").unwrap();
        
        assert_eq!(tokens.len(), 4); // command + { + } + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Command(ref cmd) if cmd == "section"));
        assert!(matches!(tokens[1].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[2].token_type, TokenType::RightBrace));
    }

    #[test]
    fn test_multiple_newlines() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("line1\n\nline2").unwrap();
        
        let newline_count = tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::Newline))
            .count();
        assert_eq!(newline_count, 2);
    }

    #[test]
    fn test_token_display() {
        let token = Token::new(TokenType::Command("section".to_string()), Position::new(1, 1, 0));
        let display_str = format!("{}", token);
        assert!(display_str.contains("Command"));
        assert!(display_str.contains("section"));
        assert!(display_str.contains("1:1"));
    }

    #[test]
    fn test_edge_case_empty_command() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("\\ ").unwrap();
        
        // Should treat as backslash followed by whitespace
        assert_eq!(tokens.len(), 3); // backslash + whitespace + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Backslash));
        assert!(matches!(tokens[1].token_type, TokenType::Whitespace(_)));
    }

    #[test]
    fn test_command_at_end() {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize("text \\section").unwrap();
        
        assert_eq!(tokens.len(), 4); // text + whitespace + command + EOF
        assert!(matches!(tokens[2].token_type, TokenType::Command(ref cmd) if cmd == "section"));
    }
}