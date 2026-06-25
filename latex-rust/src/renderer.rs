//! Renderers for converting AST to various output formats

use crate::ast::*;
use std::collections::HashMap;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

/// HTML renderer for LaTeX documents
pub struct HtmlRenderer {
    math_renderer: MathRenderer,
    command_handlers: HashMap<String, fn(&HtmlRenderer, &[Argument]) -> String>,
}

impl HtmlRenderer {
    /// Create a new HTML renderer
    pub fn new() -> Self {
        let mut renderer = Self {
            math_renderer: MathRenderer::new(),
            command_handlers: HashMap::new(),
        };
        renderer.register_default_commands();
        renderer
    }

    /// Render a document to HTML
    pub fn render(&self, document: &Document) -> String {
        let mut html = String::new();
        
        // HTML document structure
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        
        // Add title if available
        if let Some(title) = &document.metadata.title {
            html.push_str(&format!("    <title>{}</title>\n", self.escape_html(title)));
        } else {
            html.push_str("    <title>LaTeX Document</title>\n");
        }
        
        // Add CSS for styling
        html.push_str("    <style>\n");
        html.push_str(self.get_default_css());
        html.push_str("    </style>\n");
        
        // Add MathJax for math rendering
        html.push_str("    <script src=\"https://polyfill.io/v3/polyfill.min.js?features=es6\"></script>\n");
        html.push_str("    <script id=\"MathJax-script\" async src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\"></script>\n");
        html.push_str("    <script>\n");
        html.push_str("        window.MathJax = {\n");
        html.push_str("            tex: {\n");
        html.push_str("                inlineMath: [['$', '$'], ['\\\\(', '\\\\)']],\n");
        html.push_str("                displayMath: [['$$', '$$'], ['\\\\[', '\\\\]']]\n");
        html.push_str("            }\n");
        html.push_str("        };\n");
        html.push_str("    </script>\n");
        
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        
        // Render document metadata
        if document.metadata.title.is_some() || document.metadata.author.is_some() {
            html.push_str("    <header class=\"document-header\">\n");
            
            if let Some(title) = &document.metadata.title {
                html.push_str(&format!("        <h1 class=\"document-title\">{}</h1>\n", self.escape_html(title)));
            }
            
            if let Some(author) = &document.metadata.author {
                html.push_str(&format!("        <p class=\"document-author\">{}</p>\n", self.escape_html(author)));
            }
            
            if let Some(date) = &document.metadata.date {
                html.push_str(&format!("        <p class=\"document-date\">{}</p>\n", self.escape_html(date)));
            }
            
            html.push_str("    </header>\n");
        }
        
        // Render preamble packages as comments
        for node in &document.preamble {
            if let Node::Package { name, options } = node {
                html.push_str(&format!("    {}", self.render_package(name, options)));
            }
        }
        
        // Render document body
        html.push_str("    <main class=\"document-body\">\n");
        for node in &document.body {
            html.push_str(&self.render_node(node));
        }
        html.push_str("    </main>\n");
        
        html.push_str("</body>\n");
        html.push_str("</html>\n");
        
        html
    }

    /// Render a single node
    pub fn render_node(&self, node: &Node) -> String {
        match node {
            Node::Text(text) => self.escape_html(text),
            
            Node::Command { name, args, content } => {
                self.render_command(name, args, content.as_deref())
            }
            
            Node::Environment { name, args, content } => {
                self.render_environment(name, args, content)
            }
            
            Node::Math { display, content } => {
                self.math_renderer.render(content, *display)
            }
            
            Node::Group(nodes) => {
                let mut html = String::new();
                for node in nodes {
                    html.push_str(&self.render_node(node));
                }
                html
            }
            
            Node::List { list_type, items } => {
                self.render_list(list_type, items)
            }
            
            Node::Table { table_type, alignment, rows, caption, label, position } => {
                self.render_table(table_type, alignment, rows, caption.as_deref(), label.as_deref(), position.as_deref())
            }
            
            Node::Figure { path, caption, label, width, height } => {
                self.render_figure(path, caption.as_deref(), label.as_deref(), width.as_deref(), height.as_deref())
            }
            
            Node::Section { level, title, content } => {
                self.render_section(level, title, content)
            }
            
            Node::MathEnvironment { env_type, content, label, numbered, equation_number } => {
                self.render_math_environment(env_type, content, label.as_deref(), *numbered, *equation_number)
            }
            
            Node::Label(label) => {
                format!("<span id=\"{}\" class=\"label\"></span>", self.escape_html(label))
            }
            
            Node::Reference { ref_type, label } => {
                self.render_reference(ref_type, label)
            }
            
            Node::Footnote { content } => {
                self.render_footnote(content)
            }
            
            Node::Spacing { space_type, amount } => {
                self.render_spacing(space_type, amount.as_deref())
            }
            
            Node::CommandDefinition { name, num_args, default_args, definition } => {
                self.render_command_definition(name, num_args.as_ref(), default_args, definition)
            }
            
            Node::Citation { cite_type, keys, prenote, postnote } => {
                self.render_citation(cite_type, keys, prenote.as_deref(), postnote.as_deref())
            }
            
            Node::BibliographyEntry { key, entry_type, fields } => {
                self.render_bibliography_entry(key, entry_type, fields)
            }
            
            Node::Bibliography { style, entries } => {
                self.render_bibliography(style.as_deref(), entries)
            }
            
            Node::Package { name, options } => {
                self.render_package(name, options)
            }
            
            Node::CodeBlock { language, code, style, line_numbers, caption, label } => {
                self.render_code_block(language.as_deref(), code, style, *line_numbers, caption.as_deref(), label.as_deref())
            }
            
            Node::Paragraph => "<p></p>\n".to_string(),
            
            Node::LineBreak => "<br>\n".to_string(),
            
            Node::Whitespace(ws) => ws.clone(),
        }
    }

    /// Render a command
    fn render_command(&self, name: &str, args: &[Argument], content: Option<&Node>) -> String {
        // Check for custom handler
        if let Some(handler) = self.command_handlers.get(name) {
            return handler(self, args);
        }

        // Default command rendering
        match name {
            "textbf" => self.render_text_formatting("strong", args),
            "textit" => self.render_text_formatting("em", args),
            "texttt" => self.render_text_formatting("code", args),
            "underline" => self.render_text_formatting("u", args),
            "emph" => self.render_text_formatting("em", args),
            
            // Font sizes
            "tiny" => self.render_font_size("tiny", args),
            "scriptsize" => self.render_font_size("scriptsize", args),
            "footnotesize" => self.render_font_size("footnotesize", args),
            "small" => self.render_font_size("small", args),
            "normalsize" => self.render_font_size("normalsize", args),
            "large" => self.render_font_size("large", args),
            "Large" => self.render_font_size("Large", args),
            "LARGE" => self.render_font_size("LARGE", args),
            "huge" => self.render_font_size("huge", args),
            "Huge" => self.render_font_size("Huge", args),
            
            // Font families
            "textrm" => self.render_text_formatting("span", args).replace("<span>", "<span class=\"font-roman\">"),
            "textsf" => self.render_text_formatting("span", args).replace("<span>", "<span class=\"font-sans\">"),
            "textsc" => self.render_text_formatting("span", args).replace("<span>", "<span class=\"font-smallcaps\">"),
            
            // Text alignment
            "centering" => "<div class=\"text-center\">".to_string(),
            "raggedright" => "<div class=\"text-left\">".to_string(),
            "raggedleft" => "<div class=\"text-right\">".to_string(),
            "maketitle" => String::new(), // Handled in document rendering
            "newpage" => "<div class=\"page-break\"></div>\n".to_string(),
            "vspace" => "<div class=\"vspace\"></div>\n".to_string(),
            "hspace" => "<span class=\"hspace\"></span>".to_string(),
            
            // Mathematical commands
            "frac" => self.render_frac(args),
            "sqrt" => self.render_sqrt(args),
            "sum" | "int" | "prod" | "lim" => self.render_math_operator(name, args),
            "displaystyle" | "textstyle" => self.render_math_style(name, args),
            
            // List item
            "item" => self.render_item(args),
            
            _ => {
                // Unknown command - render as-is with warning comment
                format!("<!-- Unknown command: {name} -->")
            }
        }
    }

