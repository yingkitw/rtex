//! Convert LaTeX math expressions to presentation MathML.
//!
//! Handles the most common LaTeX math constructs:
//! variables, numbers, operators, fractions, roots, scripts,
//! Greek/special symbols, text, matrices, accents, and fenced groups.

use crate::math::symbols;

/// Convert a LaTeX math string into a presentation MathML string.
///
/// The output is a sequence of MathML elements suitable for wrapping in
/// `<math>` (inline) or `<math display="block">` (display) tags.
pub fn latex_to_mathml(latex: &str) -> String {
    let mut parser = MathMLParser::new(latex);
    parser.parse_sequence()
}

struct MathMLParser<'a> {
    chars: Vec<char>,
    pos: usize,
    _input: &'a str,
}

impl<'a> MathMLParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _input: input,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Parse a sequence of elements until end-of-input or a closing brace.
    fn parse_sequence(&mut self) -> String {
        let mut out = String::new();
        self.skip_whitespace();
        while self.pos < self.chars.len() {
            let ch = self.peek().unwrap();
            if ch == '}' {
                break;
            }
            let elem = self.parse_element();
            out.push_str(&elem);
            self.skip_whitespace();
        }
        out
    }

    /// Parse a single element (base + optional scripts).
    fn parse_element(&mut self) -> String {
        let base = self.parse_base();
        if base.is_empty() {
            return base;
        }

        // Check for subscript and/or superscript
        let mut has_sub = false;
        let mut has_sup = false;
        let mut sub = String::new();
        let mut sup = String::new();

        // Peek ahead, handling order: _^ or ^_ or _ or ^
        let lookahead = self.pos;
        if lookahead < self.chars.len() {
            let ch = self.chars[lookahead];
            if ch == '_' || ch == '^' {
                let first_is_sub = ch == '_';
                self.pos += 1;
                let first_script = self.parse_script_arg();

                // Check for second script marker
                let has_second = self.pos < self.chars.len()
                    && (self.chars[self.pos] == '_' || self.chars[self.pos] == '^')
                    && self.chars[self.pos] != ch;

                if has_second {
                    let second_is_sub = self.chars[self.pos] == '_';
                    self.pos += 1;
                    let second_script = self.parse_script_arg();
                    if first_is_sub {
                        sub = first_script;
                        has_sub = true;
                    } else {
                        sup = first_script;
                        has_sup = true;
                    }
                    if second_is_sub {
                        sub = second_script;
                        has_sub = true;
                    } else {
                        sup = second_script;
                        has_sup = true;
                    }
                } else if first_is_sub {
                    sub = first_script;
                    has_sub = true;
                } else {
                    sup = first_script;
                    has_sup = true;
                }
            }
        }

        if has_sub && has_sup {
            format!("<msubsup>{base}{sub}{sup}</msubsup>")
        } else if has_sub {
            format!("<msub>{base}{sub}</msub>")
        } else if has_sup {
            format!("<msup>{base}{sup}</msup>")
        } else {
            base
        }
    }

    /// Parse the argument of `^` or `_` (single char, single command, or braced group).
    fn parse_script_arg(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() == Some('{') {
            self.next();
            let inner = self.parse_sequence();
            if self.peek() == Some('}') {
                self.next();
            }
            inner
        } else if self.peek() == Some('\\') {
            // Parse a single command as the script
            self.parse_element()
        } else if let Some(ch) = self.next() {
            self.char_to_mathml(ch)
        } else {
            String::new()
        }
    }

    /// Parse a base element (the thing before any scripts).
    fn parse_base(&mut self) -> String {
        self.skip_whitespace();
        let ch = match self.peek() {
            Some(c) => c,
            None => return String::new(),
        };

        match ch {
            '{' => {
                self.next();
                let inner = self.parse_sequence();
                if self.peek() == Some('}') {
                    self.next();
                }
                format!("<mrow>{inner}</mrow>")
            }
            '\\' => self.parse_command(),
            _ => {
                self.next();
                self.char_to_mathml(ch)
            }
        }
    }

    /// Parse a `\command` token.
    fn parse_command(&mut self) -> String {
        self.next(); // consume '\'

        // Special: \\ (line break in matrix)
        if self.peek() == Some('\\') {
            self.next();
            return String::new(); // handled by matrix parser
        }

        // Special: \, \; \: \! (spacing)
        if let Some(ch) = self.peek() {
            if matches!(ch, ',' | ';' | ':' | '!') {
                self.next();
                if ch == '!' {
                    return String::new();
                }
                return "<mspace width=\"0.167em\" />".to_string();
            }
        }

        // Read command name
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphabetic() {
                name.push(ch);
                self.next();
            } else {
                break;
            }
        }

        if name.is_empty() {
            // Non-alpha command like \{ or \%
            if let Some(ch) = self.next() {
                return self.char_to_mathml(ch);
            }
            return String::new();
        }

        // Strip trailing * (e.g. \sum* etc.)
        if self.peek() == Some('*') {
            self.next();
        }

        self.dispatch_command(&name)
    }

    /// Dispatch a named command to its MathML representation.
    fn dispatch_command(&mut self, name: &str) -> String {
        match name {
            // Fractions
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let num = self.parse_required_group();
                let den = self.parse_required_group();
                format!("<mfrac>{num}{den}</mfrac>")
            }

            // Binomial
            "binom" | "dbinom" | "tbinom" => {
                let num = self.parse_required_group();
                let den = self.parse_required_group();
                format!(
                    "<mrow><mo>(</mo><mfrac linethickness=\"0\">{num}{den}</mfrac><mo>)</mo></mrow>"
                )
            }

            // Roots
            "sqrt" => self.parse_sqrt(),

            // Text
            "text" | "textrm" | "textnormal" | "mathrm" | "mathit" | "mathbf"
            | "mathsf" | "mathtt" | "mathscr" | "mathnormal" => {
                let arg = self.parse_raw_group();
                let escaped = escape_xml_text(&arg);
                format!("<mtext>{escaped}</mtext>")
            }

            // Math alphabet wrappers — render inner content with appropriate mi variant
            "mathbb" | "mathcal" | "mathfrak" => {
                let arg = self.parse_required_group();
                let escaped = escape_xml_text(&arg);
                format!("<mi>{escaped}</mi>")
            }

            // Accents
            "vec" | "hat" | "tilde" | "bar" | "dot" | "ddot" | "check" | "breve"
            | "acute" | "grave" | "mathring" | "widehat" | "widetilde" | "overrightarrow"
            | "overleftarrow" | "overline" | "underline" => {
                let arg = self.parse_required_group();
                let accent_char = match name {
                    "vec" | "overrightarrow" => "→",
                    "hat" | "widehat" => "^",
                    "tilde" | "widetilde" => "~",
                    "bar" | "overline" => "¯",
                    "dot" => "˙",
                    "ddot" => "¨",
                    "check" => "ˇ",
                    "breve" => "˘",
                    "acute" => "´",
                    "grave" => "`",
                    "mathring" => "˚",
                    "underline" => "_",
                    "overleftarrow" => "←",
                    _ => "",
                };
                let inner = if arg.contains('<') {
                    arg
                } else {
                    format!("<mi>{}</mi>", escape_xml_text(&arg))
                };
                if name == "overline" {
                    format!("<mover accent=\"false\">{inner}<mo>¯</mo></mover>")
                } else if name == "underline" {
                    format!("<munder accent=\"false\">{inner}<mo>_</mo></munder>")
                } else {
                    format!("<mover accent=\"true\">{inner}<mo>{accent_char}</mo></mover>")
                }
            }

            // Over/under sets
            "overset" | "underset" | "stackrel" => {
                let top = self.parse_required_group();
                let base = self.parse_element();
                if name == "underset" {
                    format!("<munder>{base}{top}</munder>")
                } else {
                    format!("<mover>{base}{top}</mover>")
                }
            }

            // \left and \right — just emit the delimiter character
            "left" => {
                self.skip_whitespace();
                if let Some(d) = self.peek() {
                    self.next();
                    if d == '.' {
                        return String::new();
                    }
                    return format!("<mo>{}</mo>", escape_xml_text(&d.to_string()));
                }
                String::new()
            }
            "right" => {
                self.skip_whitespace();
                if let Some(d) = self.peek() {
                    self.next();
                    if d == '.' {
                        return String::new();
                    }
                    return format!("<mo>{}</mo>", escape_xml_text(&d.to_string()));
                }
                String::new()
            }

            // Big delimiters — skip the size command, emit the delimiter
            "big" | "Big" | "bigg" | "Bigg" | "bigl" | "Bigl" | "bigr" | "Bigr" => {
                self.skip_whitespace();
                if let Some(d) = self.peek() {
                    self.next();
                    return format!("<mo>{}</mo>", escape_xml_text(&d.to_string()));
                }
                String::new()
            }

            // Matrix environments
            "begin" => self.parse_environment(),
            "end" => {
                // Consume the environment name and return empty
                self.skip_whitespace();
                if self.peek() == Some('{') {
                    self.next();
                    while let Some(ch) = self.peek() {
                        if ch == '}' {
                            self.next();
                            break;
                        }
                        self.next();
                    }
                }
                String::new()
            }

            // Spacing
            "quad" => "<mspace width=\"1em\" />".to_string(),
            "qquad" => "<mspace width=\"2em\" />".to_string(),
            "thinspace" | "negthinspace" => String::new(),
            "medspace" | "negmedspace" => String::new(),
            "thickspace" | "negthickspace" => String::new(),
            "enspace" | "enskip" => "<mspace width=\"0.5em\" />".to_string(),
            "emsp" => "<mspace width=\"1em\" />".to_string(),
            "hspace" | "vspace" | "kern" | "mskip" | "mkern" => {
                // Consume optional argument
                self.skip_whitespace();
                if self.peek() == Some('{') {
                    let _ = self.parse_required_group();
                }
                String::new()
            }

            // Phantom
            "phantom" | "hphantom" | "vphantom" | "smash" => {
                let _ = self.parse_required_group();
                String::new()
            }

            // \text subscripts like \textsuperscript — rare in math, skip
            "textsuperscript" | "textsubscript" => {
                let _ = self.parse_required_group();
                String::new()
            }

            // Function names (rendered as <mi> with upright style)
            "sin" | "cos" | "tan" | "cot" | "sec" | "csc" | "sinh" | "cosh"
            | "tanh" | "coth" | "log" | "ln" | "lg" | "exp" | "lim" | "limsup"
            | "liminf" | "arg" | "gcd" | "hom" | "ker" | "Pr" | "max" | "min"
            | "sup" | "inf" | "projlim" | "injlim" | "varlimsup" | "varliminf"
            | "varinjlim" | "varprojlim" => {
                let display = match name {
                    "limsup" => "lim sup",
                    "liminf" => "lim inf",
                    "projlim" => "proj lim",
                    "injlim" => "inj lim",
                    "varlimsup" => "lim sup",
                    "varliminf" => "lim inf",
                    "varinjlim" => "inj lim",
                    "varprojlim" => "proj lim",
                    _ => name,
                };
                format!("<mi mathvariant=\"normal\">{display}</mi>")
            }

            // \operatorname{name}
            "operatorname" => {
                let arg = self.parse_required_group();
                format!("<mi mathvariant=\"normal\">{}</mi>", escape_xml_text(&arg))
            }

            // \not — negation, render as slash overlay (simplified: just emit /)
            "not" => "<mo>/</mo>".to_string(),

            // \, control space
            " " => "<mspace width=\"0.167em\" />".to_string(),

            // \colon
            "colon" => "<mo>:</mo>".to_string(),

            // \cdots, \ldots, \dots, \vdots, \ddots
            "cdots" | "dots" | "dotsc" | "dotsb" | "dotsm" | "dotsi" | "dotso" => {
                "<mo>⋯</mo>".to_string()
            }
            "ldots" => "<mo>…</mo>".to_string(),
            "vdots" => "<mo>⋮</mo>".to_string(),
            "ddots" => "<mo>⋱</mo>".to_string(),
            "adots" => "<mo>⋰</mo>".to_string(),

            // \prime
            "prime" => "<mo>′</mo>".to_string(),

            // \degree
            "degree" => "<mo>°</mo>".to_string(),

            // \bmod, \pmod
            "bmod" => "<mo>mod</mo>".to_string(),
            "pmod" => {
                let arg = self.parse_required_group();
                format!(
                    "<mrow><mo>(</mo><mi>mod</mi><mspace width=\"0.167em\" />{arg}</mrow><mo>)</mo>"
                )
            }
            "mod" => {
                let arg = self.parse_required_group();
                format!("<mi>mod</mi><mspace width=\"0.167em\" />{arg}")
            }
            "pod" => {
                let arg = self.parse_required_group();
                format!("<mrow><mo>(</mo>{arg}<mo>)</mo></mrow>")
            }

            // Additional function names not in the main list above
            "arcsec" | "arccsc" | "arccot" | "arcsinh" | "arccosh" | "arctanh" => {
                format!("<mi mathvariant=\"normal\">{name}</mi>")
            }

            // \boxed{...} — draw a box around content
            "boxed" => {
                let arg = self.parse_required_group();
                format!("<menclose notation=\"box\">{arg}</menclose>")
            }

            // \substack for multi-line subscripts
            "substack" => {
                let arg = self.parse_raw_group();
                let rows: Vec<&str> = arg.split("\\\\").collect();
                let rows_xml: String = rows
                    .iter()
                    .map(|r| {
                        let mut sub = MathMLParser::new(r.trim());
                        format!("<mtr><mtd>{}</mtd></mtr>", sub.parse_sequence())
                    })
                    .collect::<Vec<_>>()
                    .join("");
                format!("<mtable>{rows_xml}</mtable>")
            }

            // \textstyle, \displaystyle, etc. — style switches, no MathML equivalent
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" => {
                // Consume but don't render — MathML handles sizing automatically
                String::new()
            }

            // \boldsymbol, \pmb — bold math
            "boldsymbol" | "pmb" => {
                let arg = self.parse_required_group();
                // Wrap each mi/mo in mathvariant="bold"
                if arg.contains('<') {
                    arg.replace("<mi>", "<mi mathvariant=\"bold\">")
                        .replace("<mn>", "<mn mathvariant=\"bold\">")
                } else {
                    format!("<mi mathvariant=\"bold\">{}</mi>", escape_xml_text(&arg))
                }
            }

            // \limits, \nolimits, \displaylimits — placement control, no-op
            "limits" | "nolimits" | "displaylimits" => String::new(),

            // Default: look up in symbol table
            _ => {
                if let Some(sym) = symbols::lookup_symbol(name) {
                    if sym.is_empty() {
                        return String::new();
                    }
                    // Determine if it's an operator or identifier
                    if is_operator_symbol(sym) {
                        format!("<mo>{}</mo>", escape_xml_text(sym))
                    } else {
                        format!("<mi>{}</mi>", escape_xml_text(sym))
                    }
                } else {
                    // Unknown command — emit as text
                    format!("<mtext>\\{}</mtext>", escape_xml_text(name))
                }
            }
        }
    }

    /// Parse `\sqrt` which may have an optional `[n]` index.
    fn parse_sqrt(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() == Some('[') {
            self.next();
            let mut index = String::new();
            while let Some(ch) = self.peek() {
                if ch == ']' {
                    self.next();
                    break;
                }
                index.push(ch);
                self.next();
            }
            let arg = self.parse_required_group();
            // Wrap radicand in <mrow> if it contains multiple elements to ensure
            // <mroot> has exactly 2 children (radicand + index) per MathML spec
            let radicand = if arg.starts_with("<mrow>") || !arg.contains('<') || arg.chars().filter(|&c| c == '<').count() == 2 {
                arg
            } else {
                format!("<mrow>{arg}</mrow>")
            };
            let index_mathml = if index.contains('<') {
                index
            } else {
                format!("<mn>{}</mn>", escape_xml_text(&index))
            };
            format!("<mroot>{radicand}{index_mathml}</mroot>")
        } else {
            let arg = self.parse_required_group();
            format!("<msqrt>{arg}</msqrt>")
        }
    }

    /// Parse `\begin{env} ... \end{env}` environments (matrices, cases, etc.).
    fn parse_environment(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() != Some('{') {
            return String::new();
        }
        self.next();
        let mut env_name = String::new();
        while let Some(ch) = self.peek() {
            if ch == '}' {
                self.next();
                break;
            }
            env_name.push(ch);
            self.next();
        }

        // Consume optional column spec for some envs (e.g. {align*}{c|c})
        self.skip_whitespace();
        if self.peek() == Some('{') {
            // Check if it looks like a column spec (not content)
            // Heuristic: if it's short and contains |, l, c, r
            let save = self.pos;
            self.next();
            let mut spec = String::new();
            let mut brace_depth = 1;
            while let Some(ch) = self.peek() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        break;
                    }
                }
                spec.push(ch);
                self.next();
            }
            if spec.chars().all(|c| matches!(c, '|' | 'l' | 'c' | 'r' | ' ')) && !spec.is_empty() {
                // It's a column spec, consume it
                if self.peek() == Some('}') {
                    self.next();
                }
            } else {
                // Not a column spec, restore position
                self.pos = save;
            }
        }

        // Read until \end{env}
        let mut body = String::new();
        let end_marker = format!("\\end{{{env_name}}}");
        let end_chars: Vec<char> = end_marker.chars().collect();
        loop {
            if self.pos + end_chars.len() <= self.chars.len() {
                let window: Vec<char> = self.chars[self.pos..self.pos + end_chars.len()].to_vec();
                if window == end_chars {
                    self.pos += end_chars.len();
                    break;
                }
            }
            if let Some(ch) = self.next() {
                body.push(ch);
            } else {
                break;
            }
        }

        match env_name.as_str() {
            "matrix" | "pmatrix" | "bmatrix" | "Bmatrix" | "vmatrix" | "Vmatrix"
            | "smallmatrix" => {
                let table = self.build_matrix_table(&body);
                let (open, close) = match env_name.as_str() {
                    "pmatrix" | "smallmatrix" => ("(", ")"),
                    "bmatrix" => ("[", "]"),
                    "Bmatrix" => ("{", "}"),
                    "vmatrix" => ("|", "|"),
                    "Vmatrix" => ("‖", "‖"),
                    _ => ("", ""),
                };
                if open.is_empty() {
                    format!("<mtable>{table}</mtable>")
                } else {
                    format!(
                        "<mrow><mo>{open}</mo><mtable>{table}</mtable><mo>{close}</mo></mrow>"
                    )
                }
            }
            "cases" => {
                let table = self.build_matrix_table(&body);
                format!("<mrow><mo>{{</mo><mtable>{table}</mtable><mo>}}</mo></mrow>")
            }
            "aligned" | "align" | "align*" | "gather" | "gather*" | "multline"
            | "multline*" | "split" | "gathered" | "eqnarray" | "displaymath" | "math" => {
                // Render as mtable with rows
                let table = self.build_matrix_table(&body);
                format!("<mtable>{table}</mtable>")
            }
            "array" => {
                let table = self.build_matrix_table(&body);
                format!("<mtable>{table}</mtable>")
            }
            _ => {
                // Unknown environment — render body as-is
                let inner = latex_to_mathml(&body);
                format!("<mrow>{inner}</mrow>")
            }
        }
    }

    /// Build an `<mtr><mtd>...</mtd>...</mtr>` table from matrix body.
    fn build_matrix_table(&self, body: &str) -> String {
        let mut out = String::new();
        for row in body.split("\\\\") {
            let row = row.trim();
            if row.is_empty() {
                continue;
            }
            out.push_str("<mtr>");
            for cell in row.split('&') {
                let cell = cell.trim();
                let inner = latex_to_mathml(cell);
                out.push_str(&format!("<mtd>{inner}</mtd>"));
            }
            out.push_str("</mtr>");
        }
        out
    }

    /// Parse a required `{...}` group, returning raw text content (no math parsing).
    fn parse_raw_group(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() == Some('{') {
            self.next();
            let mut depth = 1;
            let mut text = String::new();
            while let Some(ch) = self.next() {
                if ch == '{' {
                    depth += 1;
                    text.push(ch);
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    text.push(ch);
                } else {
                    text.push(ch);
                }
            }
            text
        } else if let Some(ch) = self.next() {
            ch.to_string()
        } else {
            String::new()
        }
    }

    /// Parse a required `{...}` group.
    fn parse_required_group(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() == Some('{') {
            self.next();
            let inner = self.parse_sequence();
            if self.peek() == Some('}') {
                self.next();
            }
            inner
        } else if self.peek() == Some('\\') {
            // Single command as argument
            self.parse_element()
        } else if let Some(ch) = self.next() {
            self.char_to_mathml(ch)
        } else {
            String::new()
        }
    }

    /// Convert a single character to its MathML representation.
    fn char_to_mathml(&self, ch: char) -> String {
        match ch {
            '0'..='9' => format!("<mn>{ch}</mn>"),
            'a'..='z' | 'A'..='Z' => format!("<mi>{ch}</mi>"),
            // Greek lowercase → identifier
            'α' | 'β' | 'γ' | 'δ' | 'ε' | 'ζ' | 'η' | 'θ' | 'ι' | 'κ' | 'λ'
            | 'μ' | 'ν' | 'ξ' | 'π' | 'ρ' | 'σ' | 'τ' | 'υ' | 'φ' | 'χ' | 'ψ'
            | 'ω' => format!("<mi>{ch}</mi>"),
            // Greek uppercase → identifier
            'Γ' | 'Δ' | 'Θ' | 'Λ' | 'Ξ' | 'Π' | 'Σ' | 'Υ' | 'Φ' | 'Ψ' | 'Ω' => {
                format!("<mi>{ch}</mi>")
            }
            ' ' => String::new(),
            // Everything else (operators, symbols, punctuation) → operator
            _ => {
                let escaped = escape_xml_text(&ch.to_string());
                format!("<mo>{escaped}</mo>")
            }
        }
    }
}

