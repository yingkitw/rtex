//! Layout engine for multi-page PDF generation.
//!
//! Tracks current page, Y position, margins, and handles automatic
//! page breaks when content overflows the printable area.

use crate::pdf_core::ContentStream;
use crate::page_layout::PageLayout;

/// State machine that manages content placement across multiple pages.
pub struct LayoutState {
    pub layout: PageLayout,
    /// Current vertical position on the active page (PDF coords, top-down).
    pub current_y: f32,
    /// One content stream per page.
    pub pages: Vec<ContentStream>,
    /// Current font size in points.
    pub current_font_size: f32,
}

impl LayoutState {
    /// Initialise a layout using the given page dimensions.
    pub fn new(layout: PageLayout) -> Self {
        Self {
            layout,
            current_y: layout.content_top(),
            pages: vec![ContentStream::new()],
            current_font_size: 11.0,
        }
    }

    /// Mutable reference to the content stream of the active page.
    pub fn current_stream(&mut self) -> &mut ContentStream {
        self.pages.last_mut().unwrap()
    }

    /// Left margin X coordinate.
    pub fn left_margin(&self) -> f32 {
        self.layout.margin_left
    }

    /// Usable width inside the margins.
    pub fn content_width(&self) -> f32 {
        self.layout.content_width()
    }

    /// Bottom edge of the printable area.
    pub fn content_bottom(&self) -> f32 {
        self.layout.margin_bottom
    }

    /// Top edge of the printable area.
    pub fn content_top(&self) -> f32 {
        self.layout.content_top()
    }

    /// Standard line height for a given font size.
    pub fn line_height(&self, font_size: f32) -> f32 {
        font_size + 4.0
    }

    /// Vertical space still available on the current page.
    pub fn available_height(&self) -> f32 {
        self.current_y - self.content_bottom()
    }

    /// Whether `height` fits on the current page.
    pub fn would_fit(&self, height: f32) -> bool {
        height <= self.available_height()
    }

    /// Move the cursor down by `height`.
    pub fn advance(&mut self, height: f32) {
        self.current_y -= height;
    }

    /// Finish the current page and start a fresh one.
    pub fn new_page(&mut self) {
        self.pages.push(ContentStream::new());
        self.current_y = self.content_top();
    }

    /// If `required_height` does not fit, start a new page first.
    pub fn ensure_space(&mut self, required_height: f32) {
        if !self.would_fit(required_height) {
            self.new_page();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_state_initial() {
        let layout = PageLayout::a4_portrait();
        let state = LayoutState::new(layout);
        assert_eq!(state.pages.len(), 1);
        assert_eq!(state.current_y, layout.content_top());
        assert!(state.would_fit(layout.content_height()));
    }

    #[test]
    fn test_new_page() {
        let layout = PageLayout::a4_portrait();
        let mut state = LayoutState::new(layout);
        state.new_page();
        assert_eq!(state.pages.len(), 2);
        assert_eq!(state.current_y, layout.content_top());
    }

    #[test]
    fn test_ensure_space() {
        let layout = PageLayout::a4_portrait();
        let mut state = LayoutState::new(layout);
        // consume almost all space
        state.current_y = state.content_bottom() + 10.0;
        state.ensure_space(20.0); // 20 > 10, so new page
        assert_eq!(state.pages.len(), 2);
        assert_eq!(state.current_y, layout.content_top());
    }
}
