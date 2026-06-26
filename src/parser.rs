//! LaTeX parser - converts raw LaTeX source into structured elements.
//!
//! Handles document structure, environments, commands, inline math,
//! display math, lists, tables, and metadata extraction.

use crate::table::Table;
use crate::utils::extract_braced;
use std::path::{Path, PathBuf};

/// A structured element parsed from a LaTeX document.
#[derive(Debug, Clone, PartialEq)]
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
    /// An image inclusion (`\includegraphics`).
    Image { path: String, width: Option<String>, height: Option<String> },
    /// A table from `tabular` or `table` environment.
    Table(Table),
    /// Colored text from `\textcolor{color}{text}`.
    ColoredText { color: String, text: String },
    /// A citation command `\cite{key1,key2}`.
    Citation { keys: Vec<String> },
    /// A bibliography list from `thebibliography`.
    Bibliography { entries: Vec<BibEntry> },
    /// A label definition `\label{key}`.
    Label { key: String },
    /// A reference `\ref{key}`.
    Ref { key: String },
    /// A page reference `\pageref{key}`.
    PageRef { key: String },
}

/// A single bibliography entry for `thebibliography`.
#[derive(Debug, Clone, PartialEq)]
pub struct BibEntry {
    pub key: String,
    pub text: String,
}

/// Stateful parser for a single LaTeX document.
pub struct TexParser {
    content: String,
    position: usize,
    plugins: Option<crate::plugins::PluginRegistry>,
    base_dir: Option<PathBuf>,
}

impl TexParser {
    /// Create a new parser for the given LaTeX source.
    /// Macro definitions (`\newcommand`, `\def`) are extracted and
    /// all macro calls are expanded before structured parsing begins.
    pub fn new(content: String) -> Self {
        let mut store = crate::macros::MacroStore::new();
        let stripped = store.extract_definitions(&content);
        let expanded = store.expand_all(&stripped);
        Self { content: expanded, position: 0, plugins: None, base_dir: None }
    }

    /// Create a parser with a plugin registry for custom command and
    /// environment handlers.
    pub fn with_plugins(content: String, plugins: crate::plugins::PluginRegistry) -> Self {
        let mut parser = Self::new(content);
        parser.plugins = Some(plugins);
        parser
    }

    /// Set the base directory for resolving relative paths in `\input`.
    pub fn with_base_dir(mut self, base_dir: &Path) -> Self {
        self.base_dir = Some(base_dir.to_path_buf());
        self
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

        if remaining.starts_with("\\textcolor") {
            return self.parse_textcolor();
        }

        if remaining.starts_with("\\texttt{") || remaining.starts_with("\\textbf{") || remaining.starts_with("\\textit{") || remaining.starts_with("\\emph{") {
            return self.parse_text_command();
        }

        if remaining.starts_with("\\includegraphics") {
            return self.parse_includegraphics();
        }

        if remaining.starts_with("\\cite") {
            return self.parse_cite();
        }

        if remaining.starts_with("\\label") {
            return self.parse_label();
        }

        if remaining.starts_with("\\ref") {
            return self.parse_ref();
        }

        if remaining.starts_with("\\pageref") {
            return self.parse_pageref();
        }

        if remaining.starts_with('$') {
            return self.parse_math();
        }

        // Page breaks
        if remaining.starts_with("\\newpage") || remaining.starts_with("\\clearpage") || remaining.starts_with("\\pagebreak") {
            return self.parse_pagebreak();
        }

        // Include external file content inline
        if remaining.starts_with("\\input{") {
            return self.parse_input();
        }

        // Vertical spacing
        if remaining.starts_with("\\vspace{") {
            return self.parse_vspace();
        }

        // Underline
        if remaining.starts_with("\\underline{") {
            return self.parse_underline();
        }

        // Font size commands
        let sizes = [
            ("\\tiny", "tiny"),
            ("\\scriptsize", "scriptsize"),
            ("\\footnotesize", "footnotesize"),
            ("\\small", "small"),
            ("\\normalsize", "normalsize"),
            ("\\large", "large"),
            ("\\Large", "Large"),
            ("\\LARGE", "LARGE"),
            ("\\huge", "huge"),
            ("\\Huge", "Huge"),
        ];
        for (prefix, name) in &sizes {
            if remaining.starts_with(prefix) {
                self.position += prefix.len();
                return Some(TexElement::Command { name: name.to_string(), args: vec![] });
            }
        }

        if remaining.starts_with('\\') {
            if let Some(elem) = self.try_plugin_command() {
                return Some(elem);
            }
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
        } else if remaining.starts_with("\\emph{") {
            ("emph", 6)
        } else {
            return None;
        };

        self.position += cmd_len;
        
        self.parse_braced_content().map(TexElement::Text)
    }

