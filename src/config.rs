//! Configuration system for latex-rs.
//!
//! Provides quality presets and configurable options for PDF generation.
//! Supports TOML and JSON serialization via `serde`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Quality preset for PDF generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QualityPreset {
    /// Draft quality - fast generation, larger files
    Draft,
    /// Standard quality - balanced
    #[default]
    Standard,
    /// High quality - better output, slower
    High,
    /// Print quality - best output, slowest
    Print,
}


/// Font embedding options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FontEmbedding {
    /// Don't embed fonts (smallest files, may not display correctly)
    None,
    /// Embed only used characters (subset)
    Subset,
    /// Embed full fonts
    #[default]
    Full,
}


/// Configuration for latex-rs operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Quality preset.
    pub quality: QualityPreset,
    /// Font embedding strategy.
    pub font_embedding: FontEmbedding,
    /// Custom font path (optional).
    pub custom_font: Option<PathBuf>,
    /// Enable verbose output.
    pub verbose: bool,
    /// Enable debug mode.
    pub debug: bool,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Keep intermediate files.
    pub keep_intermediate: bool,
    /// Compression level (0–9, where 9 is maximum compression).
    pub compression_level: u8,
    /// Enable math symbol caching.
    pub cache_math: bool,
    /// Enable font metrics caching.
    pub cache_fonts: bool,
    /// Maximum cache size in MB.
    pub max_cache_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            quality: QualityPreset::default(),
            font_embedding: FontEmbedding::default(),
            custom_font: None,
            verbose: false,
            debug: false,
            output_dir: PathBuf::from("output"),
            keep_intermediate: false,
            compression_level: 6,
            cache_math: true,
            cache_fonts: true,
            max_cache_size: 100, // 100 MB
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create configuration with a specific quality preset
    pub fn with_quality(quality: QualityPreset) -> Self {
        let mut config = Self {
            quality,
            ..Self::default()
        };
        config.apply_quality_preset();
        config
    }
    
    /// Apply quality preset settings
    fn apply_quality_preset(&mut self) {
        match self.quality {
            QualityPreset::Draft => {
                self.compression_level = 3;
                self.font_embedding = FontEmbedding::Subset;
                self.cache_math = true;
                self.cache_fonts = true;
            }
            QualityPreset::Standard => {
                self.compression_level = 6;
                self.font_embedding = FontEmbedding::Full;
                self.cache_math = true;
                self.cache_fonts = true;
            }
            QualityPreset::High => {
                self.compression_level = 7;
                self.font_embedding = FontEmbedding::Full;
                self.cache_math = false; // Ensure fresh rendering
                self.cache_fonts = true;
            }
            QualityPreset::Print => {
                self.compression_level = 9;
                self.font_embedding = FontEmbedding::Full;
                self.cache_math = false;
                self.cache_fonts = false;
            }
        }
    }
    
    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Set debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// Set output directory
    pub fn output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }
    
    /// Set custom font
    pub fn custom_font(mut self, font_path: PathBuf) -> Self {
        self.custom_font = Some(font_path);
        self
    }
    
    /// Set compression level
    pub fn compression_level(mut self, level: u8) -> Self {
        self.compression_level = level.min(9);
        self
    }
    
    /// Enable or disable caching.
    pub fn caching(mut self, enabled: bool) -> Self {
        self.cache_math = enabled;
        self.cache_fonts = enabled;
        self
    }

    /// Keep intermediate build artifacts alongside the output.
    pub fn keep_intermediate(mut self, keep: bool) -> Self {
        self.keep_intermediate = keep;
        self
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserialize from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.quality, QualityPreset::Standard);
        assert_eq!(config.compression_level, 6);
        assert!(config.cache_math);
    }

    #[test]
    fn test_quality_presets() {
        let draft = Config::with_quality(QualityPreset::Draft);
        assert_eq!(draft.compression_level, 3);
        assert_eq!(draft.font_embedding, FontEmbedding::Subset);
        
        let print = Config::with_quality(QualityPreset::Print);
        assert_eq!(print.compression_level, 9);
        assert!(!print.cache_math);
    }

    #[test]
    fn test_builder_pattern() {
        let config = Config::new()
            .verbose(true)
            .debug(true)
            .compression_level(8)
            .caching(false);
        
        assert!(config.verbose);
        assert!(config.debug);
        assert_eq!(config.compression_level, 8);
        assert!(!config.cache_math);
    }

    #[test]
    fn test_toml_roundtrip() {
        let config = Config::new()
            .verbose(true)
            .compression_level(7);
        let toml_str = config.to_toml().unwrap();
        let restored = Config::from_toml(&toml_str).unwrap();
        assert!(restored.verbose);
        assert_eq!(restored.compression_level, 7);
    }

    #[test]
    fn test_json_roundtrip() {
        let config = Config::new()
            .debug(true)
            .caching(false);
        let json_str = config.to_json().unwrap();
        let restored = Config::from_json(&json_str).unwrap();
        assert!(restored.debug);
        assert!(!restored.cache_math);
    }
}
