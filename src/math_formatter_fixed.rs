pub struct MathFormatter;

impl MathFormatter {
    pub fn format(math: &str) -> String {
        let mut result = math.to_string();
        
        // Apply formatting in correct order
        result = Self::format_sqrt(&result);
        result = Self::format_fractions(&result);
        result = Self::format_superscripts(&result);
        result = Self::format_subscripts(&result);
        
        // Symbol replacements (rest of the existing code)
        // ... keeping all existing symbol replacements ...
        
        result
    }
    
    // Keep all existing helper methods...
}
