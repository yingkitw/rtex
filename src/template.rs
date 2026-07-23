//! Document template / style system for configurable PDF output.
//!
//! Loads styling parameters from TOML or JSON files so that document
//! appearance can be customised without editing Rust code.

use crate::page_layout::{PageLayout, PageOrientation};
use serde::{Deserialize, Serialize};

/// Colour scheme for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub text: String,
    pub background: String,
    pub link: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            text: "#000000".to_string(),
            background: "#FFFFFF".to_string(),
            link: "#0000EE".to_string(),
        }
    }
}

/// Typography scale for headings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingScale {
    pub h1: f32,
    pub h2: f32,
    pub h3: f32,
    pub h4: f32,
    pub h5: f32,
    pub h6: f32,
}

impl Default for HeadingScale {
    fn default() -> Self {
        Self {
            h1: 24.0,
            h2: 18.0,
            h3: 14.0,
            h4: 12.0,
            h5: 10.0,
            h6: 9.0,
        }
    }
}

/// Document template controlling page layout, fonts, colours and metadata rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    /// Paper size preset or explicit dimensions.
    pub paper: PaperSize,
    /// Page orientation.
    pub orientation: PageOrientation,
    /// Margins in PDF points (1/72 inch).
    pub margins: Margins,
    /// Base body font size in points.
    pub base_font_size: f32,
    /// Line spacing multiplier (1.0 = single, 1.5 = one-and-a-half, etc.).
    pub line_spacing: f32,
    /// Heading sizes.
    pub headings: HeadingScale,
    /// Title page settings.
    pub title_page: TitlePageConfig,
    /// Colour scheme.
    pub colors: ColorScheme,
    /// Header text (optional).
    pub header: Option<String>,
    /// Footer text (optional).
    pub footer: Option<String>,
}

/// Paper size presets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaperSize {
    A4,
    Letter,
    Legal,
    Custom { width: f32, height: f32 },
}

/// Margins in points.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Margins {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            left: 72.0,
            right: 72.0,
            top: 72.0,
            bottom: 72.0,
        }
    }
}

/// Title-page rendering configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitlePageConfig {
    pub enabled: bool,
    pub title_font_size: f32,
    pub author_font_size: f32,
    pub date_font_size: f32,
    pub spacing_after_title: f32,
    pub spacing_after_author: f32,
    pub spacing_after_date: f32,
}

impl Default for TitlePageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            title_font_size: 24.0,
            author_font_size: 12.0,
            date_font_size: 10.0,
            spacing_after_title: 30.0,
            spacing_after_author: 20.0,
            spacing_after_date: 25.0,
        }
    }
}

impl Default for DocumentTemplate {
    fn default() -> Self {
        Self {
            paper: PaperSize::A4,
            orientation: PageOrientation::Portrait,
            margins: Margins::default(),
            base_font_size: 11.0,
            line_spacing: 1.2,
            headings: HeadingScale::default(),
            title_page: TitlePageConfig::default(),
            colors: ColorScheme::default(),
            header: None,
            footer: None,
        }
    }
}

impl DocumentTemplate {
    /// Load a template from a TOML file.
    pub fn from_toml(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let template: DocumentTemplate = toml::from_str(&content)?;
        Ok(template)
    }

    /// Load a template from a JSON file.
    pub fn from_json(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let template: DocumentTemplate = serde_json::from_str(&content)?;
        Ok(template)
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert the template into a concrete `PageLayout`.
    pub fn page_layout(&self) -> PageLayout {
        let (width, height) = match self.paper {
            PaperSize::A4 => (595.0, 842.0),
            PaperSize::Letter => (612.0, 792.0),
            PaperSize::Legal => (612.0, 1008.0),
            PaperSize::Custom { width, height } => (width, height),
        };
        let (width, height) = match self.orientation {
            PageOrientation::Portrait => (width, height),
            PageOrientation::Landscape => (height, width),
        };
        PageLayout {
            width,
            height,
            margin_left: self.margins.left,
            margin_right: self.margins.right,
            margin_top: self.margins.top,
            margin_bottom: self.margins.bottom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_roundtrip_toml() {
        let t = DocumentTemplate::default();
        let s = t.to_toml().unwrap();
        let t2: DocumentTemplate = toml::from_str(&s).unwrap();
        assert_eq!(t2.base_font_size, 11.0);
        assert_eq!(t2.margins.left, 72.0);
        assert!(matches!(t2.paper, PaperSize::A4));
    }

    #[test]
    fn default_template_roundtrip_json() {
        let t = DocumentTemplate::default();
        let s = t.to_json().unwrap();
        let t2: DocumentTemplate = serde_json::from_str(&s).unwrap();
        assert_eq!(t2.base_font_size, 11.0);
        assert_eq!(t2.line_spacing, 1.2);
    }

    #[test]
    fn custom_paper_size() {
        let t = DocumentTemplate {
            paper: PaperSize::Custom {
                width: 300.0,
                height: 400.0,
            },
            orientation: PageOrientation::Landscape,
            ..Default::default()
        };
        let layout = t.page_layout();
        assert_eq!(layout.width, 400.0);
        assert_eq!(layout.height, 300.0);
    }

    #[test]
    fn letter_portrait_layout() {
        let t = DocumentTemplate {
            paper: PaperSize::Letter,
            orientation: PageOrientation::Portrait,
            ..Default::default()
        };
        let layout = t.page_layout();
        assert_eq!(layout.width, 612.0);
        assert_eq!(layout.height, 792.0);
    }

    #[test]
    fn heading_scale_defaults() {
        let h = HeadingScale::default();
        assert_eq!(h.h1, 24.0);
        assert_eq!(h.h2, 18.0);
    }

    #[test]
    fn color_scheme_defaults() {
        let c = ColorScheme::default();
        assert_eq!(c.text, "#000000");
    }
}
