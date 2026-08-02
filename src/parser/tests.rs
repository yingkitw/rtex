use super::{MathLineKind, TexElement, TexParser};

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

        assert!(
            elements.is_empty()
                || elements
                    .iter()
                    .all(|e| !matches!(e, TexElement::Command { .. }))
        );
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

        assert!(
            elements.iter().any(|e| {
                matches!(e, TexElement::Section { level: 1, title } if title == "Intro")
            })
        );
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 2, title } if title == "Method")
        }));
        assert!(elements.iter().any(|e| {
            matches!(e, TexElement::Section { level: 3, title } if title == "Details")
        }));
        assert!(
            elements.iter().any(|e| {
                matches!(e, TexElement::Section { level: 4, title } if title == "Note")
            })
        );
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

        assert!(
            elements
                .iter()
                .any(|e| { matches!(e, TexElement::TableOfContents) })
        );
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
        let cmds: Vec<_> = elements
            .iter()
            .filter_map(|e| {
                if let TexElement::Command { name, args } = e {
                    Some((name.clone(), args.clone()))
                } else {
                    None
                }
            })
            .collect();
        assert!(
            cmds.iter()
                .any(|(n, a)| n == "textbf" && a == &["bold".to_string()])
        );
        assert!(
            cmds.iter()
                .any(|(n, a)| n == "textit" && a == &["italic".to_string()])
        );
        assert!(
            cmds.iter()
                .any(|(n, a)| n == "texttt" && a == &["mono".to_string()])
        );
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
                table.columns
                    == vec![
                        crate::table::Align::Left,
                        crate::table::Align::Center,
                        crate::table::Align::Right,
                    ]
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

        assert!(
            elements
                .iter()
                .any(|element| { matches!(element, TexElement::Text(t) if t.contains("hi hi")) })
        );
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

        assert!(
            elements.iter().any(|element| {
                matches!(element, TexElement::Label { key } if key == "sec:intro")
            })
        );
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

        assert!(
            elements.iter().any(|element| {
                matches!(element, TexElement::Ref { key } if key == "sec:intro")
            })
        );
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
        let sizes = [
            "\\tiny",
            "\\scriptsize",
            "\\footnotesize",
            "\\small",
            "\\normalsize",
            "\\large",
            "\\Large",
            "\\LARGE",
            "\\huge",
            "\\Huge",
        ];
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
        let mut parser = TexParser::new(content.to_string()).with_base_dir(tmp.path());
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
        let mut parser = TexParser::new(content.to_string()).with_base_dir(tmp.path());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| {
            if let TexElement::Text(t) = e {
                t.contains("Chapter text")
            } else {
                false
            }
        }));
    }

    #[test]
    fn parser_input_missing_file_is_noop() {
        let content = r#"\input{nonexistent}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        // Should not panic; parser skips the command and continues
        assert!(
            !elements
                .iter()
                .any(|e| matches!(e, TexElement::Command { name, .. } if name == "newpage"))
        );
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
                inner
                    .iter()
                    .any(|i| matches!(i, TexElement::Text(t) if t.contains("centered")))
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
        assert!(
            elements
                .iter()
                .any(|e| matches!(e, TexElement::Footnote { text } if text == "This is a note."))
        );
    }

    #[test]
    fn parser_parses_part() {
        let content = r#"\part{The Beginning}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(
            |e| matches!(e, TexElement::Section { level: 0, title } if title == "The Beginning")
        ));
    }

    #[test]
    fn parser_parses_chapter() {
        let content = r#"\chapter{Intro}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(
            elements
                .iter()
                .any(|e| matches!(e, TexElement::Section { level: 6, title } if title == "Intro"))
        );
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
        assert!(
            elements
                .iter()
                .any(|e| matches!(e, TexElement::ListOfFigures))
        );
    }

    #[test]
    fn parser_parses_listoftables() {
        let content = r#"\listoftables"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(
            elements
                .iter()
                .any(|e| matches!(e, TexElement::ListOfTables))
        );
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
        let names: Vec<&str> = elements
            .iter()
            .filter_map(|e| {
                if let TexElement::Command { name, args } = e {
                    if args.is_empty() {
                        Some(name.as_str())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
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
            assert!(
                elements
                    .iter()
                    .any(|e| matches!(e, TexElement::Text(t) if t == *expected)),
                "Command {} should produce text {}",
                cmd,
                expected
            );
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
            "\\hfill",
            "\\vfill",
            "\\hrulefill",
            "\\dotfill",
            "\\medskip",
            "\\bigskip",
            "\\smallskip",
            "\\strut",
            "\\mathstrut",
            "\\qquad",
            "\\quad",
            "\\;",
            "\\,",
            "\\!",
            "\\:",
            "\\ ",
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

    #[test]
    fn parser_parses_description_environment() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{description}
    \item[Apple] A red fruit.
    \item[Bear] A large mammal.
\end{description}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let desc = elements.iter().find_map(|e| match e {
            TexElement::DescriptionList { items } => Some(items),
            _ => None,
        });
        let items = desc.expect("should produce a DescriptionList");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].term, "Apple");
        assert_eq!(items[1].term, "Bear");
        assert!(
            items[0]
                .body
                .iter()
                .any(|e| matches!(e, TexElement::Text(t) if t.contains("red fruit")))
        );
    }

    #[test]
    fn parser_parses_description_with_unlabeled_items() {
        let content = r#"\begin{document}
\begin{description}
    \item plain body text
    \item[Labeled] second body
\end{description}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let items = elements
            .iter()
            .find_map(|e| match e {
                TexElement::DescriptionList { items } => Some(items),
                _ => None,
            })
            .expect("DescriptionList present");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].term, "");
        assert_eq!(items[1].term, "Labeled");
    }

    #[test]
    fn parser_parses_align_environment() {
        let content = r#"\begin{document}
\begin{align}
    a &= b + c \\
    d &= e + f
\end{align}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let ml = elements.iter().find_map(|e| match e {
            TexElement::MathLines { lines, kind } => Some((lines, kind)),
            _ => None,
        });
        let (lines, kind) = ml.expect("should produce a MathLines element");
        assert_eq!(lines.len(), 2);
        assert!(matches!(kind, super::MathLineKind::Align));
        assert!(
            lines[0].contains('&'),
            "& should be preserved as an alignment marker for the PDF backend"
        );
    }

    #[test]
    fn parser_parses_align_starred_environment() {
        let content = r#"\begin{document}
\begin{align*}
    x &= 1 \\
    y &= 2
\end{align*}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        assert!(
            elements
                .iter()
                .any(|e| matches!(e, TexElement::MathLines { lines, .. } if lines.len() == 2))
        );
    }

    #[test]
    fn parser_parses_gather_environment() {
        let content = r#"\begin{document}
\begin{gather}
    x = 1 \\
    y = 2 \\
    z = 3
\end{gather}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let lines = elements
            .iter()
            .find_map(|e| match e {
                TexElement::MathLines { lines, .. } => Some(lines),
                _ => None,
            })
            .expect("gather should produce MathLines");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn parser_parses_multline_environment() {
        let content = r#"\begin{document}
\begin{multline}
    a + b + c + d + e + f \\
    + g + h
\end{multline}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let ml = elements.iter().find_map(|e| match e {
            TexElement::MathLines { lines, kind } => Some((lines, kind)),
            _ => None,
        });
        let (lines, kind) = ml.expect("multline should produce MathLines");
        assert_eq!(lines.len(), 2);
        assert!(matches!(kind, super::MathLineKind::Multline));
    }

    #[test]
    fn parser_parses_cases_environment() {
        let content = r#"\begin{document}
\begin{cases}
    x & \text{if } x \geq 0 \\
    -x & \text{if } x < 0
\end{cases}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let ml = elements.iter().find_map(|e| match e {
            TexElement::MathLines { lines, kind } => Some((lines, kind)),
            _ => None,
        });
        let (lines, kind) = ml.expect("cases should produce MathLines");
        assert_eq!(lines.len(), 2);
        assert!(matches!(kind, super::MathLineKind::Cases));
        assert!(
            lines[0].contains("if"),
            "cases should include 'if' between value and condition"
        );
    }

    #[test]
    fn parser_parses_href_command() {
        let content = r#"\documentclass{article}
\begin{document}
Visit \href{https://example.com}{Example Site} for more.
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let href = elements.iter().find_map(|e| match e {
            TexElement::Command { name, args } if name == "href" => Some(args),
            _ => None,
        });
        let args = href.expect("\\href should produce a Command");
        assert_eq!(args[0], "https://example.com");
        assert_eq!(args[1], "Example Site");
    }

    #[test]
    fn parser_parses_bracket_math_delimiters() {
        // TeX-style \(...\) inline and \[...\] display math.
        let inline = r#"\documentclass{article}
\begin{document}
Inline \(E = mc^2\) math.
\end{document}"#;
        let mut p = TexParser::new(inline.to_string());
        let els = p.parse();
        assert!(
            els.iter()
                .any(|e| matches!(e, TexElement::MathInline(s) if s.contains("E = mc^2")))
        );

        let display = r#"\documentclass{article}
\begin{document}
Display \[ \int_0^\infty e^{-x} dx = 1 \] math.
\end{document}"#;
        let mut p = TexParser::new(display.to_string());
        let els = p.parse();
        assert!(
            els.iter()
                .any(|e| matches!(e, TexElement::MathDisplay(s) if s.contains("infty")))
        );
    }

    #[test]
    fn parser_parses_theorem_environment() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{theorem}[Pythagoras]
For a right triangle with legs $a, b$ and hypotenuse $c$, $a^2 + b^2 = c^2$.
\end{theorem}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let thm = elements.iter().find_map(|e| match e {
            TexElement::Theorem { kind, title, body } => Some((kind, title, body)),
            _ => None,
        });
        let (kind, title, body) = thm.expect("should produce a Theorem element");
        assert_eq!(kind, "theorem");
        assert_eq!(title.as_deref(), Some("Pythagoras"));
        assert!(
            body.iter()
                .any(|e| matches!(e, TexElement::Text(t) if t.contains("right triangle")))
        );
    }

    #[test]
    fn parser_parses_proof_environment_without_title() {
        let content = r#"\begin{document}
\begin{proof}
By induction. QED.
\end{proof}
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let thm = elements.iter().find_map(|e| match e {
            TexElement::Theorem { kind, title, .. } => Some((kind, title)),
            _ => None,
        });
        let (kind, title) = thm.expect("proof should produce a Theorem element");
        assert_eq!(kind, "proof");
        assert!(title.is_none());
    }

    #[test]
    fn parser_parses_starred_sections() {
        // `\section*`, `\subsection*`, `\subsubsection*` should be accepted.
        let content = r#"\documentclass{article}
\begin{document}
\section*{Acknowledgement}
Thanks.
\subsection*{Preface}
Lead-in text.
\subsubsection*{Notes}
Body.
\end{document}
"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        let titles: Vec<(&usize, &String)> = elements
            .iter()
            .filter_map(|e| match e {
                TexElement::Section { level, title } => Some((level, title)),
                _ => None,
            })
            .collect();
        assert_eq!(
            titles.len(),
            3,
            "expected 3 starred sections, got {titles:?}"
        );
        assert_eq!(titles[0], (&1, &"Acknowledgement".to_string()));
        assert_eq!(titles[1], (&2, &"Preface".to_string()));
        assert_eq!(titles[2], (&3, &"Notes".to_string()));
    }

    #[test]
    fn parser_splits_cases_inside_display_math() {
        // \[ f(x) = \begin{cases} … \end{cases} \] should produce a
        // MathLines element (cases) with the prefix "f(x) = " merged
        // into the first line.
        let content = r#"\documentclass{article}
\begin{document}
\[
f(x) = \begin{cases}
    x^2  & \text{if } x \geq 0 \\
    -x   & \text{if } x < 0
\end{cases}
\]
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();

        // The key invariant: the prefix "f(x) = " is preserved (either as
        // the start of the first MathLines line, or inside a MathDisplay).
        let mut found_prefix = false;
        for elem in &elements {
            match elem {
                TexElement::MathLines { lines, .. } => {
                    if lines.first().map(|s| s.contains("f(x) =")).unwrap_or(false) {
                        found_prefix = true;
                    }
                }
                TexElement::MathDisplay(text) => {
                    if text.contains("f(x) =") {
                        found_prefix = true;
                    }
                }
                _ => {}
            }
        }
        assert!(found_prefix, "expected 'f(x) =' prefix to be preserved");
    }

    #[test]
    fn parser_parses_line_break() {
        let content = r#"\documentclass{article}
\begin{document}
First line \\ Second line
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::LineBreak)),
            "Expected a LineBreak element");
    }

    #[test]
    fn parser_parses_line_break_with_opt_arg() {
        let content = r#"\documentclass{article}
\begin{document}
A \\[2em] B
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::LineBreak)),
            "Expected LineBreak with optional [length] argument");
    }

    #[test]
    fn parser_parses_extended_special_chars() {
        let cases = [
            ("\\copyright", "©"),
            ("\\pounds", "£"),
            ("\\S", "§"),
            ("\\P", "¶"),
            ("\\dag", "†"),
            ("\\ddag", "‡"),
            ("\\ldots", "…"),
            ("\\dots", "…"),
            ("\\LaTeX", "LaTeX"),
            ("\\TeX", "TeX"),
            ("\\AA", "Å"),
            ("\\aa", "å"),
            ("\\ss", "ß"),
        ];
        for (cmd, expected) in &cases {
            let mut parser = TexParser::new(cmd.to_string());
            let elements = parser.parse();
            assert!(
                elements.iter().any(|e| matches!(e, TexElement::Text(t) if t == *expected)),
                "Command {} should produce text {}",
                cmd,
                expected
            );
        }
    }

    #[test]
    fn parser_special_chars_word_boundary() {
        // \i should produce ı, but \it should produce a Command named "it"
        let mut parser = TexParser::new("\\it".to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, .. } if name == "it")),
            "\\it should be a Command, not consumed by \\i special char");
    }

    #[test]
    fn parser_parses_flushleft() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{flushleft}
