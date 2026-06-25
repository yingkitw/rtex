//! Advanced mathematical processing for LaTeX documents

use crate::ast::*;
use crate::error::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Math processor for handling advanced mathematical features
#[derive(Debug, Clone)]
pub struct MathProcessor {
    equation_counter: usize,
    equation_labels: HashMap<String, usize>,
    math_commands: HashMap<String, MathCommandHandler>,
}

/// Handler for mathematical commands
#[derive(Debug, Clone)]
pub struct MathCommandHandler {
    pub name: String,
    pub args_count: usize,
    pub optional_args: usize,
    pub handler_type: MathCommandType,
}

/// Types of mathematical commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathCommandType {
    Fraction,
    Root,
    Operator,
    Symbol,
    Accent,
    Delimiter,
    Matrix,
    Function,
    Style,
}

/// Advanced mathematical node for complex expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathNode {
    /// Simple text content
    Text(String),
    
    /// Mathematical command with arguments
    Command {
        name: String,
        args: Vec<MathArgument>,
        command_type: MathCommandType,
    },
    
    /// Superscript
    Superscript {
        base: Box<MathNode>,
        exponent: Box<MathNode>,
    },
    
    /// Subscript
    Subscript {
        base: Box<MathNode>,
        index: Box<MathNode>,
    },
    
    /// Combined sub and superscript
    SubSuperscript {
        base: Box<MathNode>,
        subscript: Box<MathNode>,
        superscript: Box<MathNode>,
    },
    
    /// Fraction
    Fraction {
        numerator: Box<MathNode>,
        denominator: Box<MathNode>,
    },
    
    /// Root (square root or nth root)
    Root {
        index: Option<Box<MathNode>>,
        radicand: Box<MathNode>,
    },
    
    /// Matrix
    Matrix {
        matrix_type: MatrixType,
        rows: Vec<Vec<MathNode>>,
    },
    
    /// Delimiter pair
    Delimited {
        left: String,
        right: String,
        content: Box<MathNode>,
    },
    
    /// Group of math nodes
    Group(Vec<MathNode>),
}

/// Mathematical argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathArgument {
    Required(Vec<MathNode>),
    Optional(Vec<MathNode>),
}

/// Matrix types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatrixType {
    Plain,      // matrix
    Parentheses, // pmatrix
    Brackets,   // bmatrix
    Braces,     // Bmatrix
    Pipes,      // vmatrix
    DoublePipes, // Vmatrix
    Small,      // smallmatrix
}

impl MathProcessor {
    /// Create a new math processor
    pub fn new() -> Self {
        let mut processor = Self {
            equation_counter: 0,
            equation_labels: HashMap::new(),
            math_commands: HashMap::new(),
        };
        processor.initialize_commands();
        processor
    }
    
    /// Initialize built-in math commands
    fn initialize_commands(&mut self) {
        self.initialize_fraction_commands();
        self.initialize_root_commands();
        self.initialize_operator_commands();
        self.initialize_function_commands();
        self.initialize_accent_commands();
        self.initialize_style_commands();
        self.initialize_matrix_commands();
    }
    
    /// Initialize fraction commands
    fn initialize_fraction_commands(&mut self) {
        self.add_command("frac", 2, 0, MathCommandType::Fraction);
        self.add_command("dfrac", 2, 0, MathCommandType::Fraction);
        self.add_command("tfrac", 2, 0, MathCommandType::Fraction);
        self.add_command("cfrac", 2, 0, MathCommandType::Fraction);
    }
    
    /// Initialize root commands
    fn initialize_root_commands(&mut self) {
        self.add_command("sqrt", 1, 1, MathCommandType::Root);
    }
    
    /// Initialize operator commands
    fn initialize_operator_commands(&mut self) {
        let operators = [
            "sum", "prod", "int", "oint", "iint", "iiint",
            "lim", "limsup", "liminf", "max", "min", "sup", "inf"
        ];
        for op in &operators {
            self.add_command(op, 0, 0, MathCommandType::Operator);
        }
    }
    
    /// Initialize function commands
    fn initialize_function_commands(&mut self) {
        let functions = [
            "sin", "cos", "tan", "sec", "csc", "cot",
            "sinh", "cosh", "tanh", "ln", "log", "exp",
            "det", "gcd", "lcm"
        ];
        for func in &functions {
            self.add_command(func, 0, 0, MathCommandType::Function);
        }
    }
    
