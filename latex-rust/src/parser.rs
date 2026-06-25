//! LaTeX parser for converting tokens to AST

use crate::ast::*;
use crate::error::{LaTeXError, LaTeXResult};
use crate::lexer::{Token, TokenType};
use crate::bibtex::{BibTeXParser, BibEntry};
use std::collections::HashMap;

/// LaTeX parser
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    command_handlers: HashMap<String, fn(&mut Parser) -> LaTeXResult<Node>>,
    custom_commands: HashMap<String, (Option<usize>, Vec<String>, Vec<Node>)>, // (num_args, default_args, definition)
    labels: HashMap<String, usize>, // Track defined labels with their token positions
    references: Vec<(String, usize)>, // Track references with their token positions
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        let mut parser = Parser {
            tokens: Vec::new(),
            current: 0,
            command_handlers: HashMap::new(),
            custom_commands: HashMap::new(),
            labels: HashMap::new(),
            references: Vec::new(),
        };
        parser.register_default_commands();
        parser
    }

    /// Parse tokens into a document
    pub fn parse(&mut self, tokens: Vec<Token>) -> LaTeXResult<Document> {
        self.tokens = tokens;
        self.current = 0;
        
        // Clear previous tracking data
        self.labels.clear();
        self.references.clear();
        
        let mut document = Document::new();
        let mut in_document = false;
        
        while !self.is_at_end() {
            let node = self.parse_node()?;
            
            // Handle document structure
            match &node {
                Node::Environment { name, content, .. } => {
                    if name == "document" {
                        // Extract content from document environment and add to body
                        for content_node in content {
                            document.add_node(content_node.clone());
                        }
                        break; // End of document
                    } else if in_document {
                        document.add_node(node);
                    } else {
                        document.add_preamble_node(node);
                    }
                }
                Node::Command { name, args, .. } => {
                    match name.as_str() {
                        "documentclass" => {
                            if let Some(Argument::Required(class_nodes)) = args.first() {
                                if let Some(Node::Text(class_name)) = class_nodes.first() {
                                    document.metadata.document_class = Some(class_name.clone());
                                }
                            }
                            document.add_preamble_node(node);
                        }
                        "usepackage" => {
                            if let Some(Argument::Required(pkg_nodes)) = args.first() {
                                if let Some(Node::Text(pkg_name)) = pkg_nodes.first() {
                                    document.add_package(pkg_name.clone());
                                }
                            }
                            document.add_preamble_node(node);
                        }
                        "title" => {
                            if let Some(Argument::Required(title_nodes)) = args.first() {
                                let title = self.nodes_to_text(title_nodes);
                                document.set_title(title);
                            }
                            document.add_preamble_node(node);
                        }
                        "author" => {
                            if let Some(Argument::Required(author_nodes)) = args.first() {
                                let author = self.nodes_to_text(author_nodes);
                                document.set_author(author);
                            }
                            document.add_preamble_node(node);
                        }
                        "begin" => {
                            if let Some(Argument::Required(env_nodes)) = args.first() {
                                if let Some(Node::Text(env_name)) = env_nodes.first() {
                                    if env_name == "document" {
                                        in_document = true;
                                        continue;
                                    }
                                }
                            }
                            if in_document {
                                document.add_node(node);
                            } else {
                                document.add_preamble_node(node);
                            }
                        }
                        "end" => {
                            if let Some(Argument::Required(env_nodes)) = args.first() {
                                if let Some(Node::Text(env_name)) = env_nodes.first() {
                                    if env_name == "document" {
                                        break;
                                    }
                                }
                            }
                            document.add_node(node);
                        }
                        _ => {
                            if in_document {
                                document.add_node(node);
                            } else {
                                document.add_preamble_node(node);
                            }
                        }
                    }
                }
                _ => {
                    if in_document {
                        document.add_node(node);
                    } else {
                        document.add_preamble_node(node);
                    }
                }
            }
        }
        
        // Validate cross-references
        self.validate_references()?;
        
        Ok(document)
    }

    /// Parse a single node
    fn parse_node(&mut self) -> LaTeXResult<Node> {
        if self.is_at_end() {
            return Err(LaTeXError::ParserError {
                position: crate::error::Position::new(0, 0, self.current),
                message: "Unexpected end of input".to_string(),
            });
        }

        let token = &self.tokens[self.current].clone();
        
        match &token.token_type {
            TokenType::Command(name) => {
                self.advance();
                self.parse_command(name.clone())
            }
            TokenType::Text(text) => {
                self.advance();
                Ok(Node::Text(text.clone()))
            }
            TokenType::LeftBrace => {
                self.parse_group()
            }
            TokenType::Dollar => {
                self.parse_math(false)
            }
            TokenType::DoubleDollar => {
                self.parse_math(true)
            }
            TokenType::Newline => {
                self.advance();
                // Check for paragraph break (double newline)
                if self.match_token(&TokenType::Newline) {
                    Ok(Node::Paragraph)
                } else {
                    Ok(Node::LineBreak)
                }
            }
            TokenType::Whitespace(ws) => {
                self.advance();
                Ok(Node::Whitespace(ws.clone()))
            }
            TokenType::Backslash => {
                self.advance();
                Ok(Node::LineBreak)
            }
            TokenType::Ampersand => {
                self.advance();
                Ok(Node::Text("&".to_string()))
            }
            _ => {
                let pos = token.position;
                self.advance();
                Err(LaTeXError::ParserError {
                    position: pos,
                    message: format!("Unexpected token: {:?}", token.token_type),
                })
            }
        }
    }

    /// Parse a LaTeX command
    fn parse_command(&mut self, name: String) -> LaTeXResult<Node> {
        // Try custom handler first
        if let Some(result) = self.try_custom_handler(&name)? {
            return Ok(result);
        }

        // Try custom command definition
        if let Some(result) = self.try_custom_command(&name)? {
            return Ok(result);
        }

        // Default command parsing
        self.parse_default_command(name)
    }

    /// Try to handle command with custom handler
    fn try_custom_handler(&mut self, name: &str) -> LaTeXResult<Option<Node>> {
        if let Some(handler) = self.command_handlers.get(name).copied() {
            Ok(Some(handler(self)?))
        } else {
            Ok(None)
        }
    }

    /// Try to handle custom command definition
    fn try_custom_command(&mut self, name: &str) -> LaTeXResult<Option<Node>> {
        if let Some((num_args, _default_args, definition)) = self.custom_commands.get(name).cloned() {
            let _args = self.parse_custom_command_args(num_args)?;
            
            // For now, return the definition as a group node
            // In a full implementation, we would substitute arguments
            Ok(Some(Node::Group(definition)))
        } else {
            Ok(None)
        }
    }

    /// Parse arguments for custom command
    fn parse_custom_command_args(&mut self, num_args: Option<usize>) -> LaTeXResult<Vec<Argument>> {
        let mut args = Vec::new();
        
        if let Some(arg_count) = num_args {
            for _ in 0..arg_count {
                args.push(self.parse_required_argument()?);
            }
        }
        
        Ok(args)
    }

    /// Parse default command with standard argument parsing
    fn parse_default_command(&mut self, name: String) -> LaTeXResult<Node> {
        let mut args = Vec::new();
        
        // Parse optional arguments
        args.extend(self.parse_all_optional_arguments()?);
        
        // Parse required arguments
        args.extend(self.parse_all_required_arguments()?);

        Ok(Node::Command {
            name,
            args,
            content: None,
        })
    }

    /// Parse all consecutive optional arguments
    fn parse_all_optional_arguments(&mut self) -> LaTeXResult<Vec<Argument>> {
        let mut args = Vec::new();
        
        while self.check(&TokenType::LeftBracket) {
            args.push(self.parse_optional_argument()?);
        }
        
        Ok(args)
    }

    /// Parse all consecutive required arguments
    fn parse_all_required_arguments(&mut self) -> LaTeXResult<Vec<Argument>> {
        let mut args = Vec::new();
        
        while self.check(&TokenType::LeftBrace) {
            args.push(self.parse_required_argument()?);
        }
        
        Ok(args)
    }

    /// Parse a group (content within braces)
    fn parse_group(&mut self) -> LaTeXResult<Node> {
        self.consume(&TokenType::LeftBrace, "Expected '{'".to_string())?;
        
        let mut nodes = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            nodes.push(self.parse_node()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}'".to_string())?;
        
        Ok(Node::Group(nodes))
    }

    /// Parse mathematical expression
    fn parse_math(&mut self, display: bool) -> LaTeXResult<Node> {
        let start_token = if display {
            TokenType::DoubleDollar
        } else {
            TokenType::Dollar
        };
        
        self.consume(&start_token, "Expected math delimiter".to_string())?;
        
        let mut content = String::new();
        
        while !self.check(&start_token) && !self.is_at_end() {
            let token = &self.tokens[self.current];
            match &token.token_type {
                TokenType::Text(text) => content.push_str(text),
                TokenType::Command(cmd) => {
                    content.push('\\');
                    content.push_str(cmd);
                }
                TokenType::LeftBrace => content.push('{'),
                TokenType::RightBrace => content.push('}'),
                TokenType::Whitespace(ws) => content.push_str(ws),
                _ => {}
            }
            self.advance();
        }
        
        self.consume(&start_token, "Expected closing math delimiter".to_string())?;
        
        Ok(Node::Math { display, content })
    }

    /// Parse required argument {content}
    fn parse_required_argument(&mut self) -> LaTeXResult<Argument> {
        self.consume(&TokenType::LeftBrace, "Expected '{'".to_string())?;
        
        let mut nodes = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            nodes.push(self.parse_node()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}'".to_string())?;
        
        Ok(Argument::Required(nodes))
    }

    /// Parse optional argument [content]
    fn parse_optional_argument(&mut self) -> LaTeXResult<Argument> {
        self.consume(&TokenType::LeftBracket, "Expected '['".to_string())?;
        
        let mut nodes = Vec::new();
        
        while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
            nodes.push(self.parse_node()?);
        }
        
        self.consume(&TokenType::RightBracket, "Expected ']'".to_string())?;
        
        Ok(Argument::Optional(nodes))
    }

    /// Register default command handlers
    fn register_default_commands(&mut self) {
        // Section commands
        self.command_handlers.insert("section".to_string(), Self::parse_section);
        self.command_handlers.insert("subsection".to_string(), Self::parse_section);
        self.command_handlers.insert("subsubsection".to_string(), Self::parse_section);
        
        // Environment commands
        self.command_handlers.insert("begin".to_string(), Self::parse_environment);
        
        // Reference commands
        self.command_handlers.insert("label".to_string(), Self::parse_label);
        self.command_handlers.insert("ref".to_string(), Self::parse_reference);
        self.command_handlers.insert("pageref".to_string(), Self::parse_reference);
        self.command_handlers.insert("eqref".to_string(), Self::parse_reference);
        
        // Footnote commands
        self.command_handlers.insert("footnote".to_string(), Self::parse_footnote);
        
        // Spacing commands
        self.command_handlers.insert("vspace".to_string(), Self::parse_spacing);
        self.command_handlers.insert("hspace".to_string(), Self::parse_spacing);
        self.command_handlers.insert("vfill".to_string(), Self::parse_spacing);
        self.command_handlers.insert("hfill".to_string(), Self::parse_spacing);
        self.command_handlers.insert("newpage".to_string(), Self::parse_spacing);
        self.command_handlers.insert("clearpage".to_string(), Self::parse_spacing);
        self.command_handlers.insert("pagebreak".to_string(), Self::parse_spacing);
        self.command_handlers.insert("par".to_string(), Self::parse_spacing);
        
        // Mathematical commands
        self.command_handlers.insert("frac".to_string(), Self::parse_math_command);
        self.command_handlers.insert("sqrt".to_string(), Self::parse_math_command);
        self.command_handlers.insert("sum".to_string(), Self::parse_math_command);
        self.command_handlers.insert("int".to_string(), Self::parse_math_command);
        self.command_handlers.insert("prod".to_string(), Self::parse_math_command);
        self.command_handlers.insert("lim".to_string(), Self::parse_math_command);
        self.command_handlers.insert("displaystyle".to_string(), Self::parse_math_command);
        self.command_handlers.insert("textstyle".to_string(), Self::parse_math_command);
        
        // List item command
        self.command_handlers.insert("item".to_string(), Self::parse_item);
        
        // Custom command definition commands
        self.command_handlers.insert("newcommand".to_string(), Self::parse_command_definition);
        self.command_handlers.insert("renewcommand".to_string(), Self::parse_command_definition);
        self.command_handlers.insert("def".to_string(), Self::parse_def_command);
        
        // Citation commands
        self.command_handlers.insert("cite".to_string(), Self::parse_citation);
        self.command_handlers.insert("citep".to_string(), Self::parse_citation);
        self.command_handlers.insert("citet".to_string(), Self::parse_citation);
        self.command_handlers.insert("citeauthor".to_string(), Self::parse_citation);
        self.command_handlers.insert("citeyear".to_string(), Self::parse_citation);
        self.command_handlers.insert("nocite".to_string(), Self::parse_citation);
        
        // Bibliography commands
        self.command_handlers.insert("bibliographystyle".to_string(), Self::parse_bibliography_style);
        self.command_handlers.insert("bibliography".to_string(), Self::parse_bibliography);
        
        // Package commands
        self.command_handlers.insert("usepackage".to_string(), Self::parse_usepackage);
    }

    /// Parse section command
    fn parse_section(&mut self) -> LaTeXResult<Node> {
        let cmd_name = match &self.tokens[self.current - 1].token_type {
            TokenType::Command(name) => name.clone(),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() {
                    self.tokens[self.current].position
                } else {
                    crate::error::Position::new(0, 0, self.current)
                },
                message: "Expected section command".to_string(),
            }),
        };

        let level = SectionLevel::from_command(&cmd_name)
            .ok_or_else(|| LaTeXError::ParserError {
                position: if self.current < self.tokens.len() {
                    self.tokens[self.current].position
                } else {
                    crate::error::Position::new(0, 0, self.current)
                },
                message: format!("Unknown section command: {cmd_name}"),
            })?;

        let title_arg = self.parse_required_argument()?;
        let title = match title_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => String::new(),
        };

        Ok(Node::Section {
            level,
            title,
            content: Vec::new(),
        })
    }

    /// Parse environment
    fn parse_environment(&mut self) -> LaTeXResult<Node> {
        let env_arg = self.parse_required_argument()?;
        let env_name = match env_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() {
                    self.tokens[self.current].position
                } else {
                    crate::error::Position::new(0, 0, self.current)
                },
                message: "Expected environment name".to_string(),
            }),
        };

        // Handle table environments specially
        if matches!(env_name.as_str(), "tabular" | "longtable" | "array" | "tabularx") {
            return self.parse_table_environment(&env_name);
        }
        
        // Handle figure environment specially
        if env_name == "figure" {
            return self.parse_figure_environment();
        }
        
        // Handle mathematical environments specially
        if matches!(env_name.as_str(), "equation" | "align" | "gather" | "multline" | "array" | "matrix" | "pmatrix" | "bmatrix" | "vmatrix" | "smallmatrix" | "cases" | "split" | "aligned" | "eqnarray") {
            return self.parse_math_environment(&env_name);
        }
        
        // Handle list environments specially
        if matches!(env_name.as_str(), "itemize" | "enumerate" | "description") {
            return self.parse_list_environment(&env_name);
        }
        
        // Handle code environments specially
        if matches!(env_name.as_str(), "verbatim" | "verbatim*" | "lstlisting" | "minted") {
            return self.parse_code_environment(&env_name);
        }

        let mut content = Vec::new();
        
        // Parse content until \end{env_name}
        while !self.is_at_end() {
            if self.check_command("end") {
                self.advance(); // consume \end
                let end_arg = self.parse_required_argument()?;
                let end_name = match end_arg {
                    Argument::Required(nodes) => self.nodes_to_text(&nodes),
                    _ => String::new(),
                };
                
                if end_name == env_name {
                    break;
                }
            }
            
            content.push(self.parse_node()?);
        }

        Ok(Node::Environment {
            name: env_name,
            args: Vec::new(),
            content,
        })
    }

    /// Helper methods
    pub(crate) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.tokens[self.current].token_type, TokenType::Eof)
    }

    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    pub(crate) fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.tokens[self.current].token_type) == std::mem::discriminant(token_type)
        }
    }

    pub(crate) fn check_command(&self, cmd: &str) -> bool {
        if self.is_at_end() {
            false
        } else {
            matches!(&self.tokens[self.current].token_type, TokenType::Command(name) if name == cmd)
        }
    }

    pub(crate) fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn consume(&mut self, token_type: &TokenType, message: String) -> LaTeXResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() {
                    self.tokens[self.current].position
                } else {
                    crate::error::Position::new(0, 0, self.current)
                },
                message,
            })
        }
    }

    fn parse_table_environment(&mut self, env_name: &str) -> LaTeXResult<Node> {
        // Determine table type
        let table_type = match env_name {
            "tabular" => TableType::Tabular,
            "longtable" => TableType::Longtable,
            "array" => TableType::Array,
            "tabularx" => TableType::Tabularx,
            _ => TableType::Tabular,
        };
        
        // Parse optional position argument for longtable
        let position = if env_name == "longtable" && self.check(&TokenType::LeftBracket) {
            self.advance(); // consume [
            let pos_arg = self.parse_optional_argument()?;
            match pos_arg {
                Argument::Optional(nodes) => Some(self.nodes_to_text(&nodes)),
                _ => None,
            }
        } else {
            None
        };
        
        // Parse width argument for tabularx
        let _width = if env_name == "tabularx" {
            let width_arg = self.parse_required_argument()?;
            match width_arg {
                Argument::Required(nodes) => Some(self.nodes_to_text(&nodes)),
                _ => None,
            }
        } else {
            None
        };
        // Parse column specification {l|c|r|...}
        let column_spec_arg = self.parse_required_argument()?;
        let column_spec = match column_spec_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => String::new(),
        };

        // Parse column alignments from specification
        let alignment = self.parse_column_alignment(&column_spec);
        let mut rows = Vec::new();
        let mut current_row_cells = Vec::new();
        let mut current_cell = Vec::new();
        let mut pending_rules = Vec::new();

        // Parse table content until \end{tabular}
        while !self.is_at_end() {
            if self.check_command("end") {
                // Look ahead to see if it's \end{tabular}
                let saved_pos = self.current;
                self.advance(); // consume \end
                let end_arg = self.parse_required_argument()?;
                let end_name = match end_arg {
                    Argument::Required(nodes) => self.nodes_to_text(&nodes),
                    _ => String::new(),
                };
                
                if end_name == "tabular" {
                    // Finish the current cell and row if they have content
                    if !current_cell.is_empty() {
                        current_row_cells.push(TableCell::Normal(current_cell));
                    }
                    if !current_row_cells.is_empty() {
                        rows.push(TableRow { 
                            cells: current_row_cells,
                            rules_before: pending_rules.clone(),
                            rules_after: Vec::new(),
                        });
                    }
                    break;
                } else {
                    // Not the end of tabular, restore position and continue
                    self.current = saved_pos;
                }
            }
            
            let token = &self.tokens[self.current].clone();
            match &token.token_type {
                TokenType::Ampersand => {
                    // End current cell, start new cell
                    current_row_cells.push(TableCell::Normal(current_cell));
                    current_cell = Vec::new();
                    self.advance();
                }
                TokenType::Command(cmd) if cmd == "\\\\" => {
                    // End current row
                    if !current_cell.is_empty() {
                        current_row_cells.push(TableCell::Normal(current_cell));
                    }
                    if !current_row_cells.is_empty() {
                        rows.push(TableRow { 
                            cells: current_row_cells,
                            rules_before: pending_rules.clone(),
                            rules_after: Vec::new(),
                        });
                    }
                    current_row_cells = Vec::new();
                    current_cell = Vec::new();
                    pending_rules.clear();
                    self.advance();
                }
                TokenType::Command(cmd) if cmd == "hline" => {
                    pending_rules.push(TableRule::Hline);
                    self.advance();
                }
                TokenType::Command(cmd) if cmd == "cline" => {
                    self.advance(); // consume \cline
                    let range_arg = self.parse_required_argument()?;
                    let range = match range_arg {
                        Argument::Required(nodes) => self.nodes_to_text(&nodes),
                        _ => String::new(),
                    };
                    pending_rules.push(TableRule::Cline(range));
                }
                TokenType::Command(cmd) if cmd == "toprule" => {
                    pending_rules.push(TableRule::Toprule);
                    self.advance();
                }
                TokenType::Command(cmd) if cmd == "midrule" => {
                    pending_rules.push(TableRule::Midrule);
                    self.advance();
                }
                TokenType::Command(cmd) if cmd == "bottomrule" => {
                    pending_rules.push(TableRule::Bottomrule);
                    self.advance();
                }
                TokenType::Command(cmd) if cmd == "multicolumn" => {
                    let multicolumn = self.parse_multicolumn()?;
                    current_row_cells.push(TableCell::MultiColumn(multicolumn));
                }
                TokenType::Command(cmd) if cmd == "multirow" => {
                    let multirow = self.parse_multirow()?;
                    current_row_cells.push(TableCell::MultiRow(multirow));
                }
                TokenType::Command(cmd) if cmd == "caption" => {
                    // Handle caption parsing - this should be stored in the table
                    self.advance(); // consume \caption
                    let caption_arg = self.parse_required_argument()?;
                    // Note: caption will be handled at table level, skip for now
                }
                TokenType::Command(cmd) if cmd == "label" => {
                    // Handle label parsing - this should be stored in the table
                    self.advance(); // consume \label
                    let _label_arg = self.parse_required_argument()?;
                    // Note: label will be handled at table level, skip for now
                }
                _ => {
                    // Add content to current cell
                    current_cell.push(self.parse_node()?);
                }
            }
        }

        Ok(Node::Table { 
            table_type,
            alignment, 
            rows,
            caption: None,
            label: None,
            position,
        })
    }

    /// Parse multicolumn command
    fn parse_multicolumn(&mut self) -> LaTeXResult<MultiColumn> {
        self.advance(); // consume \multicolumn
        
        // Parse span argument {n}
        let span_arg = self.parse_required_argument()?;
        let span_str = match span_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => "1".to_string(),
        };
        let span = span_str.parse::<usize>().unwrap_or(1);
        
        // Parse alignment argument {c|l|r}
        let align_arg = self.parse_required_argument()?;
        let align_spec = match align_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => "l".to_string(),
        };
        let alignment = self.parse_single_column_alignment(&align_spec);
        
        // Parse content argument {content}
        let content_arg = self.parse_required_argument()?;
        let content = match content_arg {
            Argument::Required(nodes) => nodes,
            _ => Vec::new(),
        };
        
        Ok(MultiColumn {
            span,
            alignment,
            content,
        })
    }
    
    /// Parse multirow command
    fn parse_multirow(&mut self) -> LaTeXResult<MultiRow> {
        self.advance(); // consume \multirow
        
        // Parse span argument {n}
        let span_arg = self.parse_required_argument()?;
        let span_str = match span_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => "1".to_string(),
        };
        let span = span_str.parse::<usize>().unwrap_or(1);
        
        // Parse width argument {width} (optional)
        let width_arg = self.parse_required_argument()?;
        let width = match width_arg {
            Argument::Required(nodes) => {
                let width_text = self.nodes_to_text(&nodes);
                if width_text.trim() == "*" {
                    None
                } else {
                    Some(width_text)
                }
            },
            _ => None,
        };
        
        // Parse content argument {content}
        let content_arg = self.parse_required_argument()?;
        let content = match content_arg {
            Argument::Required(nodes) => nodes,
            _ => Vec::new(),
        };
        
        Ok(MultiRow {
            span,
            width,
            content,
        })
    }
    
    /// Parse a single column alignment specification
    fn parse_single_column_alignment(&self, spec: &str) -> ColumnAlignment {
        let spec = spec.trim();
        if spec.starts_with('p') {
            // Extract width from p{width}
            if let Some(start) = spec.find('{') {
                if let Some(end) = spec.find('}') {
                    let width = spec[start+1..end].to_string();
                    return ColumnAlignment::Paragraph(width);
                }
            }
            ColumnAlignment::Paragraph("1cm".to_string())
        } else if spec.starts_with('m') || spec.starts_with('b') {
            // Extract width from m{width} or b{width}
            if let Some(start) = spec.find('{') {
                if let Some(end) = spec.find('}') {
                    let width = spec[start+1..end].to_string();
                    return ColumnAlignment::FixedWidth(width);
                }
            }
            ColumnAlignment::FixedWidth("1cm".to_string())
        } else if spec.starts_with('X') {
            ColumnAlignment::XColumn
        } else if spec.starts_with('S') {
            ColumnAlignment::SColumn("default".to_string())
        } else {
            match spec.chars().next().unwrap_or('l') {
                'c' => ColumnAlignment::Center,
                'r' => ColumnAlignment::Right,
                '@' => ColumnAlignment::ArrayColumn("c".to_string()),
                _ => ColumnAlignment::Left,
            }
        }
    }

    fn parse_figure_environment(&mut self) -> LaTeXResult<Node> {
        let mut path = String::new();
        let mut caption = None;
        let mut label = None;
        let mut width = None;
        let mut height = None;
        
        // Parse figure content until \end{figure}
        while !self.is_at_end() {
            if self.check_command("end") {
                // Look ahead to see if it's \end{figure}
                let saved_pos = self.current;
                self.advance(); // consume \end
                let end_arg = self.parse_required_argument()?;
                let end_name = match end_arg {
                    Argument::Required(nodes) => self.nodes_to_text(&nodes),
                    _ => String::new(),
                };
                
                if end_name == "figure" {
                    break;
                } else {
                    // Not the end of figure, restore position and continue
                    self.current = saved_pos;
                }
            }
            
            if self.check_command("includegraphics") {
                self.advance(); // consume \includegraphics
                
                // Parse optional arguments for width/height
                if self.check(&TokenType::LeftBracket) {
                    let options_arg = self.parse_optional_argument()?;
                    if let Argument::Optional(nodes) = options_arg {
                        let options_text = self.nodes_to_text(&nodes);
                        // Parse width and height from options like "width=5cm,height=3cm"
                        for option in options_text.split(',') {
                            let option = option.trim();
                            if option.starts_with("width=") {
                                width = Some(option[6..].to_string());
                            } else if option.starts_with("height=") {
                                height = Some(option[7..].to_string());
                            }
                        }
                    }
                }
                
                // Parse required argument for file path
                let path_arg = self.parse_required_argument()?;
                if let Argument::Required(nodes) = path_arg {
                    path = self.nodes_to_text(&nodes);
                }
            } else if self.check_command("caption") {
                self.advance(); // consume \caption
                let caption_arg = self.parse_required_argument()?;
                if let Argument::Required(nodes) = caption_arg {
                    caption = Some(self.nodes_to_text(&nodes));
                }
            } else if self.check_command("label") {
                self.advance(); // consume \label
                let label_arg = self.parse_required_argument()?;
                if let Argument::Required(nodes) = label_arg {
                    label = Some(self.nodes_to_text(&nodes));
                }
            } else {
                // Skip other content
                self.advance();
            }
        }
        
        Ok(Node::Figure {
            path,
            caption,
            label,
            width,
            height,
        })
    }

    fn parse_column_alignment(&self, spec: &str) -> Vec<ColumnAlignment> {
        let mut alignments = Vec::new();
        let mut chars = spec.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                'l' => alignments.push(ColumnAlignment::Left),
                'c' => alignments.push(ColumnAlignment::Center),
                'r' => alignments.push(ColumnAlignment::Right),
                'X' => alignments.push(ColumnAlignment::XColumn),
                'S' => alignments.push(ColumnAlignment::SColumn("default".to_string())),
                '@' => alignments.push(ColumnAlignment::ArrayColumn("c".to_string())),
                'p' | 'm' | 'b' => {
                    let width = self.extract_width_specification(&mut chars);
                    let alignment = self.create_width_based_alignment(ch, width);
                    alignments.push(alignment);
                }
                '|' => {}, // Ignore vertical lines for now
                _ => {}, // Ignore other characters
            }
        }
        alignments
    }

    /// Extract width specification from column alignment
    fn extract_width_specification(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut width = String::new();
        let mut brace_count = 0;
        let mut found_opening = false;
        
        while let Some(&next_ch) = chars.peek() {
            if next_ch == '{' {
                found_opening = true;
                brace_count += 1;
                chars.next();
            } else if next_ch == '}' && found_opening {
                brace_count -= 1;
                chars.next();
                if brace_count == 0 {
                    break;
                }
            } else if found_opening {
                width.push(next_ch);
                chars.next();
            } else {
                break;
            }
        }
        
        if width.is_empty() {
            "1cm".to_string()
        } else {
            width
        }
    }

    /// Create column alignment based on character and width
    fn create_width_based_alignment(&self, ch: char, width: String) -> ColumnAlignment {
        match ch {
            'p' => ColumnAlignment::Paragraph(width),
            'm' | 'b' => ColumnAlignment::FixedWidth(width),
            _ => unreachable!("Invalid width-based alignment character: {}", ch),
        }
    }

    fn nodes_to_text(&self, nodes: &[Node]) -> String {
        nodes.iter()
            .filter_map(|node| match node {
                Node::Text(text) => Some(text.as_str()),
                Node::Whitespace(ws) => Some(ws.as_str()),
                Node::Command { name, .. } => {
                    // Handle special commands that should be preserved as text
                    match name.as_str() {
                        "textwidth" => Some("\\textwidth"),
                        _ => None,
                    }
                },
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
    
    /// Parse mathematical environment
    fn parse_math_environment(&mut self, env_name: &str) -> LaTeXResult<Node> {
        let env_type = match env_name {
            "equation" => MathEnvironmentType::Equation,
            "align" => MathEnvironmentType::Align,
            "gather" => MathEnvironmentType::Gather,
            "multline" => MathEnvironmentType::Multline,
            "array" => MathEnvironmentType::Array,
            "matrix" => MathEnvironmentType::Matrix,
            "pmatrix" => MathEnvironmentType::Pmatrix,
            "bmatrix" => MathEnvironmentType::Bmatrix,
            "vmatrix" => MathEnvironmentType::Vmatrix,
            "smallmatrix" => MathEnvironmentType::Smallmatrix,
            "cases" => MathEnvironmentType::Cases,
            "split" => MathEnvironmentType::Split,
            "aligned" => MathEnvironmentType::Aligned,
            "eqnarray" => MathEnvironmentType::Eqnarray,
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() {
                    self.tokens[self.current].position
                } else {
                    crate::error::Position::new(0, 0, self.current)
                },
                message: format!("Unknown math environment: {env_name}"),
            }),
        };
        
        let mut content = Vec::new();
        let mut label = None;
        let numbered = matches!(env_type, MathEnvironmentType::Equation | MathEnvironmentType::Align | MathEnvironmentType::Gather);
        
        // Parse content until \end{env_name}
        while !self.is_at_end() {
            if self.check_command("end") {
                let saved_pos = self.current;
                self.advance(); // consume \end
                let end_arg = self.parse_required_argument()?;
                let end_name = match end_arg {
                    Argument::Required(nodes) => self.nodes_to_text(&nodes),
                    _ => String::new(),
                };
                if end_name == env_name {
                    break;
                }
                // Not the end of this environment, restore position
                self.current = saved_pos;
            } else if self.check_command("label") {
                self.advance(); // consume \label
                let label_arg = self.parse_required_argument()?;
                if let Argument::Required(nodes) = label_arg {
                    label = Some(self.nodes_to_text(&nodes));
                }
            } else {
                // Parse content as nodes
                content.push(self.parse_node()?);
            }
        }
        
        Ok(Node::MathEnvironment {
            env_type,
            content,
            label,
            numbered,
            equation_number: None, // Will be set during processing
        })
    }
    
    /// Parse label command
    fn parse_label(&mut self) -> LaTeXResult<Node> {
        let label_arg = self.parse_required_argument()?;
        let label = match label_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: "Label command requires argument".to_string(),
            }),
        };
        
        // Track the label for cross-reference validation
        if self.labels.contains_key(&label) {
            // Warn about duplicate labels but don't fail
            eprintln!("Warning: Duplicate label '{label}' found");
        }
        self.labels.insert(label.clone(), self.current);
        
        Ok(Node::Label(label))
    }
    
    /// Parse reference command
    fn parse_reference(&mut self) -> LaTeXResult<Node> {
        let cmd_name = match &self.tokens[self.current - 1].token_type {
            TokenType::Command(name) => name.clone(),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: "Expected reference command".to_string(),
            }),
        };
        
        let ref_type = match cmd_name.as_str() {
            "ref" => ReferenceType::Ref,
            "pageref" => ReferenceType::Pageref,
            "eqref" => ReferenceType::Eqref,
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: format!("Unknown reference command: {cmd_name}"),
            }),
        };
        
        let label_arg = self.parse_required_argument()?;
        let label = match label_arg {
            Argument::Required(nodes) => self.nodes_to_text(&nodes),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: "Reference command requires argument".to_string(),
            }),
        };
        
        // Track the reference for validation
        self.references.push((label.clone(), self.current));
        
        Ok(Node::Reference { ref_type, label })
    }
    
    /// Parse footnote command
    fn parse_footnote(&mut self) -> LaTeXResult<Node> {
        let content_arg = self.parse_required_argument()?;
        let content = match content_arg {
            Argument::Required(nodes) => nodes,
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: "Footnote command requires argument".to_string(),
            }),
        };
        Ok(Node::Footnote { content })
    }
    
    /// Parse spacing command
    fn parse_spacing(&mut self) -> LaTeXResult<Node> {
        let cmd_name = match &self.tokens[self.current - 1].token_type {
            TokenType::Command(name) => name.clone(),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: "Expected spacing command".to_string(),
            }),
        };
        
        let space_type = match cmd_name.as_str() {
            "vspace" => SpacingType::Vspace,
            "hspace" => SpacingType::Hspace,
            "vfill" => SpacingType::Vfill,
            "hfill" => SpacingType::Hfill,
            "newpage" => SpacingType::Newpage,
            "clearpage" => SpacingType::Clearpage,
            "pagebreak" => SpacingType::Pagebreak,
            "par" => SpacingType::Par,
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: format!("Unknown spacing command: {cmd_name}"),
            }),
        };
        
        // Some spacing commands take arguments
        let amount = match space_type {
            SpacingType::Vspace | SpacingType::Hspace => {
                if self.check(&TokenType::LeftBrace) {
                    let arg = self.parse_required_argument()?;
                    match arg {
                        Argument::Required(nodes) => Some(self.nodes_to_text(&nodes)),
                        _ => None,
                    }
                } else {
                    None
                }
            },
            _ => None,
        };
        
        Ok(Node::Spacing { space_type, amount })
    }
    
    /// Parse mathematical command
    fn parse_math_command(&mut self) -> LaTeXResult<Node> {
        let cmd_name = match &self.tokens[self.current - 1].token_type {
            TokenType::Command(name) => name.clone(),
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: "Expected math command".to_string(),
            }),
        };
        
        let mut args = Vec::new();
        
        // Parse arguments based on command type
        match cmd_name.as_str() {
            "frac" => {
                // \frac{numerator}{denominator}
                args.push(self.parse_required_argument()?);
                args.push(self.parse_required_argument()?);
            }
            "sqrt" => {
                // \sqrt[n]{content} or \sqrt{content}
                if self.check(&TokenType::LeftBracket) {
                    args.push(self.parse_optional_argument()?);
                }
                args.push(self.parse_required_argument()?);
            }
            "sum" | "int" | "prod" | "lim" => {
                // These can have subscripts and superscripts, but we'll parse them as regular commands
                // The actual subscript/superscript handling would be done in math mode parsing
            }
            "displaystyle" | "textstyle" => {
                // These are style commands that don't take arguments
            }
            _ => {}
        }
        
        Ok(Node::Command {
            name: cmd_name,
            args,
            content: None,
        })
    }
    
    /// Parse list environment (itemize, enumerate, description)
    fn parse_list_environment(&mut self, env_name: &str) -> LaTeXResult<Node> {
        let list_type = match env_name {
            "itemize" => ListType::Itemize,
            "enumerate" => ListType::Enumerate,
            "description" => ListType::Description,
            _ => return Err(LaTeXError::ParserError {
                position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                message: format!("Unknown list environment: {env_name}"),
            }),
        };
        
        let mut items = Vec::new();
        let mut current_item_content = Vec::new();
        let mut current_item_label = None;
        let mut in_item = false;
        
        // Parse content until \end{env_name}
        while !self.is_at_end() {
            if self.check_command("end") {
                let saved_pos = self.current;
                self.advance(); // consume \end
                let end_arg = self.parse_required_argument()?;
                let end_name = match end_arg {
                    Argument::Required(nodes) => self.nodes_to_text(&nodes),
                    _ => String::new(),
                };
                
                if end_name == env_name {
                    // Finish the current item if we have one
                    if in_item {
                        items.push(ListItem {
                            label: current_item_label,
                            content: current_item_content,
                        });
                    }
                    break;
                } else {
                    // Not the end of this environment, restore position
                    self.current = saved_pos;
                }
            } else if self.check_command("item") {
                // Start new item
                if in_item {
                    // Finish previous item
                    items.push(ListItem {
                        label: current_item_label,
                        content: current_item_content,
                    });
                }
                
                self.advance(); // consume \item
                
                // Parse optional label for description lists
                current_item_label = if list_type == ListType::Description && self.check(&TokenType::LeftBracket) {
                    let label_arg = self.parse_optional_argument()?;
                    match label_arg {
                        Argument::Optional(nodes) => Some(self.nodes_to_text(&nodes)),
                        _ => None,
                    }
                } else {
                    None
                };
                
                current_item_content = Vec::new();
                in_item = true;
            } else {
                // Add content to current item
                if in_item {
                    current_item_content.push(self.parse_node()?);
                } else {
                    // Content before first \item, skip it
                    self.advance();
                }
            }
        }
        
        Ok(Node::List { list_type, items })
    }
    
    /// Parse item command
     fn parse_item(&mut self) -> LaTeXResult<Node> {
         // Parse optional label
         let mut args = Vec::new();
         if self.check(&TokenType::LeftBracket) {
             args.push(self.parse_optional_argument()?);
         }
         
         Ok(Node::Command {
             name: "item".to_string(),
             args,
             content: None,
         })
     }
     
     /// Parse command definition (\newcommand, \renewcommand)
     fn parse_command_definition(&mut self) -> LaTeXResult<Node> {
         // Parse command name
         let name_arg = self.parse_required_argument()?;
         let name = match name_arg {
             Argument::Required(nodes) => {
                 if let Some(node) = nodes.first() {
                     match node {
                         Node::Text(text) => {
                             // Remove leading backslash if present
                             text.strip_prefix('\\').unwrap_or(text).to_string()
                         }
                         Node::Command { name, .. } => {
                             // Command node already has the name without backslash
                             name.clone()
                         }
                         _ => {
                             return Err(LaTeXError::ParserError {
                                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                                 message: "Command name must be text or command".to_string(),
                             });
                         }
                     }
                 } else {
                     return Err(LaTeXError::ParserError {
                         position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                         message: "Command name must be text or command".to_string(),
                     });
                 }
             }
             _ => return Err(LaTeXError::ParserError {
                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                 message: "Command definition requires name argument".to_string(),
             }),
         };
         
         // Parse optional number of arguments
         let num_args = if self.check(&TokenType::LeftBracket) {
             let arg = self.parse_optional_argument()?;
             match arg {
                 Argument::Optional(nodes) => {
                     if let Some(Node::Text(text)) = nodes.first() {
                         text.parse::<usize>().ok()
                     } else {
                         None
                     }
                 }
                 _ => None,
             }
         } else {
             None
         };
         
         // Parse optional default arguments (for optional parameters)
         let mut default_args = Vec::new();
         if self.check(&TokenType::LeftBracket) {
             let arg = self.parse_optional_argument()?;
             if let Argument::Optional(nodes) = arg {
                 if let Some(Node::Text(text)) = nodes.first() {
                     default_args.push(text.clone());
                 }
             }
         }
         
         // Parse definition body
         let def_arg = self.parse_required_argument()?;
         let definition = match def_arg {
             Argument::Required(nodes) => nodes,
             _ => return Err(LaTeXError::ParserError {
                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                 message: "Command definition requires body argument".to_string(),
             }),
         };
         
         // Store the custom command definition
         self.custom_commands.insert(name.clone(), (num_args, default_args.clone(), definition.clone()));
         
         Ok(Node::CommandDefinition {
            name,
            num_args: None,
            default_args: Vec::new(),
            definition,
        })
     }
     
     /// Parse \def command
     fn parse_def_command(&mut self) -> LaTeXResult<Node> {
         // \def\commandname{definition}
         // For simplicity, we'll parse this as a basic command definition
         
         // Expect a command token next
         if self.current >= self.tokens.len() {
             return Err(LaTeXError::ParserError {
                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                 message: "Expected command name after \\def".to_string(),
             });
         }
         
         let name = match &self.tokens[self.current].token_type {
             TokenType::Command(cmd_name) => {
                 let name = cmd_name.clone();
                 self.advance();
                 name
             }
             _ => return Err(LaTeXError::ParserError {
                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                 message: "\\def requires a command name".to_string(),
             }),
         };
         
         // Parse definition body
         let def_arg = self.parse_required_argument()?;
         let definition = match def_arg {
             Argument::Required(nodes) => nodes,
             _ => return Err(LaTeXError::ParserError {
                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                 message: "\\def requires definition body".to_string(),
             }),
         };
         
         Ok(Node::CommandDefinition {
             name,
             num_args: None,
             default_args: Vec::new(),
             definition,
         })
     }
     
     /// Parse citation command
     fn parse_citation(&mut self) -> LaTeXResult<Node> {
         let command_name = if let Some(Token { token_type: TokenType::Command(name), .. }) = self.tokens.get(self.current - 1) {
             name.clone()
         } else {
             return Err(LaTeXError::ParserError {
                 position: if self.current < self.tokens.len() { self.tokens[self.current].position } else { crate::error::Position::new(0, 0, self.current) },
                 message: "Expected citation command".to_string(),
             });
         };
         
         let cite_type = match command_name.as_str() {
             "cite" => CitationType::Cite,
             "citep" => CitationType::Citep,
             "citet" => CitationType::Citet,
             "citeauthor" => CitationType::Citeauthor,
             "citeyear" => CitationType::Citeyear,
             "nocite" => CitationType::Nocite,
             _ => CitationType::Cite,
         };
         
         // Parse optional prenote
         let prenote = if self.check(&TokenType::LeftBracket) {
             self.advance(); // consume '['
             let mut prenote_text = String::new();
             while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
                 if let Token { token_type: TokenType::Text(text), .. } = self.advance() {
                     prenote_text.push_str(text);
                 }
             }
             self.consume(&TokenType::RightBracket, "Expected ']' after prenote".to_string())?;
             Some(prenote_text)
         } else {
             None
         };
         
         // Parse optional postnote
         let postnote = if self.check(&TokenType::LeftBracket) {
             self.advance(); // consume '['
             let mut postnote_text = String::new();
             while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
                 if let Token { token_type: TokenType::Text(text), .. } = self.advance() {
                     postnote_text.push_str(text);
                 }
             }
             self.consume(&TokenType::RightBracket, "Expected ']' after postnote".to_string())?;
             Some(postnote_text)
         } else {
             None
         };
         
         // Parse required citation keys
         let keys_arg = self.parse_required_argument()?;
         let keys = if let Argument::Required(nodes) = keys_arg {
             let keys_text = self.nodes_to_text(&nodes);
             keys_text.split(',').map(|s| s.trim().to_string()).collect()
         } else {
             vec![]
         };
         
         Ok(Node::Citation {
             cite_type,
             keys,
             prenote,
             postnote,
         })
     }
     
     /// Parse bibliography style command
     fn parse_bibliography_style(&mut self) -> LaTeXResult<Node> {
         let style_arg = self.parse_required_argument()?;
         let style = if let Argument::Required(nodes) = style_arg {
             self.nodes_to_text(&nodes)
         } else {
             String::new()
         };
         
         // Store bibliography style for later use
         Ok(Node::Bibliography {
             style: Some(style),
             entries: Vec::new(),
         })
     }
     
     /// Parse bibliography command
    fn parse_bibliography(&mut self) -> LaTeXResult<Node> {
        let bib_arg = self.parse_required_argument()?;
        let bib_files = if let Argument::Required(nodes) = bib_arg {
            self.nodes_to_text(&nodes)
        } else {
            String::new()
        };
        
        // Parse bibliography files
        let mut entries = Vec::new();
        
        // Split multiple bibliography files by comma
        for bib_file in bib_files.split(',') {
            let bib_file = bib_file.trim();
            if !bib_file.is_empty() {
                // Add .bib extension if not present
                let bib_path = if bib_file.ends_with(".bib") {
                    bib_file.to_string()
                } else {
                    format!("{bib_file}.bib")
                };
                
                // Try to parse the BibTeX file
                match BibTeXParser::parse_file(&bib_path) {
                    Ok(bib_entries) => {
                        for bib_entry in bib_entries {
                            entries.push(self.convert_bib_entry_to_node(bib_entry));
                        }
                    }
                    Err(_) => {
                        // If file not found, create a placeholder entry
                        entries.push(Node::BibliographyEntry {
                            key: "missing".to_string(),
                            entry_type: "misc".to_string(),
                            fields: {
                                let mut fields = std::collections::HashMap::new();
                                fields.insert("note".to_string(), format!("Bibliography file '{bib_path}' not found"));
                                fields
                            },
                        });
                    }
                }
            }
        }
        
        Ok(Node::Bibliography {
            style: None,
            entries,
        })
    }
    
    /// Convert BibEntry to Node::BibliographyEntry
    fn convert_bib_entry_to_node(&self, bib_entry: BibEntry) -> Node {
        Node::BibliographyEntry {
            key: bib_entry.key,
            entry_type: format!("{:?}", bib_entry.entry_type).to_lowercase(),
            fields: bib_entry.fields,
        }
    }
     
     /// Parse usepackage command
     fn parse_usepackage(&mut self) -> LaTeXResult<Node> {
         // Parse optional package options
         let options = if self.check(&TokenType::LeftBracket) {
             self.advance(); // consume '['
             let mut opts = Vec::new();
             
             while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
                 if let TokenType::Text(text) = &self.advance().token_type {
                     opts.push(text.clone());
                 }
                 // Skip commas and whitespace
                 if self.check(&TokenType::Text(",".to_string())) {
                     self.advance();
                 }
             }
             
             if !self.is_at_end() {
                 self.advance(); // consume ']'
             }
             opts
         } else {
             Vec::new()
         };
         
         // Parse required package name
         let package_arg = self.parse_required_argument()?;
         let package_name = if let Argument::Required(nodes) = package_arg {
             self.nodes_to_text(&nodes)
         } else {
             return Err(LaTeXError::InvalidSyntax {
                  message: "\\usepackage requires package name".to_string(),
              });
         };
         
         Ok(Node::Package {
             name: package_name,
             options,
         })
     }
     
     fn parse_code_environment(&mut self, env_name: &str) -> LaTeXResult<Node> {
         let mut language = None;
         let mut line_numbers = false;
         let mut caption = None;
         let mut label = None;
         
         // Parse optional arguments for lstlisting and minted
         if matches!(env_name, "lstlisting" | "minted") {
             // For minted, the first argument is the language
             if env_name == "minted" {
                 let lang_arg = self.parse_required_argument()?;
                 language = Some(match lang_arg {
                     Argument::Required(nodes) => self.nodes_to_text(&nodes),
                     _ => "text".to_string(),
                 });
             }
             
             // Parse optional parameters
             if self.check(&TokenType::LeftBracket) {
                 self.advance(); // consume [
                 let mut param_text = String::new();
                 
                 while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
                     let token = self.advance();
                     match &token.token_type {
                         TokenType::Text(text) => param_text.push_str(text),
                         TokenType::Whitespace(ws) => param_text.push_str(ws),
                         _ => {}
                     }
                 }
                 
                 self.consume(&TokenType::RightBracket, "Expected ']' after code parameters".to_string())?;
                 
                 // Parse common parameters
                 if param_text.contains("numbers=left") || param_text.contains("linenos") {
                     line_numbers = true;
                 }
                 
                 // Extract language from parameters (for lstlisting)
                 if env_name == "lstlisting" {
                     if let Some(lang_start) = param_text.find("language=") {
                         let lang_content = &param_text[lang_start + 9..];
                         if let Some(lang_end) = lang_content.find(',') {
                             language = Some(lang_content[..lang_end].trim().to_string());
                         } else {
                             language = Some(lang_content.trim().to_string());
                         }
                     }
                 }
                 
                 // Extract caption and label from parameters
                 if let Some(cap_start) = param_text.find("caption=") {
                     let cap_content = &param_text[cap_start + 8..];
                     if let Some(cap_end) = cap_content.find(',') {
                         caption = Some(cap_content[..cap_end].trim().to_string());
                     } else {
                         caption = Some(cap_content.trim().to_string());
                     }
                 }
                 
                 if let Some(label_start) = param_text.find("label=") {
                     let label_content = &param_text[label_start + 6..];
                     if let Some(label_end) = label_content.find(',') {
                         label = Some(label_content[..label_end].trim().to_string());
                     } else {
                         label = Some(label_content.trim().to_string());
                     }
                 }
             }
         }
         
         // Collect raw code content until \\end{env_name}
         let mut code_content = String::new();
         
         while !self.is_at_end() {
             if self.check_command("end") {
                 // Look ahead to see if this is our end command
                 let saved_pos = self.current;
                 self.advance(); // consume \\end
                 
                 if let Ok(end_arg) = self.parse_required_argument() {
                     let end_name = match end_arg {
                         Argument::Required(nodes) => self.nodes_to_text(&nodes),
                         _ => String::new(),
                     };
                     
                     if end_name == env_name {
                         break; // Found matching end
                     }
                 }
                 
                 // Not our end command, restore position and continue
                 self.current = saved_pos;
             }
             
             let token = self.advance();
             match &token.token_type {
                 TokenType::Text(text) => code_content.push_str(text),
                 TokenType::Whitespace(ws) => code_content.push_str(ws),
                 TokenType::Newline => code_content.push('\n'),
                 TokenType::Command(cmd) => {
                     code_content.push('\\');
                     code_content.push_str(cmd);
                 }
                 TokenType::LeftBrace => code_content.push('{'),
                 TokenType::RightBrace => code_content.push('}'),
                 TokenType::LeftBracket => code_content.push('['),
                 TokenType::RightBracket => code_content.push(']'),
                 _ => {}
             }
         }
         
         let style = match env_name {
             "verbatim" | "verbatim*" => CodeStyle::Verbatim,
             "lstlisting" => CodeStyle::Listings,
             "minted" => CodeStyle::Minted,
             _ => CodeStyle::Verbatim,
         };
         
         Ok(Node::CodeBlock {
             language,
             code: code_content,
             style,
             line_numbers,
             caption,
             label,
         })
     }
     
     /// Validate that all references have corresponding labels
     fn validate_references(&self) -> LaTeXResult<()> {
         for (ref_label, token_pos) in &self.references {
             if !self.labels.contains_key(ref_label) {
                 eprintln!("Warning: Undefined reference '{ref_label}' at token {token_pos}");
                 // Could return an error instead of warning:
                 // return Err(LaTeXError::ParserError {
                 //     token_index: *token_pos,
                 //     message: format!("Undefined reference: {}", ref_label),
                 // });
             }
         }
         Ok(())
     }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}