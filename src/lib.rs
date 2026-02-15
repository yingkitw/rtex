use std::path::Path;
use std::fs;
use thiserror::Error;

mod parser;
mod pdf_builder;
mod math_formatter;
pub mod error;
pub mod config;
pub mod traits;

use parser::TexParser;
use pdf_builder::PdfBuilder;

#[derive(Error, Debug)]
pub enum TexError {
    #[error("Failed to read input file: {0}")]
    ReadError(#[from] std::io::Error),
    
    #[error("LaTeX compilation failed: {0}")]
    CompilationError(String),
    
    #[error("PDF generation failed: {0}")]
    PdfGenerationError(String),
    
    #[error("Invalid file path")]
    InvalidPath,
}

pub trait TexConverter {
    fn convert(&self, input: &Path, output: &Path) -> Result<(), TexError>;
}

pub struct NativeTexConverter;

impl NativeTexConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeTexConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TexConverter for NativeTexConverter {
    fn convert(&self, input: &Path, output: &Path) -> Result<(), TexError> {
        if !input.exists() {
            return Err(TexError::InvalidPath);
        }

        let content = fs::read_to_string(input)?;
        
        let mut parser = TexParser::new(content);
        let elements = parser.parse();
        
        let mut builder = PdfBuilder::new();
        builder.build(elements, output)
            .map_err(|e| TexError::PdfGenerationError(e))?;

        Ok(())
    }
}

pub fn convert_tex_to_pdf(input: &Path, output: &Path) -> Result<(), TexError> {
    let converter = NativeTexConverter::new();
    converter.convert(input, output)
}

#[cfg(test)]
mod tests;
