//! LaTeX parser - converts raw LaTeX source into structured elements.
//!
//! Handles document structure, environments, commands, inline math,
//! display math, lists, and metadata extraction.

/// A structured element parsed from a LaTeX document.
#[derive(Debug, Clone)]
pub enum TexElement {
    /// Plain text content.
    Text(String),
    /// A LaTeX command such as `\title{...}`.
    Command { name: String, args: Vec<String> },
    /// A section or subsection heading.
    Section { level: usize, title: String },
    /// A paragraph break (`\n\n`).
    Paragraph,
    /// Inline math delimited by `$...$`.
    MathInline(String),
    /// Display math from `\begin{equation}` or `$$...$$`.
    MathDisplay(String),
    /// A list (`itemize` or `enumerate`).
    ItemList { ordered: bool, items: Vec<Vec<TexElement>> },
    /// A code block from `lstlisting`.
    CodeBlock(String),
}

/// Stateful parser for a single LaTeX document.
pub struct TexParser {
    content: String,
    position: usize,
}

impl TexParser {
    /// Create a new parser for the given LaTeX source.
    pub fn new(content: String) -> Self {
        Self { content, position: 0 }
    }

    /// Parse the entire document into a sequence of elements.
    pub fn parse(&mut self) -> Vec<TexElement> {
        let mut elements = Vec::new();
        
        // Extract metadata from preamble before skipping
        elements.extend(self.extract_preamble_metadata());
        
        self.skip_preamble();
        
        while self.position < self.content.len() {
            let start_position = self.position;
            if let Some(element) = self.parse_next() {
                elements.push(element);
            }

            if self.position == start_position {
                if let Some(ch) = self.content[self.position..].chars().next() {
                    self.position += ch.len_utf8();
                } else {
                    break;
                }
            }
        }
        
        elements
    }

    fn extract_preamble_metadata(&mut self) -> Vec<TexElement> {
        let mut metadata = Vec::new();
        let original_position = self.position;
        
        if let Some(begin_doc) = self.content.find("\\begin{document}") {
            let preamble = &self.content[..begin_doc];
            
            // Extract all metadata commands using a helper
            for (cmd, offset) in &[("title", 7), ("author", 8), ("date", 6)] {
                if let Some(element) = self.extract_metadata_command(preamble, cmd, *offset) {
                    metadata.push(element);
                }
            }
        }
        
        self.position = original_position;
        metadata
    }

    fn extract_metadata_command(&self, preamble: &str, cmd: &str, offset: usize) -> Option<TexElement> {
        let search_str = format!("\\{}{{", cmd);
        if let Some(start) = preamble.find(&search_str) {
            let after_cmd = &preamble[start + offset..];
            if let Some(content) = crate::utils::extract_braced_inner(after_cmd) {
                return Some(TexElement::Command {
                    name: cmd.to_string(),
                    args: vec![content],
                });
            }
        }
        None
    }

    fn skip_preamble(&mut self) {
        if let Some(begin_doc) = self.content.find("\\begin{document}") {
            self.position = begin_doc + "\\begin{document}".len();
        }
    }

