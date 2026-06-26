//! LaTeX parser - converts raw LaTeX source into structured elements.
//!
//! Handles document structure, environments, commands, inline math,
//! display math, lists, tables, and metadata extraction.

use crate::table::Table;
use crate::utils::extract_braced;
use std::path::{Path, PathBuf};

mod text;
mod math;
mod commands;


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
    ItemList { ordered: bool, labels: Vec<Option<String>>, items: Vec<Vec<TexElement>> },
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
    /// A centered block from `\begin{center}`.
    Center(Vec<TexElement>),
    /// A table of contents placeholder (`\tableofcontents`).
    TableOfContents,
    /// A list of figures placeholder (`\listoffigures`).
    ListOfFigures,
    /// A list of tables placeholder (`\listoftables`).
    ListOfTables,
    /// A footnote `\footnote{text}`.
    Footnote { text: String },
    /// A caption `\caption{text}` for tables or figures.
    Caption { text: String },
    /// A quote or quotation block (`\begin{quote}` or `\begin{quotation}`).
    Quote(Vec<TexElement>),
    /// An abstract block (`\begin{abstract}`).
    Abstract(Vec<TexElement>),
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

    /// Skip the document preamble (everything before `\begin{document}`).
    fn skip_preamble(&mut self) {
        if let Some(begin_doc) = self.content.find("\\begin{document}") {
            self.position = begin_doc + "\\begin{document}".len();
        }
    }

    /// Scan the next syntactic construct and return a [`TexElement`].
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

        if remaining.starts_with("\\part") {
            return self.parse_section(0);
        }

        if remaining.starts_with("\\chapter") {
            return self.parse_section(6);
        }

        if remaining.starts_with("\\section") {
            return self.parse_section(1);
        }
        
        if remaining.starts_with("\\subsection") {
            return self.parse_section(2);
        }

        if remaining.starts_with("\\subsubsection") {
            return self.parse_section(3);
        }

        if remaining.starts_with("\\paragraph") {
            return self.parse_section(4);
        }

        if remaining.starts_with("\\subparagraph") {
            return self.parse_section(5);
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

        if remaining.starts_with("\\tableofcontents") {
            self.position += "\\tableofcontents".len();
            return Some(TexElement::TableOfContents);
        }

        if remaining.starts_with("\\listoffigures") {
            self.position += "\\listoffigures".len();
            return Some(TexElement::ListOfFigures);
        }

        if remaining.starts_with("\\listoftables") {
            self.position += "\\listoftables".len();
            return Some(TexElement::ListOfTables);
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

        // Paragraph break
        if remaining.starts_with("\\par") {
            self.position += "\\par".len();
            return Some(TexElement::Paragraph);
        }

        // Alignment commands
        if remaining.starts_with("\\raggedright") || remaining.starts_with("\\flushleft") {
            self.position += if remaining.starts_with("\\raggedright") { "\\raggedright".len() } else { "\\flushleft".len() };
            return Some(TexElement::Command { name: "raggedright".to_string(), args: vec![] });
        }
        if remaining.starts_with("\\raggedleft") || remaining.starts_with("\\flushright") {
            self.position += if remaining.starts_with("\\raggedleft") { "\\raggedleft".len() } else { "\\flushright".len() };
            return Some(TexElement::Command { name: "raggedleft".to_string(), args: vec![] });
        }

        // No indent
        if remaining.starts_with("\\noindent") {
            self.position += "\\noindent".len();
            return Some(TexElement::Command { name: "noindent".to_string(), args: vec![] });
        }

        // Today
        if remaining.starts_with("\\today") {
            self.position += "\\today".len();
            return Some(TexElement::Command { name: "today".to_string(), args: vec![] });
        }

        // Special text characters (insert literal symbols)
        let text_chars = [
            ("\\textasciicircum", "^"),
            ("\\textasciitilde", "~"),
            ("\\textbackslash", "\\"),
            ("\\textbar", "|"),
            ("\\textbraceleft", "{"),
            ("\\textbraceright", "}"),
            ("\\textdollar", "$"),
            ("\\textgreater", ">"),
            ("\\textless", "<"),
        ];
        for (prefix, ch) in &text_chars {
            if remaining.starts_with(prefix) {
                self.position += prefix.len();
                return Some(TexElement::Text(ch.to_string()));
            }
        }

        // URL
        if remaining.starts_with("\\url{") {
            return self.parse_url();
        }

        // Additional text formatting commands
        if remaining.starts_with("\\text{") {
            return self.parse_simple_braced_command("text", 6);
        }
        if remaining.starts_with("\\ensuremath{") {
            return self.parse_simple_braced_command("ensuremath", 12);
        }
        if remaining.starts_with("\\overline{") {
            return self.parse_simple_braced_command("overline", 10);
        }
        if remaining.starts_with("\\sout{") {
            return self.parse_simple_braced_command("sout", 6);
        }
        if remaining.starts_with("\\textsc{") {
            return self.parse_simple_braced_command("textsc", 8);
        }
        if remaining.starts_with("\\textrm{") {
            return self.parse_simple_braced_command("textrm", 8);
        }
        if remaining.starts_with("\\textsf{") {
            return self.parse_simple_braced_command("textsf", 8);
        }
        if remaining.starts_with("\\textsl{") {
            return self.parse_simple_braced_command("textsl", 8);
        }
        if remaining.starts_with("\\textup{") {
            return self.parse_simple_braced_command("textup", 8);
        }
        if remaining.starts_with("\\textmd{") {
            return self.parse_simple_braced_command("textmd", 8);
        }

        // Phantom commands (invisible spacing)
        if remaining.starts_with("\\phantom{") {
            return self.parse_simple_braced_command("phantom", 9);
        }
        if remaining.starts_with("\\vphantom{") {
            return self.parse_simple_braced_command("vphantom", 10);
        }
        if remaining.starts_with("\\hphantom{") {
            return self.parse_simple_braced_command("hphantom", 10);
        }

        // Raise box
        if remaining.starts_with("\\raisebox{") {
            return self.parse_raisebox();
        }

        // Rotate and scale boxes
        if remaining.starts_with("\\rotatebox{") {
            return self.parse_rotatebox();
        }
        if remaining.starts_with("\\scalebox{") {
            return self.parse_scalebox();
        }

        // Colored boxes
        if remaining.starts_with("\\colorbox{") {
            return self.parse_colorbox();
        }
        if remaining.starts_with("\\fcolorbox{") {
            return self.parse_fcolorbox();
        }

        // Superscript / subscript
        if remaining.starts_with("\\textsuperscript{") {
            return self.parse_simple_braced_command("textsuperscript", 17);
        }
        if remaining.starts_with("\\textsubscript{") {
            return self.parse_simple_braced_command("textsubscript", 15);
        }

        // Framed box
        if remaining.starts_with("\\fbox{") {
            return self.parse_simple_braced_command("fbox", 6);
        }

        // Rule / horizontal line
        if remaining.starts_with("\\rule{") {
            return self.parse_rule();
        }

        // Spacing commands
        let spacing_cmds = [
            ("\\hfill", "hfill"),
            ("\\vfill", "vfill"),
            ("\\hrulefill", "hrulefill"),
            ("\\dotfill", "dotfill"),
            ("\\medskip", "medskip"),
            ("\\bigskip", "bigskip"),
            ("\\smallskip", "smallskip"),
            ("\\strut", "strut"),
            ("\\mathstrut", "mathstrut"),
            ("\\qquad", "qquad"),
            ("\\quad", "quad"),
            ("\\;", "semicolon"),
            ("\\,", "comma"),
            ("\\!", "bang"),
            ("\\:", "colon"),
            ("\\ ", "control_space"),
        ];
        for (prefix, name) in &spacing_cmds {
            if remaining.starts_with(prefix) {
                self.position += prefix.len();
                return Some(TexElement::Command { name: name.to_string(), args: vec![] });
            }
        }

        // Include external file content inline
        if remaining.starts_with("\\input{") {
            return self.parse_input();
        }

        // Vertical spacing
        if remaining.starts_with("\\vspace{") {
            return self.parse_vspace();
        }

        // Bibliography commands
        if remaining.starts_with("\\bibliography{") {
            return self.parse_simple_braced_command("bibliography", 14);
        }
        if remaining.starts_with("\\bibliographystyle{") {
            return self.parse_simple_braced_command("bibliographystyle", 19);
        }

        // Appendix marker
        if remaining.starts_with("\\appendix") {
            self.position += "\\appendix".len();
            return Some(TexElement::Command { name: "appendix".to_string(), args: vec![] });
        }

        // Index and glossary entries
        if remaining.starts_with("\\index{") {
            return self.parse_simple_braced_command("index", 7);
        }
        if remaining.starts_with("\\glossary{") {
            return self.parse_simple_braced_command("glossary", 10);
        }

        // Underline
        if remaining.starts_with("\\underline{") {
            return self.parse_underline();
        }

        // Footnote
        if remaining.starts_with("\\footnote{") {
            return self.parse_footnote();
        }

        // Caption
        if remaining.starts_with("\\caption{") {
            return self.parse_caption();
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

        // Centering declaration
        if remaining.starts_with("\\centering") {
            self.position += "\\centering".len();
            return Some(TexElement::Command { name: "centering".to_string(), args: vec![] });
        }

        // Deprecated font declarations (still widely used)
        let font_decls = [
            ("\\em", "em"),
            ("\\bf", "bf"),
            ("\\it", "it"),
            ("\\rm", "rm"),
            ("\\sf", "sf"),
            ("\\tt", "tt"),
            ("\\sc", "sc"),
            ("\\sl", "sl"),
        ];
        for (prefix, name) in &font_decls {
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

    /// Advance past whitespace characters and `%` comments.
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

    /// Parse a `\begin{center} … \end{center}` block.
    fn parse_center(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{center}";
        let body_start = self.position;
        let body_end = self.content[self.position..].find(end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::Center(elements))
    }

    /// Parse a `\begin{quote}` or `\begin{quotation}` block.
    fn parse_quote(&mut self, env_name: &str) -> Option<TexElement> {
        let end_marker = format!("\\end{{{}}}", env_name);
        let body_start = self.position;
        let body_end = self.content[self.position..].find(&end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::Quote(elements))
    }

    /// Parse a `\begin{abstract}` block.
    fn parse_abstract(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{abstract}";
        let body_start = self.position;
        let body_end = self.content[self.position..].find(end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::Abstract(elements))
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
    /// Parse a `\begin{thebibliography} … \end{thebibliography}` block.
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

    /// Parse an environment: dispatch to the concrete parser based on the name.
    fn parse_environment(&mut self) -> Option<TexElement> {
        self.position += "\\begin{".len();

        let env_name = self.read_until('}');
        self.position += 1;

        match env_name.as_str() {
            "itemize" => self.parse_itemize(false),
            "enumerate" => self.parse_itemize(true),
            "equation" => self.parse_equation(),
            "lstlisting" => self.parse_lstlisting(),
            "verbatim" => self.parse_lstlisting(),
            "tabular" => self.parse_tabular(),
            "table" => self.parse_table(),
            "thebibliography" => self.parse_thebibliography(),
            "center" => self.parse_center(),
            "quote" | "quotation" => self.parse_quote(&env_name),
            "abstract" => self.parse_abstract(),
            _ => {
                if let Some(elem) = self.try_plugin_environment(&env_name) {
                    return Some(elem);
                }
                self.skip_until(&format!("\\end{{{}}}", env_name));
                None
            }
        }
    }

    /// Parse `\begin{itemize}` / `\begin{enumerate}` and their `\item`s.
    fn parse_itemize(&mut self, ordered: bool) -> Option<TexElement> {
        let mut items: Vec<Vec<TexElement>> = Vec::new();
        let mut labels: Vec<Option<String>> = Vec::new();
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
                
                // Check for optional [label]
                let label = if self.content[self.position..].starts_with('[') {
                    self.position += 1;
                    let label_text = self.read_until(']');
                    self.position += 1; // skip ]
                    self.skip_whitespace_and_comments();
                    Some(label_text)
                } else {
                    None
                };
                
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
                
                labels.push(label);
                items.push(item_content);
            } else {
                self.position += 1;
            }
        }

        Some(TexElement::ItemList { ordered, labels, items })
    }

    /// Parse `\begin{equation} … \end{equation}`.
    fn parse_equation(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{equation}");
        self.position += "\\end{equation}".len();

        Some(TexElement::MathDisplay(content.trim().to_string()))
    }

    /// Parse `\begin{lstlisting}` or `\begin{verbatim}`.
    fn parse_lstlisting(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{lstlisting}");
        self.position += "\\end{lstlisting}".len();

        Some(TexElement::CodeBlock(content.trim().to_string()))
    }

    /// Parse `\begin{tabular} … \end{tabular}`.
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

    /// Parse `\begin{table} … \end{table}` (extracts the inner tabular).
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
    /// Read text between matching `{…}` braces and return the inner content.
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

    /// Read characters until `delimiter` is found, returning the text before it.
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

    /// Read characters until `delimiter` is found, returning the text before it.
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

    /// Advance past everything up to and including `marker`.
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
    fn parser_parses_deep_sections() {
        let content = r#"\documentclass{article}
\begin{document}
\section{Intro}
\subsection{Method}
\subsubsection{Details}
\paragraph{Note}
\subparagraph{Fine Print}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 1, title } if title == "Intro")
        }));
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 2, title } if title == "Method")
        }));
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 3, title } if title == "Details")
        }));
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 4, title } if title == "Note")
        }));
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 5, title } if title == "Fine Print")
        }));
    }

    #[test]
    fn parser_parses_verbatim_environment() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{verbatim}