    fn parse_includegraphics(&mut self) -> Option<TexElement> {
        self.position += "\\includegraphics".len();
        self.skip_whitespace_and_comments();

        let mut width = None;
        let mut height = None;

        // Optional [width=...,height=...] arguments
        if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
            self.position += 1;
            let opts = self.read_until(']');
            self.position += 1;

            for part in opts.split(',') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("width=") {
                    width = Some(val.trim().to_string());
                } else if let Some(val) = part.strip_prefix("height=") {
                    height = Some(val.trim().to_string());
                }
            }
        }

        self.skip_whitespace_and_comments();

        let path = self.parse_braced_content()?;
        Some(TexElement::Image { path, width, height })
    }

    fn parse_textcolor(&mut self) -> Option<TexElement> {
        self.position += "\\textcolor".len();
        self.skip_whitespace_and_comments();

        let color = self.parse_braced_content()?;
        self.skip_whitespace_and_comments();

        let text = self.parse_braced_content()?;
        Some(TexElement::ColoredText { color, text })
    }

    fn parse_cite(&mut self) -> Option<TexElement> {
        self.position += "\\cite".len();
        self.skip_whitespace_and_comments();

        let keys_str = self.parse_braced_content()?;
        let keys: Vec<String> = keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Some(TexElement::Citation { keys })
    }

    fn parse_label(&mut self) -> Option<TexElement> {
        self.position += "\\label".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::Label { key })
    }

    fn parse_ref(&mut self) -> Option<TexElement> {
        self.position += "\\ref".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::Ref { key })
    }

    fn parse_pageref(&mut self) -> Option<TexElement> {
        self.position += "\\pageref".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::PageRef { key })
    }

    fn parse_pagebreak(&mut self) -> Option<TexElement> {
        if self.content[self.position..].starts_with("\\newpage") {
            self.position += "\\newpage".len();
        } else if self.content[self.position..].starts_with("\\clearpage") {
            self.position += "\\clearpage".len();
        } else if self.content[self.position..].starts_with("\\pagebreak") {
            self.position += "\\pagebreak".len();
        }
        Some(TexElement::Command { name: "newpage".to_string(), args: vec![] })
    }

    fn parse_input(&mut self) -> Option<TexElement> {
        self.position += "\\input{".len();
        let filename = self.read_until('}');
        self.position += 1; // skip closing brace

        let mut path = if let Some(ref base) = self.base_dir {
            base.join(&filename)
        } else {
            PathBuf::from(&filename)
        };
        // Ensure .tex extension if missing
        if path.extension().is_none() {
            path.set_extension("tex");
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            // Splice included content into current source at current position
            let before = &self.content[..self.position];
            let after = &self.content[self.position..];
            self.content = format!("{}{}\n{}", before, content, after);
        }
        // Return None so the loop re-parses the spliced content
        None
    }

    fn parse_vspace(&mut self) -> Option<TexElement> {
        self.position += "\\vspace{".len();
        let length = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command { name: "vspace".to_string(), args: vec![length] })
    }

    fn parse_underline(&mut self) -> Option<TexElement> {
        self.position += "\\underline{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command { name: "underline".to_string(), args: vec![text] })
    }

    #[allow(clippy::question_mark)]
    fn try_plugin_command(&mut self) -> Option<TexElement> {
        if self.plugins.is_none() {
            return None;
        }

        // Peek command name without advancing.
        let start = self.position + 1; // skip '\'
        let mut name_end = start;
        while name_end < self.content.len() {
            let ch = self.content[name_end..].chars().next()?;
            if ch.is_alphabetic() || ch == '*' {
                name_end += ch.len_utf8();
            } else {
                break;
            }
        }
        let name = self.content[start..name_end].to_string();
        if name.is_empty() {
            return None;
        }

        // Temporarily advance to collect arguments.
        let saved = self.position;
        self.position = name_end;
        self.skip_whitespace_and_comments();

        let mut args = Vec::new();
        while let Some(arg) = self.parse_braced_content() {
            args.push(arg);
            self.skip_whitespace_and_comments();
        }

        // Now borrow plugins mutably, after all self-borrows are done.
        let plugins = self.plugins.as_mut().unwrap();
        if let Some(elem) = plugins.try_command(&name, &args) {
            return Some(elem);
        }

        // No plugin handled it — restore position.
        self.position = saved;
        None
    }

    #[allow(clippy::question_mark)]
    fn try_plugin_environment(&mut self, env_name: &str) -> Option<TexElement> {
        if self.plugins.is_none() {
            return None;
        }
        let end_marker = format!("\\end{{{}}}", env_name);
        let body_start = self.position;
        let body_end = self.content[self.position..].find(&end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();

        let plugins = self.plugins.as_mut().unwrap();
        if let Some(elem) = plugins.try_environment(env_name, &body) {
            self.position = body_start + body_end + end_marker.len();
            return Some(elem);
        }
        None
    }

    #[allow(clippy::collapsible_if)]
    fn parse_thebibliography(&mut self) -> Option<TexElement> {
        // Skip optional argument {number}
        self.skip_whitespace_and_comments();
        if self.content[self.position..].starts_with('{') {
            if let Some((_inner, end)) = extract_braced(&self.content, self.position) {
                self.position = end;
            }
        }

        let mut entries = Vec::new();
        let end_marker = "\\end{thebibliography}";

        while self.position < self.content.len() {
            self.skip_whitespace_and_comments();
            let remaining = &self.content[self.position..];

            if remaining.starts_with(end_marker) {
                self.position += end_marker.len();
                break;
            }

            if remaining.starts_with("\\bibitem") {
                self.position += "\\bibitem".len();
                self.skip_whitespace_and_comments();

                let key = if self.content[self.position..].starts_with('{') {
                    self.parse_braced_content().unwrap_or_default()
                } else {
                    // Plain key (no braces)
                    let start = self.position;
                    while self.position < self.content.len() {
                        let ch = self.content[self.position..].chars().next().unwrap();
                        if ch.is_whitespace() || ch == '\\' {
                            break;
                        }
                        self.position += ch.len_utf8();
                    }
                    self.content[start..self.position].to_string()
                };
                self.skip_whitespace_and_comments();

                // Read text until next \bibitem or \end
                let text_start = self.position;
                while self.position < self.content.len() {
                    let rem = &self.content[self.position..];
                    if rem.starts_with("\\bibitem") || rem.starts_with(end_marker) {
                        break;
                    }
                    self.position += rem.chars().next().unwrap().len_utf8();
                }
                let text = self.content[text_start..self.position].trim().to_string();
                entries.push(BibEntry { key, text });
            } else {
                // Skip unknown content
                self.position += remaining.chars().next().unwrap().len_utf8();
            }
        }

        Some(TexElement::Bibliography { entries })
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
            "thebibliography" => self.parse_thebibliography(),
            _ => {
                if let Some(elem) = self.try_plugin_environment(&env_name) {
                    return Some(elem);
                }
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
        let raw = self.read_until_str("\\end{tabular}");
        self.position += "\\end{tabular}".len();

        // Extract column spec from the start of raw content: {lc|r}...
        let (spec, body) = if let Some(end) = raw.find('}') {
            (&raw[..=end], &raw[end + 1..])
        } else {
            ("", raw.as_str())
        };

        Some(TexElement::Table(Table::parse(spec, body)))
    }

    fn parse_table(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{table}");
        self.position += "\\end{table}".len();

        // Look for tabular environment within table
        if let Some(tabular_start) = content.find("\\begin{tabular}") {
            let after_begin = &content[tabular_start + "\\begin{tabular}".len()..];
            let (spec, body_with_end) = if let Some(end) = after_begin.find('}') {
                (&after_begin[..=end], &after_begin[end + 1..])
            } else {
                ("", after_begin)
            };
            if let Some(tabular_end) = body_with_end.find("\\end{tabular}") {
                let body = &body_with_end[..tabular_end];
                return Some(TexElement::Table(Table::parse(spec, body)));
            }
        }

        Some(TexElement::Text(String::new()))
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

    #[test]
    fn parser_parses_includegraphics() {
        let content = r#"\documentclass{article}
\begin{document}
\includegraphics{logo.png}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Image { path, width, height } if path == "logo.png" && width.is_none() && height.is_none())
        }));
    }

    #[test]
    fn parser_parses_includegraphics_with_options() {
        let content = r#"\documentclass{article}
\begin{document}
\includegraphics[width=5cm,height=3cm]{logo.png}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Image { path, width, height }
                if path == "logo.png"
                && width.as_deref() == Some("5cm")
                && height.as_deref() == Some("3cm"))
        }));
    }

    #[test]
    fn parser_parses_tabular() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{tabular}{lcr}
