pub struct MathFormatter;

impl MathFormatter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn format(math: &str) -> String {
        let mut result = math.to_string();
        
        result = Self::format_sqrt(&result);
        result = Self::format_fractions(&result);
        result = Self::format_superscripts(&result);
        result = Self::format_subscripts(&result);
        
        result = result.replace("\\int", "∫");
        result = result.replace("\\sum", "∑");
        result = result.replace("\\alpha", "α");
        result = result.replace("\\beta", "β");
        result = result.replace("\\gamma", "γ");
        
        result
    }
    
    fn format_sqrt(text: &str) -> String {
        text.replace("\\sqrt", "√")
    }
    
    fn format_fractions(text: &str) -> String {
        text.to_string()
    }
    
    fn format_superscripts(text: &str) -> String {
        text.to_string()
    }
    
    fn format_subscripts(text: &str) -> String {
        text.to_string()
    }
}

impl Default for MathFormatter {
    fn default() -> Self {
        Self::new()
    }
}
