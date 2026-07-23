//! Static analysis diagnostics for LaTeX source.

use super::positions::{TexRange, offset_to_position};
use crate::error::Position;

/// Severity aligned with LSP diagnostic levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
}

/// A single diagnostic finding in a `.tex` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexDiagnostic {
    pub range: TexRange,
    pub severity: Severity,
    pub message: String,
    pub code: Option<String>,
}

pub fn analyze_diagnostics(text: &str) -> Vec<TexDiagnostic> {
    let mut out = Vec::new();
    out.extend(check_braces(text));
    out.extend(check_environments(text));
    out.extend(check_math_delimiters(text));
    out.extend(check_document_structure(text));
    out
}

fn check_braces(text: &str) -> Vec<TexDiagnostic> {
    let mut stack: Vec<(usize, char)> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut in_comment = false;
    let mut offset = 0usize;

    for ch in text.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            offset += ch.len_utf8();
            continue;
        }

        if ch == '%' {
            in_comment = true;
            offset += ch.len_utf8();
            continue;
        }

        if ch == '\\' {
            offset += ch.len_utf8();
            continue;
        }

        match ch {
            '{' => stack.push((offset, '{')),
            '}' => {
                if stack.pop().is_none() {
                    diagnostics.push(TexDiagnostic {
                        range: TexRange::single(offset),
                        severity: Severity::Error,
                        message: "Unmatched closing brace '}'".to_string(),
                        code: Some("unmatched-brace".to_string()),
                    });
                }
            }
            _ => {}
        }
        offset += ch.len_utf8();
    }

    for (start, _) in stack {
        diagnostics.push(TexDiagnostic {
            range: TexRange::single(start),
            severity: Severity::Error,
            message: "Unclosed '{' — missing closing brace".to_string(),
            code: Some("unclosed-brace".to_string()),
        });
    }

    diagnostics
}

fn check_environments(text: &str) -> Vec<TexDiagnostic> {
    let mut stack: Vec<(String, usize)> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut pos = 0usize;

    while pos < text.len() {
        if let Some(rel) = text[pos..].find("\\begin{") {
            let start = pos + rel;
            if let Some(name) = read_braced_name(text, start + "\\begin".len()) {
                stack.push((name, start));
            }
            pos = start + 1;
            continue;
        }

        if let Some(rel) = text[pos..].find("\\end{") {
            let start = pos + rel;
            if let Some(name) = read_braced_name(text, start + "\\end".len()) {
                match stack.pop() {
                    Some((open, _open_start)) if open == name => {}
                    Some((open, open_start)) => {
                        diagnostics.push(TexDiagnostic {
                            range: TexRange::span(start, start + "\\end{".len() + name.len() + 2),
                            severity: Severity::Error,
                            message: format!(
                                "Environment mismatch: expected \\end{{{open}}} (opened at {}), found \\end{{{name}}}",
                                format_position(text, open_start)
                            ),
                            code: Some("env-mismatch".to_string()),
                        });
                    }
                    None => {
                        diagnostics.push(TexDiagnostic {
                            range: TexRange::span(start, start + "\\end{".len() + name.len() + 2),
                            severity: Severity::Error,
                            message: format!("\\end{{{name}}} without matching \\begin{{{name}}}"),
                            code: Some("env-unmatched-end".to_string()),
                        });
                    }
                }
            }
            pos = start + 1;
            continue;
        }

        break;
    }

    for (name, start) in stack {
        diagnostics.push(TexDiagnostic {
            range: TexRange::span(start, start + "\\begin{".len() + name.len() + 2),
            severity: Severity::Error,
            message: format!("Unclosed environment '{name}'"),
            code: Some("env-unclosed".to_string()),
        });
    }

    diagnostics
}

