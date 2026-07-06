//! TeX box model primitives.
//!
//! Boxes are rectangular areas with width, height, and depth (extent below
//! the baseline). Horizontal and vertical boxes are the building blocks for
//! the TeX layout engine. Full box packing is reserved for future work.

use super::dimensions::Dimension;

/// Box orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxDirection {
    Horizontal,
    Vertical,
}

/// A TeX box with dimensions relative to the baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeXBox {
    pub direction: BoxDirection,
    pub width: Dimension,
    pub height: Dimension,
    pub depth: Dimension,
}

impl TeXBox {
    /// Empty horizontal box.
    pub fn hbox_empty() -> Self {
        Self {
            direction: BoxDirection::Horizontal,
            width: Dimension::from_sp(0),
            height: Dimension::from_sp(0),
            depth: Dimension::from_sp(0),
        }
    }

    /// Total vertical extent (height + depth).
    pub fn total_height(&self) -> Dimension {
        Dimension::from_sp(self.height.sp() + self.depth.sp())
    }

    /// Create a horizontal box with explicit dimensions.
    pub fn hbox(width: Dimension, height: Dimension, depth: Dimension) -> Self {
        Self {
            direction: BoxDirection::Horizontal,
            width,
            height,
            depth,
        }
    }

    /// Create a vertical box with explicit dimensions.
    pub fn vbox(width: Dimension, height: Dimension, depth: Dimension) -> Self {
        Self {
            direction: BoxDirection::Vertical,
            width,
            height,
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hbox_total_height() {
        let height = Dimension::parse("10pt").unwrap();
        let depth = Dimension::parse("2pt").unwrap();
        let b = TeXBox::hbox(Dimension::parse("50pt").unwrap(), height, depth);
        assert_eq!(b.total_height().pt(), 12.0);
    }

    #[test]
    fn empty_hbox_is_zero() {
        let b = TeXBox::hbox_empty();
        assert_eq!(b.width.sp(), 0);
        assert_eq!(b.total_height().sp(), 0);
    }
}
