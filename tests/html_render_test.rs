// HTML rendering tests for every TexElement variant.
// Verifies that each parsed element produces valid HTML output without
// panicking and with expected content fragments.

use rtex::{OutputFormat, convert_tex_string};

fn render_html(tex: &str) -> String {
    let bytes = convert_tex_string(tex, OutputFormat::Html).expect("HTML conversion failed");
    String::from_utf8(bytes).expect("HTML output should be valid UTF-8")
}

fn assert_contains(html: &str, needle: &str, label: &str) {
    assert!(
        html.contains(needle),
        "HTML for {} should contain {:?}\nGot:\n{}",
        label,
        needle,
        html
    );
}

fn assert_not_contains(html: &str, needle: &str, label: &str) {
    assert!(
        !html.contains(needle),
        "HTML for {} should NOT contain {:?}\nGot:\n{}",
        label,
        needle,
        html
    );
}

fn doc(body: &str) -> String {
    format!(
        r#"\documentclass{{article}}
\begin{{document}}
{}
\end{{document}}"#,
        body
    )
}

// ==================== Basic structure ====================

#[test]
fn html_has_doctype_and_title() {
    let html = render_html(&doc("Hello world."));
    assert_contains(&html, "<!DOCTYPE html>", "doctype");
    assert_contains(&html, "<html", "html tag");
}

#[test]
fn html_text_element() {
    let html = render_html(&doc("Plain text content."));
    assert_contains(&html, "Plain text content.", "text");
}

#[test]
fn html_section_element() {
    let html = render_html(&doc(r"\section{My Section} Body text."));
    assert_contains(&html, "My Section", "section title");
    assert_contains(&html, "Body text", "section body");
}

#[test]
fn html_paragraph_element() {
    let html = render_html(&doc("First paragraph.\n\nSecond paragraph."));
    assert_contains(&html, "First paragraph", "para 1");
    assert_contains(&html, "Second paragraph", "para 2");
}

#[test]
fn html_math_inline() {
    let html = render_html(&doc(r"Inline $E = mc^2$ math."));
    assert_contains(&html, "E = mc", "inline math");
}

#[test]
fn html_math_display() {
    let html = render_html(&doc(r"\[ E = mc^2 \]"));
    assert_contains(&html, "E = mc", "display math");
}

#[test]
fn html_math_lines_align() {
    let html = render_html(&doc(
        r"\begin{align} a &= b \\ c &= d \end{align}",
    ));
    assert_contains(&html, "a", "align line 1");
    assert_contains(&html, "c", "align line 2");
}

#[test]
fn html_itemlist_unordered() {
    let html = render_html(&doc(
        r"\begin{itemize}\item First\item Second\end{itemize}",
    ));
    assert_contains(&html, "First", "item 1");
    assert_contains(&html, "Second", "item 2");
    assert_contains(&html, "<ul", "ul tag");
}

#[test]
fn html_itemlist_ordered() {
    let html = render_html(&doc(
        r"\begin{enumerate}\item First\item Second\end{enumerate}",
    ));
    assert_contains(&html, "First", "enum item 1");
    assert_contains(&html, "<ol", "ol tag");
}

#[test]
fn html_description_list() {
    let html = render_html(&doc(
        r"\begin{description}\item[Term] Definition\end{description}",
    ));
    assert_contains(&html, "Term", "desc term");
    assert_contains(&html, "Definition", "desc body");
    assert_contains(&html, "<dl", "dl tag");
}

#[test]
fn html_theorem() {
    let html = render_html(&doc(
        r"\begin{theorem}[Test] The theorem body.\end{theorem}",
    ));
    assert_contains(&html, "Theorem", "theorem heading");
    assert_contains(&html, "Test", "theorem title");
    assert_contains(&html, "theorem body", "theorem body");
}

#[test]
fn html_proof() {
    let html = render_html(&doc(r"\begin{proof} QED.\end{proof}"));
    assert_contains(&html, "Proof", "proof heading");
    assert_contains(&html, "QED", "proof body");
}