Left aligned text
\end{flushleft}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::FlushLeft(_))),
            "Expected FlushLeft element");
    }

    #[test]
    fn parser_parses_flushright() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{flushright}
Right aligned text
\end{flushright}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::FlushRight(_))),
            "Expected FlushRight element");
    }

    #[test]
    fn parser_parses_figure() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{figure}
\centering
\caption{Test figure}
\end{figure}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Center(_))),
            "Expected figure to produce Center element");
    }

    #[test]
    fn parser_parses_displaymath_env() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{displaymath}
E = mc^2
\end{displaymath}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::MathDisplay(_))),
            "Expected displaymath environment to produce MathDisplay");
    }

    #[test]
    fn parser_parses_math_env() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{math} x + y \end{math}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::MathInline(_))),
            "Expected math environment to produce MathInline");
    }

    #[test]
    fn parser_skips_setlength() {
        let content = r#"\documentclass{article}
\begin{document}
\setlength{\parindent}{0pt}
Hello
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(!elements.iter().any(|e| matches!(e, TexElement::Command { name, .. } if name == "setlength")),
            "setlength should be silently consumed");
    }

    #[test]
    fn parser_skips_setcounter() {
        let content = r#"\documentclass{article}
\begin{document}
\setcounter{page}{3}
Hello
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(!elements.iter().any(|e| matches!(e, TexElement::Command { name, .. } if name == "setcounter")),
            "setcounter should be silently consumed");
    }

    #[test]
    fn parser_parses_hspace() {
        let content = r#"\documentclass{article}
\begin{document}
A\hspace{1em}B
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "hspace" && args == &["1em"])),
            "Expected hspace command");
    }

    #[test]
    fn parser_parses_multicolumn() {
        let content = r#"\documentclass{article}
\begin{document}
\multicolumn{2}{c}{Header}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "multicolumn" && args.len() == 3)),
            "Expected multicolumn command with 3 args");
    }

    #[test]
    fn parser_parses_enquote() {
        let content = r#"\documentclass{article}
\begin{document}
\enquote{Hello world}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "enquote" && args == &["Hello world"])),
            "Expected enquote command");
    }

    #[test]
    fn parser_parses_mbox() {
        let content = r#"\documentclass{article}
\begin{document}
\mbox{Hello}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "mbox" && args == &["Hello"])),
            "Expected mbox command");
    }

    #[test]
    fn parser_parses_parbox() {
        let content = r#"\documentclass{article}
\begin{document}
\parbox[c]{3cm}{Centered text}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Command { name, args } if name == "parbox" && args == &["Centered text"])),
            "Expected parbox command with text content");
    }

    #[test]
    fn parser_parses_minipage() {
        let content = r#"\documentclass{article}
\begin{document}
\begin{minipage}[c]{0.5\textwidth}
Hello minipage
\end{minipage}
\end{document}"#;
        let mut parser = TexParser::new(content.to_string());
        let elements = parser.parse();
        assert!(elements.iter().any(|e| matches!(e, TexElement::Center(_))),
            "Expected minipage to produce Center element");
    }
}
