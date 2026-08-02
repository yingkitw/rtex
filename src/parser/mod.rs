//! LaTeX parser - converts raw LaTeX source into structured elements.
//!
//! Handles document structure, environments, commands, inline math,
//! display math, lists, tables, and metadata extraction.

use crate::table::Table;
use crate::utils::extract_braced;
use serde::Serialize;
use std::path::{Path, PathBuf};

mod commands;
mod math;
mod text;

/// A structured element parsed from a LaTeX document.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// Multi-line display math (`align`, `gather`, `multline`, `cases`).
    /// `lines` contains the formatted per-line content; `kind` records the
    /// source environment for downstream formatting choices.
    MathLines {
        lines: Vec<String>,
        kind: MathLineKind,
    },
    /// A list (`itemize` or `enumerate`).
    ItemList {
        ordered: bool,
        labels: Vec<Option<String>>,
        items: Vec<Vec<TexElement>>,
    },
    /// A `description` list — each item is a (term, body) pair.
    DescriptionList { items: Vec<DescItem> },
    /// A theorem-like block (`theorem`, `lemma`, `proof`, `definition`,
    /// `corollary`, `proposition`, `remark`, `example`). `kind` is the
    /// environment name; `title` is set when `\begin{theorem}[name]` is used;
    /// `body` holds the inner parsed elements.
    Theorem {
        kind: String,
        title: Option<String>,
        body: Vec<TexElement>,
    },
    /// A code block from `lstlisting`.
    CodeBlock(String),
    /// An image inclusion (`\includegraphics`).
    Image {
        path: String,
        width: Option<String>,
        height: Option<String>,
    },
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
    /// A forced line break (`\\`).
    LineBreak,
    /// A flush-left block (`\begin{flushleft}`).
    FlushLeft(Vec<TexElement>),
    /// A flush-right block (`\begin{flushright}`).
    FlushRight(Vec<TexElement>),
}

/// A single bibliography entry for `thebibliography`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BibEntry {
    pub key: String,
    pub text: String,
}

/// One entry in a `description` list: a `term` and its body content.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DescItem {
    pub term: String,
    pub body: Vec<TexElement>,
}

/// Multi-line math environment flavour — affects per-line rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MathLineKind {
    /// `align` / `align*` — lines aligned at `&` (rendered with `&` stripped for now).
    Align,
    /// `gather` / `gather*` — centered, each line independent.
    Gather,
    /// `multline` / `multline*` — first line flush left, last line flush right.
    Multline,
    /// `cases` — piecewise definition with a leading brace and 2-column body.
    Cases,
}

/// Stateful parser for a single LaTeX document.
pub struct TexParser {
    content: String,
    position: usize,
    plugins: Option<crate::plugins::PluginRegistry>,
    base_dir: Option<PathBuf>,
    search_paths: Vec<PathBuf>,
}

