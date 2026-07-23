//! Advanced math processing — equation numbering, command registry,
//! and environment helpers.
//!
//! Complements the existing `MathFormatter` by tracking display-math
//! equation numbers and providing a registry of standard math
//! commands (fractions, roots, operators, functions, matrices, etc.).
//!
//! Ported and simplified from `latex-rust/src/math.rs`.

use std::collections::HashMap;

/// Types of mathematical commands recognized by the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Metadata for a single math command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathCommandInfo {
    pub name: String,
    pub arg_count: usize,
    pub optional_count: usize,
    pub cmd_type: MathCommandType,
}

/// Registry of standard math commands plus an equation counter.
#[derive(Debug, Clone)]
pub struct MathProcessor {
    equation_counter: usize,
    equation_labels: HashMap<String, usize>,
    commands: HashMap<String, MathCommandInfo>,
}

impl Default for MathProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl MathProcessor {
    pub fn new() -> Self {
        let mut p = Self {
            equation_counter: 0,
            equation_labels: HashMap::new(),
            commands: HashMap::new(),
        };
        p.init_commands();
        p
    }

    // ── Equation numbering ────────────────────────────────────────

    /// Increment and return the next equation number.
    pub fn next_equation_number(&mut self) -> usize {
        self.equation_counter += 1;
        self.equation_counter
    }

    /// Current equation counter value.
    pub fn current_number(&self) -> usize {
        self.equation_counter
    }

    /// Associate `label` with `number` for cross-referencing.
    pub fn add_label(&mut self, label: String, number: usize) {
        self.equation_labels.insert(label, number);
    }

    /// Look up the equation number for `label`.
    pub fn get_label(&self, label: &str) -> Option<usize> {
        self.equation_labels.get(label).copied()
    }

    /// Reset counter and discard all labels.
    pub fn reset(&mut self) {
        self.equation_counter = 0;
        self.equation_labels.clear();
    }

    // ── Command registry ──────────────────────────────────────────

    pub fn is_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    pub fn get_command(&self, name: &str) -> Option<&MathCommandInfo> {
        self.commands.get(name)
    }

    pub fn command_type(&self, name: &str) -> Option<MathCommandType> {
        self.commands.get(name).map(|c| c.cmd_type)
    }

    pub fn arg_count(&self, name: &str) -> Option<usize> {
        self.commands.get(name).map(|c| c.arg_count)
    }

    /// All registered command names.
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    // ── Environment helpers ───────────────────────────────────────

    /// Return `true` if `env_name` should receive an equation number
    /// (i.e. it does **not** end with `'*'`).
    pub fn should_number(env_name: &str) -> bool {
        !env_name.ends_with('*')
    }