    fn parse_next(&mut self) -> Option<TexElement> {
        self.skip_whitespace_and_comments();
        
        if self.position >= self.content.len() {
            return None;
        }

        let remaining = &self.content[self.position..];

        if remaining.starts_with("\\end{document}") {
            self.position = self.content.len();
            return None;
        }

        if remaining.starts_with("\\section") {
            return self.parse_section(1);
        }
        
        if remaining.starts_with("\\subsection") {
            return self.parse_section(2);
        }

        if remaining.starts_with("\\title") {
            return self.parse_command("title");
        }

        if remaining.starts_with("\\author") {
            return self.parse_command("author");
        }

        if remaining.starts_with("\\date") {
            return self.parse_command("date");
        }

        if remaining.starts_with("\\maketitle") {
            self.position += "\\maketitle".len();
            return Some(TexElement::Command {
                name: "maketitle".to_string(),
                args: vec![],
            });
        }

        if remaining.starts_with("\\begin{") {
            return self.parse_environment();
        }

        if remaining.starts_with("\\texttt{") || remaining.starts_with("\\textbf{") || remaining.starts_with("\\textit{") {
            return self.parse_text_command();
        }

        if remaining.starts_with('$') {
            return self.parse_math();
        }

        if remaining.starts_with('\\') {
            self.parse_unknown_command();
            return None;
        }

        if remaining.starts_with("\n\n") {
            self.position += 2;
            return Some(TexElement::Paragraph);
        }

        self.parse_text()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            
            if remaining.starts_with('%') {
                if let Some(newline) = remaining.find('\n') {
                    self.position += newline + 1;
                } else {
                    self.position = self.content.len();
                }
            } else if remaining.starts_with(|c: char| c.is_whitespace() && c != '\n') {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn parse_section(&mut self, level: u8) -> Option<TexElement> {
        let cmd = if level == 1 { "\\section" } else { "\\subsection" };
        self.position += cmd.len();
        
        self.skip_whitespace_and_comments();
        
        self.parse_braced_content().map(|title| TexElement::Section { level: level as usize, title })
    }

    fn parse_command(&mut self, name: &str) -> Option<TexElement> {
        self.position += name.len() + 1;
        
        self.skip_whitespace_and_comments();
        
        self.parse_braced_content().map(|arg| TexElement::Command {
                name: name.to_string(),
                args: vec![arg],
            })
    }

    fn parse_text_command(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        
        let (_cmd_name, cmd_len) = if remaining.starts_with("\\texttt{") {
            ("texttt", 8)
        } else if remaining.starts_with("\\textbf{") {
            ("textbf", 8)
        } else if remaining.starts_with("\\textit{") {
            ("textit", 8)
        } else {
            return None;
        };

        self.position += cmd_len;
        
        self.parse_braced_content().map(TexElement::Text)
    }

    fn parse_environment(&mut self) -> Option<TexElement> {
        self.position += "\\begin{".len();

        let env_name = self.read_until('}');
        self.position += 1;

        match env_name.as_str() {
            "itemize" => self.parse_itemize(false),
            "enumerate" => self.parse_itemize(true),
            "equation" => self.parse_equation(),
            "lstlisting" => self.parse_lstlisting(),
            "tabular" => self.parse_tabular(),
            "table" => self.parse_table(),
            _ => {
                self.skip_until(&format!("\\end{{{}}}", env_name));
                None
            }
        }
    }

    fn parse_itemize(&mut self, ordered: bool) -> Option<TexElement> {
        let mut items = Vec::new();
        let end_marker = if ordered { "\\end{enumerate}" } else { "\\end{itemize}" };

        while self.position < self.content.len() {
            self.skip_whitespace_and_comments();
            
            let remaining = &self.content[self.position..];
            
            if remaining.starts_with(end_marker) {
                self.position += end_marker.len();
                break;
            }

            if remaining.starts_with("\\item") {
                self.position += "\\item".len();
                self.skip_whitespace_and_comments();
                
                let mut item_content = Vec::new();
                
                while self.position < self.content.len() {
                    let remaining = &self.content[self.position..];
                    
                    if remaining.starts_with("\\item") || remaining.starts_with(end_marker) {
                        break;
                    }
                    
                    if let Some(elem) = self.parse_next() {
                        item_content.push(elem);
                    } else {
                        break;
                    }
                }
                
                items.push(item_content);
            } else {
                self.position += 1;
            }
        }

        Some(TexElement::ItemList { ordered, items })
    }

    fn parse_equation(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{equation}");
        self.position += "\\end{equation}".len();

        Some(TexElement::MathDisplay(content.trim().to_string()))
    }

    fn parse_lstlisting(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{lstlisting}");
        self.position += "\\end{lstlisting}".len();

        Some(TexElement::CodeBlock(content.trim().to_string()))
    }

    fn parse_tabular(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{tabular}");
        self.position += "\\end{tabular}".len();

        Some(TexElement::Text(format!("\n{}\n", self.format_table_content(&content))))
    }

    fn parse_table(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{table}");
        self.position += "\\end{table}".len();
        
        // Look for tabular environment within table
        if let Some(tabular_start) = content.find("\\begin{tabular}") {
            let tabular_content = &content[tabular_start..];
            if let Some(tabular_end) = tabular_content.find("\\end{tabular}") {
                let tabular_only = &tabular_content[..tabular_end + "\\end{tabular}".len()];
                return Some(TexElement::Text(format!("\n{}\n", self.format_table_content(tabular_only))));
            }
        }
        
        Some(TexElement::Text(String::new()))
    }

    fn format_table_content(&self, content: &str) -> String {
        let mut table_text = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Handle table rules
            if line.starts_with("\\toprule") || line.starts_with("\\midrule") || line.starts_with("\\bottomrule") {
                table_text.push_str(&"─".repeat(30));
                table_text.push('\n');
                continue;
            }
            
            // Handle table rows
            if line.contains('&') && !line.starts_with('\\') {
                let cells: Vec<&str> = line.split('&').collect();
                for (i, cell) in cells.iter().enumerate() {
                    let cell = cell.trim()
                        .trim_end_matches("\\\\")
                        .replace("\\$", "$");
                    table_text.push_str(&cell);
                    if i < cells.len() - 1 {
                        table_text.push_str(" | ");
                    }
                }
                table_text.push('\n');
            }
        }
        
        table_text.trim().to_string()
    }

    fn parse_math(&mut self) -> Option<TexElement> {
        self.position += 1;
        
        let remaining = &self.content[self.position..];
        
        if remaining.starts_with('$') {
            self.position += 1;
            let content = self.read_until_str("$$");
            self.position += 2;
            Some(TexElement::MathDisplay(content))
        } else {
            let content = self.read_until('$');
            self.position += 1;
            Some(TexElement::MathInline(content))
        }
    }

    fn parse_text(&mut self) -> Option<TexElement> {
        let mut text = String::new();
        
        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            
            if remaining.starts_with('\\') || remaining.starts_with("\n\n") || remaining.starts_with('}') {
                break;
            }
            
            if remaining.starts_with('$') {
                self.position += 1;
                
                if self.position < self.content.len() && self.content[self.position..].starts_with('$') {
                    self.position -= 1;
                    break;
                }
                
                let math_content = self.read_until('$');
                self.position += 1;
                
                // Keep $ delimiters so PDF builder can format the math
                text.push_str(&format!("${}$", math_content));
            } else if let Some(ch) = remaining.chars().next() {
                text.push(ch);
                self.position += ch.len_utf8();
            }
        }
        
        if text.is_empty() {
            None
        } else {
            Some(TexElement::Text(text.trim().to_string()))
        }
    }