impl TexParser {
    /// Create a new parser for the given LaTeX source.
    /// Macro definitions (`\newcommand`, `\def`) are extracted and
    /// all macro calls are expanded before structured parsing begins.
    pub fn new(content: String) -> Self {
        let mut store = crate::macros::MacroStore::new();
        let stripped = store.extract_definitions(&content);
        let expanded = store.expand_all(&stripped);
        Self {
            content: expanded,
            position: 0,
            plugins: None,
            base_dir: None,
            search_paths: Vec::new(),
        }
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

    /// Add directories searched after `base_dir` when resolving `\input{...}`.
    pub fn with_search_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.search_paths = paths;
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

    fn extract_metadata_command(
        &self,
        preamble: &str,
        cmd: &str,
        offset: usize,
    ) -> Option<TexElement> {
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

        if remaining.starts_with("\\texttt{")
            || remaining.starts_with("\\textbf{")
            || remaining.starts_with("\\textit{")
            || remaining.starts_with("\\emph{")
        {
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

        // TeX-style math delimiters: \(...\) for inline, \[...\] for display.
        if remaining.starts_with("\\[") {
            self.position += "\\[".len();
            let content = self.read_until_str("\\]");
            self.position += "\\]".len();
            // If the content contains a `\begin{cases}` block, recurse on it
            // so the cases become a `MathLines` element rather than literal text.
            if content.contains("\\begin{cases}") {
                return self.split_display_math_with_cases(&content);
            }
            return Some(TexElement::MathDisplay(content));
        }
        if remaining.starts_with("\\(") {
            self.position += "\\(".len();
            let content = self.read_until_str("\\)");
            self.position += "\\)".len();
            // Same recursive handling for inline math.
            if content.contains("\\begin{cases}") {
                return self.split_inline_math_with_cases(&content);
            }
            return Some(TexElement::MathInline(content));
        }

        // Page breaks
        if remaining.starts_with("\\newpage")
            || remaining.starts_with("\\clearpage")
            || remaining.starts_with("\\pagebreak")
        {
            return self.parse_pagebreak();
        }

        // Paragraph break (must check after \parbox to avoid matching \parbox as \par)
        if remaining.starts_with("\\par") && !remaining.starts_with("\\parbox") {
            self.position += "\\par".len();
            return Some(TexElement::Paragraph);
        }

        // Alignment commands
        if remaining.starts_with("\\raggedright") || remaining.starts_with("\\flushleft") {
            self.position += if remaining.starts_with("\\raggedright") {
                "\\raggedright".len()
            } else {
                "\\flushleft".len()
            };
            return Some(TexElement::Command {
                name: "raggedright".to_string(),
                args: vec![],
            });
        }
        if remaining.starts_with("\\raggedleft") || remaining.starts_with("\\flushright") {
            self.position += if remaining.starts_with("\\raggedleft") {
                "\\raggedleft".len()
            } else {
                "\\flushright".len()
            };
            return Some(TexElement::Command {
                name: "raggedleft".to_string(),
                args: vec![],
            });
        }

        // No indent
        if remaining.starts_with("\\noindent") {
            self.position += "\\noindent".len();
            return Some(TexElement::Command {
                name: "noindent".to_string(),
                args: vec![],
            });
        }

        // Today
        if remaining.starts_with("\\today") {
            self.position += "\\today".len();
            return Some(TexElement::Command {
                name: "today".to_string(),
                args: vec![],
            });
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
            ("\\textellipsis", "…"),
            ("\\textcopyright", "©"),
            ("\\textregistered", "®"),
            ("\\texttrademark", "™"),
            ("\\copyright", "©"),
            ("\\pounds", "£"),
            ("\\S", "§"),
            ("\\P", "¶"),
            ("\\dag", "†"),
            ("\\ddag", "‡"),
            ("\\ldots", "…"),
            ("\\dots", "…"),
            ("\\LaTeXe", "LaTeX2ε"),
            ("\\LaTeX", "LaTeX"),
            ("\\TeX", "TeX"),
            ("\\AA", "Å"),
            ("\\aa", "å"),
            ("\\AE", "Æ"),
            ("\\ae", "æ"),
            ("\\OE", "Œ"),
            ("\\oe", "œ"),
            ("\\ss", "ß"),
            ("\\L", "Ł"),
            ("\\l", "ł"),
            ("\\O", "Ø"),
            ("\\o", "ø"),
            ("\\i", "ı"),
            ("\\j", "ȷ"),
        ];
        for (prefix, ch) in &text_chars {
            if let Some(after) = remaining.strip_prefix(prefix) {
                // Ensure we don't match a prefix of a longer command name.
                // E.g. \i should not match \it, \o should not match \oe.
                if after.starts_with(|c: char| c.is_alphabetic()) {
                    continue;
                }
                self.position += prefix.len();
                return Some(TexElement::Text(ch.to_string()));
            }
        }

        // URL
        if remaining.starts_with("\\url{") {
            return self.parse_url();
        }
        if remaining.starts_with("\\href{") {
            return self.parse_href();
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
                return Some(TexElement::Command {
                    name: name.to_string(),
                    args: vec![],
                });
            }
        }

        // Include external file content inline
        if remaining.starts_with("\\input{") {
            return self.parse_input();
        }

        // Vertical spacing
        if remaining.starts_with("\\vspace*{") {
            return self.parse_simple_braced_command("vspace*", 9);
        }
        if remaining.starts_with("\\vspace{") {
            return self.parse_vspace();
        }

        // Horizontal spacing
        if remaining.starts_with("\\hspace*{") {
            return self.parse_simple_braced_command("hspace*", 9);
        }
        if remaining.starts_with("\\hspace{") {
            return self.parse_simple_braced_command("hspace", 8);
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
            return Some(TexElement::Command {
                name: "appendix".to_string(),
                args: vec![],
            });
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

        // Text normal font
        if remaining.starts_with("\\textnormal{") {
            return self.parse_simple_braced_command("textnormal", 12);
        }

        // Enquote (csquotes)
        if remaining.starts_with("\\enquote{") {
            return self.parse_simple_braced_command("enquote", 9);
        }

        // mbox — horizontal box
        if remaining.starts_with("\\mbox{") {
            return self.parse_simple_braced_command("mbox", 6);
        }

        // parbox[alignment]{width}{text}
        if remaining.starts_with("\\parbox") {
            return self.parse_parbox();
        }

        // makebox[width][position]{text}
        if remaining.starts_with("\\makebox") {
            return self.parse_makebox();
        }

        // Footnote companions
        if remaining.starts_with("\\footnotemark") {
            self.position += "\\footnotemark".len();
            // Optional [number]
            self.skip_whitespace_and_comments();
            if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
                self.position += 1;
                let _ = self.read_until(']');
                self.position += 1;
            }
            return Some(TexElement::Command {
                name: "footnotemark".to_string(),
                args: vec![],
            });
        }
        if remaining.starts_with("\\footnotetext{") {
            return self.parse_simple_braced_command("footnotetext", 13);
        }

        // Table commands
        if remaining.starts_with("\\multicolumn{") {
            return self.parse_multicolumn();
        }
        if remaining.starts_with("\\cline{") {
            return self.parse_simple_braced_command("cline", 7);
        }

        // Page control commands (no-op, just consume)
        let page_cmds = [
            ("\\linebreak", "linebreak"),
            ("\\nopagebreak", "nopagebreak"),
            ("\\samepage", "samepage"),
            ("\\enlargethispage", "enlargethispage"),
        ];
        for (prefix, name) in &page_cmds {
            if remaining.starts_with(prefix) {
                self.position += prefix.len();
                self.skip_whitespace_and_comments();
                // Optional [length] argument
                if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
                    self.position += 1;
                    let _ = self.read_until(']');
                    self.position += 1;
                }
                return Some(TexElement::Command {
                    name: name.to_string(),
                    args: vec![],
                });
            }
        }

        // Skip commands — consume and produce no output
        let skip_cmds = [
            "\\setlength", "\\addtolength", "\\newlength", "\\setcounter", "\\newcounter",
            "\\stepcounter", "\\refstepcounter", "\\ignorespaces", "\\ignorespacesafterend",
            "\\protect", "\\typeout", "\\obeylines", "\\obeyspaces", "\\baselinestretch",
            "\\linespread", "\\selectfont", "\\resetfontparameters", "\\normalfont",
            "\\rmfamily", "\\sffamily", "\\ttfamily", "\\bfseries", "\\mdseries",
            "\\upshape", "\\itshape", "\\slshape", "\\scshape",
            "\\sloppy", "\\fussy", "\\raggedbottom", "\\flushbottom",
            "\\columnsep", "\\columnwidth", "\\textwidth", "\\linewidth",
            "\\pagewidth", "\\paperwidth", "\\paperheight", "\\textheight",
            "\\unitlength", "\\tabcolsep", "\\arraycolsep", "\\arrayrulewidth",
            "\\doublerulesep", "\\arraystretch",
        ];
        for prefix in &skip_cmds {
            if remaining.starts_with(prefix) {
                self.position += prefix.len();
                self.skip_whitespace_and_comments();
                // Consume any braced arguments
                while self.position < self.content.len() && self.content[self.position..].starts_with('{') {
                    let _ = self.parse_braced_content();
                    self.skip_whitespace_and_comments();
                }
                // Consume optional [...] argument
                if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
                    self.position += 1;
                    let _ = self.read_until(']');
                    self.position += 1;
                }
                return None;
            }
        }

