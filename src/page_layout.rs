/// Page layout configuration for PDF generation
/// Inspired by pdfrs architecture for better maintainability

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy)]
pub struct PageLayout {
    pub width: f32,
    pub height: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
}

impl PageLayout {
    /// Create a standard portrait layout (8.5" x 11")
    pub fn portrait() -> Self {
        PageLayout {
            width: 612.0,   // 8.5 inches
            height: 792.0,  // 11 inches
            margin_left: 72.0,
            margin_right: 72.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
        }
    }

    /// Create a standard landscape layout (11" x 8.5")
    pub fn landscape() -> Self {
        PageLayout {
            width: 792.0,   // 11 inches
            height: 612.0,  // 8.5 inches
            margin_left: 72.0,
            margin_right: 72.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
        }
    }

    /// Create A4 portrait layout
    pub fn a4_portrait() -> Self {
        PageLayout {
            width: 595.0,   // A4 width
            height: 842.0,  // A4 height
            margin_left: 72.0,
            margin_right: 72.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
        }
    }

    /// Create A4 landscape layout
    pub fn a4_landscape() -> Self {
        PageLayout {
            width: 842.0,   // A4 height
            height: 595.0,  // A4 width
            margin_left: 72.0,
            margin_right: 72.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
        }
    }

    /// Create layout from orientation
    pub fn from_orientation(orientation: PageOrientation) -> Self {
        match orientation {
            PageOrientation::Portrait => Self::portrait(),
            PageOrientation::Landscape => Self::landscape(),
        }
    }

    /// Set uniform margins for all sides
    pub fn with_margins(mut self, margin: f32) -> Self {
        self.margin_left = margin;
        self.margin_right = margin;
        self.margin_top = margin;
        self.margin_bottom = margin;
        self
    }

    /// Set individual margins
    pub fn with_custom_margins(mut self, left: f32, right: f32, top: f32, bottom: f32) -> Self {
        self.margin_left = left;
        self.margin_right = right;
        self.margin_top = top;
        self.margin_bottom = bottom;
        self
    }

    /// Get the top Y coordinate of the content area
    pub fn content_top(&self) -> f32 {
        self.height - self.margin_top
    }

    /// Get the bottom Y coordinate of the content area
    pub fn content_bottom(&self) -> f32 {
        self.margin_bottom
    }

    /// Get the width of the content area
    pub fn content_width(&self) -> f32 {
        self.width - self.margin_left - self.margin_right
    }

    /// Get the height of the content area
    pub fn content_height(&self) -> f32 {
        self.height - self.margin_top - self.margin_bottom
    }

    /// Get the left X coordinate of the content area
    pub fn content_left(&self) -> f32 {
        self.margin_left
    }

    /// Get the right X coordinate of the content area
    pub fn content_right(&self) -> f32 {
        self.width - self.margin_right
    }
}

impl Default for PageLayout {
    fn default() -> Self {
        Self::a4_portrait()
    }
}

/// Font size helpers for different heading levels
pub fn heading_font_size(level: usize, base: f32) -> f32 {
    match level {
        1 => base * 2.0,
        2 => base * 1.6,
        3 => base * 1.3,
        4 => base * 1.1,
        5 => base * 1.0,
        _ => base * 0.9,
    }
}

/// Calculate line height from font size
pub fn line_height(font_size: f32) -> f32 {
    font_size + 4.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portrait_dimensions() {
        let layout = PageLayout::portrait();
        assert_eq!(layout.width, 612.0);
        assert_eq!(layout.height, 792.0);
    }

    #[test]
    fn test_landscape_dimensions() {
        let layout = PageLayout::landscape();
        assert_eq!(layout.width, 792.0);
        assert_eq!(layout.height, 612.0);
    }

    #[test]
    fn test_content_area() {
        let layout = PageLayout::portrait();
        assert_eq!(layout.content_width(), 468.0); // 612 - 72 - 72
        assert_eq!(layout.content_height(), 648.0); // 792 - 72 - 72
    }

    #[test]
    fn test_custom_margins() {
        let layout = PageLayout::portrait()
            .with_custom_margins(50.0, 50.0, 100.0, 100.0);
        assert_eq!(layout.margin_left, 50.0);
        assert_eq!(layout.margin_top, 100.0);
    }

    #[test]
    fn test_heading_sizes() {
        let base = 12.0;
        assert_eq!(heading_font_size(1, base), 24.0);
        assert!((heading_font_size(2, base) - 19.2).abs() < 0.01);
        assert!((heading_font_size(3, base) - 15.6).abs() < 0.01);
    }
}
