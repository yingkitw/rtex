//! Document outline symbols from the parsed AST.

use crate::parser::{TexElement, TexParser};

/// A symbol in the document outline (sections, labels).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range_start: usize,
    pub range_end: usize,
    pub children: Vec<TexSymbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Section,
    Subsection,
    Label,
}

/// Build a hierarchical outline from LaTeX source.
pub fn document_symbols(text: &str) -> Vec<TexSymbol> {
    let elements = TexParser::new(text.to_string()).parse();
    let mut root = Vec::new();
    let mut stack: Vec<(usize, TexSymbol)> = Vec::new();

    for element in &elements {
        match element {
            TexElement::Section { level, title } => {
                let symbol = TexSymbol {
                    name: title.clone(),
                    kind: if *level <= 1 {
                        SymbolKind::Section
                    } else {
                        SymbolKind::Subsection
                    },
                    range_start: find_section_offset(text, title).unwrap_or(0),
                    range_end: find_section_offset(text, title)
                        .map(|s| s + title.len())
                        .unwrap_or(0),
                    children: Vec::new(),
                };

                while stack.last().is_some_and(|(lvl, _)| *lvl >= *level) {
                    attach_symbol(stack.pop().unwrap().1, &mut stack, &mut root);
                }
                stack.push((*level, symbol));
            }
            TexElement::Label { key } => {
                let start = find_label_offset(text, key);
                root.push(TexSymbol {
                    name: format!("#{key}"),
                    kind: SymbolKind::Label,
                    range_start: start,
                    range_end: start.saturating_add(key.len() + "\\label{}".len()),
                    children: Vec::new(),
                });
            }
            _ => {}
        }
    }

    while let Some((_, symbol)) = stack.pop() {
        attach_symbol(symbol, &mut stack, &mut root);
    }

    root
}

fn attach_symbol(symbol: TexSymbol, stack: &mut [(usize, TexSymbol)], root: &mut Vec<TexSymbol>) {
    if let Some((_, parent)) = stack.last_mut() {
        parent.children.push(symbol);
    } else {
        root.push(symbol);
    }
}

fn find_label_offset(text: &str, key: &str) -> usize {
    let needle = format!("\\label{{{key}}}");
    text.find(&needle).unwrap_or(0)
}

fn find_section_offset(text: &str, title: &str) -> Option<usize> {
    for cmd in [
        "\\section",
        "\\subsection",
        "\\subsubsection",
        "\\paragraph",
        "\\chapter",
        "\\part",
    ] {
        let pattern = format!("{cmd}{{{title}}}");
        if let Some(pos) = text.find(&pattern) {
            return Some(pos);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_section_symbols() {
        let text = r"\documentclass{article}
\begin{document}
\section{Introduction}
Hello
\subsection{Background}
\end{document}";
        let symbols = document_symbols(text);
        let intro = symbols.iter().find(|s| s.name == "Introduction").unwrap();
        assert!(intro.children.iter().any(|c| c.name == "Background"));
    }

    #[test]
    fn extracts_label_symbols() {
        let text = r"\begin{document}
\section{A}\label{sec:a}
\end{document}";
        let symbols = document_symbols(text);
        assert!(symbols.iter().any(|s| s.name == "#sec:a"));
    }

    #[test]
    fn multiple_top_level_sections() {
        let text = r"\begin{document}
\section{One}
\section{Two}
\end{document}";
        let symbols = document_symbols(text);
        assert_eq!(
            symbols
                .iter()
                .filter(|s| s.kind == SymbolKind::Section)
                .count(),
            2
        );
    }
}
