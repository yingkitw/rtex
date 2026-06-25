//! Color management for PDF text and boxes.
//!
//! Supports RGB colors and common named colors. Colors are stored as
//! normalized `[0.0, 1.0]` RGB values and written to the PDF content stream
//! via the `rg` (non-stroking) operator.

use std::str::FromStr;

/// A normalized RGB color for PDF rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    /// White.
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0 };
    /// Black.
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0 };
    /// Red.
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0 };
    /// Green.
    pub const GREEN: Color = Color { r: 0.0, g: 0.5, b: 0.0 };
    /// Blue.
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0 };
    /// Cyan.
    pub const CYAN: Color = Color { r: 0.0, g: 1.0, b: 1.0 };
    /// Magenta.
    pub const MAGENTA: Color = Color { r: 1.0, g: 0.0, b: 1.0 };
    /// Yellow.
    pub const YELLOW: Color = Color { r: 1.0, g: 1.0, b: 0.0 };
    /// Orange.
    pub const ORANGE: Color = Color { r: 1.0, g: 0.65, b: 0.0 };
    /// Purple.
    pub const PURPLE: Color = Color { r: 0.5, g: 0.0, b: 0.5 };
    /// Gray (50%).
    pub const GRAY: Color = Color { r: 0.5, g: 0.5, b: 0.5 };
    /// Dark gray.
    pub const DARK_GRAY: Color = Color { r: 0.25, g: 0.25, b: 0.25 };
    /// Light gray.
    pub const LIGHT_GRAY: Color = Color { r: 0.75, g: 0.75, b: 0.75 };

    /// Create a color from an HTML hex string (`#RRGGBB` or `RGB`).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::from_u8(r, g, b))
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color::from_u8(r, g, b))
            }
            _ => None,
        }
    }

    /// Create a color from 8-bit RGB components.
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    /// Look up a named color (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "white" => Some(Color::WHITE),
            "black" => Some(Color::BLACK),
            "red" => Some(Color::RED),
            "green" => Some(Color::GREEN),
            "blue" => Some(Color::BLUE),
            "cyan" => Some(Color::CYAN),
            "magenta" => Some(Color::MAGENTA),
            "yellow" => Some(Color::YELLOW),
            "orange" => Some(Color::ORANGE),
            "purple" => Some(Color::PURPLE),
            "gray" | "grey" => Some(Color::GRAY),
            "darkgray" | "darkgrey" => Some(Color::DARK_GRAY),
            "lightgray" | "lightgrey" => Some(Color::LIGHT_GRAY),
            _ => None,
        }
    }

    /// Parse a color from a LaTeX color argument.
    ///
    /// Accepts named colors, hex strings, and `r,g,b` decimal triples.
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if let Some(c) = Color::from_name(value) {
            return Some(c);
        }
        if let Some(c) = Color::from_hex(value) {
            return Some(c);
        }
        // Try r,g,b decimal
        let parts: Vec<&str> = value.split(',').collect();
        if parts.len() == 3 {
            let r = f32::from_str(parts[0].trim()).ok()?;
            let g = f32::from_str(parts[1].trim()).ok()?;
            let b = f32::from_str(parts[2].trim()).ok()?;
            return Some(Color { r, g, b });
        }
        None
    }

    /// Return the PDF content stream command for non-stroking color.
    #[cfg(test)]
    pub fn pdf_cmd(&self) -> String {
        format!("{} {} {} rg\n", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_colors() {
        assert_eq!(Color::from_name("red"), Some(Color::RED));
        assert_eq!(Color::from_name("Red"), Some(Color::RED));
        assert_eq!(Color::from_name("BLUE"), Some(Color::BLUE));
        assert_eq!(Color::from_name("unknown"), None);
    }

    #[test]
    fn test_hex_colors() {
        assert_eq!(Color::from_hex("#FF0000"), Some(Color::RED));
        assert_eq!(Color::from_hex("#00FF00"), Some(Color { r: 0.0, g: 1.0, b: 0.0 }));
        assert_eq!(Color::from_hex("F00"), Some(Color::RED));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn test_parse_rgb() {
        let c = Color::parse("0.5,0.25,0.75").unwrap();
        assert!((c.r - 0.5).abs() < 0.001);
        assert!((c.g - 0.25).abs() < 0.001);
        assert!((c.b - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_pdf_cmd() {
        assert_eq!(Color::RED.pdf_cmd(), "1 0 0 rg\n");
        assert_eq!(Color::BLACK.pdf_cmd(), "0 0 0 rg\n");
    }
}
