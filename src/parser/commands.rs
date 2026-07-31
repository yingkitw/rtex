//! Command parsing — backslash commands and their arguments.

use super::{TexElement, TexParser};
use std::path::PathBuf;

impl TexParser {
    /// Parse a sectioning command (\part, \chapter, \section, \subsection, …, \subparagraph).
    ///
    /// A trailing `*` (e.g. `\section*{Title}`) is consumed and ignored,
    /// producing the same `TexElement::Section` as the numbered variant.
    /// Our renderer doesn't emit section numbers, so the star is a no-op
    /// but must not break parsing.
    pub(super) fn parse_section(&mut self, level: u8) -> Option<TexElement> {
        let cmd = match level {
            0 => "\\part",
            1 => "\\section",
            2 => "\\subsection",
            3 => "\\subsubsection",
            4 => "\\paragraph",
            5 => "\\subparagraph",
            6 => "\\chapter",
            _ => "\\section",
        };
        self.position += cmd.len();

        // Optional starred variant.
        if self.content[self.position..].starts_with('*') {
            self.position += 1;
        }

        self.skip_whitespace_and_comments();

        self.parse_braced_content()
            .map(|title| TexElement::Section {
                level: level as usize,
                title,
            })
    }

    /// Parse a generic one-argument command (\title, \author, \date, …).
    pub(super) fn parse_command(&mut self, name: &str) -> Option<TexElement> {
        self.position += name.len() + 1;

        self.skip_whitespace_and_comments();

        self.parse_braced_content().map(|arg| TexElement::Command {
            name: name.to_string(),
            args: vec![arg],
        })
    }

    /// Parse `\texttt{arg}`, `\textbf{arg}`, `\textit{arg}`, or `\emph{arg}`.
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

