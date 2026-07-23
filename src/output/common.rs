//! Shared helpers for multi-format document rendering.

use crate::math_formatter::MathFormatter;
use crate::parser::{BibEntry, TexElement};

/// Document metadata extracted from parsed elements.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}

pub fn extract_metadata(elements: &[TexElement]) -> DocumentMeta {
    let mut meta = DocumentMeta::default();
    for element in elements {
        if let TexElement::Command { name, args } = element {
            match name.as_str() {
                "title" if !args.is_empty() => meta.title = Some(args[0].clone()),
                "author" if !args.is_empty() => meta.author = Some(args[0].clone()),
                "date" if !args.is_empty() => meta.date = Some(args[0].clone()),
                _ => {}
            }
        }
    }
    meta
}

pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn escape_xml(text: &str) -> String {
    escape_html(text)
}

pub fn format_math_inline(math: &str) -> String {
    MathFormatter::format(math)
}

pub fn format_math_display(math: &str) -> String {
    MathFormatter::format(math)
}

pub fn render_elements_html(elements: &[TexElement], out: &mut String) {
    for element in elements {
        render_element_html(element, out);
    }
}

pub fn render_element_html(element: &TexElement, out: &mut String) {
    match element {
        TexElement::Text(text) => out.push_str(&escape_html(text)),
        TexElement::Command { name, args } => render_command_html(name, args, out),
        TexElement::Section { level, title } => {
            let tag = match level {
                0 | 6 => "h1",
                1 => "h2",
                2 => "h3",
                3 => "h4",
                4 => "h5",
                _ => "h6",
            };
            out.push_str(&format!("<{tag}>{}</{tag}>\n", escape_html(title)));
        }
        TexElement::Paragraph => out.push_str("<p></p>\n"),
        TexElement::MathInline(math) => {
            out.push_str("<span class=\"math inline\">");
            out.push_str(&escape_html(&format_math_inline(math)));
            out.push_str("</span>");
        }
        TexElement::MathDisplay(math) => {
            out.push_str("<div class=\"math display\">");
            out.push_str(&escape_html(&format_math_display(math)));
            out.push_str("</div>\n");
        }
        TexElement::MathLines { lines, .. } => {
            out.push_str("<div class=\"math display multi\">");
            for (idx, line) in lines.iter().enumerate() {
                if idx > 0 {
                    out.push_str("<br />");
                }
                out.push_str(&escape_html(&format_math_display(line)));
            }
            out.push_str("</div>\n");
        }
        TexElement::ItemList {
            ordered,
            labels,
            items,
        } => {
            let tag = if *ordered { "ol" } else { "ul" };
            out.push_str(&format!("<{tag}>\n"));
            for (idx, item) in items.iter().enumerate() {
                out.push_str("<li>");
                if let Some(label) = labels.get(idx).and_then(|l| l.as_ref()) {
                    out.push_str(&format!(
                        "<span class=\"label\">{}</span> ",
                        escape_html(label)
                    ));
                }
                render_elements_html(item, out);
                out.push_str("</li>\n");
            }
            out.push_str(&format!("</{tag}>\n"));
        }
        TexElement::DescriptionList { items } => {
            out.push_str("<dl>\n");
            for item in items {
                if !item.term.is_empty() {
                    out.push_str(&format!("<dt>{}</dt>\n", escape_html(&item.term)));
                }
                out.push_str("<dd>");
                render_elements_html(&item.body, out);
                out.push_str("</dd>\n");
            }
            out.push_str("</dl>\n");
        }
        TexElement::Theorem { kind, title, body } => {
            let mut heading = kind[..1].to_uppercase() + &kind[1..];
            if let Some(t) = title {
                heading.push_str(&format!(" ({})", t));
            }
            out.push_str(&format!(
                "<section class=\"theorem\"><h3 class=\"theorem-heading\">{}</h3>",
                escape_html(&heading)
            ));
            render_elements_html(body, out);
            out.push_str("</section>\n");
        }
        TexElement::CodeBlock(code) => {
            out.push_str("<pre><code>");
            out.push_str(&escape_html(code));
            out.push_str("</code></pre>\n");
        }
        TexElement::Image {
            path,
            width,
            height,
        } => {
            let mut attrs = format!(
                "src=\"{}\" alt=\"{}\"",
                escape_html(path),
                escape_html(path)
            );
            if let Some(w) = width {
                attrs.push_str(&format!(" width=\"{}\"", escape_html(w)));
            }
            if let Some(h) = height {
                attrs.push_str(&format!(" height=\"{}\"", escape_html(h)));
            }
            out.push_str(&format!("<img {attrs} />\n"));
        }
        TexElement::Table(table) => render_table_html(table, out),
        TexElement::ColoredText { color, text } => {
            out.push_str(&format!(
                "<span style=\"color:{}\">{}</span>",
                escape_html(color),
                escape_html(text)
            ));
        }
        TexElement::Citation { keys } => {
            out.push_str(&format!("[{}]", escape_html(&keys.join(", "))));
        }
        TexElement::Bibliography { entries } => render_bibliography_html(entries, out),
        TexElement::Label { key } => {
            out.push_str(&format!("<span id=\"{}\"></span>", escape_html(key)));
        }
        TexElement::Ref { key } => out.push_str(&escape_html(key)),
        TexElement::PageRef { key } => {
            out.push_str(&format!("page {}", escape_html(key)));
        }
        TexElement::Center(content) => {
            out.push_str("<div class=\"center\">");
            render_elements_html(content, out);
            out.push_str("</div>\n");
        }
        TexElement::TableOfContents => {
            out.push_str("<nav class=\"toc\"><em>Table of Contents</em></nav>\n")
        }
        TexElement::ListOfFigures => {
            out.push_str("<section class=\"lof\"><em>List of Figures</em></section>\n")
        }
        TexElement::ListOfTables => {
            out.push_str("<section class=\"lot\"><em>List of Tables</em></section>\n")
        }
        TexElement::Footnote { text } => {
            out.push_str("<sup class=\"footnote\">");
            out.push_str(&escape_html(text));
            out.push_str("</sup>");
        }
        TexElement::Caption { text } => {
            out.push_str("<figcaption>");
            out.push_str(&escape_html(text));
            out.push_str("</figcaption>\n");
        }
        TexElement::Quote(content) => {
            out.push_str("<blockquote>");
            render_elements_html(content, out);
            out.push_str("</blockquote>\n");
        }
        TexElement::Abstract(content) => {
            out.push_str("<section class=\"abstract\"><h2>Abstract</h2>");
            render_elements_html(content, out);
            out.push_str("</section>\n");
        }
    }
}