    /// Render font size commands
    fn render_font_size(&self, size: &str, args: &[Argument]) -> String {
        if args.is_empty() {
            format!("<span class=\"font-{size}\">")
        } else {
            let mut html = format!("<span class=\"font-{size}\">\n");
            for arg in args {
                match arg {
                    Argument::Required(nodes) => {
                        for node in nodes {
                            html.push_str(&self.render_node(node));
                        }
                    }
                    Argument::Optional(_) => {}
                }
            }
            html.push_str("</span>");
            html
        }
    }

    /// Render text formatting commands
    fn render_text_formatting(&self, tag: &str, args: &[Argument]) -> String {
        if let Some(Argument::Required(nodes)) = args.first() {
            let content = nodes.iter()
                .map(|node| self.render_node(node))
                .collect::<String>();
            format!("<{}>{}</{}>", tag, content, tag.split_whitespace().next().unwrap_or(tag))
        } else {
            String::new()
        }
    }

    /// Render an environment
    fn render_environment(&self, name: &str, _args: &[Argument], content: &[Node]) -> String {
        let content_html = content.iter()
            .map(|node| self.render_node(node))
            .collect::<String>();

        match name {
            "itemize" => format!("<ul>\n{content_html}\n</ul>\n"),
            "enumerate" => format!("<ol>\n{content_html}\n</ol>\n"),
            "description" => format!("<dl>\n{content_html}\n</dl>\n"),
            "quote" => format!("<blockquote>\n{content_html}\n</blockquote>\n"),
            "center" => format!("<div class=\"center\">\n{content_html}\n</div>\n"),
            "flushleft" => format!("<div class=\"text-left\">\n{content_html}\n</div>\n"),
            "flushright" => format!("<div class=\"text-right\">\n{content_html}\n</div>\n"),
            "verbatim" => format!("<pre><code>\n{}\n</code></pre>\n", self.escape_html(&content_html)),
            "abstract" => format!("<div class=\"abstract\">\n<h2>Abstract</h2>\n{content_html}\n</div>\n"),
            _ => {
                format!("<div class=\"environment-{name}\">\n{content_html}\n</div>\n")
            }
        }
    }

    /// Render a list
    fn render_list(&self, list_type: &ListType, items: &[ListItem]) -> String {
        let tag = match list_type {
            ListType::Itemize => "ul",
            ListType::Enumerate => "ol",
            ListType::Description => "dl",
        };

        let mut html = format!("<{tag}>\n");
        
        for item in items {
            // Filter out leading/trailing whitespace and line breaks from list item content
            let cleaned_content: Vec<&Node> = item.content.iter()
                .skip_while(|node| matches!(node, Node::Whitespace(_) | Node::LineBreak))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .skip_while(|node| matches!(node, Node::Whitespace(_) | Node::LineBreak))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            
            let content = cleaned_content.iter()
                .map(|node| self.render_node(node))
                .collect::<String>();
            
            match list_type {
                ListType::Description => {
                    if let Some(label) = &item.label {
                        html.push_str(&format!("    <dt>{}</dt>\n", self.escape_html(label)));
                    }
                    html.push_str(&format!("    <dd>{content}</dd>\n"));
                }
                _ => {
                    html.push_str(&format!("    <li>{content}</li>\n"));
                }
            }
        }
        
        html.push_str(&format!("</{tag}>\n"));
        html
    }