#include <stdio.h>
int main() {
    printf("Hello\n");
    return 0;
}
\end{verbatim}
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::CodeBlock(code) if code.contains("#include") && code.contains("printf"))
        }));
    }

    #[test]
    fn parser_parses_tableofcontents() {
        let content = r#"\documentclass{article}
\begin{document}
\tableofcontents
\section{Intro}
Some text.
\end{document}
"#;

        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::TableOfContents)
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
            matches!(element, TexElement::ItemList { ordered: false, labels, items } if items.len() == 2 && labels.len() == 2)
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
            matches!(element, TexElement::ItemList { ordered: true, labels, items } if items.len() == 2 && labels.len() == 2)
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

        // \textbf, \textit, \texttt should produce Command elements
        let cmds: Vec<_> = elements.iter().filter_map(|e| {
            if let TexElement::Command { name, args } = e {
                Some((name.clone(), args.clone()))
            } else { None }
        }).collect();
        assert!(cmds.iter().any(|(n, a)| n == "textbf" && a == &["bold".to_string()]));
        assert!(cmds.iter().any(|(n, a)| n == "textit" && a == &["italic".to_string()]));
        assert!(cmds.iter().any(|(n, a)| n == "texttt" && a == &["mono".to_string()]));
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

        // Native parser now handles \today before plugin fallback
        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Command { name, args } if name == "today" && args.is_empty())
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

        // Native parser now handles \url{...} before plugin fallback
        assert!(elements.iter().any(|element| {
            matches!(element, TexElement::Command { name, args } if name == "url" && args == &["https://example.com"])
        }));
    }

    #[test]
    fn parser_parses_emph() {
        let content = r#"\emph{important}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "emph" && args == &["important".to_string()])));
    }

    #[test]
    fn parser_parses_newpage() {
        let content = r#"\newpage"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "newpage" && args.is_empty())));
    }

    #[test]
    fn parser_parses_clearpage() {
        let content = r#"\clearpage"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "clearpage" && args.is_empty())));
    }

    #[test]
    fn parser_parses_pagebreak() {
        let content = r#"\pagebreak"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "pagebreak" && args.is_empty())));
    }

    #[test]
    fn parser_parses_caption() {
        let content = r#"\caption{A diagram showing the workflow.}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Caption { text } if text == "A diagram showing the workflow.")));
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
    fn parser_parses_today() {
        let content = r#"\today"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "today" && args.is_empty())));
    }

    #[test]
    fn parser_parses_url() {
        let content = r#"\url{https://example.com}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "url" && args == &["https://example.com"])));
    }

    #[test]
    fn parser_parses_textsuperscript() {
        let content = r#"\textsuperscript{st}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textsuperscript" && args == &["st"])));
    }

    #[test]
    fn parser_parses_textsubscript() {
        let content = r#"\textsubscript{2}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textsubscript" && args == &["2"])));
    }

    #[test]
    fn parser_parses_raggedright() {
        let content = r#"\raggedright some text"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "raggedright" && args.is_empty())));
    }

    #[test]
    fn parser_parses_raggedleft() {
        let content = r#"\raggedleft some text"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "raggedleft" && args.is_empty())));
    }

    #[test]
    fn parser_parses_noindent() {
        let content = r#"\noindent text"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "noindent" && args.is_empty())));
    }

    #[test]
    fn parser_parses_fbox() {
        let content = r#"\fbox{boxed text}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "fbox" && args == &["boxed text"])));
    }

    #[test]
    fn parser_parses_rule() {
        let content = r#"\rule{5cm}{0.4pt}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "rule" && args == &["5cm", "0.4pt"])));
    }

    #[test]
    fn parser_parses_par() {
        let content = r#"first \par second"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Paragraph)));
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
            matches!(e, TexElement::Command { name, args } if name == "textbf" && args == &["included".to_string()])
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

    #[test]
    fn parser_parses_centering_command() {
        let content = r#"\centering centered text"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "centering" && args.is_empty())));
    }

    #[test]
    fn parser_parses_center_environment() {
        let content = r#"\begin{center}centered text\end{center}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            if let TexElement::Center(inner) = e {
                inner.iter().any(|i| matches!(i, TexElement::Text(t) if t.contains("centered")))
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_parses_footnote() {
        let content = r#"Hello\footnote{This is a note.}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Footnote { text } if text == "This is a note.")));
    }

    #[test]
    fn parser_parses_part() {
        let content = r#"\part{The Beginning}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Section { level: 0, title } if title == "The Beginning")));
    }

    #[test]
    fn parser_parses_chapter() {
        let content = r#"\chapter{Intro}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Section { level: 6, title } if title == "Intro")));
    }

    #[test]
    fn parser_parses_quote() {
        let content = r#"\begin{quote}A famous quote.\end{quote}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Quote(inner) if inner.iter().any(|i| matches!(i, TexElement::Text(t) if t.contains("famous"))))
        }));
    }

    #[test]
    fn parser_parses_quotation() {
        let content = r#"\begin{quotation}A longer quotation.\end{quotation}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Quote(inner) if inner.iter().any(|i| matches!(i, TexElement::Text(t) if t.contains("longer"))))
        }));
    }

    #[test]
    fn parser_parses_abstract() {
        let content = r#"\begin{abstract}This is the abstract.\end{abstract}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Abstract(inner) if inner.iter().any(|i| matches!(i, TexElement::Text(t) if t.contains("abstract"))))
        }));
    }

    #[test]
    fn parser_parses_item_with_label() {
        let content = r#"\begin{itemize}\item[Key] Value\end{itemize}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::ItemList { labels, items, .. } if labels.len() == 1 && labels[0] == Some("Key".to_string()) && items.len() == 1)
        }));
    }

    #[test]
    fn parser_parses_listoffigures() {
        let content = r#"\listoffigures"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::ListOfFigures)));
    }

    #[test]
    fn parser_parses_listoftables() {
        let content = r#"\listoftables"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::ListOfTables)));
    }

    #[test]
    fn parser_parses_text_command() {
        let content = r#"\text{plain text}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "text" && args == &["plain text"])));
    }

    #[test]
    fn parser_parses_ensuremath() {
        let content = r#"\ensuremath{x^2}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "ensuremath" && args == &["x^2"])));
    }

    #[test]
    fn parser_parses_overline() {
        let content = r#"\overline{AB}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "overline" && args == &["AB"])));
    }

    #[test]
    fn parser_parses_sout() {
        let content = r#"\sout{deleted}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "sout" && args == &["deleted"])));
    }

    #[test]
    fn parser_parses_textsc() {
        let content = r#"\textsc{Small Caps}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textsc" && args == &["Small Caps"])));
    }

    #[test]
    fn parser_parses_textrm() {
        let content = r#"\textrm{roman}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textrm" && args == &["roman"])));
    }

    #[test]
    fn parser_parses_textsf() {
        let content = r#"\textsf{sans}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textsf" && args == &["sans"])));
    }

    #[test]
    fn parser_parses_textsl() {
        let content = r#"\textsl{slanted}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textsl" && args == &["slanted"])));
    }

    #[test]
    fn parser_parses_textup() {
        let content = r#"\textup{upright}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textup" && args == &["upright"])));
    }

    #[test]
    fn parser_parses_textmd() {
        let content = r#"\textmd{medium}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "textmd" && args == &["medium"])));
    }

    #[test]
    fn parser_parses_phantom() {
        let content = r#"\phantom{hidden}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "phantom" && args == &["hidden"])));
    }

    #[test]
    fn parser_parses_vphantom() {
        let content = r#"\vphantom{hidden}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "vphantom" && args == &["hidden"])));
    }

    #[test]
    fn parser_parses_hphantom() {
        let content = r#"\hphantom{hidden}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "hphantom" && args == &["hidden"])));
    }

    #[test]
    fn parser_parses_raisebox() {
        let content = r#"\raisebox{2pt}{raised text}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "raisebox" && args == &["2pt", "raised text"])));
    }

    #[test]
    fn parser_parses_bibliography() {
        let content = r#"\bibliography{refs}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "bibliography" && args == &["refs"])));
    }

    #[test]
    fn parser_parses_bibliographystyle() {
        let content = r#"\bibliographystyle{plain}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "bibliographystyle" && args == &["plain"])));
    }

    #[test]
    fn parser_parses_appendix() {
        let content = r#"\appendix"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "appendix" && args.is_empty())));
    }

    #[test]
    fn parser_parses_index() {
        let content = r#"\index{term}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "index" && args == &["term"])));
    }

    #[test]
    fn parser_parses_glossary() {
        let content = r#"\glossary{term}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "glossary" && args == &["term"])));
    }

    #[test]
    fn parser_parses_rotatebox() {
        let content = r#"\rotatebox{90}{text}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "rotatebox" && args == &["90", "text"])));
    }

    #[test]
    fn parser_parses_scalebox() {
        let content = r#"\scalebox{2}{text}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "scalebox" && args == &["2", "text"])));
    }

    #[test]
    fn parser_parses_font_declarations() {
        let mut parser = TexParser::new(r#"\em \bf \it \rm \sf \tt \sc \sl"#.to_string());
        let elements = parser.parse();
        let names: Vec<&str> = elements.iter().filter_map(|e| {
            if let TexElement::Command { name, args } = e {
                if args.is_empty() { Some(name.as_str()) } else { None }
            } else { None }
        }).collect();
        assert!(names.contains(&"em"));
        assert!(names.contains(&"bf"));
        assert!(names.contains(&"it"));
        assert!(names.contains(&"rm"));
        assert!(names.contains(&"sf"));
        assert!(names.contains(&"tt"));
        assert!(names.contains(&"sc"));
        assert!(names.contains(&"sl"));
    }

    #[test]
    fn parser_parses_text_special_chars() {
        let cases = [
            ("\\textasciicircum", "^"),
            ("\\textasciitilde", "~"),
            ("\\textbackslash", "\\"),
            ("\\textbar", "|"),
            ("\\textbraceleft", "{"),
            ("\\textbraceright", "}"),
            ("\\textdollar", "$"),
            ("\\textgreater", ">"),
            ("\\textless", "<"),
        ];
        for (cmd, expected) in &cases {
            let mut parser = TexParser::new(cmd.to_string());
            let elements = parser.parse();
            assert!(elements.iter().any(|e| matches!(e, TexElement::Text(t) if t == *expected)),
                "Command {} should produce text {}", cmd, expected);
        }
    }

    #[test]
    fn parser_parses_colorbox() {
        let content = r#"\colorbox{red}{hello}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "colorbox" && args == &["red", "hello"])));
    }

    #[test]
    fn parser_parses_fcolorbox() {
        let content = r#"\fcolorbox{black}{yellow}{hello}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "fcolorbox" && args == &["black", "yellow", "hello"])));
    }

    #[test]
    fn parser_parses_spacing_commands() {
        let cmds = [
            "\\hfill", "\\vfill", "\\hrulefill", "\\dotfill",
            "\\medskip", "\\bigskip", "\\smallskip",
            "\\strut", "\\mathstrut",
            "\\qquad", "\\quad",
            "\\;", "\\,", "\\!", "\\:", "\\ ",
        ];
        for cmd in &cmds {
            let mut parser = TexParser::new(cmd.to_string());
            let elements = parser.parse();
            let expected_name = match *cmd {
                "\\hfill" => "hfill",
                "\\vfill" => "vfill",
                "\\hrulefill" => "hrulefill",
                "\\dotfill" => "dotfill",
                "\\medskip" => "medskip",
                "\\bigskip" => "bigskip",
                "\\smallskip" => "smallskip",
                "\\strut" => "strut",
                "\\mathstrut" => "mathstrut",
                "\\qquad" => "qquad",
                "\\quad" => "quad",
                "\\;" => "semicolon",
                "\\," => "comma",
                "\\!" => "bang",
                "\\:" => "colon",
                "\\ " => "control_space",
                _ => "",
            };
            assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == expected_name && args.is_empty())),
                "Command {} should produce Command {{ name: {}, args: [] }}", cmd, expected_name);
        }
    }
}
