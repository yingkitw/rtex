//! Image loading and PDF embedding support.
//!
//! Handles PNG and JPEG images for `\includegraphics`.

use std::path::Path;

/// Information about an image to be embedded in a PDF.
#[derive(Debug)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub color_space: ColorSpace,
    pub bits_per_component: u8,
    pub data: Vec<u8>,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgb,
}

impl ImageInfo {
    /// Load an image from a file path.
    ///
    /// Supports PNG and JPEG. Returns an error for unsupported formats or I/O failures.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
        let (width, height) = (img.width(), img.height());

        // Convert to RGB8 for PDF embedding
        let rgb_img = img.to_rgb8();
        let data = rgb_img.into_raw();

        Ok(ImageInfo {
            width,
            height,
            color_space: ColorSpace::Rgb,
            bits_per_component: 8,
            data,
            filter: None,
        })
    }

    /// Generate the PDF XObject dictionary for this image.
    pub fn xobject_dict(&self, length: usize) -> String {
        let cs = match self.color_space {
            ColorSpace::Rgb => "/DeviceRGB",
        };

        if let Some(ref filter) = self.filter {
            format!(
                "<<\n/Type /XObject\n/Subtype /Image\n/Width {}\n/Height {}\n/ColorSpace {}\n/BitsPerComponent {}\n/Filter /{}\n/Length {}\n>>\n",
                self.width, self.height, cs, self.bits_per_component, filter, length
            )
        } else {
            format!(
                "<<\n/Type /XObject\n/Subtype /Image\n/Width {}\n/Height {}\n/ColorSpace {}\n/BitsPerComponent {}\n/Length {}\n>>\n",
                self.width, self.height, cs, self.bits_per_component, length
            )
        }
    }
}

/// Parse a LaTeX dimension string (e.g. "5cm", "2in", "100pt") into PDF points (1/72 inch).
///
/// Returns `None` if the format is not recognized.
pub fn parse_dimension(value: &str) -> Option<f32> {
    let value = value.trim();
    if let Some(v) = value.strip_suffix("pt") {
        v.trim().parse::<f32>().ok()
    } else if let Some(v) = value.strip_suffix("cm") {
        v.trim().parse::<f32>().ok().map(|x| x * 28.3465)
    } else if let Some(v) = value.strip_suffix("mm") {
        v.trim().parse::<f32>().ok().map(|x| x * 2.83465)
    } else if let Some(v) = value.strip_suffix("in") {
        v.trim().parse::<f32>().ok().map(|x| x * 72.0)
    } else {
        value.parse::<f32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dimension_cm() {
        assert!((parse_dimension("5cm").unwrap() - 141.7325).abs() < 0.1);
    }

    #[test]
    fn test_parse_dimension_pt() {
        assert_eq!(parse_dimension("12pt"), Some(12.0));
    }

    #[test]
    fn test_parse_dimension_in() {
        assert_eq!(parse_dimension("2in"), Some(144.0));
    }

    #[test]
    fn test_parse_dimension_raw() {
        assert_eq!(parse_dimension("100"), Some(100.0));
    }
}
