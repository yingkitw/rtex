//! Command parsing — backslash commands and their arguments.

use super::{TexElement, TexParser};
use std::path::PathBuf;

impl TexParser {
    pub(super) fn parse_section(&mut self, level: u8) -> Option<TexElement> {
        let cmd = match level {
            1 => "\\section",
            2 => "\\subsection",
            3 => "\\subsubsection",
            4 => "\\paragraph",
            5 => "\\subparagraph",
            _ => "\\section",
        };
        self.position += cmd.len();

        self.skip_whitespace_and_comments();

        self.parse_braced_content().map(|title| TexElement::Section { level: level as usize, title })
    }

    pub(super) fn parse_command(&mut self, name: &str) -> Option<TexElement> {
        self.position += name.len() + 1;

        self.skip_whitespace_and_comments();

        self.parse_braced_content().map(|arg| TexElement::Command {
                name: name.to_string(),
                args: vec![arg],
            })
    }

    pub(super) fn parse_text_command(&mut self) -> Option<TexElement> {
        let remaining = &self.content[self.position..];

        let (_cmd_name, cmd_len) = if remaining.starts_with("\\texttt{") {
            ("texttt", 7)
        } else if remaining.starts_with("\\textbf{") {
            ("textbf", 7)
        } else if remaining.starts_with("\\textit{") {
            ("textit", 7)
        } else if remaining.starts_with("\\emph{") {
            ("emph", 5)
        } else {
            return None;
        };

        self.position += cmd_len;

        self.parse_braced_content().map(|arg| TexElement::Command {
            name: _cmd_name.to_string(),
            args: vec![arg],
        })
    }

    pub(super) fn parse_includegraphics(&mut self) -> Option<TexElement> {
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

    pub(super) fn parse_textcolor(&mut self) -> Option<TexElement> {
        self.position += "\\textcolor".len();
        self.skip_whitespace_and_comments();

        let color = self.parse_braced_content()?;
        self.skip_whitespace_and_comments();

        let text = self.parse_braced_content()?;
        Some(TexElement::ColoredText { color, text })
    }

    pub(super) fn parse_cite(&mut self) -> Option<TexElement> {
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

    pub(super) fn parse_label(&mut self) -> Option<TexElement> {
        self.position += "\\label".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::Label { key })
    }

    pub(super) fn parse_ref(&mut self) -> Option<TexElement> {
        self.position += "\\ref".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::Ref { key })
    }

    pub(super) fn parse_pageref(&mut self) -> Option<TexElement> {
        self.position += "\\pageref".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::PageRef { key })
    }

    pub(super) fn parse_pagebreak(&mut self) -> Option<TexElement> {
        let name = if self.content[self.position..].starts_with("\\newpage") {
            self.position += "\\newpage".len();
            "newpage"
        } else if self.content[self.position..].starts_with("\\clearpage") {
            self.position += "\\clearpage".len();
            "clearpage"
        } else if self.content[self.position..].starts_with("\\pagebreak") {
            self.position += "\\pagebreak".len();
            "pagebreak"
        } else {
            "newpage"
        };
        Some(TexElement::Command { name: name.to_string(), args: vec![] })
    }

    pub(super) fn parse_input(&mut self) -> Option<TexElement> {
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

    pub(super) fn parse_vspace(&mut self) -> Option<TexElement> {
        self.position += "\\vspace{".len();
        let length = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command { name: "vspace".to_string(), args: vec![length] })
    }

    pub(super) fn parse_underline(&mut self) -> Option<TexElement> {
        self.position += "\\underline{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command { name: "underline".to_string(), args: vec![text] })
    }

    pub(super) fn parse_footnote(&mut self) -> Option<TexElement> {
        self.position += "\\footnote{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Footnote { text })
    }

    pub(super) fn parse_caption(&mut self) -> Option<TexElement> {
        self.position += "\\caption{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Caption { text })
    }

    pub(super) fn parse_unknown_command(&mut self) {
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
}
