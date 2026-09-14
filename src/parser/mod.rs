//! LaTeX parser - converts raw LaTeX source into structured elements.
//!
//! Handles document structure, environments, commands, inline math,
//! display math, lists, tables, and metadata extraction.

use crate::table::Table;
use crate::utils::extract_braced;
use serde::Serialize;
use std::path::{Path, PathBuf};

mod accents;
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
    Citation { keys: Vec<String>, kind: String },
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
    /// Acronym/glossary entries from `\newacronym`/`\newglossaryentry`:
    /// key → (short form, long form).
    acronyms: std::collections::HashMap<String, (String, String)>,
}

/// Scan raw source for `\newacronym{key}{short}{long}` (with optional
/// `[...]` before the key) and `\newglossaryentry{key}{name=..., ...}`
/// so `\gls`/`\acr*` commands can render real terms.
fn extract_acronyms(content: &str) -> std::collections::HashMap<String, (String, String)> {
    let mut map = std::collections::HashMap::new();
    let mut rest = content;
    while let Some(pos) = rest.find("\\newacronym") {
        rest = &rest[pos + "\\newacronym".len()..];
        let mut fields = Vec::new();
        let mut cursor = rest.trim_start();
        // Optional [...] argument(s), then three {...} groups.
        while let Some(stripped) = cursor.strip_prefix('[') {
            if let Some(end) = stripped.find(']') {
                cursor = stripped[end + 1..].trim_start();
            } else {
                break;
            }
        }
        let mut ok = true;
        for _ in 0..3 {
            let Some(stripped) = cursor.strip_prefix('{') else {
                ok = false;
                break;
            };
            let Some(end) = stripped.find('}') else {
                ok = false;
                break;
            };
            fields.push(stripped[..end].to_string());
            cursor = stripped[end + 1..].trim_start();
        }
        if ok && fields.len() == 3 {
            map.insert(fields[0].clone(), (fields[1].clone(), fields[2].clone()));
        }
    }
    rest = content;
    while let Some(pos) = rest.find("\\newglossaryentry") {
        rest = &rest[pos + "\\newglossaryentry".len()..];
        let cursor = rest.trim_start();
        let Some(stripped) = cursor.strip_prefix('{') else {
            continue;
        };
        let Some(key_end) = stripped.find('}') else {
            continue;
        };
        let key = stripped[..key_end].to_string();
        let Some(opts) = stripped[key_end + 1..].strip_prefix('{') else {
            continue;
        };
        let Some(opts_end) = opts.find('}') else {
            continue;
        };
        // `name=Term` is the display form; the long description is the
        // rest of the option list up to the first top-level comma.
        let opts_body = &opts[..opts_end];
        let name = opts_body
            .split(',')
            .find_map(|f| f.trim().strip_prefix("name="))
            .map(str::trim)
            .unwrap_or("");
        if !name.is_empty() {
            map.insert(key, (name.to_string(), name.to_string()));
        }
    }
    map
}