/// Escape XML special characters in text content.
fn escape_xml_text(text: &str) -> String {
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

/// Determine whether a Unicode symbol should be rendered as an `<mo>` (operator)
/// rather than an `<mi>` (identifier).
fn is_operator_symbol(sym: &str) -> bool {
    if sym.chars().count() != 1 {
        return false;
    }
    let ch = sym.chars().next().unwrap();
    matches!(
        ch,
        '±' | '∓' | '×' | '·' | '∗' | '⋆' | '∘' | '•' | '⋄' | '⊕' | '⊖'
        | '⊗' | '⊙' | '≤' | '≥' | '≪' | '≫' | '≺' | '≻' | '≼' | '≽'
        | '≡' | '∼' | '≃' | '≅' | '≈' | '⊂' | '⊆' | '⊃' | '⊇' | '∉'
        | '≠' | '⊥' | '∥' | '⊨' | '∝' | '→' | '←' | '⇒' | '⇐' | '↔'
        | '⇔' | '↑' | '↓' | '⇑' | '⇓' | '↕' | '↗' | '↘' | '↙' | '↖'
        | '↦' | '⟶' | '⟵' | '⟹' | '⟸' | '⟷' | '⟺' | '⟼' | '↪' | '↩'
        | '⇀' | '⇁' | '↼' | '↽' | '⇌' | '⇋' | '↣' | '↢' | '↠' | '↞'
        | '↬' | '↫' | '↷' | '↶' | '↺' | '↻' | '⇝' | '↭' | '↛' | '↚'
        | '⇏' | '⇍' | '↮' | '⇎' | '↤' | '⟻' | '∑' | '∫' | '∏' | '∐'
        | '⋂' | '⋃' | '∮' | '∬' | '∭' | '∯' | '∰' | '⋁' | '⋀' | '⨆'
        | '⨄' | '⨅' | '⨋' | '⨿' | '∞' | '∂' | '∇' | '∀' | '∃' | '¬'
        | '∅' | '√' | '∠' | '∡' | '∢' | 'ℵ' | 'ℏ' | 'ℓ' | '∖' | '∴'
        | '∵' | '≀' | '⋔' | '⊺' | '⌟' | '∔' | '⋉' | '⋊' | '⋋' | '⋌'
        | '⋏' | '⋎' | '⊼' | '⊻' | '⊞' | '⊟' | '⊠' | '⊡' | '⋇' | '⋖'
        | '⋗' | '⋅' | '⊎' | '⊓' | '⊔' | '⋒' | '⋓' | '⩞' | '⩟' | '…'
        | '⋯' | '⋮' | '⋱' | '⋰' | '⌈' | '⌉' | '⌊' | '⌋' | '⟨' | '⟩'
        | '‖' | '°' | '′' | '‵' | '♭' | '♮' | '♯' | '△' | '▽' | '□'
        | '■' | '◊' | '◆' | '◯' | '⊛' | '⊚' | '⊝' | '®' | 'Ⓢ' | '★'
        | '☉' | '☿' | '♀' | '♂' | '♃' | '♄' | '♅' | '♆' | '♇' | '☽'
        | '℧' | 'Ⅎ' | '⅁' | '∁' | 'ð' | '⊢' | '⊣' | '⊩' | '⊪' | '⊫'
        | '⊬' | '⊭' | '⊮' | '⊯' | '⊏' | '⊐' | '⊑' | '⊒' | '⊊' | '⊋'
        | '⫋' | '⫌' | '⊈' | '⊉' | '≦' | '≧' | '⩽' | '⩾' | '⪕' | '⪖'
        | '≲' | '≳' | '⪅' | '⪆' | '≊' | '≶' | '≷' | '⪋' | '⪌' | '≐'
        | '≾' | '≿' | '⊀' | '⊁' | '⋠' | '⋡' | '⋨' | '⋩' | '⪹' | '⪺'
        | '≮' | '≯' | '≰' | '≱' | '⪇' | '⪈' | '≨' | '≩' | '⋘' | '⋙'
        | '⋚' | '⋛' | '≍' | '≎' | '≏' | '≬' | '∽' | '⋍' | '𝕜' | 'ℂ'
        | 'ℕ' | 'ℚ' | 'ℝ' | 'ℤ'
    )
}

/// Wrap MathML content in `<math>` tags for inline rendering.
pub fn inline_mathml(latex: &str) -> String {
    format!("<math>{}</math>", latex_to_mathml(latex))
}

/// Wrap MathML content in `<math display="block">` tags for display rendering.
pub fn display_mathml(latex: &str) -> String {
    format!(
        "<math display=\"block\">{}</math>",
        latex_to_mathml(latex)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable() {
        let ml = latex_to_mathml("x");
        assert_eq!(ml, "<mi>x</mi>");
    }

    #[test]
    fn test_simple_number() {
        let ml = latex_to_mathml("42");
        assert_eq!(ml, "<mn>4</mn><mn>2</mn>");
    }

    #[test]
    fn test_addition() {
        let ml = latex_to_mathml("x + y");
        assert_eq!(ml, "<mi>x</mi><mo>+</mo><mi>y</mi>");
    }

    #[test]
    fn test_superscript() {
        let ml = latex_to_mathml("x^2");
        assert_eq!(ml, "<msup><mi>x</mi><mn>2</mn></msup>");
    }

    #[test]
    fn test_subscript() {
        let ml = latex_to_mathml("x_i");
        assert_eq!(ml, "<msub><mi>x</mi><mi>i</mi></msub>");
    }

    #[test]
    fn test_subsuperscript() {
        let ml = latex_to_mathml("x_i^2");
        assert_eq!(ml, "<msubsup><mi>x</mi><mi>i</mi><mn>2</mn></msubsup>");
    }

    #[test]
    fn test_fraction() {
        let ml = latex_to_mathml("\\frac{a}{b}");
        assert_eq!(ml, "<mfrac><mi>a</mi><mi>b</mi></mfrac>");
    }

    #[test]
    fn test_sqrt() {
        let ml = latex_to_mathml("\\sqrt{x}");
        assert_eq!(ml, "<msqrt><mi>x</mi></msqrt>");
    }

    #[test]
    fn test_nth_root() {
        let ml = latex_to_mathml("\\sqrt[3]{x}");
        assert_eq!(ml, "<mroot><mi>x</mi><mn>3</mn></mroot>");
    }

    #[test]
    fn test_nth_root_multi_digit_radicand() {
        // Multi-digit radicand should be wrapped in <mrow> to ensure
        // <mroot> has exactly 2 children per MathML spec
        let ml = latex_to_mathml("\\sqrt[4]{16}");
        assert_eq!(ml, "<mroot><mrow><mn>1</mn><mn>6</mn></mrow><mn>4</mn></mroot>");
    }

    #[test]
    fn test_nth_root_multi_term_radicand() {
        // Multi-term radicand (like x+y) should be wrapped in <mrow>
        let ml = latex_to_mathml("\\sqrt[3]{x+y}");
        assert_eq!(
            ml,
            "<mroot><mrow><mi>x</mi><mo>+</mo><mi>y</mi></mrow><mn>3</mn></mroot>"
        );
    }

    #[test]
    fn test_greek_letter() {
        let ml = latex_to_mathml("\\alpha");
        assert_eq!(ml, "<mi>α</mi>");
    }

    #[test]
    fn test_operator_symbol() {
        let ml = latex_to_mathml("\\sum");
        assert_eq!(ml, "<mo>∑</mo>");
    }

    #[test]
    fn test_text() {
        let ml = latex_to_mathml("\\text{if}");
        assert_eq!(ml, "<mtext>if</mtext>");
    }

    #[test]
    fn test_vec_accent() {
        let ml = latex_to_mathml("\\vec{x}");
        assert_eq!(
            ml,
            "<mover accent=\"true\"><mi>x</mi><mo>→</mo></mover>"
        );
    }

    #[test]
    fn test_pmatrix() {
        let ml = latex_to_mathml("\\begin{pmatrix}1 & 2 \\\\ 3 & 4\\end{pmatrix}");
        assert!(ml.contains("<mo>(</mo>"));
        assert!(ml.contains("<mtable>"));
        assert!(ml.contains("<mtr>"));
        assert!(ml.contains("<mtd><mn>1</mn></mtd>"));
        assert!(ml.contains("<mtd><mn>2</mn></mtd>"));
        assert!(ml.contains("<mtd><mn>3</mn></mtd>"));
        assert!(ml.contains("<mtd><mn>4</mn></mtd>"));
        assert!(ml.contains("<mo>)</mo>"));
    }

    #[test]
    fn test_cases() {
        let ml = latex_to_mathml("\\begin{cases}1 & \\text{if } x > 0 \\\\ 0 & \\text{otherwise}\\end{cases}");
        assert!(ml.contains("<mo>{</mo>"));
        assert!(ml.contains("<mtable>"));
        assert!(ml.contains("<mtext>if </mtext>"));
    }

    #[test]
    fn test_sum_with_limits() {
        let ml = latex_to_mathml("\\sum_{i=0}^{n}");
        assert!(ml.contains("<mo>∑</mo>"));
        assert!(ml.contains("<msubsup>"));
        assert!(ml.contains("<mi>i</mi>"));
        assert!(ml.contains("<mn>0</mn>"));
        assert!(ml.contains("<mi>n</mi>"));
    }

    #[test]
    fn test_function_name() {
        let ml = latex_to_mathml("\\sin(x)");
        assert!(ml.contains("<mi mathvariant=\"normal\">sin</mi>"));
        assert!(ml.contains("<mo>(</mo>"));
        assert!(ml.contains("<mi>x</mi>"));
        assert!(ml.contains("<mo>)</mo>"));
    }

    #[test]
    fn test_inline_wrapper() {
        let ml = inline_mathml("x^2");
        assert_eq!(ml, "<math><msup><mi>x</mi><mn>2</mn></msup></math>");
    }

    #[test]
    fn test_display_wrapper() {
        let ml = display_mathml("x^2");
        assert_eq!(
            ml,
            "<math display=\"block\"><msup><mi>x</mi><mn>2</mn></msup></math>"
        );
    }

    #[test]
    fn test_left_right() {
        let ml = latex_to_mathml("\\left( x + y \\right)");
        assert!(ml.contains("<mo>(</mo>"));
        assert!(ml.contains("<mi>x</mi>"));
        assert!(ml.contains("<mo>+</mo>"));
        assert!(ml.contains("<mi>y</mi>"));
        assert!(ml.contains("<mo>)</mo>"));
    }

    #[test]
    fn test_braced_group() {
        let ml = latex_to_mathml("{a+b}");
        assert_eq!(ml, "<mrow><mi>a</mi><mo>+</mo><mi>b</mi></mrow>");
    }

    #[test]
    fn test_complex_expression() {
        let ml = latex_to_mathml("\\frac{1}{1+x^2}");
        assert!(ml.contains("<mfrac>"));
        assert!(ml.contains("<mn>1</mn>"));
        assert!(ml.contains("<msup><mi>x</mi><mn>2</mn></msup>"));
    }

    #[test]
    fn test_xml_escaping() {
        let ml = latex_to_mathml("x < y");
        assert!(ml.contains("&lt;"));
    }

    #[test]
    fn test_mod_command() {
        let ml = latex_to_mathml("\\mod{n}");
        assert!(ml.contains("mod"));
        assert!(ml.contains("<mi>n</mi>"));
    }

    #[test]
    fn test_pod_command() {
        let ml = latex_to_mathml("\\pod{n}");
        assert!(ml.contains("<mo>(</mo>"));
        assert!(ml.contains("<mi>n</mi>"));
        assert!(ml.contains("<mo>)</mo>"));
    }

    #[test]
    fn test_boxed_command() {
        let ml = latex_to_mathml("\\boxed{x}");
        assert!(ml.contains("<menclose"));
        assert!(ml.contains("<mi>x</mi>"));
    }

    #[test]
    fn test_substack_command() {
        let ml = latex_to_mathml("\\substack{a \\\\ b}");
        assert!(ml.contains("<mtable>"));
        assert!(ml.contains("<mtr>"));
        assert!(ml.contains("<mtd>"));
    }

    #[test]
    fn test_boldsymbol_mathml() {
        let ml = latex_to_mathml("\\boldsymbol{x}");
        assert!(ml.contains("bold"));
        assert!(ml.contains("<mi"));
    }

    #[test]
    fn test_displaystyle_mathml() {
        let ml = latex_to_mathml("\\displaystyle x^2");
        assert!(ml.contains("<mi>x</mi>"));
        assert!(!ml.contains("displaystyle"));
    }

    #[test]
    fn test_limits_nolimits() {
        let ml = latex_to_mathml("\\sum\\limits_{i=0}^n");
        assert!(ml.contains("<mo>∑</mo>"));
    }

    #[test]
    fn test_additional_function_names_mathml() {
        let ml = latex_to_mathml("\\arcsec");
        assert!(ml.contains("arcsec"));
        assert!(ml.contains("mathvariant=\"normal\""));
    }
}