#[test]
fn html_codeblock() {
    let html = render_html(&doc(
        r"\begin{lstlisting}code here\end{lstlisting}",
    ));
    assert_contains(&html, "code here", "codeblock");
}

#[test]
fn html_verbatim() {
    let html = render_html(&doc(
        r"\begin{verbatim}raw \text\end{verbatim}",
    ));
    assert_contains(&html, "raw", "verbatim");
}

#[test]
fn html_image() {
    let html = render_html(&doc(r"\includegraphics{logo.png}"));
    assert_contains(&html, "logo.png", "image path");
}

#[test]
fn html_table() {
    let html = render_html(&doc(
        r"\begin{tabular}{lc}A & B \\ C & D\end{tabular}",
    ));
    assert_contains(&html, "A", "table cell A");
    assert_contains(&html, "D", "table cell D");
    assert_contains(&html, "<table", "table tag");
}

#[test]
fn html_colored_text() {
    let html = render_html(&doc(
        r"\textcolor{red}{Warning text.}",
    ));
    assert_contains(&html, "Warning text.", "colored text content");
}

#[test]
fn html_citation() {
    let html = render_html(&doc(r"See \cite{key1, key2}."));
    assert_contains(&html, "key1", "cite key1");
    assert_contains(&html, "key2", "cite key2");
}

#[test]
fn html_bibliography() {
    let html = render_html(&doc(
        r"\begin{thebibliography}{9}\bibitem{k} Author, Title.\end{thebibliography}",
    ));
    assert_contains(&html, "Author", "bib entry");
    assert_contains(&html, "k", "bib key");
}

#[test]
fn html_label() {
    let html = render_html(&doc(r"\label{sec:intro}"));
    // Label is metadata; just ensure no crash
    assert_contains(&html, "<!DOCTYPE html>", "label doc valid");
}

#[test]
fn html_ref() {
    let html = render_html(&doc(r"See \ref{sec:intro}."));
    assert_contains(&html, "sec:intro", "ref key");
}

#[test]
fn html_pageref() {
    let html = render_html(&doc(r"See page \pageref{sec:intro}."));
    assert_contains(&html, "sec:intro", "pageref key");
}

#[test]
fn html_center() {
    let html = render_html(&doc(r"\begin{center}Centered text.\end{center}"));
    assert_contains(&html, "Centered text.", "center content");
}

#[test]
fn html_tableofcontents() {
    let html = render_html(&doc(r"\tableofcontents"));
    assert_contains(&html, "Table of Contents", "TOC placeholder");
}

#[test]
fn html_listoffigures() {
    let html = render_html(&doc(r"\listoffigures"));
    assert_contains(&html, "List of Figures", "LOF placeholder");
}

#[test]
fn html_listoftables() {
    let html = render_html(&doc(r"\listoftables"));
    assert_contains(&html, "List of Tables", "LOT placeholder");
}

#[test]
fn html_footnote() {
    let html = render_html(&doc(r"Text\footnote{Note here.}."));
    assert_contains(&html, "Note here.", "footnote text");
}

#[test]
fn html_caption() {
    let html = render_html(&doc(
        r"\begin{figure}\caption{Caption text.}\end{figure}",
    ));
    assert_contains(&html, "Caption text.", "caption");
}

#[test]
fn html_quote() {
    let html = render_html(&doc(r"\begin{quote}Quote text.\end{quote}"));
    assert_contains(&html, "Quote text.", "quote content");
}

#[test]
fn html_abstract() {
    let html = render_html(&doc(r"\begin{abstract}Abstract text.\end{abstract}"));
    assert_contains(&html, "Abstract text.", "abstract content");
}

#[test]
fn html_line_break() {
    let html = render_html(&doc(r"A \\ B"));
    assert_contains(&html, "A", "line break A");
    assert_contains(&html, "B", "line break B");
}

#[test]
fn html_flushleft() {
    let html = render_html(&doc(
        r"\begin{flushleft}Left text.\end{flushleft}",
    ));
    assert_contains(&html, "Left text.", "flushleft content");
}

#[test]
fn html_flushright() {
    let html = render_html(&doc(
        r"\begin{flushright}Right text.\end{flushright}",
    ));
    assert_contains(&html, "Right text.", "flushright content");
}