impl TexParser {
    /// Create a new parser for the given LaTeX source.
    /// Macro definitions (`\newcommand`, `\def`) are extracted and
    /// all macro calls are expanded before structured parsing begins.
    pub fn new(content: String) -> Self {
        let mut store = crate::macros::MacroStore::new();
        let stripped = store.extract_definitions(&content);
        let expanded = store.expand_all(&stripped);
        let acronyms = extract_acronyms(&expanded);
        Self {
            content: expanded,
            position: 0,
            plugins: None,
            base_dir: None,
            search_paths: Vec::new(),
            acronyms,
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
                // Source-adjacent text concatenates without a space
                // (TeX semantics): a symbol command like `\ss` inside
                // a word must not split it into "Stra ß e". Line and
                // sentence boundaries are already preserved inside the
                // Text content itself (parse_text keeps `\n`).
                match (elements.last_mut(), &element) {
                    (Some(TexElement::Text(prev)), TexElement::Text(next)) => {
                        prev.push_str(next);
                    }
                    _ => elements.push(element),
                }
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
        self.skip_comments();

        if self.position >= self.content.len() {
            return None;
        }

        // Terminate parsing at the end of the document body.
        if self.content[self.position..].starts_with("\\end{document}") {
            self.position = self.content.len();
            return None;
        }

        // Bare brace groups: `{}` terminates a command scope
        // (`\LaTeX{} rocks`); a non-empty group scopes its contents,
        // which are parsed normally.
        if self.content[self.position..].starts_with('{') {
            let base = self.position;
            let mut depth = 0usize;
            let mut close = None;
            for (i, ch) in self.content[base..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(base + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            self.position += 1; // consume '{'
            let limit = close.unwrap_or(self.content.len());
            while self.position < limit {
                let before = self.position;
                self.skip_comments();
                if self.content[self.position..].starts_with('}') {
                    self.position += 1;
                    return None;
                }
                match self.parse_next() {
                    Some(element) => return Some(element),
                    None => {
                        if self.position == before {
                            self.position += 1;
                        }
                    }
                }
            }
            if close.is_some() && self.content[self.position..].starts_with('}') {
                self.position += 1;
            }
            return None;
        }

        // Ordered dispatch phases. Order is significant: many command
        // prefixes overlap (for example `\cite` must be checked after
        // `\citeauthor`). A phase returning `None` falls through to the
        // next phase ONLY while the position is unchanged; if the phase
        // consumed input (skip commands, `\input` splicing, unknown
        // environments), dispatch stops and the driver re-enters
        // `parse_next` so all phases see the updated content again.
        macro_rules! dispatch {
            ($phase:ident) => {{
                let before = self.position;
                match self.$phase() {
                    Some(element) => return Some(element),
                    None if self.position > before => return None,
                    None => {}
                }
            }};
        }
        dispatch!(try_structure_and_sections);
        dispatch!(try_inline_and_citations);
        dispatch!(try_math_delimiters);
        dispatch!(try_text_accents);
        dispatch!(try_flow_and_breaks);
        dispatch!(try_text_symbols);
        dispatch!(try_links_and_text_cmds);
        dispatch!(try_spacing_and_files);
        dispatch!(try_bibliography_and_skips);
        dispatch!(try_misc_commands);

        // Fallback: plugin commands, unknown commands, blank lines, text.
        let remaining = &self.content[self.position..];
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

    /// Parse document structure: sections, title block, TOC placeholders, environments.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_structure_and_sections(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
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

        None
    }

    /// Parse inline styling, citations, labels and cross-references.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_inline_and_citations(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
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

        if remaining.starts_with("\\nocite") {
            return self.parse_nocite();
        }

        // Biblatex citation variants — must be checked before \cite
        // since \citeauthor, \citeyear, \citetitle start with \cite
        if remaining.starts_with("\\textcite") {
            return self.parse_cite_variant("\\textcite");
        }
        if remaining.starts_with("\\parencite") {
            return self.parse_cite_variant("\\parencite");
        }
        if remaining.starts_with("\\footcite") {
            return self.parse_cite_variant("\\footcite");
        }
        if remaining.starts_with("\\citeauthor") {
            return self.parse_cite_variant("\\citeauthor");
        }
        if remaining.starts_with("\\citeyear") {
            return self.parse_cite_variant("\\citeyear");
        }
        if remaining.starts_with("\\citetitle") {
            return self.parse_cite_variant("\\citetitle");
        }
        if remaining.starts_with("\\fullcite") {
            return self.parse_cite_variant("\\fullcite");
        }

        if remaining.starts_with("\\cite") {
            return self.parse_cite();
        }

        if remaining.starts_with("\\label") {
            return self.parse_label();
        }

        if remaining.starts_with("\\eqref")
            || remaining.starts_with("\\autoref")
            || remaining.starts_with("\\nameref")
            || remaining.starts_with("\\cref")
            || remaining.starts_with("\\Cref")
            || (remaining.starts_with("\\ref") && !remaining.starts_with("\\reflectbox"))
        {
            return self.parse_ref();
        }

        if remaining.starts_with("\\pageref") {
            return self.parse_pageref();
        }

        // Glossaries / acronyms — render from the scanned definitions
        // (`extract_acronyms`); unknown keys fall back to the key text.
        if remaining.starts_with("\\gls{")
            || remaining.starts_with("\\glspl{")
            || remaining.starts_with("\\Gls{")
            || remaining.starts_with("\\GLS{")
            || remaining.starts_with("\\acrshort{")
            || remaining.starts_with("\\acrlong{")
            || remaining.starts_with("\\acrfull{")
        {
            let name_end = remaining.find('{').unwrap_or(1);
            let name = remaining[1..name_end].to_string();
            self.position += name_end + 1;
            let key = self.read_until('}');
            self.position += 1;
            let (short, long) = match self.acronyms.get(&key) {
                Some((s, l)) => (s.clone(), l.clone()),
                None => (key.clone(), key),
            };
            let text = match name.as_str() {
                "acrlong" => long,
                "acrfull" => format!("{long} ({short})"),
                "Gls" => {
                    let mut c = short.chars();
                    match c.next() {
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        None => short,
                    }
                }
                "GLS" => short.to_uppercase(),
                "glspl" => format!("{short}s"),
                _ => short,
            };
            return Some(TexElement::Text(text));
        }
        // Definitions appearing in the body are consumed silently (the
        // pre-parse scan already harvested them). Consume the optional
        // [...] and ALL braced groups, or leftovers leak as text.
        if remaining.starts_with("\\newacronym")
            || remaining.starts_with("\\newglossaryentry")
        {
            self.position += if remaining.starts_with("\\newacronym") {
                "\\newacronym".len()
            } else {
                "\\newglossaryentry".len()
            };
            self.skip_whitespace_and_comments();
            if self.content[self.position..].starts_with('[') {
                self.position += 1;
                let _ = self.read_until(']');
                self.position += 1;
                self.skip_whitespace_and_comments();
            }
            while self.content[self.position..].starts_with('{') {
                let _ = self.parse_braced_content();
                self.skip_whitespace_and_comments();
            }
            return None;
        }

        // \todo[...]{note} — todonotes margin note: consumed, no output.
        if remaining.starts_with("\\todo") {
            self.position += "\\todo".len();
            self.skip_whitespace_and_comments();
            if self.content[self.position..].starts_with('[') {
                self.position += 1;
                let _ = self.read_until(']');
                self.position += 1;
                self.skip_whitespace_and_comments();
            }
            if self.content[self.position..].starts_with('{') {
                let _ = self.parse_braced_content();
            }
            return None;
        }

        // \footref{key} — a reference to a footnote label.
        if remaining.starts_with("\\footref{") {
            self.position += "\\footref".len();
            self.skip_whitespace_and_comments();
            let key = self.parse_braced_content().unwrap_or_default();
            return Some(TexElement::Ref { key });
        }

        None
    }

    /// Parse dollar-delimited inline math and TeX-style display/inline math.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_math_delimiters(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
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


        None
    }

    /// Parse text accent commands (`\'e`, `\"{u}`, `\c{c}`, `\v{s}`,
    /// `\H{o}`, `\t{oo}`, ...) into composed Unicode text.
    ///
    /// The control sequence NAME is extracted first so `\cline` (name
    /// `cline`) never matches the `\c` accent, mirroring TeX's
    /// longest-letters tokenization.
    fn try_text_accents(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        if !remaining.starts_with('\\') {
            return None;
        }
        let after_backslash = &remaining[1..];
        let first = after_backslash.chars().next()?;
        // Name: a run of letters, or a single non-letter character.
        let name_len: usize = if first.is_alphabetic() {
            after_backslash
                .char_indices()
                .take_while(|(_, c)| c.is_alphabetic())
                .map(|(_, c)| c.len_utf8())
                .sum()
        } else {
            1
        };
        if !accents::is_accent_command(&after_backslash[..name_len]) {
            return None;
        }
        let accent = after_backslash[..name_len].chars().next()?;
        self.position += 1 + name_len;

        // Argument: braced group, a single attached letter, or nothing.
        // A `\i`/`\j` dotless base inside the argument resolves first
        // (e.g. `\"{\i}` → "ï").
        let resolve_base = |group: &str| -> String {
            if let Some(rest) = group.strip_prefix("\\i") {
                format!("ı{rest}")
            } else if let Some(rest) = group.strip_prefix("\\j") {
                format!("ȷ{rest}")
            } else {
                group.to_string()
            }
        };
        let remaining = &self.content[self.position..];
        if remaining.starts_with('{') {
            self.position += 1;
            let group = self.read_until('}');
            self.position += 1; // closing brace
            let group = resolve_base(&group);
            let mut chars = group.chars();
            let out = match chars.next() {
                Some(base) => {
                    let mut text = accents::compose_accent(accent, base);
                    text.push_str(chars.as_str());
                    text
                }
                None => String::new(),
            };
            return Some(TexElement::Text(out));
        }
        if let Some(base) = remaining.chars().next() {
            if base.is_alphabetic() {
                self.position += base.len_utf8();
                return Some(TexElement::Text(accents::compose_accent(accent, base)));
            }
        }
        // Accent with no argument (e.g. `\'` before whitespace).
        Some(TexElement::Text(String::new()))
    }

    /// Parse page/paragraph flow controls, alignment declarations and today.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_flow_and_breaks(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
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

        None
    }