    /// Standard math environments supported by LaTeX / amsmath.
    pub fn environments() -> Vec<&'static str> {
        vec![
            "equation",
            "equation*",
            "align",
            "align*",
            "gather",
            "gather*",
            "multline",
            "multline*",
            "eqnarray",
            "eqnarray*",
            "split",
            "aligned",
            "cases",
            "matrix",
            "pmatrix",
            "bmatrix",
            "Bmatrix",
            "vmatrix",
            "Vmatrix",
            "smallmatrix",
            "array",
        ]
    }

    // ── Initialization ──────────────────────────────────────────

    fn init_commands(&mut self) {
        // Fractions
        for name in ["frac", "dfrac", "tfrac", "cfrac"] {
            self.add(name, 2, 0, MathCommandType::Fraction);
        }
        // Roots
        self.add("sqrt", 1, 1, MathCommandType::Root);
        // Big operators
        for name in [
            "sum", "prod", "int", "oint", "iint", "iiint", "lim", "limsup", "liminf", "max", "min",
            "sup", "inf",
        ] {
            self.add(name, 0, 0, MathCommandType::Operator);
        }
        // Functions
        for name in [
            "sin", "cos", "tan", "sec", "csc", "cot", "sinh", "cosh", "tanh", "ln", "log", "exp",
            "det", "gcd", "lcm",
        ] {
            self.add(name, 0, 0, MathCommandType::Function);
        }
        // Accents
        for name in [
            "hat",
            "widehat",
            "tilde",
            "widetilde",
            "bar",
            "overline",
            "underline",
            "vec",
            "dot",
            "ddot",
        ] {
            self.add(name, 1, 0, MathCommandType::Accent);
        }
        // Styles
        for name in [
            "displaystyle",
            "textstyle",
            "scriptstyle",
            "scriptscriptstyle",
        ] {
            self.add(name, 0, 0, MathCommandType::Style);
        }
        // Matrices
        for name in [
            "matrix",
            "pmatrix",
            "bmatrix",
            "Bmatrix",
            "vmatrix",
            "Vmatrix",
            "smallmatrix",
        ] {
            self.add(name, 0, 0, MathCommandType::Matrix);
        }
        // Delimiters
        for name in [
            "left", "right", "bigl", "bigr", "Bigl", "Bigr", "biggl", "biggr", "Biggl", "Biggr",
            "big", "Big", "bigg", "Bigg",
        ] {
            self.add(name, 1, 0, MathCommandType::Delimiter);
        }
        // Common symbols that are commands
        for name in [
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
            "lambda", "mu", "nu", "xi", "pi", "rho", "sigma", "tau", "upsilon", "phi", "chi",
            "psi", "omega", "Gamma", "Delta", "Theta", "Lambda", "Xi", "Pi", "Sigma", "Upsilon",
            "Phi", "Psi", "Omega", "infty", "partial", "nabla", "forall", "exists", "in", "notin",
            "subset", "supset", "subseteq", "supseteq", "cup", "cap", "emptyset", "times", "div",
            "pm", "mp", "leq", "geq", "neq", "approx", "equiv", "sim", "propto", "cdot", "ldots",
            "cdots", "vdots", "ddots", "vec", "hat",
        ] {
            self.add(name, 0, 0, MathCommandType::Symbol);
        }
    }

    fn add(&mut self, name: &str, args: usize, opt: usize, ty: MathCommandType) {
        self.commands.insert(
            name.to_string(),
            MathCommandInfo {
                name: name.to_string(),
                arg_count: args,
                optional_count: opt,
                cmd_type: ty,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equation_numbering() {
        let mut p = MathProcessor::new();
        assert_eq!(p.next_equation_number(), 1);
        assert_eq!(p.next_equation_number(), 2);
        assert_eq!(p.current_number(), 2);
    }

    #[test]
    fn equation_labels() {
        let mut p = MathProcessor::new();
        p.next_equation_number();
        p.add_label("eq:hello".to_string(), 1);
        assert_eq!(p.get_label("eq:hello"), Some(1));
        assert_eq!(p.get_label("missing"), None);
    }

    #[test]
    fn reset_clears_everything() {
        let mut p = MathProcessor::new();
        p.next_equation_number();
        p.add_label("a".to_string(), 1);
        p.reset();
        assert_eq!(p.current_number(), 0);
        assert_eq!(p.get_label("a"), None);
    }

    #[test]
    fn command_registry_fractions() {
        let p = MathProcessor::new();
        assert!(p.is_command("frac"));
        assert_eq!(p.command_type("frac"), Some(MathCommandType::Fraction));
        assert_eq!(p.arg_count("frac"), Some(2));
    }

    #[test]
    fn command_registry_symbols() {
        let p = MathProcessor::new();
        assert!(p.is_command("alpha"));
        assert!(p.is_command("Omega"));
        assert_eq!(p.command_type("alpha"), Some(MathCommandType::Symbol));
    }

    #[test]
    fn command_registry_operators() {
        let p = MathProcessor::new();
        assert!(p.is_command("sum"));
        assert!(p.is_command("int"));
        assert_eq!(p.command_type("lim"), Some(MathCommandType::Operator));
    }

    #[test]
    fn should_number_starless() {
        assert!(MathProcessor::should_number("equation"));
        assert!(!MathProcessor::should_number("equation*"));
        assert!(MathProcessor::should_number("align"));
        assert!(!MathProcessor::should_number("align*"));
    }

    #[test]
    fn environments_listed() {
        let envs = MathProcessor::environments();
        assert!(envs.contains(&"equation"));
        assert!(envs.contains(&"align*"));
        assert!(envs.contains(&"pmatrix"));
    }

    #[test]
    fn unknown_command_not_found() {
        let p = MathProcessor::new();
        assert!(!p.is_command("notamathcmd"));
        assert_eq!(p.get_command("nope"), None);
    }

    #[test]
    fn registry_is_populated() {
        let p = MathProcessor::new();
        assert!(p.len() > 50);
        assert!(!p.is_empty());
    }
}