    /// Initialize accent commands
    fn initialize_accent_commands(&mut self) {
        let accents = [
            ("hat", 1), ("widehat", 1), ("tilde", 1), ("widetilde", 1),
            ("bar", 1), ("overline", 1), ("underline", 1),
            ("vec", 1), ("dot", 1), ("ddot", 1)
        ];
        for (accent, args) in &accents {
            self.add_command(accent, *args, 0, MathCommandType::Accent);
        }
    }
    
    /// Initialize style commands
    fn initialize_style_commands(&mut self) {
        let styles = ["displaystyle", "textstyle", "scriptstyle", "scriptscriptstyle"];
        for style in &styles {
            self.add_command(style, 0, 0, MathCommandType::Style);
        }
    }
    
    /// Initialize matrix commands
    fn initialize_matrix_commands(&mut self) {
        let matrices = ["matrix", "pmatrix", "bmatrix", "Bmatrix", "vmatrix", "Vmatrix", "smallmatrix"];
        for matrix in &matrices {
            self.add_command(matrix, 0, 0, MathCommandType::Matrix);
        }
    }
    
    /// Add a math command handler
    fn add_command(&mut self, name: &str, args_count: usize, optional_args: usize, command_type: MathCommandType) {
        self.math_commands.insert(name.to_string(), MathCommandHandler {
            name: name.to_string(),
            args_count,
            optional_args,
            handler_type: command_type,
        });
    }
    
    /// Get next equation number
    pub fn next_equation_number(&mut self) -> usize {
        self.equation_counter += 1;
        self.equation_counter
    }
    
    /// Add equation label
    pub fn add_equation_label(&mut self, label: String, number: usize) {
        self.equation_labels.insert(label, number);
    }
    
    /// Get equation number by label
    pub fn get_equation_number(&self, label: &str) -> Option<usize> {
        self.equation_labels.get(label).copied()
    }
    
    /// Reset equation counter
    pub fn reset_equation_counter(&mut self) {
        self.equation_counter = 0;
        self.equation_labels.clear();
    }
    
    /// Check if command is a math command
    pub fn is_math_command(&self, name: &str) -> bool {
        self.math_commands.contains_key(name)
    }
    
    /// Get math command handler
    pub fn get_math_command(&self, name: &str) -> Option<&MathCommandHandler> {
        self.math_commands.get(name)
    }
    
    /// Parse advanced math expression
    pub fn parse_math_expression(&self, content: &str) -> Result<Vec<MathNode>, LaTeXError> {
        // This is a simplified parser - in a full implementation,
        // this would be a complete math expression parser
        
        // For now, just create a text node
        // TODO: Implement full math expression parsing
        let nodes = vec![MathNode::Text(content.to_string())];
        
        Ok(nodes)
    }
    
    /// Create math environment with equation numbering
    pub fn create_math_environment(
        &mut self,
        env_type: MathEnvironmentType,
        content: Vec<Node>,
        label: Option<String>,
        numbered: bool,
    ) -> Node {
        let equation_number = if numbered {
            let num = self.next_equation_number();
            if let Some(ref label_str) = label {
                self.add_equation_label(label_str.clone(), num);
            }
            Some(num)
        } else {
            None
        };
        
        Node::MathEnvironment {
            env_type,
            content,
            label,
            numbered,
            equation_number,
        }
    }
    
    /// Get supported math environments
    pub fn get_supported_environments() -> Vec<&'static str> {
        vec![
            "equation", "equation*",
            "align", "align*",
            "gather", "gather*",
            "multline", "multline*",
            "eqnarray", "eqnarray*",
            "split", "aligned",
            "cases",
            "matrix", "pmatrix", "bmatrix", "Bmatrix", "vmatrix", "Vmatrix", "smallmatrix",
            "array",
        ]
    }
    
    /// Check if environment should be numbered
    pub fn should_number_environment(env_name: &str) -> bool {
        !env_name.ends_with('*')
    }
    
    /// Get matrix type from environment name
    pub fn get_matrix_type(env_name: &str) -> Option<MatrixType> {
        match env_name {
            "matrix" => Some(MatrixType::Plain),
            "pmatrix" => Some(MatrixType::Parentheses),
            "bmatrix" => Some(MatrixType::Brackets),
            "Bmatrix" => Some(MatrixType::Braces),
            "vmatrix" => Some(MatrixType::Pipes),
            "Vmatrix" => Some(MatrixType::DoublePipes),
            "smallmatrix" => Some(MatrixType::Small),
            _ => None,
        }
    }
}

impl Default for MathProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Math utilities
pub struct MathUtils;