fn render_command_html(name: &str, args: &[String], out: &mut String) {
    match name {
        // Document metadata commands are surfaced via the <header> block
        // emitted by `render_elements_html`. Skip them in body output so
        // they don't appear twice.
        "title" | "author" | "date" => (),
        "textbf" | "bf" if !args.is_empty() => {
            out.push_str("<strong>");
            out.push_str(&escape_html(&args[0]));
            out.push_str("</strong>");
        }
        "textit" | "it" | "emph" if !args.is_empty() => {
            out.push_str("<em>");
            out.push_str(&escape_html(&args[0]));
            out.push_str("</em>");
        }
        "texttt" | "tt" if !args.is_empty() => {
            out.push_str("<code>");
            out.push_str(&escape_html(&args[0]));
            out.push_str("</code>");
        }
        "underline" if !args.is_empty() => {
            out.push_str("<u>");
            out.push_str(&escape_html(&args[0]));
            out.push_str("</u>");
        }
        "url" if !args.is_empty() => {
            let url = &args[0];
            out.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                escape_html(url),
                escape_html(url)
            ));
        }
        "href" if args.len() >= 2 => {
            let url = &args[0];
            let text = &args[1];
            out.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                escape_html(url),
                escape_html(text)
            ));
        }
        "newline" | "linebreak" => out.push_str("<br />\n"),
        "hfill" => out.push_str("<span class=\"hfill\"></span>"),
        _ if !args.is_empty() => {
            out.push_str(&escape_html(&args[0]));
        }
        _ => {}
    }
}

