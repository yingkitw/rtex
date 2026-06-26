//! TeX token system.
//!
//! TeX reads characters into *tokens* before macro expansion and
//! execution.  A token is either a character token (carrying a
//! category code) or a control-sequence token.

use super::catcodes::{CatCode, CatCodeTable};

/// A token produced by the TeX lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A character together with its current category code.
    Char { ch: char, cat: CatCode },
    /// A control sequence (e.g. `\section`, `\` followed by a single
    /// non-letter, or `\` followed by a run of letters).
    ControlSequence(String),
    /// End-of-file sentinel.
    EndOfFile,
}

/// A minimal TeX lexer that converts raw text into a sequence of
/// tokens according to a `CatCodeTable`.
#[derive(Debug, Clone)]
pub struct TexLexer<'a> {
    input: &'a str,
    position: usize,
    catcodes: CatCodeTable,
}

impl<'a> TexLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            catcodes: CatCodeTable::new(),
        }
    }

    pub fn with_catcodes(input: &'a str, catcodes: CatCodeTable) -> Self {
        Self {
            input,
            position: 0,
            catcodes,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input[self.position..].chars().next()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Skip everything from a `%` comment to the end of the line,
    /// consuming the line terminator as well.
    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                self.advance(); // consume the EOL
                break;
            }
            self.advance();
        }
    }

    /// Read a control sequence name after the backslash.
    fn read_cs_name(&mut self) -> String {
        match self.peek() {
            // Single non-letter after backslash → one-char csname.
            Some(ch) if self.catcodes.get(ch) != CatCode::Letter => {
                self.advance();
                ch.to_string()
            }
            // Letter(s) after backslash → run of letters.
            Some(_) => {
                let mut name = String::new();
                while let Some(ch) = self.peek() {
                    if self.catcodes.get(ch) == CatCode::Letter {
                        name.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                name
            }
            None => String::new(),
        }
    }

    /// Return the next token, or `None` when the input is exhausted.
    pub fn next_token(&mut self) -> Option<Token> {
        // Skip spaces and comments, normalising end-of-line.
        loop {
            match self.peek() {
                None => return None,
                Some(ch) => {
                    let cat = self.catcodes.get(ch);
                    match cat {
                        CatCode::Comment => {
                            self.advance(); // consume '%'
                            self.skip_comment();
                        }
                        CatCode::Space => {
                            self.advance();
                            // Collapse consecutive spaces into one.
                            while let Some(c) = self.peek() {
                                if self.catcodes.get(c) == CatCode::Space {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            return Some(Token::Char { ch: ' ', cat: CatCode::Space });
                        }
                        CatCode::EndOfLine => {
                            self.advance();
                            // Collapse consecutive EOLs into one space.
                            while let Some(c) = self.peek() {
                                if self.catcodes.get(c) == CatCode::EndOfLine {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            return Some(Token::Char { ch: ' ', cat: CatCode::Space });
                        }
                        _ => break,
                    }
                }
            }
        }

        let ch = self.advance()?;
        let cat = self.catcodes.get(ch);

        if cat == CatCode::Escape {
            let name = self.read_cs_name();
            Some(Token::ControlSequence(name))
        } else {
            Some(Token::Char { ch, cat })
        }
    }

    /// Collect all remaining tokens into a `Vec`.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(tok) = self.next_token() {
            tokens.push(tok);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_text() {
        let mut lexer = TexLexer::new("Hello world");
        let toks = lexer.tokenize();
        // In TeX every character is its own token.
        assert_eq!(toks.len(), 11);
        assert_eq!(toks[0], Token::Char { ch: 'H', cat: CatCode::Letter });
        assert_eq!(toks[5], Token::Char { ch: ' ', cat: CatCode::Space });
        assert_eq!(toks[6], Token::Char { ch: 'w', cat: CatCode::Letter });
    }

    #[test]
    fn tokenize_control_sequence() {
        let mut lexer = TexLexer::new("\\section{Intro}");
        let toks = lexer.tokenize();
        assert_eq!(toks[0], Token::ControlSequence("section".to_string()));
        assert_eq!(toks[1], Token::Char { ch: '{', cat: CatCode::BeginGroup });
    }

    #[test]
    fn tokenize_single_char_cs() {
        let mut lexer = TexLexer::new("\\$");
        let toks = lexer.tokenize();
        assert_eq!(toks[0], Token::ControlSequence("$".to_string()));
    }

    #[test]
    fn tokenize_comments_stripped() {
        let mut lexer = TexLexer::new("Hello % this is a comment\nworld");
        let toks = lexer.tokenize();
        // Space before comment is preserved; EOL consumed by skip_comment.
        // Hello (5) + space (1) + world (5) = 11 tokens
        assert_eq!(toks.len(), 11);
        assert_eq!(toks[6], Token::Char { ch: 'w', cat: CatCode::Letter });
    }

    #[test]
    fn tokenize_collapses_spaces() {
        let mut lexer = TexLexer::new("a   b");
        let toks = lexer.tokenize();
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[1], Token::Char { ch: ' ', cat: CatCode::Space });
    }

    #[test]
    fn tokenize_empty_input() {
        let mut lexer = TexLexer::new("");
        assert!(lexer.tokenize().is_empty());
    }
}