\hline
A & B & C \\
\hline
1 & 2 & 3 \\
\end{tabular}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            if let TexElement::Table(table) = element {
                table.columns == vec![crate::table::Align::Left, crate::table::Align::Center, crate::table::Align::Right]
                    && table.rows.len() == 4
                    && table.rows[0].is_separator
                    && table.rows[1].cells == vec!["A", "B", "C"]
                    && table.rows[2].is_separator
                    && table.rows[3].cells == vec!["1", "2", "3"]
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_parses_table_environment() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{table}
\begin{tabular}{cc}
X & Y \\
\end{tabular}
\end{table}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            if let TexElement::Table(table) = element {
                table.columns == vec![crate::table::Align::Center, crate::table::Align::Center]
                    && table.rows.len() == 1
                    && table.rows[0].cells == vec!["X", "Y"]
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_parses_textcolor() {
        let content = r#"\documentclass{article}
\begin{document}
\textcolor{red}{Important!}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::ColoredText { color, text } if color == "red" && text == "Important!")
        }));
    }

    #[test]
    fn parser_expands_newcommand_macro() {
        let content = r#"\documentclass{article}
\newcommand{\hello}{Hello World}
\begin{document}
\hello
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Text(t) if t.contains("Hello World"))
        }));
    }

    #[test]
    fn parser_expands_newcommand_with_args() {
        let content = r#"\documentclass{article}
\newcommand{\greet}[1]{Hello, #1!}
\begin{document}
\greet{Alice}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Text(t) if t.contains("Hello, Alice!"))
        }));
    }

    #[test]
    fn parser_expands_def_macro() {
        let content = r#"\documentclass{article}
\def\twice#1{#1 #1}
\begin{document}
\twice{hi}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Text(t) if t.contains("hi hi"))
        }));
    }

    #[test]
    fn parser_parses_cite() {
        let content = r#"\documentclass{article}
\begin{document}
See \cite{smith2024} for details.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Citation { keys } if keys == &["smith2024"])
        }));
    }

    #[test]
    fn parser_parses_cite_multiple() {
        let content = r#"\documentclass{article}
\begin{document}
See \cite{smith2024, jones2023}.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Citation { keys } if keys == &["smith2024", "jones2023"])
        }));
    }

    #[test]
    fn parser_parses_thebibliography() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{thebibliography}{9}