    /// Parse `\includegraphics[…]{path}` with optional width/height.
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
        Some(TexElement::Image {
            path,
            width,
            height,
        })
    }

    pub(super) fn parse_textcolor(&mut self) -> Option<TexElement> {
        self.position += "\\textcolor".len();
        self.skip_whitespace_and_comments();

        let color = self.parse_braced_content()?;
        self.skip_whitespace_and_comments();

        let text = self.parse_braced_content()?;
        Some(TexElement::ColoredText { color, text })
    }

    /// Parse `\cite{key1,key2}` into a citation element.
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

    /// Parse `\label{key}`.
    pub(super) fn parse_label(&mut self) -> Option<TexElement> {
        self.position += "\\label".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::Label { key })
    }

    /// Parse `\ref{key}`.
    pub(super) fn parse_ref(&mut self) -> Option<TexElement> {
        self.position += "\\ref".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::Ref { key })
    }

    /// Parse `\pageref{key}`.
    pub(super) fn parse_pageref(&mut self) -> Option<TexElement> {
        self.position += "\\pageref".len();
        self.skip_whitespace_and_comments();
        let key = self.parse_braced_content()?;
        Some(TexElement::PageRef { key })
    }

    /// Parse page-break commands: `\newpage`, `\clearpage`, `\pagebreak`.
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
        Some(TexElement::Command {
            name: name.to_string(),
            args: vec![],
        })
    }

    /// Parse `\input{filename}` and splice the referenced file inline.
    pub(super) fn parse_input(&mut self) -> Option<TexElement> {
        self.position += "\\input{".len();
        let filename = self.read_until('}');
        self.position += 1; // skip closing brace

        let mut candidates = Vec::new();
        if let Some(ref base) = self.base_dir {
            candidates.push(base.join(&filename));
        }
        for dir in &self.search_paths {
            candidates.push(dir.join(&filename));
        }
        candidates.push(PathBuf::from(&filename));

        for mut path in candidates {
            if path.extension().is_none() {
                path.set_extension("tex");
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                let before = &self.content[..self.position];
                let after = &self.content[self.position..];
                self.content = format!("{}{}\n{}", before, content, after);
                break;
            }
        }
        None
    }

    /// Parse `\vspace{length}`.
    pub(super) fn parse_vspace(&mut self) -> Option<TexElement> {
        self.position += "\\vspace{".len();
        let length = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command {
            name: "vspace".to_string(),
            args: vec![length],
        })
    }

    /// Parse `\underline{text}`.
    pub(super) fn parse_underline(&mut self) -> Option<TexElement> {
        self.position += "\\underline{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command {
            name: "underline".to_string(),
            args: vec![text],
        })
    }

    /// Parse `\footnote{text}`.
    pub(super) fn parse_footnote(&mut self) -> Option<TexElement> {
        self.position += "\\footnote{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Footnote { text })
    }

    /// Parse `\caption{text}`.
    pub(super) fn parse_caption(&mut self) -> Option<TexElement> {
        self.position += "\\caption{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Caption { text })
    }

    /// Parse `\url{link}`.
    pub(super) fn parse_url(&mut self) -> Option<TexElement> {
        self.position += "\\url{".len();
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command {
            name: "url".to_string(),
            args: vec![text],
        })
    }

    /// Parse `\href{url}{text}` (hyperref).
    pub(super) fn parse_href(&mut self) -> Option<TexElement> {
        self.position += "\\href{".len();
        let url = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let text = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let t = self.read_until('}');
            self.position += 1;
            t
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "href".to_string(),
            args: vec![url, text],
        })
    }

    /// Parse `\raisebox{distance}{text}`.
    pub(super) fn parse_raisebox(&mut self) -> Option<TexElement> {
        self.position += "\\raisebox{".len();
        let distance = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let text = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let t = self.read_until('}');
            self.position += 1;
            t
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "raisebox".to_string(),
            args: vec![distance, text],
        })
    }

    /// Parse `\rotatebox{angle}{text}`.
    pub(super) fn parse_rotatebox(&mut self) -> Option<TexElement> {
        self.position += "\\rotatebox{".len();
        let angle = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let text = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let t = self.read_until('}');
            self.position += 1;
            t
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "rotatebox".to_string(),
            args: vec![angle, text],
        })
    }

    /// Parse `\scalebox{factor}{text}`.
    pub(super) fn parse_scalebox(&mut self) -> Option<TexElement> {
        self.position += "\\scalebox{".len();
        let factor = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let text = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let t = self.read_until('}');
            self.position += 1;
            t
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "scalebox".to_string(),
            args: vec![factor, text],
        })
    }

    /// Parse `\colorbox{color}{text}`.
    pub(super) fn parse_colorbox(&mut self) -> Option<TexElement> {
        self.position += "\\colorbox{".len();
        let color = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let text = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let t = self.read_until('}');
            self.position += 1;
            t
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "colorbox".to_string(),
            args: vec![color, text],
        })
    }

    /// Parse `\fcolorbox{framecolor}{backcolor}{text}`.
    pub(super) fn parse_fcolorbox(&mut self) -> Option<TexElement> {
        self.position += "\\fcolorbox{".len();
        let frame = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let back = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let b = self.read_until('}');
            self.position += 1;
            b
        } else {
            String::new()
        };
        self.skip_whitespace_and_comments();
        let text = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let t = self.read_until('}');
            self.position += 1;
            t
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "fcolorbox".to_string(),
            args: vec![frame, back, text],
        })
    }

    /// Parse any command of the form `\name{text}` into a [`TexElement::Command`].
    pub(super) fn parse_simple_braced_command(
        &mut self,
        name: &str,
        prefix_len: usize,
    ) -> Option<TexElement> {
        self.position += prefix_len;
        let text = self.read_until('}');
        self.position += 1; // skip closing brace
        Some(TexElement::Command {
            name: name.to_string(),
            args: vec![text],
        })
    }

    /// Parse `\rule{width}{height}` for horizontal rules or vertical struts.
    pub(super) fn parse_rule(&mut self) -> Option<TexElement> {
        self.position += "\\rule{".len();
        let width = self.read_until('}');
        self.position += 1; // skip closing brace
        self.skip_whitespace_and_comments();
        let height = if self.content[self.position..].starts_with('{') {
            self.position += 1;
            let h = self.read_until('}');
            self.position += 1;
            h
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "rule".to_string(),
            args: vec![width, height],
        })
    }

    /// Skip past an unknown command so parsing can continue.
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

    /// Parse `\parbox[alignment]{width}{text}`.
    pub(super) fn parse_parbox(&mut self) -> Option<TexElement> {
        self.position += "\\parbox".len();
        self.skip_whitespace_and_comments();

        // Optional [alignment]
        if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
            self.position += 1;
            let _ = self.read_until(']');
            self.position += 1;
            self.skip_whitespace_and_comments();
        }

        // {width}
        if self.position < self.content.len() && self.content[self.position..].starts_with('{') {
            let _ = self.parse_braced_content();
            self.skip_whitespace_and_comments();
        }

        // {text}
        let text = if self.position < self.content.len() && self.content[self.position..].starts_with('{') {
            self.parse_braced_content().unwrap_or_default()
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "parbox".to_string(),
            args: vec![text],
        })
    }

    /// Parse `\makebox[width][position]{text}`.
    pub(super) fn parse_makebox(&mut self) -> Option<TexElement> {
        self.position += "\\makebox".len();
        self.skip_whitespace_and_comments();

        // Optional [width]
        if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
            self.position += 1;
            let _ = self.read_until(']');
            self.position += 1;
            self.skip_whitespace_and_comments();
        }

        // Optional [position]
        if self.position < self.content.len() && self.content[self.position..].starts_with('[') {
            self.position += 1;
            let _ = self.read_until(']');
            self.position += 1;
            self.skip_whitespace_and_comments();
        }

        // {text}
        let text = if self.position < self.content.len() && self.content[self.position..].starts_with('{') {
            self.parse_braced_content().unwrap_or_default()
        } else {
            String::new()
        };
        Some(TexElement::Command {
            name: "makebox".to_string(),
            args: vec![text],
        })
    }

    /// Parse `\multicolumn{n}{align}{content}`.
    pub(super) fn parse_multicolumn(&mut self) -> Option<TexElement> {
        self.position += "\\multicolumn".len();
        self.skip_whitespace_and_comments();

        let n = self.parse_braced_content().unwrap_or_default();
        self.skip_whitespace_and_comments();
        let align = self.parse_braced_content().unwrap_or_default();
        self.skip_whitespace_and_comments();
        let content = self.parse_braced_content().unwrap_or_default();

        Some(TexElement::Command {
            name: "multicolumn".to_string(),
            args: vec![n, align, content],
        })
    }
}