        // Counter formatting commands — consume and produce empty text
        let counter_cmds = [
            "\\value", "\\arabic", "\\roman", "\\Roman", "\\alph", "\\Alph",
        ];
        for prefix in &counter_cmds {
            if remaining.starts_with(prefix) {
                self.position += prefix.len();
                self.skip_whitespace_and_comments();
                let _ = self.parse_braced_content();
                return Some(TexElement::Text(String::new()));
            }
        }

        // \the<counter> — consume
        if remaining.starts_with("\\the") {
            self.position += 4;
            // Consume following alphabetic command name
            while self.position < self.content.len() {
                let ch = self.content[self.position..].chars().next().unwrap();
                if ch.is_alphabetic() {
                    self.position += ch.len_utf8();
                } else {
                    break;
                }
            }
            return Some(TexElement::Text(String::new()));
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
                return Some(TexElement::Command {
                    name: name.to_string(),
                    args: vec![],
                });
            }
        }

        // Centering declaration
        if remaining.starts_with("\\centering") {
            self.position += "\\centering".len();
            return Some(TexElement::Command {
                name: "centering".to_string(),
                args: vec![],
            });
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
                return Some(TexElement::Command {
                    name: name.to_string(),
                    args: vec![],
                });
            }
        }

        // Line break: \\  (possibly followed by [length] or *)
        if remaining.starts_with("\\\\") {
            self.position += 2;
            // Optional * (prevents page break)
            if self.position < self.content.len() && self.content[self.position..].starts_with('*') {
                self.position += 1;
            }
            // Optional [length] extra vertical space
            self.skip_whitespace_and_comments();
            if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
                self.position += 1;
                let _ = self.read_until(']');
                self.position += 1;
            }
            return Some(TexElement::LineBreak);
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

    /// Parse a `\begin{figure} … \end{figure}` block.
    /// Returns the inner content as a `Center` element.
    fn parse_figure(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{figure}";
        let body_start = self.position;
        let body_end = self.content[self.position..].find(end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::Center(elements))
    }

    /// Parse a `\begin{flushleft} … \end{flushleft}` block.
    fn parse_flushleft(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{flushleft}";
        let body_start = self.position;
        let body_end = self.content[self.position..].find(end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::FlushLeft(elements))
    }

    /// Parse a `\begin{flushright} … \end{flushright}` block.
    fn parse_flushright(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{flushright}";
        let body_start = self.position;
        let body_end = self.content[self.position..].find(end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::FlushRight(elements))
    }

    /// Parse a `\begin{minipage}[alignment]{width} … \end{minipage}` block.
    fn parse_minipage(&mut self) -> Option<TexElement> {
        // Skip optional [alignment] argument
        self.skip_whitespace_and_comments();
        if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
            self.position += 1;
            let _ = self.read_until(']');
            self.position += 1;
        }
        // Skip {width} argument
        self.skip_whitespace_and_comments();
        if self.position < self.content.len() && self.content[self.position..].starts_with('{') {
            let _ = self.parse_braced_content();
        }

        let end_marker = "\\end{minipage}";
        let body_start = self.position;
        let body_end = self.content[self.position..].find(end_marker)?;
        let body = self.content[body_start..body_start + body_end].to_string();
        self.position = body_start + body_end + end_marker.len();

        let mut inner_parser = TexParser::new(body);
        let elements = inner_parser.parse();
        Some(TexElement::Center(elements))
    }

    /// Parse a `\begin{displaymath} … \end{displaymath}` block.
    fn parse_displaymath_env(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{displaymath}");
        self.position += "\\end{displaymath}".len();
        Some(TexElement::MathDisplay(content.trim().to_string()))
    }

    /// Parse a `\begin{math} … \end{math}` block (inline math).
    fn parse_math_env(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{math}");
        self.position += "\\end{math}".len();
        Some(TexElement::MathInline(content.trim().to_string()))
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

    /// Parse a `\begin{thebibliography} … \end{thebibliography}` block.
    fn parse_thebibliography(&mut self) -> Option<TexElement> {
        // Skip optional argument {number}
        self.skip_whitespace_and_comments();
        if self.content[self.position..].starts_with('{')
            && let Some((_inner, end)) = extract_braced(&self.content, self.position)
        {
            self.position = end;
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
            "description" => self.parse_description(),
            "enumerate" => self.parse_itemize(true),
            "theorem" | "lemma" | "proof" | "definition" | "corollary" | "proposition"
            | "remark" | "example" => self.parse_theorem(&env_name),
            "equation" => self.parse_equation(),
            "equation*" => self.parse_equation(),
            "align" => self.parse_math_lines(MathLineKind::Align),
            "align*" => self.parse_math_lines(MathLineKind::Align),
            "gather" => self.parse_math_lines(MathLineKind::Gather),
            "gather*" => self.parse_math_lines(MathLineKind::Gather),
            "multline" => self.parse_math_lines(MathLineKind::Multline),
            "multline*" => self.parse_math_lines(MathLineKind::Multline),
            "cases" => self.parse_math_lines(MathLineKind::Cases),
            "lstlisting" => self.parse_lstlisting(),
            "verbatim" => self.parse_lstlisting(),
            "tabular" => self.parse_tabular(),
            "table" => self.parse_table(),
            "thebibliography" => self.parse_thebibliography(),
            "center" => self.parse_center(),
            "quote" | "quotation" => self.parse_quote(&env_name),
            "abstract" => self.parse_abstract(),
            "figure" => self.parse_figure(),
            "figure*" => self.parse_figure(),
            "flushleft" => self.parse_flushleft(),
            "flushright" => self.parse_flushright(),
            "minipage" => self.parse_minipage(),
            "displaymath" => self.parse_displaymath_env(),
            "math" => self.parse_math_env(),
            "eqnarray" => self.parse_math_lines(MathLineKind::Align),
            "eqnarray*" => self.parse_math_lines(MathLineKind::Align),
            "split" => self.parse_math_lines(MathLineKind::Align),
            "aligned" => self.parse_math_lines(MathLineKind::Align),
            "gathered" => self.parse_math_lines(MathLineKind::Gather),
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
        let end_marker = if ordered {
            "\\end{enumerate}"
        } else {
            "\\end{itemize}"
        };

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

        Some(TexElement::ItemList {
            ordered,
            labels,
            items,
        })
    }

    /// Parse `\begin{description} … \end{description}`.
    ///
    /// Each `\item[term]` produces a (term, body) pair. `\item` without
    /// a label is permitted and gets an empty term.
    fn parse_description(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{description}";
        let mut items: Vec<DescItem> = Vec::new();

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

                let term = if self.content[self.position..].starts_with('[') {
                    self.position += 1;
                    let label_text = self.read_until(']');
                    self.position += 1;
                    self.skip_whitespace_and_comments();
                    label_text
                } else {
                    String::new()
                };

                let mut body: Vec<TexElement> = Vec::new();
                while self.position < self.content.len() {
                    let remaining = &self.content[self.position..];

                    if remaining.starts_with("\\item") || remaining.starts_with(end_marker) {
                        break;
                    }

                    if let Some(elem) = self.parse_next() {
                        body.push(elem);
                    } else {
                        break;
                    }
                }

                items.push(DescItem { term, body });
            } else {
                self.position += 1;
            }
        }

        Some(TexElement::DescriptionList { items })
    }

    /// Parse `\begin{equation} … \end{equation}`.
    fn parse_equation(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{equation}");
        self.position += "\\end{equation}".len();

        Some(TexElement::MathDisplay(content.trim().to_string()))
    }

    /// Parse a theorem-like environment (`theorem`, `lemma`, `proof`,
    /// `definition`, `corollary`, `proposition`, `remark`, `example`).
    ///
    /// The optional `[title]` argument on the begin line is captured as
    /// `title`; otherwise the body is rendered with just the kind name.
    fn parse_theorem(&mut self, kind: &str) -> Option<TexElement> {
        // Optional `[title]` argument.
        self.skip_whitespace_and_comments();
        let mut title: Option<String> = None;
        if self.content[self.position..].starts_with('[') {
            self.position += 1;
            let t = self.read_until(']');
            self.position += 1;
            self.skip_whitespace_and_comments();
            if !t.is_empty() {
                title = Some(t);
            }
        }

        let end_marker = format!("\\end{{{kind}}}");
        let body = self.read_until_str(&end_marker);
        self.position += end_marker.len();

        let mut inner = TexParser::new(body);
        let body_elements = inner.parse();
        Some(TexElement::Theorem {
            kind: kind.to_string(),
            title,
            body: body_elements,
        })
    }

    /// Parse a multi-line math environment (`align`, `gather`, `multline`,
    /// `cases`, and their starred variants).
    ///
    /// The body is split on `\\`. For each non-empty line:
    /// - `align`/`gather`/`multline`: `&` alignment markers are stripped.
    /// - `cases`: the `&` separator splits the value from the condition;
    ///   each line is rendered as `{ value  if  condition`.
    fn parse_math_lines(&mut self, kind: MathLineKind) -> Option<TexElement> {
        let env_name = match kind {
            MathLineKind::Align => "align",
            MathLineKind::Gather => "gather",
            MathLineKind::Multline => "multline",
            MathLineKind::Cases => "cases",
        };
        // Try the explicit name first, then its starred variant.
        let end_marker = format!("\\end{{{env_name}}}");
        let body = if self.content[self.position..].contains(&end_marker) {
            let body = self.read_until_str(&end_marker);
            self.position += end_marker.len();
            body
        } else {
            let starred = format!("\\end{{{env_name}*}}");
            if self.content[self.position..].contains(&starred) {
                let body = self.read_until_str(&starred);
                self.position += starred.len();
                body
            } else {
                return None;
            }
        };

        Some(Self::build_math_lines(&body, kind))
    }

    /// Build a `MathLines` element from a math environment body, splitting
    /// on `\\` and formatting each line for its environment kind. Pure
    /// function — does not touch `self`.
    fn build_math_lines(body: &str, kind: MathLineKind) -> TexElement {
        let mut lines: Vec<String> = Vec::new();
        for raw_line in body.split("\\\\") {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            let formatted = match kind {
                MathLineKind::Cases => Self::format_cases_line(line),
                MathLineKind::Align => line.replace('&', " & "),
                _ => line.replace('&', "  "),
            };
            lines.push(formatted);
        }

        if lines.is_empty() {
            return TexElement::MathDisplay(String::new());
        }
        if lines.len() == 1 {
            return TexElement::MathDisplay(lines.into_iter().next().unwrap());
        }
        TexElement::MathLines { lines, kind }
    }

    /// Format a single `\begin{cases}` line: `{ value  if  condition`.
    /// If `rhs` already starts with `if`, the connector is omitted.
    fn format_cases_line(line: &str) -> String {
        let parts: Vec<&str> = line.split('&').collect();
        let lhs = parts.first().copied().unwrap_or("").trim();
        let rhs = parts.get(1).copied().unwrap_or("").trim();
        if rhs.is_empty() {
            format!("{{ {lhs}")
        } else if rhs.starts_with("\\text{if") || rhs.to_ascii_lowercase().starts_with("if ") {
            format!("{{ {lhs}  {rhs}")
        } else {
            format!("{{ {lhs}  if  {rhs}")
        }
    }

    /// Handle display-math content (`\[...\]`) that contains a `\begin{cases}`
    /// block by splitting it into: prefix text + MathLines(cases) + suffix text.
    /// Since `parse_next` returns a single element, we fold the prefix onto
    /// the first case line so the cases environment renders with leading
    /// context.
    fn split_display_math_with_cases(&mut self, content: &str) -> Option<TexElement> {
        self.split_math_with_cases(content, true)
    }

    /// Same as `split_display_math_with_cases`, for inline math `\(`...`\)`.
    fn split_inline_math_with_cases(&mut self, content: &str) -> Option<TexElement> {
        self.split_math_with_cases(content, false)
    }

    fn split_math_with_cases(&mut self, content: &str, _is_display: bool) -> Option<TexElement> {
        let begin = "\\begin{cases}";
        let end = "\\end{cases}";
        let begin_idx = content.find(begin)?;
        let after_begin = begin_idx + begin.len();
        let end_rel = content[after_begin..].find(end)?;
        let cases_body = &content[after_begin..after_begin + end_rel];

        let prefix = content[..begin_idx].trim();
        let suffix = content[after_begin + end_rel + end.len()..].trim();

        // Format the cases block as if encountered at top level.
        let cases_elem = Self::build_math_lines(cases_body, MathLineKind::Cases);

        // Merge prefix (and suffix) into the cases element so the renderer
        // sees a single element rather than a sequence.
        match cases_elem {
            TexElement::MathDisplay(text) => {
                let mut merged = String::new();
                if !prefix.is_empty() {
                    merged.push_str(prefix);
                    merged.push(' ');
                }
                merged.push_str(&text);
                if !suffix.is_empty() {
                    merged.push(' ');
                    merged.push_str(suffix);
                }
                Some(TexElement::MathDisplay(merged))
            }
            TexElement::MathLines { mut lines, kind } => {
                if !prefix.is_empty() {
                    if let Some(first) = lines.first_mut() {
                        let mut combined = String::with_capacity(prefix.len() + first.len() + 1);
                        combined.push_str(prefix);
                        combined.push(' ');
                        combined.push_str(first);
                        *first = combined;
                    } else {
                        lines.push(prefix.to_string());
                    }
                }
                if !suffix.is_empty() {
                    if let Some(last) = lines.last_mut() {
                        last.push(' ');
                        last.push_str(suffix);
                    } else {
                        lines.push(suffix.to_string());
                    }
                }
                Some(TexElement::MathLines { lines, kind })
            }
            other => Some(other),
        }
    }

    /// Parse `\begin{lstlisting}` or `\begin{verbatim}`.
    fn parse_lstlisting(&mut self) -> Option<TexElement> {
        // Strip the optional `[language=...]` argument on the begin line.
        self.skip_whitespace_and_comments();
        if self.content[self.position..].starts_with('[') {
            self.position += 1;
            let _ = self.read_until(']');
            self.position += 1;
            self.skip_whitespace_and_comments();
        }
        // `verbatim` uses the same closing marker as `lstlisting`.
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
            self.position += self.content[self.position..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
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
mod tests;
