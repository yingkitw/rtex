//! HTML renderer for parsed LaTeX documents.

use crate::error::LatexError;
use crate::parser::TexElement;

use super::common::{extract_metadata, render_elements_html};

pub fn render(elements: &[TexElement]) -> Result<Vec<u8>, LatexError> {
    let meta = extract_metadata(elements);
    let title = meta.title.as_deref().unwrap_or("Document");
    let mut body = String::new();
    render_elements_html(elements, &mut body);

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"utf-8\" />\n");
    html.push_str(&format!(
        "  <title>{}</title>\n",
        super::common::escape_html(title)
    ));
    if let Some(author) = &meta.author {
        html.push_str(&format!(
            "  <meta name=\"author\" content=\"{}\" />\n",
            super::common::escape_html(author)
        ));
    }
    html.push_str("  <style>body{font-family:Georgia,serif;max-width:48rem;margin:2rem auto;line-height:1.5}.math.display{text-align:center;margin:1rem 0}.center{text-align:center}.abstract{border:1px solid #ccc;padding:1rem;margin:1rem 0}</style>\n");
    html.push_str("</head>\n<body>\n");
    if meta.title.is_some() || meta.author.is_some() || meta.date.is_some() {
        html.push_str("<header>\n");
        if let Some(t) = &meta.title {
            html.push_str(&format!("<h1>{}</h1>\n", super::common::escape_html(t)));
        }
        if let Some(a) = &meta.author {
            html.push_str(&format!(
                "<p class=\"author\">{}</p>\n",
                super::common::escape_html(a)
            ));
        }
        if let Some(d) = &meta.date {
            html.push_str(&format!(
                "<p class=\"date\">{}</p>\n",
                super::common::escape_html(d)
            ));
        }
        html.push_str("</header>\n");
    }
    html.push_str(&body);
    html.push_str("</body>\n</html>\n");
    Ok(html.into_bytes())
}