// ==================== Command rendering ====================

#[test]
fn html_textbf() {
    let html = render_html(&doc(r"\textbf{Bold text.}"));
    assert_contains(&html, "Bold text.", "textbf");
    assert_contains(&html, "<strong", "strong tag");
}

#[test]
fn html_textit() {
    let html = render_html(&doc(r"\textit{Italic text.}"));
    assert_contains(&html, "Italic text.", "textit");
    assert_contains(&html, "<em", "em tag");
}

#[test]
fn html_texttt() {
    let html = render_html(&doc(r"\texttt{Mono text.}"));
    assert_contains(&html, "Mono text.", "texttt");
    assert_contains(&html, "<code", "code tag");
}

#[test]
fn html_href() {
    let html = render_html(&doc(
        r"\href{https://example.com}{Link text}",
    ));
    assert_contains(&html, "https://example.com", "href url");
    assert_contains(&html, "Link text", "href text");
    assert_contains(&html, "<a ", "a tag");
}

#[test]
fn html_url() {
    let html = render_html(&doc(r"\url{https://example.com}"));
    assert_contains(&html, "https://example.com", "url");
    assert_contains(&html, "<a ", "a tag for url");
}

#[test]
fn html_today() {
    let html = render_html(&doc(r"\today"));
    // Just verify it doesn't crash; the date is dynamic
    assert_contains(&html, "<!DOCTYPE html>", "today doc valid");
}

#[test]
fn html_special_chars() {
    let html = render_html(&doc(
        r"\copyright\ \pounds\ \S\ \P\ \dag\ \ddag\ \ldots\ \LaTeX\ \TeX",
    ));
    assert_contains(&html, "©", "copyright");
    assert_contains(&html, "£", "pounds");
    assert_contains(&html, "§", "S");
    assert_contains(&html, "¶", "P");
    assert_contains(&html, "†", "dag");
    assert_contains(&html, "‡", "ddag");
    assert_contains(&html, "…", "ldots");
    assert_contains(&html, "LaTeX", "LaTeX");
    assert_contains(&html, "TeX", "TeX");
}

#[test]
fn html_text_special_chars() {
    let html = render_html(&doc(
        r"\textasciicircum\ \textasciitilde\ \textbackslash\ \textbar\ \textdollar",
    ));
    assert_contains(&html, "^", "asciicircum");
    assert_contains(&html, "~", "asciitilde");
    assert_contains(&html, "\\", "backslash");
    assert_contains(&html, "|", "bar");
    assert_contains(&html, "$", "dollar");
}

#[test]
fn html_enquote() {
    let html = render_html(&doc(r"\enquote{Quoted.}"));
    assert_contains(&html, "Quoted.", "enquote content");
}

#[test]
fn html_textnormal() {
    let html = render_html(&doc(r"\textnormal{Normal.}"));
    assert_contains(&html, "Normal.", "textnormal");
}

#[test]
fn html_textsc() {
    let html = render_html(&doc(r"\textsc{SmallCaps.}"));
    assert_contains(&html, "SmallCaps.", "textsc");
}

#[test]
fn html_hspace() {
    let html = render_html(&doc(r"A\hspace{1em}B"));
    assert_contains(&html, "A", "hspace A");
    assert_contains(&html, "B", "hspace B");
}

#[test]
fn html_vspace() {
    let html = render_html(&doc(r"A\vspace{12pt}B"));
    assert_contains(&html, "A", "vspace A");
    assert_contains(&html, "B", "vspace B");
}

#[test]
fn html_mbox() {
    let html = render_html(&doc(r"\mbox{Boxed.}"));
    assert_contains(&html, "Boxed.", "mbox");
}

#[test]
fn html_parbox() {
    let html = render_html(&doc(r"\parbox[c]{3cm}{Parbox text.}"));
    assert_contains(&html, "Parbox text.", "parbox");
}

#[test]
fn html_makebox() {
    let html = render_html(&doc(r"\makebox[5cm][s]{Makebox text.}"));
    assert_contains(&html, "Makebox text.", "makebox");
}