    fn parse_unknown_command(&mut self) {
        if self.position >= self.content.len() {
            return;
        }

        self.position += 1;

        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            let Some(ch) = remaining.chars().next() else {
                break;
            };

            if ch.is_alphabetic() || ch == '*' {
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }

        self.skip_whitespace_and_comments();
    }

    fn parse_braced_content(&mut self) -> Option<String> {
        if self.position >= self.content.len() {
            return None;
        }

        let remaining = &self.content[self.position..];
        
        if !remaining.starts_with('{') {
            return None;
        }

        self.position += 1;
        let content = self.read_until('}');
        self.position += 1;
        
        Some(content)
    }

    fn read_until(&mut self, delimiter: char) -> String {
        let start = self.position;
        let mut depth = 0;
        
        while self.position < self.content.len() {
            let ch = self.content[self.position..].chars().next().unwrap();
            
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                if depth == 0 && delimiter == '}' {
                    break;
                }
                depth -= 1;
            } else if ch == delimiter && depth == 0 {
                break;
            }
            
            self.position += ch.len_utf8();
        }
        
        self.content[start..self.position].to_string()
    }

    fn read_until_str(&mut self, delimiter: &str) -> String {
        let start = self.position;
        
        while self.position < self.content.len() {
            if self.content[self.position..].starts_with(delimiter) {
                break;
            }
            self.position += 1;
        }
        
        self.content[start..self.position].to_string()
    }

    fn skip_until(&mut self, marker: &str) {
        if let Some(pos) = self.content[self.position..].find(marker) {
            self.position += pos + marker.len();
        } else {
            self.position = self.content.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TexElement, TexParser};

    #[test]
    fn parser_skips_unknown_command_and_continues() {
        let content = r#"\documentclass{article}
\begin{document}
\undefined_command
Visible text
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Text(text) if text.contains("Visible text"))
        }));
    }

    #[test]
    fn parser_unknown_command_with_argument_does_not_hang() {
        let content = r#"\documentclass{article}
\begin{document}
\undefined{ignored}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.is_empty() || elements.iter().all(|e| !matches!(e, TexElement::Command { .. })));
    }

    #[test]
    fn parser_parses_lstlisting_environment() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{lstlisting}[language=Rust]
fn main() {
    println!("Hello, World!");
}
\end{lstlisting}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::CodeBlock(code) if code.contains("fn main") && code.contains("println"))
        }));
    }

    #[test]
    fn parser_parses_sections() {
        let content = r#"\documentclass{article}
\begin{document}
\section{Introduction}
Some intro text.
\subsection{Background}
Background info.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Section { level: 1, title } if title == "Introduction")
        }));
        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Section { level: 2, title } if title == "Background")
        }));
    }

    #[test]
    fn parser_parses_inline_math_in_text() {
        let content = r#"\documentclass{article}
\begin{document}
The equation $E = mc^2$ is famous.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        // Inline math is kept inside Text elements with $ delimiters
        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Text(text) if text.contains("$E = mc^2$"))
        }));
    }

    #[test]
    fn parser_parses_display_math() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{equation}
a^2 + b^2 = c^2
\end{equation}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::MathDisplay(text) if text.contains("a^2 + b^2 = c^2"))
        }));
    }

    #[test]
    fn parser_parses_itemize() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{itemize}
\item First bullet
\item Second bullet
\end{itemize}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::ItemList { ordered: false, items } if items.len() == 2)
        }));
    }

    #[test]
    fn parser_parses_enumerate() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{enumerate}
\item Step one
\item Step two
\end{enumerate}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::ItemList { ordered: true, items } if items.len() == 2)
        }));
    }

    #[test]
    fn parser_extracts_metadata() {
        let content = r#"\documentclass{article}
\title{Test Doc}
\author{Jane Doe}
\date{2024-01-01}
\begin{document}
\maketitle
Body text.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Command { name, args } if name == "title" && args[0] == "Test Doc")
        }));
        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Command { name, args } if name == "author" && args[0] == "Jane Doe")
        }));
    }

    #[test]
    fn parser_parses_text_formatting() {
        let content = r#"\documentclass{article}
\begin{document}
\textbf{bold} and \textit{italic} and \texttt{mono}.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let text_elements: Vec<_> = elements.iter().filter_map(|e| {
            if let TexElement::Text(t) = e { Some(t.clone()) } else { None }
        }).collect();

        let combined = text_elements.join(" ");
        assert!(combined.contains("bold"));
        assert!(combined.contains("italic"));
        assert!(combined.contains("mono"));
    }
}