fn render_table_html(table: &crate::table::Table, out: &mut String) {
    out.push_str("<table>\n");
    for row in &table.rows {
        if row.is_separator {
            continue;
        }
        out.push_str("<tr>");
        for cell in &row.cells {
            out.push_str("<td>");
            out.push_str(&escape_html(cell));
            out.push_str("</td>");
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</table>\n");
}

fn render_bibliography_html(entries: &[BibEntry], out: &mut String) {
    out.push_str("<section class=\"bibliography\"><h2>References</h2><ol>\n");
    for (idx, entry) in entries.iter().enumerate() {
        out.push_str(&format!(
            "<li id=\"bib-{}\"><span class=\"label\">[{}]</span> {}</li>\n",
            escape_html(&entry.key),
            idx + 1,
            escape_html(&entry.text)
        ));
    }
    out.push_str("</ol></section>\n");
}

pub fn plain_text_from_elements(elements: &[TexElement]) -> String {
    let mut out = String::new();
    collect_plain_text(elements, &mut out);
    out
}

fn collect_plain_text(elements: &[TexElement], out: &mut String) {
    for element in elements {
        match element {
            TexElement::Text(text) => out.push_str(text),
            TexElement::Command { args, .. } if !args.is_empty() => out.push_str(&args[0]),
            TexElement::Command { .. } => {}
            TexElement::Section { title, .. } => {
                out.push_str(title);
                out.push('\n');
            }
            TexElement::Paragraph => out.push_str("\n\n"),
            TexElement::MathInline(math) | TexElement::MathDisplay(math) => {
                out.push_str(&format_math_inline(math));
            }
            TexElement::MathLines { lines, .. } => {
                for (idx, line) in lines.iter().enumerate() {
                    if idx > 0 {
                        out.push('\n');
                    }
                    out.push_str(&format_math_inline(line));
                }
            }
            TexElement::ItemList { items, .. } => {
                for item in items {
                    out.push_str("• ");
                    collect_plain_text(item, out);
                    out.push('\n');
                }
            }
            TexElement::DescriptionList { items } => {
                for item in items {
                    if !item.term.is_empty() {
                        out.push_str(&item.term);
                        out.push_str(": ");
                    }
                    collect_plain_text(&item.body, out);
                    out.push('\n');
                }
            }
            TexElement::CodeBlock(code) => {
                out.push_str(code);
                out.push('\n');
            }
            TexElement::Image { path, .. } => {
                out.push_str(path);
                out.push('\n');
            }
            TexElement::Table(table) => {
                for row in &table.rows {
                    if !row.is_separator {
                        out.push_str(&row.cells.join("\t"));
                        out.push('\n');
                    }
                }
            }
            TexElement::ColoredText { text, .. } => out.push_str(text),
            TexElement::Citation { keys } => {
                out.push('[');
                out.push_str(&keys.join(", "));
                out.push(']');
            }
            TexElement::Bibliography { entries } => {
                for entry in entries {
                    out.push_str(&entry.text);
                    out.push('\n');
                }
            }
            TexElement::Ref { key } | TexElement::PageRef { key } => out.push_str(key),
            TexElement::Center(content)
            | TexElement::Quote(content)
            | TexElement::Abstract(content) => collect_plain_text(content, out),
            TexElement::Theorem { kind, title, body } => {
                out.push_str(&kind.to_uppercase());
                if let Some(t) = title {
                    out.push_str(&format!(" ({t})"));
                }
                out.push_str(": ");
                collect_plain_text(body, out);
                out.push('\n');
            }
            TexElement::Footnote { text } | TexElement::Caption { text } => out.push_str(text),
            TexElement::Label { .. }
            | TexElement::TableOfContents
            | TexElement::ListOfFigures
            | TexElement::ListOfTables => {}
        }
    }
}