\bibitem{smith2024} J. Smith, A Great Paper, Journal of Testing, 2024.
\bibitem{jones2023} A. Jones, Another Paper, 2023.
\end{thebibliography}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            if let TexElement::Bibliography { entries } = element {
                entries.len() == 2
                    && entries[0].key == "smith2024"
                    && entries[0].text.contains("A Great Paper")
                    && entries[1].key == "jones2023"
                    && entries[1].text.contains("Another Paper")
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_parses_label() {
        let content = r#"\documentclass{article}
\begin{document}
\section{Intro}
\label{sec:intro}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Label { key } if key == "sec:intro")
        }));
    }

    #[test]
    fn parser_parses_ref() {
        let content = r#"\documentclass{article}
\begin{document}
See Section~\ref{sec:intro}.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Ref { key } if key == "sec:intro")
        }));
    }

    #[test]
    fn parser_parses_pageref() {
        let content = r#"\documentclass{article}
\begin{document}
See page~\pageref{sec:intro}.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::PageRef { key } if key == "sec:intro")
        }));
    }

    #[test]
    fn parser_plugin_handles_today() {
        let content = r#"\documentclass{article}
\begin{document}
Today is \today.
\end{document}
"#;

        let mut plugins = crate::plugins::PluginRegistry::new();
        plugins.register(Box::new(crate::plugins::TodayPlugin));
        let mut parser = TexParser::with_plugins(content.to_string(), plugins);
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            if let TexElement::Text(t) = element {
                t.contains("202")
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_plugin_handles_url() {
        let content = r#"\documentclass{article}
\begin{document}
Visit \url{https://example.com}.
\end{document}
"#;

        let mut plugins = crate::plugins::PluginRegistry::new();
        plugins.register(Box::new(crate::plugins::UrlPlugin));
        let mut parser = TexParser::with_plugins(content.to_string(), plugins);
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            if let TexElement::Text(t) = element {
                t.contains("https://example.com")
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_parses_emph() {
        let content = r#"\emph{important}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Text(t) if t == "important")));
    }

    #[test]
    fn parser_parses_newpage() {
        let content = r#"\newpage"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "newpage" && args.is_empty())));
    }

    #[test]
    fn parser_parses_clearpage_as_newpage() {
        let content = r#"\clearpage"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "newpage" && args.is_empty())));
    }

    #[test]
    fn parser_parses_vspace() {
        let content = r#"\vspace{12pt}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "vspace" && args == &["12pt"])));
    }

    #[test]
    fn parser_parses_underline() {
        let content = r#"\underline{key}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "underline" && args == &["key"])));
    }

    #[test]
    fn parser_parses_font_size_commands() {
        let sizes = ["\\tiny", "\\scriptsize", "\\footnotesize", "\\small",
                     "\\normalsize", "\\large", "\\Large", "\\LARGE", "\\huge", "\\Huge"];
        for cmd in &sizes {
            let content = format!("{}text", cmd);
            let name = &cmd[1..]; // strip backslash
            let mut parser = TexParser::new(content);
            let elements = parser.parse();
            assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name: n, args } if n == name && args.is_empty())),
                "failed to parse {}", cmd);
        }
    }

    #[test]
    fn parser_input_splices_file_content() {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        let included = tmp.path().join("included.tex");
        {
            let mut f = std::fs::File::create(&included).unwrap();
            f.write_all(b"\\textbf{included}").unwrap();
        }

        let content = r#"\input{included}"#;
        let mut parser = TexParser::new(content.to_string())
            .with_base_dir(tmp.path());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            if let TexElement::Text(t) = e { t.contains("included") } else { false }
        }));
    }

    #[test]
    fn parser_input_adds_tex_extension() {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        let included = tmp.path().join("chapter1.tex");
        {
            let mut f = std::fs::File::create(&included).unwrap();
            f.write_all(b"Chapter text").unwrap();
        }

        let content = r#"\input{chapter1}"#;
        let mut parser = TexParser::new(content.to_string())
            .with_base_dir(tmp.path());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            if let TexElement::Text(t) = e { t.contains("Chapter text") } else { false }
        }));
    }

    #[test]
    fn parser_input_missing_file_is_noop() {
        let content = r#"\input{nonexistent}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        // Should not panic; parser skips the command and continues
        assert!(!elements.iter().any(|e| matches!(e, TexElement::Command { name, .. } if name == "newpage")));
    }
}