fn check_math_delimiters(text: &str) -> Vec<TexDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_comment = false;
    let mut dollar_offsets = Vec::new();
    let mut offset = 0usize;

    for ch in text.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            offset += ch.len_utf8();
            continue;
        }
        if ch == '%' {
            in_comment = true;
            offset += ch.len_utf8();
            continue;
        }
        if ch == '$' {
            dollar_offsets.push(offset);
        }
        offset += ch.len_utf8();
    }

    if dollar_offsets.len() % 2 == 1 {
        let last = *dollar_offsets.last().unwrap();
        diagnostics.push(TexDiagnostic {
            range: TexRange::single(last),
            severity: Severity::Error,
            message: "Unclosed inline math delimiter '$'".to_string(),
            code: Some("unclosed-math".to_string()),
        });
    }

    diagnostics
}

fn check_document_structure(text: &str) -> Vec<TexDiagnostic> {
    let mut diagnostics = Vec::new();
    let has_documentclass = text.contains("\\documentclass");
    let has_begin_document = text.contains("\\begin{document}");
    let has_end_document = text.contains("\\end{document}");

    if !has_documentclass && text.contains("\\begin{") {
        diagnostics.push(TexDiagnostic {
            range: TexRange::single(0),
            severity: Severity::Warning,
            message: "Missing \\documentclass declaration".to_string(),
            code: Some("missing-documentclass".to_string()),
        });
    }

    if has_begin_document && !has_end_document {
        if let Some(start) = text.find("\\begin{document}") {
            diagnostics.push(TexDiagnostic {
                range: TexRange::span(start, start + "\\begin{document}".len()),
                severity: Severity::Warning,
                message: "Missing \\end{document}".to_string(),
                code: Some("missing-end-document".to_string()),
            });
        }
    }

    if has_end_document && !has_begin_document {
        if let Some(start) = text.find("\\end{document}") {
            diagnostics.push(TexDiagnostic {
                range: TexRange::span(start, start + "\\end{document}".len()),
                severity: Severity::Error,
                message: "\\end{document} without \\begin{document}".to_string(),
                code: Some("orphan-end-document".to_string()),
            });
        }
    }

    diagnostics
}

fn read_braced_name(text: &str, start: usize) -> Option<String> {
    if !text[start..].starts_with('{') {
        return None;
    }
    let inner_start = start + 1;
    let end = text[inner_start..].find('}')? + inner_start;
    Some(text[inner_start..end].to_string())
}

fn format_position(text: &str, offset: usize) -> String {
    let pos = offset_to_position(text, offset);
    Position::new(pos.line as usize + 1, pos.character as usize + 1, offset).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unclosed_brace() {
        let text = "\\section{Intro";
        let diags = analyze_diagnostics(text);
        assert!(
            diags
                .iter()
                .any(|d| d.code.as_deref() == Some("unclosed-brace"))
        );
    }

    #[test]
    fn detects_unmatched_end_brace() {
        let text = "hello } world";
        let diags = analyze_diagnostics(text);
        assert!(
            diags
                .iter()
                .any(|d| d.code.as_deref() == Some("unmatched-brace"))
        );
    }

    #[test]
    fn detects_environment_mismatch() {
        let text = "\\begin{itemize}\\end{enumerate}";
        let diags = analyze_diagnostics(text);
        assert!(
            diags
                .iter()
                .any(|d| d.code.as_deref() == Some("env-mismatch"))
        );
    }

    #[test]
    fn detects_unclosed_math() {
        let text = "The value is $x = 1";
        let diags = analyze_diagnostics(text);
        assert!(
            diags
                .iter()
                .any(|d| d.code.as_deref() == Some("unclosed-math"))
        );
    }

    #[test]
    fn warns_missing_end_document() {
        let text = "\\documentclass{article}\\begin{document}Hi";
        let diags = analyze_diagnostics(text);
        assert!(
            diags
                .iter()
                .any(|d| d.code.as_deref() == Some("missing-end-document"))
        );
    }

    #[test]
    fn ignores_braces_in_comments() {
        let text = "% { not a brace\n\\begin{document}";
        let diags = analyze_diagnostics(text);
        assert!(
            !diags
                .iter()
                .any(|d| d.code.as_deref() == Some("unmatched-brace"))
        );
    }
}