    /// Parse special text characters and logo commands.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_text_symbols(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        // Special text characters (insert literal symbols)
        let text_chars = [
            // Escaped special characters — must precede other matching
            // so `\%` yields a literal percent instead of a comment and
            // `\$` does not open math mode.
            ("\\%", "%"),
            ("\\&", "&"),
            ("\\$", "$"),
            ("\\#", "#"),
            ("\\_", "_"),
            ("\\{", "{"),
            ("\\}", "}"),
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
            ("\\textexclamdown", "¡"),
            ("\\textquestiondown", "¿"),
            ("\\textquotedblleft", "„"),
            ("\\textquotedblright", "''"),
            ("\\textquoteleft", "‘"),
            ("\\textquoteright", "’"),
            ("\\textendash", "–"),
            ("\\textemdash", "—"),
            ("\\textbullet", "•"),
            ("\\textperiodcentered", "·"),
            ("\\guillemotleft", "«"),
            ("\\guillemotright", "»"),
            ("\\guilsinglleft", "‹"),
            ("\\guilsinglright", "›"),
            ("\\textvisiblespace", "␣"),
            ("\\textcompwordmark", "\u{ad}"),
            ("\\euro", "€"),
            ("\\textunderscore", "_"),
            ("\\textdegree", "°"),
            ("\\textcelsius", "℃"),
            ("\\textmu", "µ"),
            ("\\textohm", "Ω"),
            ("\\textnumero", "№"),
            ("\\textestimated", "℮"),
            ("\\textbrokenbar", "¦"),
            ("\\textordfeminine", "ª"),
            ("\\textordmasculine", "º"),
            ("\\textacutedbl", "\u{34}"),
            ("\\textgravedbl", "\u{36}"),
            ("\\texttildelow", "\u{3f}"),
            ("\\textcent", "¢"),
            ("\\texteuro", "€"),
            ("\\textyen", "¥"),
            ("\\textcurrency", "¤"),
        ];
        for (prefix, ch) in &text_chars {
            if let Some(after) = remaining.strip_prefix(prefix) {
                // Guard only for control WORDS (alphabetic names):
                // `\i` must not match `\it`. Control SYMBOLS (`\{`,
                // `\%`, ...) are single-char names — `\{b` is `\{` + b.
                if prefix.ends_with(|c: char| c.is_ascii_alphabetic())
                    && after.starts_with(|c: char| c.is_alphabetic())
                {
                    continue;
                }
                self.position += prefix.len();
                // TeX tokenization: the space after a control WORD
                // (alphabetic name) is the name terminator and is
                // consumed; `\ss e` is "ße", while `\% x` keeps the
                // space (non-letter names have no terminator).
                if prefix.ends_with(|c: char| c.is_ascii_alphabetic()) {
                    self.skip_whitespace_and_comments();
                }
                return Some(TexElement::Text(ch.to_string()));
            }
        }