    /// Render a table
    fn render_table(&self, table_type: &TableType, alignment: &[ColumnAlignment], rows: &[TableRow], caption: Option<&str>, label: Option<&str>, position: Option<&str>) -> String {
        let table_class = match table_type {
            TableType::Tabular => "tabular",
            TableType::Longtable => "longtable",
            TableType::Array => "array",
            TableType::Tabularx => "tabularx",
        };
        
        let mut html = format!("<table class=\"{table_class}\">\n");
        
        // Add caption if present
        if let Some(cap) = caption {
            html.push_str(&format!("    <caption>{}</caption>\n", self.escape_html(cap)));
        }
        
        for row in rows {
            // Render rules before the row
            for rule in &row.rules_before {
                match rule {
                    TableRule::Hline => html.push_str("    <tr class=\"hline\"><td colspan=\"100%\"></td></tr>\n"),
                    TableRule::Cline(range) => {
                         html.push_str(&format!("    <tr class=\"cline\"><td colspan=\"100%\" data-range=\"{range}\"></td></tr>\n"));
                     }
                    TableRule::Toprule => html.push_str("    <tr class=\"toprule\"><td colspan=\"100%\"></td></tr>\n"),
                    TableRule::Midrule => html.push_str("    <tr class=\"midrule\"><td colspan=\"100%\"></td></tr>\n"),
                    TableRule::Bottomrule => html.push_str("    <tr class=\"bottomrule\"><td colspan=\"100%\"></td></tr>\n"),
                }
            }
            
            // Skip empty rows
            if row.cells.is_empty() {
                continue;
            }
            
            html.push_str("    <tr>\n");
            let mut col_index = 0;
            
            for cell in &row.cells {
                match cell {
                    TableCell::Normal(nodes) => {
                        let content = nodes.iter()
                            .filter(|node| !matches!(node, Node::LineBreak))
                            .map(|node| self.render_node(node))
                            .collect::<String>()
                            .trim()
                            .to_string();
                        
                        // Apply column alignment if available
                        let align_attr = if col_index < alignment.len() {
                            match &alignment[col_index] {
                                ColumnAlignment::Left => " style=\"text-align: left;\"",
                                ColumnAlignment::Center => " style=\"text-align: center;\"",
                                ColumnAlignment::Right => " style=\"text-align: right;\"",
                                ColumnAlignment::Paragraph(width) => &format!(" style=\"text-align: left; width: {width};\""),
                                ColumnAlignment::FixedWidth(width) => &format!(" style=\"width: {width};\""),
                                ColumnAlignment::ArrayColumn(spec) => &format!(" style=\"text-align: center;\" data-array-spec=\"{spec}\""),
                                ColumnAlignment::XColumn => " style=\"text-align: left; width: auto;\"",
                                ColumnAlignment::SColumn(spec) => &format!(" style=\"text-align: center;\" data-s-spec=\"{spec}\""),
                            }
                        } else {
                            ""
                        };
                        
                        html.push_str(&format!("        <td{align_attr}>{content}</td>\n"));
                        col_index += 1;
                    }
                    TableCell::MultiColumn(multicolumn) => {
                        let content = multicolumn.content.iter()
                            .filter(|node| !matches!(node, Node::LineBreak))
                            .map(|node| self.render_node(node))
                            .collect::<String>()
                            .trim()
                            .to_string();
                        
                        let align_attr = match multicolumn.alignment {
                            ColumnAlignment::Left => " style=\"text-align: left;\"",
                            ColumnAlignment::Center => " style=\"text-align: center;\"",
                            ColumnAlignment::Right => " style=\"text-align: right;\"",
                            ColumnAlignment::Paragraph(ref width) => &format!(" style=\"text-align: left; width: {width};\""),
                            ColumnAlignment::FixedWidth(ref width) => &format!(" style=\"width: {width};\""),
                            ColumnAlignment::ArrayColumn(ref spec) => &format!(" style=\"text-align: center;\" data-array-spec=\"{spec}\""),
                            ColumnAlignment::XColumn => " style=\"text-align: left; width: auto;\"",
                            ColumnAlignment::SColumn(ref spec) => &format!(" style=\"text-align: center;\" data-s-spec=\"{spec}\""),
                        };
                        
                        html.push_str(&format!("        <td colspan=\"{}\"{}>{})</td>\n", multicolumn.span, align_attr, content));
                        col_index += multicolumn.span;
                    }
                    TableCell::MultiRow(multirow) => {
                        let content = multirow.content.iter()
                            .filter(|node| !matches!(node, Node::LineBreak))
                            .map(|node| self.render_node(node))
                            .collect::<String>()
                            .trim()
                            .to_string();
                        
                        html.push_str(&format!("        <td rowspan=\"{}\">{})</td>\n", multirow.span, content));
                        col_index += 1;
                    }
                    TableCell::MultiColumnRow { multicolumn, multirow } => {
                        let cell_content = multicolumn.content.iter()
                            .filter(|node| !matches!(node, Node::LineBreak))
                            .map(|node| self.render_node(node))
                            .collect::<String>()
                            .trim()
                            .to_string();
                        
                        let align_attr = match &multicolumn.alignment {
                            ColumnAlignment::Left => " style=\"text-align: left;\"",
                            ColumnAlignment::Center => " style=\"text-align: center;\"",
                            ColumnAlignment::Right => " style=\"text-align: right;\"",
                            ColumnAlignment::Paragraph(ref width) => &format!(" style=\"text-align: left; width: {width};\""),
                            ColumnAlignment::FixedWidth(ref width) => &format!(" style=\"width: {width};\""),
                            ColumnAlignment::ArrayColumn(ref spec) => &format!(" style=\"text-align: center;\" data-array-spec=\"{spec}\""),
                            ColumnAlignment::XColumn => " style=\"text-align: left; width: auto;\"",
                            ColumnAlignment::SColumn(ref spec) => &format!(" style=\"text-align: center;\" data-s-spec=\"{spec}\""),
                        };
                        
                        html.push_str(&format!("        <td colspan=\"{}\" rowspan=\"{}\"{}>{})</td>\n", multicolumn.span, multirow.span, align_attr, cell_content));
                        col_index += multicolumn.span;
                    }
                }
            }
            html.push_str("    </tr>\n");
            
            // Render rules after the row
            for rule in &row.rules_after {
                match rule {
                    TableRule::Hline => html.push_str("    <tr class=\"hline\"><td colspan=\"100%\"></td></tr>\n"),
                    TableRule::Cline(range) => {
                         html.push_str(&format!("    <tr class=\"cline\"><td colspan=\"100%\" data-range=\"{range}\"></td></tr>\n"));
                     }
                    TableRule::Toprule => html.push_str("    <tr class=\"toprule\"><td colspan=\"100%\"></td></tr>\n"),
                    TableRule::Midrule => html.push_str("    <tr class=\"midrule\"><td colspan=\"100%\"></td></tr>\n"),
                    TableRule::Bottomrule => html.push_str("    <tr class=\"bottomrule\"><td colspan=\"100%\"></td></tr>\n"),
                }
            }
        }
        
        html.push_str("</table>\n");
        html
    }

    /// Render a section
    fn render_section(&self, level: &SectionLevel, title: &str, content: &[Node]) -> String {
        let heading_level = std::cmp::min(level.level() + 1, 6) as usize;
        let mut html = format!("<h{} class=\"section-{}\">{}</h{}>\n", 
                              heading_level, level.level(), self.escape_html(title), heading_level);
        
        for node in content {
            html.push_str(&self.render_node(node));
        }
        
        html
    }

    /// Render a figure
    fn render_figure(&self, path: &str, caption: Option<&str>, label: Option<&str>, width: Option<&str>, height: Option<&str>) -> String {
        let mut html = String::from("<figure class=\"latex-figure\">\n");
        
        // Build image tag with attributes
        let mut img_attrs = Vec::new();
        img_attrs.push(format!("src=\"{}\"", self.escape_html(path)));
        img_attrs.push("alt=\"Figure\"".to_string());
        
        if let Some(w) = width {
            let processed_width = self.process_latex_dimension(w);
            img_attrs.push(format!("width=\"{}\"", self.escape_html(&processed_width)));
        }
        if let Some(h) = height {
            let processed_height = self.process_latex_dimension(h);
            img_attrs.push(format!("height=\"{}\"", self.escape_html(&processed_height)));
        }
        
        html.push_str(&format!("    <img {}/>\n", img_attrs.join(" ")));
        
        // Add caption if present
        if let Some(cap) = caption {
            html.push_str(&format!("    <figcaption>{}</figcaption>\n", self.escape_html(cap)));
        }
        
        // Add label as data attribute if present
        if let Some(lbl) = label {
            html = html.replace("<figure class=\"latex-figure\">", &format!("<figure class=\"latex-figure\" data-label=\"{}\">" , self.escape_html(lbl)));
        }
        
        html.push_str("</figure>\n");
        html
    }

    /// Process LaTeX dimensions and convert them to CSS-compatible values
    fn process_latex_dimension(&self, dimension: &str) -> String {
        let dimension = dimension.trim();
        
        // Handle \textwidth units
        if dimension.contains("\\textwidth") {
            let coefficient_str = dimension.replace("\\textwidth", "");
            let coefficient = coefficient_str.trim();
            if let Ok(value) = coefficient.parse::<f64>() {
                // Convert to percentage (assuming textwidth = 100%)
                return format!("{}%", (value * 100.0) as i32);
            }
        }
        
        // Handle other LaTeX units
        if dimension.ends_with("cm") || dimension.ends_with("mm") || 
           dimension.ends_with("in") || dimension.ends_with("pt") || 
           dimension.ends_with("pc") || dimension.ends_with("em") || 
           dimension.ends_with("ex") {
            return dimension.to_string();
        }
        
        // Handle percentage values
        if dimension.ends_with("%") {
            return dimension.to_string();
        }
        
        // Handle pixel values (add px if it's just a number)
        if dimension.parse::<f64>().is_ok() {
            return format!("{dimension}px");
        }
        
        // Return as-is if we can't process it
        dimension.to_string()
    }

