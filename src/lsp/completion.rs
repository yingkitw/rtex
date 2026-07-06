//! Completion candidates for LaTeX commands and environments.

/// A completion item returned by the analysis layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexCompletion {
    pub label: String,
    pub insert_text: String,
    pub detail: Option<String>,
    pub kind: CompletionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Command,
    Environment,
}

const COMMANDS: &[(&str, &str)] = &[
    ("documentclass", "Document class declaration"),
    ("usepackage", "Load a package"),
    ("begin", "Start an environment"),
    ("end", "End an environment"),
    ("title", "Document title"),
    ("author", "Document author"),
    ("date", "Document date"),
    ("maketitle", "Render title block"),
    ("section", "Section heading"),
    ("subsection", "Subsection heading"),
    ("subsubsection", "Subsubsection heading"),
    ("paragraph", "Paragraph heading"),
    ("chapter", "Chapter heading"),
    ("part", "Part heading"),
    ("label", "Define a cross-reference label"),
    ("ref", "Reference a label"),
    ("pageref", "Reference a page number"),
    ("cite", "Citation"),
    ("textbf", "Bold text"),
    ("textit", "Italic text"),
    ("texttt", "Monospace text"),
    ("emph", "Emphasized text"),
    ("textcolor", "Colored text"),
    ("includegraphics", "Include an image"),
    ("input", "Include another file"),
    ("tableofcontents", "Table of contents"),
    ("listoffigures", "List of figures"),
    ("listoftables", "List of tables"),
    ("footnote", "Footnote"),
    ("caption", "Caption"),
    ("newcommand", "Define a macro"),
    ("def", "Define a macro (plain TeX)"),
    ("today", "Current date"),
    ("url", "URL"),
    ("hline", "Table horizontal rule"),
    ("centering", "Center content"),
    ("raggedright", "Flush left"),
    ("raggedleft", "Flush right"),
    ("newpage", "Start a new page"),
    ("clearpage", "Clear page"),
    ("pagebreak", "Page break"),
    ("abstract", "Abstract environment"),
    ("equation", "Numbered display math"),
    ("itemize", "Bulleted list"),
    ("enumerate", "Numbered list"),
    ("tabular", "Table"),
    ("table", "Floating table"),
    ("figure", "Floating figure"),
    ("center", "Centered block"),
    ("quote", "Quote block"),
    ("verbatim", "Verbatim text"),
    ("lstlisting", "Code listing"),
    ("thebibliography", "Bibliography list"),
    ("bibliography", "External bibliography file"),
    ("bibliographystyle", "Bibliography style"),
];

/// Return documentation for a command name (without backslash).
pub fn command_documentation(name: &str) -> Option<&'static str> {
    COMMANDS
        .iter()
        .find(|(cmd, _)| *cmd == name)
        .map(|(_, detail)| *detail)
}

/// Return completion items matching `prefix` (text after `\\`).
pub fn command_completions(prefix: &str) -> Vec<TexCompletion> {
    let prefix = prefix.trim_start_matches('\\');
    COMMANDS
        .iter()
        .filter(|(name, _)| prefix.is_empty() || name.starts_with(prefix))
        .map(|(name, detail)| TexCompletion {
            label: name.to_string(),
            insert_text: format!("{name}{{}}"),
            detail: Some((*detail).to_string()),
            kind: if matches!(*name, "begin" | "end" | "itemize" | "enumerate" | "tabular" | "table" | "figure" | "center" | "quote" | "abstract" | "equation" | "verbatim" | "lstlisting" | "thebibliography") {
                CompletionKind::Environment
            } else {
                CompletionKind::Command
            },
        })
        .collect()
}

/// Return environment names for `\\begin{` / `\\end{` completion.
pub fn environment_completions(prefix: &str) -> Vec<TexCompletion> {
    const ENVS: &[(&str, &str)] = &[
        ("document", "Main document body"),
        ("itemize", "Bulleted list"),
        ("enumerate", "Numbered list"),
        ("tabular", "Table"),
        ("table", "Floating table"),
        ("figure", "Floating figure"),
        ("equation", "Numbered display math"),
        ("center", "Centered block"),
        ("quote", "Quote block"),
        ("quotation", "Quotation block"),
        ("abstract", "Abstract"),
        ("verbatim", "Verbatim text"),
        ("lstlisting", "Code listing"),
        ("thebibliography", "Bibliography list"),
        ("align", "Aligned equations"),
        ("gather", "Gathered equations"),
        ("minipage", "Mini page box"),
    ];

    ENVS
        .iter()
        .filter(|(name, _)| prefix.is_empty() || name.starts_with(prefix))
        .map(|(name, detail)| TexCompletion {
            label: name.to_string(),
            insert_text: name.to_string(),
            detail: Some((*detail).to_string()),
            kind: CompletionKind::Environment,
        })
        .collect()
}

/// Infer whether the cursor is completing a command or environment name.
pub fn completions_at(text: &str, offset: usize) -> Vec<TexCompletion> {
    let raw = prefix_at_cursor(text, offset);
    let prefix = raw.trim_start_matches('\\');
    if prefix.starts_with("begin{") || prefix.starts_with("end{") {
        let env_prefix = prefix
            .strip_prefix("begin{")
            .or_else(|| prefix.strip_prefix("end{"))
            .unwrap_or("");
        environment_completions(env_prefix)
    } else {
        command_completions(prefix)
    }
}

fn prefix_at_cursor(text: &str, offset: usize) -> &str {
    let safe_offset = offset.min(text.len());
    let before = &text[..safe_offset];
    let start = before
        .rfind('\\')
        .or_else(|| before.rfind('{'))
        .unwrap_or(safe_offset);
    &before[start..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_section_command() {
        let items = command_completions("sec");
        assert!(items.iter().any(|c| c.label == "section"));
        let sub_items = command_completions("sub");
        assert!(sub_items.iter().any(|c| c.label == "subsection"));
    }

    #[test]
    fn completes_itemize_environment() {
        let items = environment_completions("item");
        assert!(items.iter().any(|c| c.label == "itemize"));
    }

    #[test]
    fn completions_at_begin_environment() {
        let text = "\\begin{ite";
        let items = completions_at(text, text.len());
        assert!(items.iter().any(|c| c.label == "itemize"));
    }
}