        None
    }

    /// Parse URLs, inline verbatim, and simple braced text commands.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_links_and_text_cmds(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        // URL
        if remaining.starts_with("\\url{") {
            return self.parse_url();
        }
        if remaining.starts_with("\\href{") {
            return self.parse_href();
        }
        if remaining.starts_with("\\nolinkurl{") {
            return self.parse_simple_braced_command("nolinkurl", 11);
        }

        // \verb<delim>text<delim> — inline verbatim
        if remaining.starts_with("\\verb") && !remaining.starts_with("\\verbatim") {
            return self.parse_verb();
        }
        if remaining.starts_with("\\lstinline") {
            return self.parse_verb();
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

        // Additional text commands
        if remaining.starts_with("\\textcircled{") {
            return self.parse_simple_braced_command("textcircled", 13);
        }
        if remaining.starts_with("\\hl{") {
            return self.parse_simple_braced_command("hl", 4);
        }
        if remaining.starts_with("\\st{") {
            return self.parse_simple_braced_command("st", 4);
        }
        if remaining.starts_with("\\uline{") {
            return self.parse_simple_braced_command("uline", 7);
        }
        if remaining.starts_with("\\uuline{") {
            return self.parse_simple_braced_command("uuline", 8);
        }
        if remaining.starts_with("\\uwave{") {
            return self.parse_simple_braced_command("uwave", 7);
        }
        if remaining.starts_with("\\dotuline{") {
            return self.parse_simple_braced_command("dotuline", 10);
        }

        // \color{name} — color declaration (switches color for subsequent text)
        if remaining.starts_with("\\color{") {
            return self.parse_simple_braced_command("color", 7);
        }

        // \normalcolor — reset color declaration
        if remaining.starts_with("\\normalcolor") {
            self.position += "\\normalcolor".len();
            return Some(TexElement::Command {
                name: "normalcolor".to_string(),
                args: vec![],
            });
        }

        // \hspace* already handled below, but also add \phantom* variants
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
        if remaining.starts_with("\\smash{") {
            return self.parse_simple_braced_command("smash", 7);
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

        None
    }

    /// Parse spacing commands and file inclusion.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_spacing_and_files(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        // Spacing commands
        let spacing_cmds = [
            ("\\hfill", "hfill"),
            ("\\vfill", "vfill"),
            ("\\hrulefill", "hrulefill"),
            ("\\dotfill", "dotfill"),
            ("\\fill", "fill"),
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
        if let Some(after) = remaining.strip_prefix("\\input") {
            // Ensure we don't match \input* or other longer commands
            if after.starts_with('{') || after.starts_with(|c: char| c.is_whitespace()) {
                return self.parse_input();
            }
        }

        // \include{filename} — like \input but with page breaks
        if remaining.starts_with("\\include{") {
            return self.parse_include();
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

        None
    }

    /// Parse bibliography commands, skip commands and counter formatting.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_bibliography_and_skips(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        // Bibliography commands
        if remaining.starts_with("\\bibliography{") {
            return self.parse_simple_braced_command("bibliography", 14);
        }
        if remaining.starts_with("\\bibliographystyle{") {
            return self.parse_simple_braced_command("bibliographystyle", 19);
        }
        // \addbibresource{file.bib} — biblatex resource declaration
        if remaining.starts_with("\\addbibresource{") {
            return self.parse_simple_braced_command("addbibresource", 16);
        }
        // \printbibliography — biblatex bibliography output (no args or optional args)
        if remaining.starts_with("\\printbibliography")
            && !remaining.starts_with("\\printbibliography{")
        {
            self.position += "\\printbibliography".len();
            // Consume optional [...] args
            self.skip_whitespace_and_comments();
            while self.position < self.content.len()
                && self.content[self.position..].starts_with('[')
            {
                // Skip until matching ]
                let rest = &self.content[self.position..];
                if let Some(end_rel) = rest.find(']') {
                    self.position += end_rel + 1;
                } else {
                    break;
                }
                self.skip_whitespace_and_comments();
            }
            return Some(TexElement::Command {
                name: "printbibliography".to_string(),
                args: vec![],
            });
        }
        if remaining.starts_with("\\printbibliography{") {
            return self.parse_simple_braced_command("printbibliography", 19);
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

        // \thanks{text} — title-page acknowledgment, rendered as a footnote
        if remaining.starts_with("\\thanks{") {
            self.position += "\\thanks".len();
            self.skip_whitespace_and_comments();
            let text = self.parse_braced_content().unwrap_or_default();
            return Some(TexElement::Footnote { text });
        }

        // \MakeUppercase{...} / \MakeLowercase{...}
        if remaining.starts_with("\\MakeUppercase{") {
            self.position += "\\MakeUppercase".len();
            self.skip_whitespace_and_comments();
            let text = self.parse_braced_content().unwrap_or_default();
            return Some(TexElement::Text(text.to_uppercase()));
        }
        if remaining.starts_with("\\MakeLowercase{") {
            self.position += "\\MakeLowercase".len();
            self.skip_whitespace_and_comments();
            let text = self.parse_braced_content().unwrap_or_default();
            return Some(TexElement::Text(text.to_lowercase()));
        }

        // \and — author separator (`\author{A \and B}`); word-boundary
        // guarded so commands like `\andothers` never match.
        if let Some(after) = remaining.strip_prefix("\\and")
            && !after.starts_with(|c: char| c.is_alphabetic())
        {
            self.position += "\\and".len();
            return Some(TexElement::Text(", ".to_string()));
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

        // \tag{...} and \tag*{...} — equation tags
        if remaining.starts_with("\\tag*{") {
            return self.parse_simple_braced_command("tag*", 6);
        }
        if remaining.starts_with("\\tag{") {
            return self.parse_simple_braced_command("tag", 5);
        }

        // \intertext{...} — text between align lines
        if remaining.starts_with("\\intertext{") {
            return self.parse_simple_braced_command("intertext", 11);
        }
        if remaining.starts_with("\\shortintertext{") {
            return self.parse_simple_braced_command("shortintertext", 15);
        }

        // \reflectbox{...} — reflect content
        if remaining.starts_with("\\reflectbox{") {
            return self.parse_simple_braced_command("reflectbox", 12);
        }

        // \resizebox{width}{height}{content}
        if remaining.starts_with("\\resizebox{") {
            return self.parse_resizebox();
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
            ("\\nolinebreak", "nolinebreak"),
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
            "\\pagestyle", "\\thispagestyle", "\\pagenumbering", "\\definecolor",
            "\\DeclareMathOperator", "\\numberwithin", "\\counterwithin",
            "\\renewcommand", "\\providecommand", "\\DeclareRobustCommand",
            "\\hypersetup", "\\urlstyle", "\\babelhyphenation",
            "\\frenchspacing", "\\nonfrenchspacing",
            "\\thicklines", "\\thinlines",
            "\\baselineskip", "\\topskip", "\\bottomskip",
            "\\parindent", "\\parskip", "\\headheight", "\\headsep",
            "\\footskip", "\\topmargin", "\\bottommargin",
            "\\leftmargin", "\\rightmargin", "\\oddsidemargin", "\\evensidemargin",
            "\\marginparwidth", "\\marginparsep", "\\marginparpush",
            "\\floatsep", "\\intextsep", "\\textfloatsep",
            "\\abovecaptionskip", "\\belowcaptionskip",
            "\\counterwithout",
            "\\let", "\\edef", "\\xdef", "\\global",
            "\\mathversion", "\\restoremathversion",
            "\\sloppypar", "\\endsloppypar",
            "\\nonumber", "\\notag", "\\qedhere",
            "\\theoremstyle", "\\swapnumbers", "\\qedsymbol",
            "\\theoremheaderfont", "\\theorembodyfont", "\\newtheoremstyle",
            "\\settowidth", "\\settodepth", "\\settoheight",
            "\\sbox", "\\savebox", "\\usebox", "\\adjustbox",
            "\\span", "\\hide",
            "\\captionof", "\\subfloat", "\\subcaption", "\\subcaptionbox",
            "\\stretch",
            "\\widowpenalty", "\\clubpenalty", "\\interlinepenalty",
            "\\hyphenpenalty", "\\exhyphenpenalty", "\\brokenpenalty",
            "\\floatingpenalty", "\\hyphenation", "\\tolerance",
            "\\pretolerance", "\\emergencystretch", "\\hbadness", "\\vbadness",
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
                if let Some(ch) = self.content[self.position..].chars().next() {
                    if ch.is_alphabetic() {
                        self.position += ch.len_utf8();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            return Some(TexElement::Text(String::new()));
        }

        None
    }

    /// Parse footnotes, captions, table helpers, font sizes and line breaks.
    ///
    /// Returns `None` when nothing matched, or when input was consumed
    /// that produces no element (skip commands); both fall through to
    /// the next dispatch phase.
    fn try_misc_commands(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];
        // Footnote
        if remaining.starts_with("\\footnote{") {
            return self.parse_footnote();
        }

        // Caption
        if remaining.starts_with("\\caption{") {
            return self.parse_caption();
        }
        // \caption*{text} — unnumbered caption
        if remaining.starts_with("\\caption*{") {
            return self.parse_simple_braced_command("caption*", 10);
        }

        // Table color commands — \rowcolor{color}, \cellcolor{color}
        if remaining.starts_with("\\rowcolor{") {
            return self.parse_simple_braced_command("rowcolor", 10);
        }
        if remaining.starts_with("\\cellcolor{") {
            return self.parse_simple_braced_command("cellcolor", 11);
        }

        // \multirow{num}{width}{text} — multi-row table cell
        if remaining.starts_with("\\multirow{") {
            self.position += "\\multirow".len();
            self.skip_whitespace_and_comments();
            let num = self.parse_braced_content()?;
            self.skip_whitespace_and_comments();
            let width = self.parse_braced_content()?;
            self.skip_whitespace_and_comments();
            let text = self.parse_braced_content()?;
            return Some(TexElement::Command {
                name: "multirow".to_string(),
                args: vec![num, width, text],
            });
        }

        // \floatplacement — skip with optional args
        if remaining.starts_with("\\floatplacement") {
            self.position += "\\floatplacement".len();
            self.skip_whitespace_and_comments();
            while self.position < self.content.len()
                && self.content[self.position..].starts_with('[')
            {
                let rest = &self.content[self.position..];
                if let Some(end_rel) = rest.find(']') {
                    self.position += end_rel + 1;
                } else {
                    break;
                }
                self.skip_whitespace_and_comments();
            }
            while self.position < self.content.len()
                && self.content[self.position..].starts_with('{')
            {
                let _ = self.parse_braced_content();
                self.skip_whitespace_and_comments();
            }
            return None;
        }
        // \floatbarrier — no args, no output
        if remaining.starts_with("\\floatbarrier") {
            self.position += "\\floatbarrier".len();
            return None;
        }

        // siunitx commands
        if remaining.starts_with("\\sisetup") {
            self.position += "\\sisetup".len();
            self.skip_whitespace_and_comments();
            let _ = self.parse_braced_content();
            return None;
        }
        if remaining.starts_with("\\SIrange{") {
            self.position += "\\SIrange".len();
            self.skip_whitespace_and_comments();
            let start = self.parse_braced_content().unwrap_or_default();
            self.skip_whitespace_and_comments();
            let end = self.parse_braced_content().unwrap_or_default();
            self.skip_whitespace_and_comments();
            let unit = self.parse_braced_content().unwrap_or_default();
            return Some(TexElement::Command {
                name: "SIrange".to_string(),
                args: vec![start, end, unit],
            });
        }
        if remaining.starts_with("\\SI{") {
            self.position += "\\SI".len();
            self.skip_whitespace_and_comments();
            let number = self.parse_braced_content().unwrap_or_default();
            self.skip_whitespace_and_comments();
            let unit = self.parse_braced_content().unwrap_or_default();
            return Some(TexElement::Command {
                name: "SI".to_string(),
                args: vec![number, unit],
            });
        }
        if remaining.starts_with("\\si{") {
            return self.parse_simple_braced_command("si", 4);
        }
        if remaining.starts_with("\\unit{") {
            return self.parse_simple_braced_command("unit", 6);
        }
        if remaining.starts_with("\\num{") {
            return self.parse_simple_braced_command("num", 5);
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

        // Page break commands
        let page_break_cmds = [
            ("\\newpage", "newpage"),
            ("\\clearpage", "clearpage"),
            ("\\pagebreak", "pagebreak"),
            ("\\noindent", "noindent"),
            ("\\indent", "indent"),
            ("\\newline", "newline"),
            ("\\raggedright", "raggedright"),
            ("\\raggedleft", "raggedleft"),
            ("\\onecolumn", "onecolumn"),
            ("\\twocolumn", "twocolumn"),
        ];
        for (prefix, name) in &page_break_cmds {
            if remaining.starts_with(prefix) {
                // Ensure we don't match a prefix of a longer command
                let after = &remaining[prefix.len()..];
                if after.starts_with(|c: char| c.is_alphabetic()) {
                    continue;
                }
                self.position += prefix.len();
                // Optional [length] for \pagebreak and \newline
                self.skip_whitespace_and_comments();
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

        // Margin notes — consume braced argument, produce no output
        if remaining.starts_with("\\marginpar") {
            self.position += "\\marginpar".len();
            self.skip_whitespace_and_comments();
            // Optional [outer note] argument
            if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
                self.position += 1;
                let _ = self.read_until(']');
                self.position += 1;
                self.skip_whitespace_and_comments();
            }
            if self.position < self.content.len() && self.content[self.position..].starts_with('{') {
                let _ = self.parse_braced_content();
            }
            return None;
        }

        // Reverse/normal margin par — skip
        if remaining.starts_with("\\reversemarginpar") || remaining.starts_with("\\normalmarginpar") {
            self.position += if remaining.starts_with("\\reversemarginpar") {
                "\\reversemarginpar".len()
            } else {
                "\\normalmarginpar".len()
            };
            return None;
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

        None
    }

    /// Advance past `%` comments only (not whitespace).
    fn skip_comments(&mut self) {
        while self.position < self.content.len() {
            let remaining = &self.content[self.position..];
            if remaining.starts_with('%') {
                if let Some(newline) = remaining.find('\n') {
                    self.position += newline + 1;
                } else {
                    self.position = self.content.len();
                }
            } else {
                break;
            }
        }
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
        if let Some(plugins) = self.plugins.as_mut() {
            if let Some(elem) = plugins.try_command(&name, &args) {
                return Some(elem);
            }
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

        if let Some(plugins) = self.plugins.as_mut() {
            if let Some(elem) = plugins.try_environment(env_name, &body) {
                self.position = body_start + body_end + end_marker.len();
                return Some(elem);
            }
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
                        if let Some(ch) = self.content[self.position..].chars().next() {
                            if ch.is_whitespace() || ch == '\\' {
                                break;
                            }
                            self.position += ch.len_utf8();
                        } else {
                            break;
                        }
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
                    if let Some(ch) = rem.chars().next() {
                        self.position += ch.len_utf8();
                    } else {
                        break;
                    }
                }
                let text = self.content[text_start..self.position].trim().to_string();
                entries.push(BibEntry { key, text });
            } else {
                // Skip unknown content
                if let Some(ch) = remaining.chars().next() {
                    self.position += ch.len_utf8();
                }
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
            "align" => self.parse_math_lines(&env_name, MathLineKind::Align),
            "align*" => self.parse_math_lines(&env_name, MathLineKind::Align),
            "gather" => self.parse_math_lines(&env_name, MathLineKind::Gather),
            "gather*" => self.parse_math_lines(&env_name, MathLineKind::Gather),
            "multline" => self.parse_math_lines(&env_name, MathLineKind::Multline),
            "multline*" => self.parse_math_lines(&env_name, MathLineKind::Multline),
            "cases" => self.parse_math_lines(&env_name, MathLineKind::Cases),
            "lstlisting" => self.parse_lstlisting(&env_name),
            "verbatim" => self.parse_lstlisting(&env_name),
            "tabular" => self.parse_tabular(),
            "tabular*" => self.parse_tabular_star(),
            "tabularx" => self.parse_tabularx(),
            "longtable" | "longtable*" => self.parse_tabular_named("longtable"),
            "array" => self.parse_array_env(),
            "table" => self.parse_table(),
            "table*" => self.parse_table(),
            "thebibliography" => self.parse_thebibliography(),
            "center" => self.parse_center(),
            "quote" | "quotation" => self.parse_quote(&env_name),
            "abstract" => self.parse_abstract(),
            "figure" => self.parse_figure(),
            "figure*" => self.parse_figure(),
            "subfigure" | "subfig" => self.parse_subfigure(&env_name),
            "flushleft" => self.parse_flushleft(),
            "flushright" => self.parse_flushright(),
            "minipage" => self.parse_minipage(),
            "displaymath" => self.parse_displaymath_env(),
            "math" => self.parse_math_env(),
            "eqnarray" | "eqnarray*" | "split" | "aligned" | "alignat" | "alignat*" | "flalign"
            | "flalign*" => self.parse_math_lines(&env_name, MathLineKind::Align),
            "gathered" => self.parse_math_lines(&env_name, MathLineKind::Gather),
            "comment" => self.parse_comment_env(),
            "multicols" => self.parse_multicols(),
            "wrapfigure" => self.parse_wrapfigure(),
            "wraptable" => self.parse_wraptable(),
            "tabbing" => self.parse_tabbing(),
            "algorithm" | "algorithm*" => self.parse_algorithm(&env_name),
            "algorithmic" => self.parse_algorithmic(),
            "alltt" => self.parse_alltt(),
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
    fn parse_math_lines(&mut self, env_name: &str, kind: MathLineKind) -> Option<TexElement> {
        // The end marker MUST derive from the actual environment name:
        // `\begin{eqnarray}` ends at `\end{eqnarray}`, not `\end{align}`
        // (same lesson as the verbatim end-tag bug).
        let end_marker = format!("\\end{{{env_name}}}");
        if !self.content[self.position..].contains(&end_marker) {
            return None;
        }
        let body = self.read_until_str(&end_marker);
        self.position += end_marker.len();

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
            return TexElement::MathDisplay(lines.into_iter().next().unwrap_or_default());
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
    fn parse_lstlisting(&mut self, env_name: &str) -> Option<TexElement> {
        // Strip the optional `[language=...]` argument on the begin line.
        self.skip_whitespace_and_comments();
        if self.content[self.position..].starts_with('[') {
            self.position += 1;
            let _ = self.read_until(']');
            self.position += 1;
            self.skip_whitespace_and_comments();
        }
        let end_marker = format!("\\end{{{env_name}}}");
        let content = self.read_until_str(&end_marker);
        self.position += end_marker.len();

        Some(TexElement::CodeBlock(content.trim().to_string()))
    }

    /// Parse `\begin{tabular} … \end{tabular}`.
    fn parse_tabular(&mut self) -> Option<TexElement> {
        self.parse_tabular_named("tabular")
    }

    /// Parse a `tabular`-family body. The end marker MUST derive from
    /// the actual environment name (`longtable` ends at
    /// `\end{longtable}`), per the shared-end-marker anti-pattern.
    fn parse_tabular_named(&mut self, env_name: &str) -> Option<TexElement> {
        let end_marker = format!("\\end{{{env_name}}}");
        let raw = self.read_until_str(&end_marker);
        self.position += end_marker.len();

        // Extract column spec from the start of raw content: {lc|r}...
        let (spec, body) = if let Some(end) = raw.find('}') {
            (&raw[..=end], &raw[end + 1..])
        } else {
            ("", raw.as_str())
        };

        Some(TexElement::Table(Table::parse(spec, body)))
    }

    /// Parse `\begin{tabular*}{width}{col spec} … \end{tabular*}`.
    fn parse_tabular_star(&mut self) -> Option<TexElement> {
        // Skip the width argument {width}
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        let raw = self.read_until_str("\\end{tabular*}");
        self.position += "\\end{tabular*}".len();
        let (spec, body) = if let Some(end) = raw.find('}') {
            (&raw[..=end], &raw[end + 1..])
        } else {
            ("", raw.as_str())
        };
        Some(TexElement::Table(Table::parse(spec, body)))
    }

    /// Parse `\begin{tabularx}{width}{col spec} … \end{tabularx}`.
    fn parse_tabularx(&mut self) -> Option<TexElement> {
        // Skip the width argument {width}
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        let raw = self.read_until_str("\\end{tabularx}");
        self.position += "\\end{tabularx}".len();
        let (spec, body) = if let Some(end) = raw.find('}') {
            (&raw[..=end], &raw[end + 1..])
        } else {
            ("", raw.as_str())
        };
        Some(TexElement::Table(Table::parse(spec, body)))
    }

    /// Parse `\begin{array}{col spec} … \end{array}` — math array environment.
    fn parse_array_env(&mut self) -> Option<TexElement> {
        let raw = self.read_until_str("\\end{array}");
        self.position += "\\end{array}".len();
        let (spec, body) = if let Some(end) = raw.find('}') {
            (&raw[..=end], &raw[end + 1..])
        } else {
            ("", raw.as_str())
        };
        Some(TexElement::Table(Table::parse(spec, body)))
    }

    /// Parse `\begin{comment} … \end{comment}` — content is silently discarded.
    fn parse_comment_env(&mut self) -> Option<TexElement> {
        self.skip_until("\\end{comment}");
        None
    }

    /// Parse `\begin{multicols}{n} … \end{multicols}` — multi-column layout.
    fn parse_multicols(&mut self) -> Option<TexElement> {
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        let end_marker = "\\end{multicols}";
        let content = self.read_until_str(end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::Command {
            name: "multicols".to_string(),
            args: vec![inner
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join("")],
        })
    }

    /// Parse `\begin{wrapfigure}{pos}{width} … \end{wrapfigure}` — wrapping figure.
    fn parse_wrapfigure(&mut self) -> Option<TexElement> {
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        let end_marker = "\\end{wrapfigure}";
        let content = self.read_until_str(end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::Center(inner))
    }

    /// Parse `\begin{wraptable}{pos}{width} … \end{wraptable}` — wrapping table.
    fn parse_wraptable(&mut self) -> Option<TexElement> {
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        let end_marker = "\\end{wraptable}";
        let content = self.read_until_str(end_marker);
        self.position += end_marker.len();
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

    /// Parse `\begin{tabbing} … \end{tabbing}` — tab stop alignment.
    fn parse_tabbing(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{tabbing}";
        let content = self.read_until_str(end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::Center(inner))
    }

    /// Parse `\begin{algorithm} … \end{algorithm}` — algorithm float.
    fn parse_algorithm(&mut self, env_name: &str) -> Option<TexElement> {
        let end_marker = format!("\\end{{{env_name}}}");
        let content = self.read_until_str(&end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::Center(inner))
    }

    /// Parse `\begin{algorithmic} … \end{algorithmic}` — algorithmic pseudocode.
    fn parse_algorithmic(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{algorithmic}";
        let content = self.read_until_str(end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::Center(inner))
    }

    /// Parse `\begin{alltt} … \end{alltt}` — like verbatim but allows commands.
    fn parse_alltt(&mut self) -> Option<TexElement> {
        let end_marker = "\\end{alltt}";
        let content = self.read_until_str(end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::CodeBlock(inner
            .iter()
            .map(|e| match e {
                TexElement::Text(t) => t.clone(),
                _ => format!("{e:?}"),
            })
            .collect::<Vec<_>>()
            .join("")))
    }

    /// Parse `\begin{subfigure}{width} … \end{subfigure}` — sub-figure container.
    fn parse_subfigure(&mut self, env_name: &str) -> Option<TexElement> {
        // Skip the width argument {width}
        self.skip_whitespace_and_comments();
        let _ = self.parse_braced_content();
        let end_marker = format!("\\end{{{env_name}}}");
        let content = self.read_until_str(&end_marker);
        self.position += end_marker.len();
        let mut sub_parser = TexParser::new(content);
        let inner = sub_parser.parse();
        Some(TexElement::Command {
            name: "subfigure".to_string(),
            args: vec![inner
                .iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join("")],
        })
    }

    /// Parse `\begin{table} … \end{table}` (extracts the inner tabular).
    fn parse_table(&mut self) -> Option<TexElement> {
        let content = self.read_until_str("\\end{table}");
        self.position += "\\end{table}".len();

        // Look for tabular, tabular*, or tabularx within table
        for (begin_marker, end_marker) in &[
            ("\\begin{tabular*}", "\\end{tabular*}"),
            ("\\begin{tabularx}", "\\end{tabularx}"),
            ("\\begin{tabular}", "\\end{tabular}"),
        ] {
            if let Some(tabular_start) = content.find(begin_marker) {
                let after_begin = &content[tabular_start + begin_marker.len()..];
                // For tabular* and tabularx, skip the width arg {width}
                let after_width = if begin_marker.contains('*') || begin_marker.contains('x') {
                    if let Some(width_end) = after_begin.find('}') {
                        &after_begin[width_end + 1..]
                    } else {
                        after_begin
                    }
                } else {
                    after_begin
                };
                let (spec, body_with_end) = if let Some(end) = after_width.find('}') {
                    (&after_width[..=end], &after_width[end + 1..])
                } else {
                    ("", after_width)
                };
                if let Some(tabular_end) = body_with_end.find(end_marker) {
                    let body = &body_with_end[..tabular_end];
                    return Some(TexElement::Table(Table::parse(spec, body)));
                }
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
            if let Some(ch) = self.content[self.position..].chars().next() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    if depth == 0 && delimiter == '}' {
                        break;
                    }
                    if depth > 0 {
                        depth -= 1;
                    }
                } else if ch == delimiter && depth == 0 {
                    break;
                }

                self.position += ch.len_utf8();
            } else {
                break;
            }
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