    fn render_math_environment(&self, env_type: &MathEnvironmentType, content: &[Node], label: Option<&str>, numbered: bool, equation_number: Option<usize>) -> String {
        let env_name = match env_type {
            MathEnvironmentType::Equation => "equation",
            MathEnvironmentType::Align => "align",
            MathEnvironmentType::Gather => "gather",
            MathEnvironmentType::Multline => "multline",
            MathEnvironmentType::Array => "array",
            MathEnvironmentType::Matrix => "matrix",
            MathEnvironmentType::Pmatrix => "pmatrix",
            MathEnvironmentType::Bmatrix => "bmatrix",
            MathEnvironmentType::Vmatrix => "vmatrix",
            MathEnvironmentType::Smallmatrix => "smallmatrix",
            MathEnvironmentType::Cases => "cases",
            MathEnvironmentType::Split => "split",
            MathEnvironmentType::Aligned => "aligned",
            MathEnvironmentType::Eqnarray => "eqnarray",
        };

        let content_str = content.iter().map(|node| self.render_node(node)).collect::<String>();
        
        let mut html = String::new();
        
        // Build container div with appropriate classes and attributes
        let mut div_attrs = vec![format!("class=\"math-environment math-{}\"", env_name)];
        
        if numbered {
            div_attrs.push("data-numbered=\"true\"".to_string());
        }
        
        if let Some(lbl) = label {
            div_attrs.push(format!("id=\"{}\"", self.escape_html(lbl)));
        }
        
        if let Some(eq_num) = equation_number {
            div_attrs.push(format!("data-equation-number=\"{eq_num}\""));
        }
        
        html.push_str(&format!("<div {}>\n", div_attrs.join(" ")));
        
        // Add equation number display if numbered
        if numbered {
            if let Some(eq_num) = equation_number {
                html.push_str(&format!("<span class=\"equation-number\">({eq_num})</span>\n"));
            }
        }
        
        // Add the math content in a display math div
        html.push_str(&format!("<div class=\"math-display\">$$\\begin{{{env_name}}}\\n{content_str}\\end{{{env_name}}}$$</div>\n"));
        html.push_str("</div>\n");
        
        html
    }

    fn render_reference(&self, ref_type: &ReferenceType, label: &str) -> String {
        match ref_type {
            ReferenceType::Ref => format!("<a href=\"#{}\" class=\"ref\">[ref:{}]</a>", self.escape_html(label), self.escape_html(label)),
            ReferenceType::Pageref => format!("<a href=\"#{}\" class=\"pageref\">[page:{}]</a>", self.escape_html(label), self.escape_html(label)),
            ReferenceType::Eqref => format!("<a href=\"#{}\" class=\"eqref\">({}))</a>", self.escape_html(label), self.escape_html(label)),
        }
    }

    fn render_footnote(&self, content: &[Node]) -> String {
        let content_str = content.iter().map(|node| self.render_node(node)).collect::<String>();
        format!("<sup class=\"footnote\">{content_str}</sup>")
    }

    fn render_spacing(&self, space_type: &SpacingType, amount: Option<&str>) -> String {
        match space_type {
            SpacingType::Vspace => {
                if let Some(amt) = amount {
                    format!("<div style=\"height: {};\"></div>", self.escape_html(amt))
                } else {
                    "<div style=\"height: 1em;\"></div>".to_string()
                }
            }
            SpacingType::Hspace => {
                if let Some(amt) = amount {
                    format!("<span style=\"margin-left: {};\"></span>", self.escape_html(amt))
                } else {
                    "<span style=\"margin-left: 1em;\"></span>".to_string()
                }
            }
            SpacingType::Vfill => "<div style=\"flex-grow: 1;\"></div>".to_string(),
            SpacingType::Hfill => "<span style=\"flex-grow: 1;\"></span>".to_string(),
            SpacingType::Newpage => "<div style=\"page-break-before: always;\"></div>".to_string(),
            SpacingType::Clearpage => "<div style=\"page-break-before: always; clear: both;\"></div>".to_string(),
            SpacingType::Pagebreak => "<div style=\"page-break-inside: avoid;\"></div>".to_string(),
            SpacingType::Linebreak => "<br>\n".to_string(),
            SpacingType::Par => "<p></p>".to_string(),
        }
    }
    
    /// Render command definition
    fn render_command_definition(&self, name: &str, num_args: Option<&usize>, default_args: &[String], definition: &[Node]) -> String {
        // For HTML output, we'll render command definitions as HTML comments
        // In a full implementation, these would be stored and used when the command is invoked
        let mut html = String::new();
        html.push_str(&format!("<!-- Command definition: \\{name}"));
        
        if let Some(args) = num_args {
            html.push_str(&format!(" with {args} arguments"));
        }
        
        if !default_args.is_empty() {
            html.push_str(&format!(" (defaults: {})", default_args.join(", ")));
        }
        
        html.push_str(" -->\n");
        
        // For demonstration, also render the definition content in a comment
        html.push_str("<!-- Definition: ");
        for node in definition {
            html.push_str(&self.render_node(node).replace("-->", "--&gt;"));
        }
        html.push_str(" -->\n");
        
        html
    }
    