impl MathUtils {
    /// Check if character is a math operator
    pub fn is_math_operator(c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '≤' | '≥' | '≠' | '±' | '∓' | '×' | '÷' | '∝' | '∞' | '∂' | '∇' | '∆' | '∑' | '∏' | '∫' | '∮' | '√' | '∛' | '∜')
    }
    
    /// Check if string starts with a Greek letter name
    pub fn is_greek_letter_start(s: &str) -> bool {
        let greek_letters = [
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
            "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi",
            "rho", "sigma", "tau", "upsilon", "phi", "chi", "psi", "omega",
            "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
            "Iota", "Kappa", "Lambda", "Mu", "Nu", "Xi", "Omicron", "Pi",
            "Rho", "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega"
        ];
        greek_letters.iter().any(|&letter| s.starts_with(letter))
    }
    
    /// Get Greek symbol for name
    pub fn get_greek_symbol(name: &str) -> Option<&'static str> {
        match name {
            "alpha" => Some("α"),
            "beta" => Some("β"),
            "gamma" => Some("γ"),
            "delta" => Some("δ"),
            "epsilon" => Some("ε"),
            "zeta" => Some("ζ"),
            "eta" => Some("η"),
            "theta" => Some("θ"),
            "iota" => Some("ι"),
            "kappa" => Some("κ"),
            "lambda" => Some("λ"),
            "mu" => Some("μ"),
            "nu" => Some("ν"),
            "xi" => Some("ξ"),
            "omicron" => Some("ο"),
            "pi" => Some("π"),
            "rho" => Some("ρ"),
            "sigma" => Some("σ"),
            "tau" => Some("τ"),
            "upsilon" => Some("υ"),
            "phi" => Some("φ"),
            "chi" => Some("χ"),
            "psi" => Some("ψ"),
            "omega" => Some("ω"),
            "Alpha" => Some("Α"),
            "Beta" => Some("Β"),
            "Gamma" => Some("Γ"),
            "Delta" => Some("Δ"),
            "Epsilon" => Some("Ε"),
            "Zeta" => Some("Ζ"),
            "Eta" => Some("Η"),
            "Theta" => Some("Θ"),
            "Iota" => Some("Ι"),
            "Kappa" => Some("Κ"),
            "Lambda" => Some("Λ"),
            "Mu" => Some("Μ"),
            "Nu" => Some("Ν"),
            "Xi" => Some("Ξ"),
            "Omicron" => Some("Ο"),
            "Pi" => Some("Π"),
            "Rho" => Some("Ρ"),
            "Sigma" => Some("Σ"),
            "Tau" => Some("Τ"),
            "Upsilon" => Some("Υ"),
            "Phi" => Some("Φ"),
            "Chi" => Some("Χ"),
            "Psi" => Some("Ψ"),
            "Omega" => Some("Ω"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_math_processor_creation() {
        let processor = MathProcessor::new();
        assert_eq!(processor.equation_counter, 0);
        assert!(processor.is_math_command("frac"));
        assert!(processor.is_math_command("sqrt"));
        assert!(!processor.is_math_command("unknown"));
    }
    
    #[test]
    fn test_equation_numbering() {
        let mut processor = MathProcessor::new();
        
        let num1 = processor.next_equation_number();
        let num2 = processor.next_equation_number();
        
        assert_eq!(num1, 1);
        assert_eq!(num2, 2);
        
        processor.add_equation_label("eq:test".to_string(), num1);
        assert_eq!(processor.get_equation_number("eq:test"), Some(1));
    }
    
    #[test]
    fn test_environment_numbering() {
        assert!(MathProcessor::should_number_environment("equation"));
        assert!(!MathProcessor::should_number_environment("equation*"));
        assert!(MathProcessor::should_number_environment("align"));
        assert!(!MathProcessor::should_number_environment("align*"));
    }
    
    #[test]
    fn test_greek_letters() {
        assert!(MathUtils::is_greek_letter_start("alpha"));
        assert!(MathUtils::is_greek_letter_start("Beta"));
        assert!(!MathUtils::is_greek_letter_start("notgreek"));
        
        assert_eq!(MathUtils::get_greek_symbol("alpha"), Some("α"));
        assert_eq!(MathUtils::get_greek_symbol("Beta"), Some("Β"));
        assert_eq!(MathUtils::get_greek_symbol("unknown"), None);
    }
    
    #[test]
    fn test_matrix_types() {
        assert_eq!(MathProcessor::get_matrix_type("matrix"), Some(MatrixType::Plain));
        assert_eq!(MathProcessor::get_matrix_type("pmatrix"), Some(MatrixType::Parentheses));
        assert_eq!(MathProcessor::get_matrix_type("unknown"), None);
    }
}