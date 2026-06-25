use latex_rust::math::*;
use latex_rust::ast::{Node, MathEnvironmentType};

#[cfg(test)]
mod math_unit_tests {
    use super::*;

    #[test]
    fn test_math_processor_creation() {
        let processor = MathProcessor::new();
        assert!(processor.is_math_command("frac"));
        assert!(processor.is_math_command("sqrt"));
        assert!(!processor.is_math_command("invalid"));
    }

    #[test]
    fn test_math_processor_default() {
        let processor = MathProcessor::default();
        assert!(processor.is_math_command("sum"));
        assert!(processor.is_math_command("int"));
    }

    #[test]
    fn test_equation_numbering() {
        let mut processor = MathProcessor::new();
        let first_number = processor.next_equation_number();
        let second_number = processor.next_equation_number();
        assert_eq!(second_number, first_number + 1);
    }

    #[test]
    fn test_equation_labels() {
        let mut processor = MathProcessor::new();
        let number = processor.next_equation_number();
        processor.add_equation_label("eq1".to_string(), number);
        assert_eq!(processor.get_equation_number("eq1"), Some(number));
        assert_eq!(processor.get_equation_number("nonexistent"), None);
    }

    #[test]
    fn test_math_command_recognition() {
        let processor = MathProcessor::new();
        assert!(processor.is_math_command("frac"));
        assert!(processor.is_math_command("sqrt"));
        assert!(processor.is_math_command("sum"));
        assert!(!processor.is_math_command("invalid_command"));
    }

    #[test]
    fn test_math_command_type_variants() {
        let command_types = vec![
            MathCommandType::Function,
            MathCommandType::Operator,
            MathCommandType::Symbol,
            MathCommandType::Delimiter,
            MathCommandType::Accent,
            MathCommandType::Fraction,
            MathCommandType::Root,
            MathCommandType::Matrix,
            MathCommandType::Style,
        ];
        
        for cmd_type in command_types {
            let debug_str = format!("{:?}", cmd_type);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_matrix_type_variants() {
        let matrix_types = vec![
            MatrixType::Plain,
            MatrixType::Parentheses,
            MatrixType::Brackets,
            MatrixType::Braces,
            MatrixType::Pipes,
            MatrixType::DoublePipes,
            MatrixType::Small,
        ];
        
        for matrix_type in matrix_types {
            let debug_str = format!("{:?}", matrix_type);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_supported_environments() {
        let environments = MathProcessor::get_supported_environments();
        assert!(environments.contains(&"equation"));
        assert!(environments.contains(&"align"));
        assert!(environments.contains(&"matrix"));
    }

    #[test]
    fn test_environment_numbering() {
        assert!(MathProcessor::should_number_environment("equation"));
        assert!(MathProcessor::should_number_environment("align"));
        assert!(!MathProcessor::should_number_environment("equation*"));
        assert!(!MathProcessor::should_number_environment("align*"));
    }

    #[test]
    fn test_matrix_type_detection() {
        assert_eq!(MathProcessor::get_matrix_type("pmatrix"), Some(MatrixType::Parentheses));
        assert_eq!(MathProcessor::get_matrix_type("bmatrix"), Some(MatrixType::Brackets));
        assert_eq!(MathProcessor::get_matrix_type("vmatrix"), Some(MatrixType::Pipes));
        assert_eq!(MathProcessor::get_matrix_type("invalid"), None);
    }

    #[test]
    fn test_math_utils_operators() {
        assert!(MathUtils::is_math_operator('+'));
        assert!(MathUtils::is_math_operator('-'));
        assert!(MathUtils::is_math_operator('*'));
        assert!(MathUtils::is_math_operator('/'));
        assert!(!MathUtils::is_math_operator('a'));
    }

    #[test]
    fn test_greek_letter_recognition() {
        assert!(MathUtils::is_greek_letter_start("alpha"));
        assert!(MathUtils::is_greek_letter_start("beta"));
        assert!(MathUtils::is_greek_letter_start("gamma"));
        assert!(!MathUtils::is_greek_letter_start("invalid"));
    }

    #[test]
    fn test_greek_symbol_lookup() {
        assert_eq!(MathUtils::get_greek_symbol("alpha"), Some("α"));
        assert_eq!(MathUtils::get_greek_symbol("beta"), Some("β"));
        assert_eq!(MathUtils::get_greek_symbol("gamma"), Some("γ"));
        assert_eq!(MathUtils::get_greek_symbol("invalid"), None);
    }

    #[test]
    fn test_math_node_variants() {
        let text_node = MathNode::Text("x".to_string());
        let command_node = MathNode::Command {
            name: "frac".to_string(),
            args: vec![],
            command_type: MathCommandType::Function,
        };
        
        match text_node {
            MathNode::Text(content) => assert_eq!(content, "x"),
            _ => panic!("Expected Text node"),
        }
        
        match command_node {
            MathNode::Command { name, .. } => assert_eq!(name, "frac"),
            _ => panic!("Expected Command node"),
        }
    }

    #[test]
    fn test_math_argument_variants() {
        let required_arg = MathArgument::Required(vec![MathNode::Text("x".to_string())]);
        let optional_arg = MathArgument::Optional(vec![MathNode::Text("y".to_string())]);
        
        match required_arg {
            MathArgument::Required(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    MathNode::Text(content) => assert_eq!(content, "x"),
                    _ => panic!("Expected Text node"),
                }
            },
            _ => panic!("Expected Required argument"),
        }
        
        match optional_arg {
            MathArgument::Optional(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    MathNode::Text(content) => assert_eq!(content, "y"),
                    _ => panic!("Expected Text node"),
                }
            },
            _ => panic!("Expected Optional argument"),
        }
    }

    #[test]
    fn test_create_math_environment() {
        let mut processor = MathProcessor::new();
        let content = vec![Node::Text("x = 1".to_string())];
        let env_node = processor.create_math_environment(
            MathEnvironmentType::Equation,
            content,
            Some("eq1".to_string()),
            true
        );
        
        match env_node {
            Node::MathEnvironment { env_type, label, numbered, .. } => {
                assert!(matches!(env_type, MathEnvironmentType::Equation));
                assert_eq!(label, Some("eq1".to_string()));
                assert_eq!(numbered, true);
            },
            _ => panic!("Expected MathEnvironment node"),
        }
    }

    #[test]
    fn test_math_environment_type_variants() {
        let env_types = vec![
            MathEnvironmentType::Equation,
            MathEnvironmentType::Align,
            MathEnvironmentType::Gather,
            MathEnvironmentType::Matrix,
            MathEnvironmentType::Pmatrix,
            MathEnvironmentType::Bmatrix,
            MathEnvironmentType::Vmatrix,
            MathEnvironmentType::Smallmatrix,
            MathEnvironmentType::Cases,
            MathEnvironmentType::Split,
            MathEnvironmentType::Aligned,
            MathEnvironmentType::Eqnarray,
        ];
        
        for env_type in env_types {
            let debug_str = format!("{:?}", env_type);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_parse_math_expression() {
        let processor = MathProcessor::new();
        let result = processor.parse_math_expression("x + y");
        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_reset_equation_counter() {
        let mut processor = MathProcessor::new();
        processor.next_equation_number();
        processor.next_equation_number();
        processor.reset_equation_counter();
        let next_number = processor.next_equation_number();
        assert_eq!(next_number, 1);
    }
}