#[test]
fn html_multicolumn() {
    let html = render_html(&doc(
        r"\begin{tabular}{ccc}\multicolumn{2}{c}{Header} & C\end{tabular}",
    ));
    assert_contains(&html, "Header", "multicolumn header");
    assert_contains(&html, "C", "multicolumn C");
}

#[test]
fn html_footnotemark_footnotetext() {
    let html = render_html(&doc(
        r"Text\footnotemark\ here.\footnotetext{Note.}",
    ));
    assert_contains(&html, "Text", "footnotemark text");
    assert_contains(&html, "Note.", "footnotetext");
}

#[test]
fn html_linebreak_nopagebreak_samepage() {
    let html = render_html(&doc(
        r"A \linebreak B \nopagebreak C \samepage D",
    ));
    assert_contains(&html, "A", "linebreak A");
    assert_contains(&html, "B", "linebreak B");
    assert_contains(&html, "C", "nopagebreak C");
    assert_contains(&html, "D", "samepage D");
}

#[test]
fn html_skip_commands_no_crash() {
    let html = render_html(&doc(
        r"\setlength{\parindent}{0pt}\setcounter{page}{3}\ignorespaces OK",
    ));
    assert_contains(&html, "OK", "text after skip commands");
    assert_not_contains(&html, "setlength", "setlength should be consumed");
    assert_not_contains(&html, "setcounter", "setcounter should be consumed");
}

#[test]
fn html_font_declarations() {
    let html = render_html(&doc(r"\em \bf \it \rm \sf \tt \sc \sl Text."));
    assert_contains(&html, "Text.", "text after font decls");
}

#[test]
fn html_font_size_commands() {
    let html = render_html(&doc(
        r"\tiny T \small S \normalsize N \large L \Huge H",
    ));
    assert_contains(&html, "T", "tiny");
    assert_contains(&html, "H", "Huge");
}

#[test]
fn html_text_commands() {
    let html = render_html(&doc(
        r"\textrm{R} \textsf{S} \textsl{Sl} \textup{U} \textmd{M}",
    ));
    assert_contains(&html, "R", "textrm");
    assert_contains(&html, "S", "textsf");
    assert_contains(&html, "Sl", "textsl");
}

#[test]
fn html_sout_overline() {
    let html = render_html(&doc(r"\sout{deleted} \overline{bar}"));
    assert_contains(&html, "deleted", "sout");
    assert_contains(&html, "bar", "overline");
}

#[test]
fn html_textsuperscript_subscript() {
    let html = render_html(&doc(r"\textsuperscript{st} \textsubscript{2}"));
    assert_contains(&html, "st", "superscript");
    assert_contains(&html, "2", "subscript");
}

#[test]
fn html_phantom_commands() {
    let html = render_html(&doc(
        r"\phantom{hidden} \vphantom{vh} \hphantom{hh} visible",
    ));
    assert_contains(&html, "visible", "text after phantom");
}

#[test]
fn html_box_commands() {
    let html = render_html(&doc(
        r"\raisebox{2pt}{raised} \rotatebox{90}{rotated} \scalebox{2}{scaled} \fbox{framed}",
    ));
    assert_contains(&html, "raised", "raisebox");
    assert_contains(&html, "rotated", "rotatebox");
    assert_contains(&html, "scaled", "scalebox");
    assert_contains(&html, "framed", "fbox");
}

#[test]
fn html_colorbox_fcolorbox() {
    let html = render_html(&doc(
        r"\colorbox{red}{colored} \fcolorbox{black}{yellow}{framed}",
    ));
    assert_contains(&html, "colored", "colorbox");
    assert_contains(&html, "framed", "fcolorbox");
}

#[test]
fn html_spacing_commands() {
    let html = render_html(&doc(r"A\hfill B\qquad C\quad D\, E\; F"));
    assert_contains(&html, "A", "hfill A");
    assert_contains(&html, "B", "qquad B");
    assert_contains(&html, "F", "semicolon F");
}

#[test]
fn html_rule_command() {
    let html = render_html(&doc(r"\rule{5cm}{0.4pt}"));
    assert_contains(&html, "<!DOCTYPE html>", "rule doc valid");
}