    /// Render citation
    fn render_citation(&self, cite_type: &CitationType, keys: &[String], prenote: Option<&str>, postnote: Option<&str>) -> String {
        let mut html = String::new();
        
        if let CitationType::Citep = cite_type { html.push('(') }
        
        if let Some(prenote) = prenote {
            html.push_str(&self.escape_html(prenote));
            html.push(' ');
        }
        
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                html.push_str(", ");
            }
            html.push_str(&format!("<a href=\"#ref-{}\" class=\"citation\">[{}]</a>", 
                self.escape_html(key), self.escape_html(key)));
        }
        
        if let Some(postnote) = postnote {
            html.push(' ');
            html.push_str(&self.escape_html(postnote));
        }
        
        if let CitationType::Citep = cite_type { html.push(')') }
        
        html
    }
    
    /// Render bibliography entry
    fn render_bibliography_entry(&self, key: &str, entry_type: &str, fields: &HashMap<String, String>) -> String {
        let mut html = String::new();
        html.push_str(&format!("<div id=\"ref-{}\" class=\"bibliography-entry\">", self.escape_html(key)));
        html.push_str(&format!("<span class=\"entry-type\">[{}]</span> ", self.escape_html(entry_type)));
        
        // Render common fields
        if let Some(author) = fields.get("author") {
            html.push_str(&format!("<span class=\"author\">{}</span>. ", self.escape_html(author)));
        }
        
        if let Some(title) = fields.get("title") {
            html.push_str(&format!("<span class=\"title\">{}</span>. ", self.escape_html(title)));
        }
        
        if let Some(journal) = fields.get("journal") {
            html.push_str(&format!("<span class=\"journal\">{}</span>. ", self.escape_html(journal)));
        }
        
        if let Some(year) = fields.get("year") {
            html.push_str(&format!("<span class=\"year\">{}</span>. ", self.escape_html(year)));
        }
        
        html.push_str("</div>");
        html
    }
    
    /// Render bibliography
    fn render_bibliography(&self, style: Option<&str>, entries: &[Node]) -> String {
        let mut html = String::new();
        html.push_str("<div class=\"bibliography\">");
        html.push_str("<h2>References</h2>");
        
        if let Some(style) = style {
            html.push_str(&format!("<!-- Bibliography Style: {} -->", self.escape_html(style)));
        }
        
        for entry in entries {
            html.push_str(&self.render_node(entry));
        }
        
        html.push_str("</div>");
        html
    }
    
    /// Render package declaration
    fn render_package(&self, name: &str, options: &[String]) -> String {
        // Render package as HTML comment for now
        if options.is_empty() {
            format!("<!-- Package: {name} -->\n")
        } else {
            format!("<!-- Package: {} [{}] -->\n", name, options.join(", "))
        }
    }
    
    pub fn render_code_block(&self, language: Option<&str>, code: &str, style: &CodeStyle, line_numbers: bool, caption: Option<&str>, label: Option<&str>) -> String {
        let mut html = String::new();
        
        // Add label if present
        if let Some(label_text) = label {
            html.push_str(&format!("<div id=\"{}\" class=\"code-block-container\">\n", self.escape_html(label_text)));
        } else {
            html.push_str("<div class=\"code-block-container\">\n");
        }
        
        // Add caption if present
        if let Some(caption_text) = caption {
            html.push_str(&format!("<div class=\"code-caption\">{}</div>\n", self.escape_html(caption_text)));
        }
        
        // Determine CSS classes based on style and language
        let mut classes = vec!["code-block"];
        
        match style {
            CodeStyle::Verbatim => classes.push("verbatim"),
            CodeStyle::Listings => classes.push("listings"),
            CodeStyle::Minted => classes.push("minted"),
            CodeStyle::Inline => classes.push("inline-code"),
        }
        
        if line_numbers {
            classes.push("line-numbers");
        }
        
        let lang_class = language.map(|lang| format!("language-{lang}"));
        if let Some(ref lang_cls) = lang_class {
            classes.push(lang_cls);
        }
        
        let class_str = classes.join(" ");
        
        // Render the code block
        if matches!(style, CodeStyle::Inline) {
            html.push_str(&format!("<code class=\"{}\">{}</code>", class_str, self.escape_html(code)));
        } else {
            html.push_str(&format!("<pre class=\"{}\"><code>{}</code></pre>\n", class_str, self.escape_html(code)));
        }
        
        html.push_str("</div>\n");
        html
    }

    /// Register default command handlers
    fn register_default_commands(&mut self) {
        // Add custom command handlers here if needed
    }

    /// Escape HTML special characters
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// Render mathematical fraction
    fn render_frac(&self, args: &[Argument]) -> String {
        if args.len() >= 2 {
            if let (Some(Argument::Required(num)), Some(Argument::Required(den))) = (args.first(), args.get(1)) {
                let numerator = num.iter().map(|n| self.render_node(n)).collect::<String>();
                let denominator = den.iter().map(|n| self.render_node(n)).collect::<String>();
                return format!("<span class=\"math-frac\">\\frac{{{numerator}}}{{{denominator}}}</span>");
            }
        }
        String::new()
    }

    /// Render square root
    fn render_sqrt(&self, args: &[Argument]) -> String {
        if let Some(Argument::Required(nodes)) = args.first() {
            let content = nodes.iter().map(|n| self.render_node(n)).collect::<String>();
            format!("<span class=\"math-sqrt\">\\sqrt{{{content}}}</span>")
        } else {
            String::new()
        }
    }

    /// Render mathematical operators
    fn render_math_operator(&self, name: &str, args: &[Argument]) -> String {
        if args.is_empty() {
            format!("<span class=\"math-operator\">\\{name}</span>")
        } else {
            let content = args.iter()
                .filter_map(|arg| match arg {
                    Argument::Required(nodes) => Some(nodes.iter().map(|n| self.render_node(n)).collect::<String>()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            format!("<span class=\"math-operator\">\\{name}{{{content}}}</span>")
        }
    }

    /// Render mathematical style commands
    fn render_math_style(&self, name: &str, args: &[Argument]) -> String {
        if let Some(Argument::Required(nodes)) = args.first() {
            let content = nodes.iter().map(|n| self.render_node(n)).collect::<String>();
            format!("<span class=\"math-style-{name}\">\\{name}{{{content}}}</span>")
        } else {
            format!("<span class=\"math-style-{name}\">\\{name}</span>")
        }
    }

    /// Render list item
    fn render_item(&self, args: &[Argument]) -> String {
        if let Some(Argument::Optional(label_nodes)) = args.first() {
            let label = label_nodes.iter().map(|n| self.render_node(n)).collect::<String>();
            format!("<li data-label=\"{}\">", self.escape_html(&label))
        } else {
            "<li>".to_string()
        }
    }

    /// Get default CSS styles
    fn get_default_css(&self) -> &'static str {
        r#"
        body {
            font-family: 'Times New Roman', serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            color: #333;
            background-color: #ffffff;
        }
        
        .document-header {
            text-align: center;
            margin-bottom: 2em;
            border-bottom: 1px solid #ccc;
            padding-bottom: 1em;
        }
        
        .document-title {
            font-size: 2em;
            margin-bottom: 0.5em;
        }
        
        .document-author {
            font-size: 1.2em;
            font-style: italic;
            margin-bottom: 0.3em;
        }
        
        .document-date {
            font-size: 1em;
            color: #666;
        }
        
        h1, h2, h3, h4, h5, h6 {
            color: #2c3e50;
            margin-top: 1.5em;
            margin-bottom: 0.5em;
        }
        
        .center {
            text-align: center;
        }
        
        .abstract {
            background-color: #f8f9fa;
            padding: 1em;
            border-left: 4px solid #007bff;
            margin: 1em 0;
        }
        
        blockquote {
            border-left: 4px solid #ccc;
            margin: 1em 0;
            padding-left: 1em;
            font-style: italic;
        }
        
        pre, code {
            font-family: 'Courier New', monospace;
            background-color: #f4f4f4;
            padding: 0.2em 0.4em;
            border-radius: 3px;
        }
        
        pre {
            padding: 1em;
            overflow-x: auto;
        }
        
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 1em 0;
        }
        
        th, td {
            border: 1px solid #ddd;
            padding: 0.5em;
            text-align: left;
        }
        
        th {
            background-color: #f2f2f2;
            font-weight: bold;
        }
        
        /* Font sizes */
        .font-tiny { font-size: 0.5em; }
        .font-scriptsize { font-size: 0.7em; }
        .font-footnotesize { font-size: 0.8em; }
        .font-small { font-size: 0.9em; }
        .font-normalsize { font-size: 1em; }
        .font-large { font-size: 1.2em; }
        .font-Large { font-size: 1.44em; }
        .font-LARGE { font-size: 1.728em; }
        .font-huge { font-size: 2.074em; }
        .font-Huge { font-size: 2.488em; }
        
        /* Font families */
        .font-roman { font-family: 'Times New Roman', serif; }
        .font-sans { font-family: 'Arial', sans-serif; }
        .font-smallcaps { font-variant: small-caps; }
        
        /* Text alignment */
        .text-center { text-align: center; }
        .text-left { text-align: left; }
        .text-right { text-align: right; }
        
        .page-break {
            page-break-before: always;
        }
        
        .vspace {
            margin: 1em 0;
        }
        
        .hspace {
            margin: 0 1em;
        }
        
        /* Math styles */
        .math-frac, .math-sqrt, .math-operator, .math-style-displaystyle, .math-style-textstyle {
            font-family: 'Times New Roman', serif;
            font-style: italic;
        }
        
        .math-display {
            text-align: center;
            margin: 1em 0;
        }
        
        .math-inline {
            display: inline;
        }
        
        /* Table rules */
        .hline td {
            border-top: 1px solid #000;
            border-bottom: none;
            border-left: none;
            border-right: none;
            padding: 0;
            height: 1px;
        }
        
        .cline td {
            border-top: 1px solid #000;
            border-bottom: none;
            border-left: none;
            border-right: none;
            padding: 0;
            height: 1px;
        }
        
        .toprule td {
            border-top: 2px solid #000;
            border-bottom: none;
            border-left: none;
            border-right: none;
            padding: 0;
            height: 2px;
        }
        
        .midrule td {
            border-top: 1px solid #000;
            border-bottom: none;
            border-left: none;
            border-right: none;
            padding: 0;
            height: 1px;
        }
        
        .bottomrule td {
            border-top: 2px solid #000;
            border-bottom: none;
            border-left: none;
            border-right: none;
            padding: 0;
            height: 2px;
        }
        "#
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Math renderer for mathematical expressions
pub struct MathRenderer;

impl MathRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render mathematical expression
    pub fn render(&self, content: &str, display: bool) -> String {
        if display {
            format!("<div class=\"math-display\">$${content}$$</div>\n")
        } else {
            format!("<span class=\"math-inline\">${content}$</span>")
        }
    }
}

impl Default for MathRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// PDF renderer for LaTeX documents
pub struct PdfRenderer {
    current_font_size: f32,
    current_font_family: String,
    current_alignment: TextAlignment,
}

#[derive(Clone, Debug)]
enum TextAlignment {
    Left,
    Center,
    Right,
}

impl PdfRenderer {
    /// Create a new PDF renderer
    pub fn new() -> Self {
        Self {
            current_font_size: 12.0,
            current_font_family: "Times-Roman".to_string(),
            current_alignment: TextAlignment::Left,
        }
    }

    /// Render to PDF file
    pub fn render_to_file(&mut self, document: &Document, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (doc, page1, layer1) = PdfDocument::new("LaTeX Document", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);
        
        // Load fonts
        let font_regular = doc.add_builtin_font(BuiltinFont::TimesRoman)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::TimesBold)?;
        let font_italic = doc.add_builtin_font(BuiltinFont::TimesItalic)?;
        let font_sans = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        
        let mut y_position = Mm(270.0); // Start near top of page
        let left_margin = Mm(20.0);
        let right_margin = Mm(190.0);
        let page_width = Mm(210.0);
        
        // Render document metadata
        if let Some(title) = &document.metadata.title {
            current_layer.use_text(title, 18.0, left_margin, y_position, &font_bold);
            y_position -= Mm(10.0);
        }
        
        if let Some(author) = &document.metadata.author {
            current_layer.use_text(author, 12.0, left_margin, y_position, &font_italic);
            y_position -= Mm(8.0);
        }
        
        y_position -= Mm(10.0); // Extra space after header
        
        // Render document body
        for node in &document.body {
            y_position = self.render_node_pdf(&current_layer, node, &font_regular, &font_bold, &font_italic, &font_sans, y_position, left_margin, right_margin, page_width)?;
        }
        
        // Save the PDF
        doc.save(&mut BufWriter::new(File::create(output_path)?))?;
        Ok(())
    }
    
    /// Render a single node to PDF
    fn render_node_pdf(
        &self,
        layer: &PdfLayerReference,
        node: &Node,
        font_regular: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        font_italic: &IndirectFontRef,
        font_sans: &IndirectFontRef,
        mut y_position: Mm,
        left_margin: Mm,
        right_margin: Mm,
        page_width: Mm,
    ) -> Result<Mm, Box<dyn std::error::Error>> {
        match node {
            Node::Text(text) => {
                let x_pos = match self.current_alignment {
                     TextAlignment::Left => left_margin,
                     TextAlignment::Center => page_width / 2.0 - Mm(text.len() as f32 * 2.0), // Rough centering
                     TextAlignment::Right => right_margin - Mm(text.len() as f32 * 4.0), // Rough right align
                 };
                layer.use_text(text, self.current_font_size, x_pos, y_position, font_regular);
                Ok(y_position)
            }
            
            Node::Command { name, args, .. } => {
                match name.as_str() {
                    "textbf" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, self.current_font_size, left_margin, y_position, font_bold);
                        }
                        Ok(y_position)
                    }
                    "textit" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, self.current_font_size, left_margin, y_position, font_italic);
                        }
                        Ok(y_position)
                    }
                    "textsf" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, self.current_font_size, left_margin, y_position, font_sans);
                        }
                        Ok(y_position)
                    }
                    // Font sizes
                    "tiny" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 6.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "scriptsize" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 8.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "footnotesize" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 9.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "small" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 10.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "normalsize" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 12.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "large" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 14.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "Large" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 17.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "LARGE" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 20.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "huge" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 24.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    "Huge" => {
                        if let Some(Argument::Required(nodes)) = args.first() {
                            let text = self.extract_text_from_nodes(nodes);
                            layer.use_text(&text, 30.0, left_margin, y_position, font_regular);
                        }
                        Ok(y_position)
                    }
                    _ => Ok(y_position)
                }
            }
            
            Node::Environment { name, content, .. } => {
                match name.as_str() {
                    "center" => {
                        for content_node in content {
                            let mut renderer = self.clone();
                            renderer.current_alignment = TextAlignment::Center;
                            y_position = renderer.render_node_pdf(layer, content_node, font_regular, font_bold, font_italic, font_sans, y_position, left_margin, right_margin, page_width)?;
                            y_position -= Mm(4.0);
                        }
                        Ok(y_position)
                    }
                    "flushleft" => {
                        for content_node in content {
                            let mut renderer = self.clone();
                            renderer.current_alignment = TextAlignment::Left;
                            y_position = renderer.render_node_pdf(layer, content_node, font_regular, font_bold, font_italic, font_sans, y_position, left_margin, right_margin, page_width)?;
                            y_position -= Mm(4.0);
                        }
                        Ok(y_position)
                    }
                    "flushright" => {
                        for content_node in content {
                            let mut renderer = self.clone();
                            renderer.current_alignment = TextAlignment::Right;
                            y_position = renderer.render_node_pdf(layer, content_node, font_regular, font_bold, font_italic, font_sans, y_position, left_margin, right_margin, page_width)?;
                            y_position -= Mm(4.0);
                        }
                        Ok(y_position)
                    }
                    "quote" => {
                        y_position -= Mm(6.0); // Space before quote
                        let quote_left_margin = left_margin + Mm(10.0);
                        let quote_right_margin = right_margin - Mm(10.0);
                        
                        for content_node in content {
                            y_position = self.render_node_pdf(layer, content_node, font_regular, font_bold, font_italic, font_sans, y_position, quote_left_margin, quote_right_margin, page_width)?;
                            y_position -= Mm(3.0);
                        }
                        y_position -= Mm(6.0); // Space after quote
                        Ok(y_position)
                    }
                    _ => {
                        for content_node in content {
                            y_position = self.render_node_pdf(layer, content_node, font_regular, font_bold, font_italic, font_sans, y_position, left_margin, right_margin, page_width)?;
                            y_position -= Mm(4.0);
                        }
                        Ok(y_position)
                    }
                }
            }
            
            Node::Section { level, title, .. } => {
                 let font_size = match level {
                     SectionLevel::Part => 20.0,
                     SectionLevel::Chapter => 18.0,
                     SectionLevel::Section => 16.0,
                     SectionLevel::Subsection => 14.0,
                     SectionLevel::Subsubsection => 12.0,
                     SectionLevel::Paragraph => 11.0,
                     SectionLevel::Subparagraph => 10.0,
                 };
                y_position -= Mm(6.0); // Extra space before section
                layer.use_text(title, font_size, left_margin, y_position, font_bold);
                y_position -= Mm(6.0); // Space after section
                Ok(y_position)
            }
            
            Node::Paragraph => {
                y_position -= Mm(4.0);
                Ok(y_position)
            }
            
            Node::LineBreak => {
                y_position -= Mm(4.0);
                Ok(y_position)
            }
            
            Node::Table { table_type: _, alignment, rows, caption: _, label: _, position: _ } => {
                self.render_table_pdf(layer, alignment, rows, font_regular, font_bold, y_position, left_margin, right_margin, page_width)
            }
            
            Node::List { list_type, items } => {
                self.render_list_pdf(layer, list_type, items, font_regular, font_bold, font_italic, font_sans, y_position, left_margin, right_margin, page_width)
            }
            
            Node::Figure { path, caption, label, .. } => {
                // For PDF, we'll render a placeholder text since we can't embed images easily
                y_position -= Mm(6.0); // Space before figure
                
                let figure_text = format!("[Figure: {path}]");
                layer.use_text(&figure_text, self.current_font_size, left_margin, y_position, font_regular);
                y_position -= Mm(4.0);
                
                if let Some(cap) = caption {
                    let caption_text = format!("Caption: {cap}");
                    layer.use_text(&caption_text, self.current_font_size * 0.9, left_margin, y_position, font_regular);
                    y_position -= Mm(4.0);
                }
                
                if let Some(lbl) = label {
                    let label_text = format!("Label: {lbl}");
                    layer.use_text(&label_text, self.current_font_size * 0.8, left_margin, y_position, font_regular);
                    y_position -= Mm(4.0);
                }
                
                y_position -= Mm(6.0); // Space after figure
                Ok(y_position)
            }
            
            Node::Whitespace(_) => Ok(y_position),
            
            Node::MathEnvironment { env_type, content, label, .. } => {
                // For PDF, render math environment as text placeholder
                let env_name = match env_type {
                    MathEnvironmentType::Equation => "equation",
                    MathEnvironmentType::Align => "align",
                    MathEnvironmentType::Gather => "gather",
                    MathEnvironmentType::Multline => "multline",
                    MathEnvironmentType::Array => "array",
                    MathEnvironmentType::Matrix => "matrix",
                    MathEnvironmentType::Pmatrix => "pmatrix",
                    MathEnvironmentType::Bmatrix => "bmatrix",
                    MathEnvironmentType::Vmatrix => "vmatrix",
                    MathEnvironmentType::Smallmatrix => "smallmatrix",
                    MathEnvironmentType::Cases => "cases",
                    MathEnvironmentType::Split => "split",
                    MathEnvironmentType::Aligned => "aligned",
                    MathEnvironmentType::Eqnarray => "eqnarray",
                };
                
                y_position -= Mm(6.0); // Space before math
                
                // Enhanced math rendering
                let math_content = self.extract_text_from_nodes(content);
                match env_name {
                    "equation" | "align" | "gather" => {
                        // Center the equation
                        let text_width = math_content.len() as f32 * self.current_font_size * 0.6;
                        let center_x = left_margin + Mm((page_width.0 - left_margin.0 - right_margin.0 - text_width) / 2.0);
                        
                        layer.use_text(&math_content, self.current_font_size * 1.1, center_x, y_position, font_italic);
                        y_position -= Mm(self.current_font_size * 1.5);
                    }
                    _ => {
                        // Other math environments with slight indentation
                        layer.use_text(&math_content, self.current_font_size, left_margin + Mm(5.0), y_position, font_italic);
                        y_position -= Mm(self.current_font_size * 1.2);
                    }
                }
                
                if let Some(lbl) = label {
                    let label_text = format!("Label: {lbl}");
                    layer.use_text(&label_text, self.current_font_size * 0.8, left_margin, y_position, font_regular);
                    y_position -= Mm(4.0);
                }
                
                y_position -= Mm(6.0); // Space after math
                Ok(y_position)
            }
            
            Node::Label(label) => {
                let label_text = format!("[Label: {label}]");
                layer.use_text(&label_text, self.current_font_size * 0.8, left_margin, y_position, font_regular);
                Ok(y_position)
            }
            
            Node::Reference { ref_type, label } => {
                let ref_text = match ref_type {
                    ReferenceType::Ref => format!("[ref:{label}]"),
                    ReferenceType::Pageref => format!("[page:{label}]"),
                    ReferenceType::Eqref => format!("({label})"),
                };
                layer.use_text(&ref_text, self.current_font_size, left_margin, y_position, font_regular);
                Ok(y_position)
            }
            
            Node::Footnote { content } => {
                let footnote_text = format!("[Footnote: {}]", self.extract_text_from_nodes(content));
                layer.use_text(&footnote_text, self.current_font_size * 0.8, left_margin, y_position, font_regular);
                Ok(y_position)
            }
            
            Node::Spacing { space_type, .. } => {
                match space_type {
                    SpacingType::Vspace | SpacingType::Vfill => {
                        y_position -= Mm(8.0); // Vertical space
                    }
                    SpacingType::Newpage | SpacingType::Clearpage | SpacingType::Pagebreak => {
                        y_position -= Mm(20.0); // Simulate page break with large space
                    }
                    SpacingType::Linebreak => {
                        y_position -= Mm(4.0); // Line break
                    }
                    SpacingType::Par => {
                        y_position -= Mm(6.0); // Paragraph break
                    }
                    _ => {} // Horizontal spacing doesn't affect y_position
                }
                Ok(y_position)
            }
            
            Node::CommandDefinition { .. } => {
                // Command definitions don't produce visible output in PDF
                Ok(y_position)
            }
            
            Node::CodeBlock { language, code, style, line_numbers, caption, label } => {
                self.render_code_block_pdf(layer, language.as_deref(), code, style, *line_numbers, caption.as_deref(), label.as_deref(), font_regular, font_bold, font_italic, font_sans, y_position, left_margin, right_margin, page_width)
            }
            
            _ => Ok(y_position), // Handle other node types as needed
        }
    }
    
    /// Render a list to PDF
    fn render_list_pdf(
        &self,
        layer: &PdfLayerReference,
        list_type: &ListType,
        items: &[ListItem],
        font_regular: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        font_italic: &IndirectFontRef,
        font_sans: &IndirectFontRef,
        mut y_position: Mm,
        left_margin: Mm,
        right_margin: Mm,
        page_width: Mm,
    ) -> Result<Mm, Box<dyn std::error::Error>> {
        y_position -= Mm(4.0); // Space before list
        
        let list_left_margin = left_margin + Mm(8.0); // Indent list items
        let bullet_margin = left_margin + Mm(4.0); // Position for bullets/numbers
        
        for (index, item) in items.iter().enumerate() {
            // Render bullet or number
            let marker = match list_type {
                ListType::Itemize => "•".to_string(),
                ListType::Enumerate => format!("{}.", index + 1),
                ListType::Description => {
                    if let Some(label) = &item.label {
                        format!("{label}:")
                    } else {
                        "•".to_string()
                    }
                }
            };
            
            layer.use_text(&marker, self.current_font_size, bullet_margin, y_position, font_regular);
            
            // Render item content
            for content_node in &item.content {
                y_position = self.render_node_pdf(layer, content_node, font_regular, font_bold, font_italic, font_sans, y_position, list_left_margin, right_margin, page_width)?;
            }
            
            y_position -= Mm(3.0); // Space between items
        }
        
        y_position -= Mm(4.0); // Space after list
        Ok(y_position)
    }
    
    /// Render a table to PDF
    fn render_table_pdf(
        &self,
        layer: &PdfLayerReference,
        alignment: &[ColumnAlignment],
        rows: &[TableRow],
        font_regular: &IndirectFontRef,
        _font_bold: &IndirectFontRef,
        mut y_position: Mm,
        left_margin: Mm,
        right_margin: Mm,
        _page_width: Mm,
    ) -> Result<Mm, Box<dyn std::error::Error>> {
        let table_width = right_margin - left_margin;
        let col_width = table_width / alignment.len() as f32;
        
        y_position -= Mm(2.0); // Space before table
        
        for row in rows.iter() {
            let row_height = Mm(6.0);
            let cell_y = y_position - Mm(1.0); // Slight padding from top
            
            let mut col_idx = 0;
            // Render each cell in the row
            for cell in &row.cells {
                match cell {
                    TableCell::Normal(nodes) => {
                        if col_idx < alignment.len() {
                            let cell_x = left_margin + col_width * (col_idx as f32);
                            let cell_text = self.extract_text_from_nodes(nodes);
                            
                            // Apply alignment
                            let text_x = match alignment.get(col_idx).unwrap_or(&ColumnAlignment::Left) {
                                ColumnAlignment::Left => cell_x + Mm(1.0),
                                ColumnAlignment::Center => cell_x + col_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                                ColumnAlignment::Right => cell_x + col_width - Mm(cell_text.len() as f32 * 3.0) - Mm(1.0),
                                ColumnAlignment::Paragraph(_) => cell_x + Mm(1.0), // Left align for paragraph
                                ColumnAlignment::FixedWidth(_) => cell_x + Mm(1.0), // Left align for fixed width
                                ColumnAlignment::ArrayColumn(_) => cell_x + col_width / 2.0 - Mm(cell_text.len() as f32 * 1.5), // Center align for array
                                ColumnAlignment::XColumn => cell_x + Mm(1.0), // Left align for X column
                                ColumnAlignment::SColumn(_) => cell_x + col_width / 2.0 - Mm(cell_text.len() as f32 * 1.5), // Center align for S column
                            };
                            
                            layer.use_text(&cell_text, 10.0, text_x, cell_y, font_regular);
                        }
                        col_idx += 1;
                    }
                    TableCell::MultiColumn(multicolumn) => {
                        if col_idx < alignment.len() {
                            let cell_x = left_margin + col_width * (col_idx as f32);
                            let cell_text = self.extract_text_from_nodes(&multicolumn.content);
                            let multicolumn_width = col_width * (multicolumn.span as f32);
                            
                            // Apply multicolumn alignment
                            let text_x = match &multicolumn.alignment {
                                ColumnAlignment::Left => cell_x + Mm(1.0),
                                ColumnAlignment::Center => cell_x + multicolumn_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                                ColumnAlignment::Right => cell_x + multicolumn_width - Mm(cell_text.len() as f32 * 3.0) - Mm(1.0),
                                ColumnAlignment::Paragraph(_) => cell_x + Mm(1.0),
                                ColumnAlignment::FixedWidth(_) => cell_x + Mm(1.0),
                                ColumnAlignment::ArrayColumn(_) => cell_x + multicolumn_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                                ColumnAlignment::XColumn => cell_x + Mm(1.0),
                                ColumnAlignment::SColumn(_) => cell_x + multicolumn_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                            };
                            
                            layer.use_text(&cell_text, 10.0, text_x, cell_y, font_regular);
                        }
                        col_idx += multicolumn.span;
                    }
                    TableCell::MultiRow(multirow) => {
                        if col_idx < alignment.len() {
                            let cell_x = left_margin + col_width * (col_idx as f32);
                            let cell_text = self.extract_text_from_nodes(&multirow.content);
                            
                            // Apply alignment (use default left alignment for multirow)
                            let text_x = cell_x + Mm(1.0);
                            
                            layer.use_text(&cell_text, 10.0, text_x, cell_y, font_regular);
                        }
                        col_idx += 1;
                    }
                    TableCell::MultiColumnRow { multicolumn, multirow: _ } => {
                        if col_idx < alignment.len() {
                            let cell_x = left_margin + col_width * (col_idx as f32);
                            let cell_text = self.extract_text_from_nodes(&multicolumn.content);
                            let multicolumn_width = col_width * (multicolumn.span as f32);
                            
                            // Apply multicolumn alignment
                            let text_x = match &multicolumn.alignment {
                                ColumnAlignment::Left => cell_x + Mm(1.0),
                                ColumnAlignment::Center => cell_x + multicolumn_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                                ColumnAlignment::Right => cell_x + multicolumn_width - Mm(cell_text.len() as f32 * 3.0) - Mm(1.0),
                                ColumnAlignment::Paragraph(_) => cell_x + Mm(1.0),
                                ColumnAlignment::FixedWidth(_) => cell_x + Mm(1.0),
                                ColumnAlignment::ArrayColumn(_) => cell_x + multicolumn_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                                ColumnAlignment::XColumn => cell_x + Mm(1.0),
                                ColumnAlignment::SColumn(_) => cell_x + multicolumn_width / 2.0 - Mm(cell_text.len() as f32 * 1.5),
                            };
                            
                            layer.use_text(&cell_text, 10.0, text_x, cell_y, font_regular);
                        }
                        col_idx += multicolumn.span;
                    }
                }
            }
            
            y_position -= row_height;
        }
        
        y_position -= Mm(4.0); // Extra space after table
        Ok(y_position)
    }
    
    fn render_code_block_pdf(
        &self,
        layer: &PdfLayerReference,
        language: Option<&str>,
        code: &str,
        style: &CodeStyle,
        line_numbers: bool,
        caption: Option<&str>,
        label: Option<&str>,
        font_regular: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        font_italic: &IndirectFontRef,
        font_sans: &IndirectFontRef,
        mut y_position: Mm,
        left_margin: Mm,
        right_margin: Mm,
        page_width: Mm,
    ) -> Result<Mm, Box<dyn std::error::Error>> {
        // Add caption if present
        if let Some(caption_text) = caption {
            layer.use_text(format!("Code: {caption_text}"), self.current_font_size, left_margin, y_position, font_bold);
            y_position -= Mm(self.current_font_size * 1.5);
        }
        
        // Add language info if present
        if let Some(lang) = language {
            let lang_text = format!("Language: {lang}");
            layer.use_text(&lang_text, self.current_font_size * 0.9, left_margin, y_position, font_italic);
            y_position -= Mm(self.current_font_size * 1.2);
        }
        
        // Render code content with monospace-like appearance
        let code_font = font_sans; // Use sans-serif as monospace substitute
        let code_size = self.current_font_size * 0.9;
        let code_margin = left_margin + Mm(10.0); // Indent code block
        
        // Split code into lines
        let lines: Vec<&str> = code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let mut line_text = String::new();
            
            // Add line numbers if requested
            if line_numbers {
                line_text.push_str(&format!("{:3}: ", line_num + 1));
            }
            
            line_text.push_str(line);
            
            // Render the line
            layer.use_text(&line_text, code_size, code_margin, y_position, code_font);
            y_position -= Mm(code_size * 1.2);
        }
        
        // Add label if present
        if let Some(label_text) = label {
            let label_display = format!("[Label: {label_text}]");
            layer.use_text(&label_display, self.current_font_size * 0.8, left_margin, y_position, font_regular);
            y_position -= Mm(self.current_font_size);
        }
        
        // Add spacing after code block
        y_position -= Mm(8.0);
        
        Ok(y_position)
    }
    
    /// Extract text content from a list of nodes
    fn extract_text_from_nodes(&self, nodes: &[Node]) -> String {
        nodes.iter()
            .filter_map(|node| match node {
                Node::Text(text) => Some(text.clone()),
                Node::Whitespace(ws) => Some(ws.clone()),
                _ => None,
            })
            .collect::<Vec<String>>()
            .join("")
    }
}

impl Clone for PdfRenderer {
    fn clone(&self) -> Self {
        Self {
            current_font_size: self.current_font_size,
            current_font_family: self.current_font_family.clone(),
            current_alignment: self.current_alignment.clone(),
        }
    }
}

impl Default for PdfRenderer {
    fn default() -> Self {
        Self::new()
    }
}