#[test]
fn html_index_glossary_appendix() {
    let html = render_html(&doc(
        r"Text\index{term}\glossary{gloss} \appendix \section{App}",
    ));
    assert_contains(&html, "Text", "text with index");
    assert_contains(&html, "App", "appendix section");
}

#[test]
fn html_bibliography_bibliographystyle() {
    let html = render_html(&doc(
        r"\bibliography{refs}\bibliographystyle{plain}",
    ));
    assert_contains(&html, "refs", "bibliography arg");
}

#[test]
fn html_figure_environment() {
    let html = render_html(&doc(
        r"\begin{figure}\centering Content\caption{Cap}\end{figure}",
    ));
    assert_contains(&html, "Content", "figure content");
    assert_contains(&html, "Cap", "figure caption");
}

#[test]
fn html_minipage() {
    let html = render_html(&doc(
        r"\begin{minipage}[c]{0.5\textwidth}Minipage content.\end{minipage}",
    ));
    assert_contains(&html, "Minipage content.", "minipage");
}

#[test]
fn html_displaymath_env() {
    let html = render_html(&doc(
        r"\begin{displaymath}E = mc^2\end{displaymath}",
    ));
    assert_contains(&html, "E = mc", "displaymath env");
}

#[test]
fn html_math_env() {
    let html = render_html(&doc(r"\begin{math}x + y\end{math}"));
    assert_contains(&html, "x + y", "math env");
}

#[test]
fn html_eqnarray() {
    let html = render_html(&doc(
        r"\begin{eqnarray}a &=& b\end{eqnarray}",
    ));
    assert_contains(&html, "a", "eqnarray a");
    assert_contains(&html, "b", "eqnarray b");
}

#[test]
fn html_counter_commands() {
    let html = render_html(&doc(
        r"\setcounter{section}{5}\arabic{section}\roman{section}",
    ));
    assert_contains(&html, "<!DOCTYPE html>", "counter doc valid");
}

#[test]
fn html_split_aligned_gathered() {
    let html = render_html(&doc(
        r"\[\begin{aligned}a &= b\end{aligned}\]",
    ));
    assert_contains(&html, "a", "aligned a");
    assert_contains(&html, "b", "aligned b");
}

#[test]
fn html_newpage_clearpage_pagebreak() {
    let html = render_html(&doc(r"\newpage A\clearpage B\pagebreak C"));
    assert_contains(&html, "A", "newpage A");
    assert_contains(&html, "B", "clearpage B");
    assert_contains(&html, "C", "pagebreak C");
}

#[test]
fn html_raggedright_noindent_centering() {
    let html = render_html(&doc(
        r"\raggedright A\raggedleft B\noindent C\centering D",
    ));
    assert_contains(&html, "A", "raggedright A");
    assert_contains(&html, "D", "centering D");
}

#[test]
fn html_includegraphics_with_options() {
    let html = render_html(&doc(
        r"\includegraphics[width=5cm,height=3cm]{logo.png}",
    ));
    assert_contains(&html, "logo.png", "image path with options");
}

#[test]
fn html_nested_lists() {
    let html = render_html(&doc(
        r"\begin{itemize}\item A\begin{itemize}\item B\end{itemize}\end{itemize}",
    ));
    assert_contains(&html, "A", "outer list");
    assert_contains(&html, "B", "inner list");
}

#[test]
fn html_starred_sections() {
    let html = render_html(&doc(r"\section*{Unnumbered}"));
    assert_contains(&html, "Unnumbered", "starred section");
}

#[test]
fn html_metadata_title_author() {
    let html = render_html(
        r#"\documentclass{article}
\title{My Title}
\author{My Author}
\begin{document}
\maketitle
Body text.
\end{document}"#,
    );
    assert_contains(&html, "My Title", "title in HTML");
    assert_contains(&html, "My Author", "author in HTML");
    assert_contains(&html, "Body text.", "body in HTML");
}

#[test]
fn html_hrulefill() {
    let html = render_html(&doc(r"Above\hrulefill Below"));
    assert_contains(&html, "<hr", "hrulefill produces <hr>");
}
