//! Image loading and PDF embedding support.
//!
//! Handles PNG, JPEG, and SVG images for `\includegraphics`.

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
    /// Supports PNG, JPEG, and SVG. SVG files are rasterized to RGB before embedding.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        if is_svg_path(path) {
            return Self::from_svg_path(path);
        }

        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
        Self::from_dynamic_image(&img)
    }

    fn from_svg_path(path: &Path) -> Result<Self, String> {
        let svg_data =
            std::fs::read(path).map_err(|e| format!("Failed to read SVG '{}': {}", path.display(), e))?;
        Self::from_svg_bytes(&svg_data)
    }

    fn from_svg_bytes(svg_data: &[u8]) -> Result<Self, String> {
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_data, &opt)
            .map_err(|e| format!("Failed to parse SVG: {e}"))?;
        let size = tree.size();
        let width = size.width().ceil().max(1.0) as u32;
        let height = size.height().ceil().max(1.0) as u32;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| "Failed to allocate SVG raster buffer".to_string())?;
        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let rgba = pixmap.data();
        let mut data = Vec::with_capacity((width as usize) * (height as usize) * 3);
        for chunk in rgba.chunks_exact(4) {
            data.push(chunk[0]);
            data.push(chunk[1]);
            data.push(chunk[2]);
        }

        Ok(ImageInfo {
            width,
            height,
            color_space: ColorSpace::Rgb,
            bits_per_component: 8,
            data,
            filter: None,
        })
    }

    fn from_dynamic_image(img: &image::DynamicImage) -> Result<Self, String> {
        let (width, height) = (img.width(), img.height());
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

fn is_svg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    const MINIMAL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">
  <rect x="0" y="0" width="20" height="10" fill="red"/>
</svg>"#;

    #[test]
    fn test_from_svg_bytes() {
        let info = ImageInfo::from_svg_bytes(MINIMAL_SVG.as_bytes()).unwrap();
        assert_eq!(info.width, 20);
        assert_eq!(info.height, 10);
        assert_eq!(info.data.len(), 20 * 10 * 3);
        assert!(info.data.iter().any(|&b| b > 0));
    }

    #[test]
    fn test_from_svg_path() {
        let mut file = NamedTempFile::with_suffix(".svg").unwrap();
        file.write_all(MINIMAL_SVG.as_bytes()).unwrap();

        let info = ImageInfo::from_path(file.path()).unwrap();
        assert_eq!(info.width, 20);
        assert_eq!(info.height, 10);
    }

    #[test]
    fn test_is_svg_path() {
        assert!(is_svg_path(Path::new("logo.svg")));
        assert!(is_svg_path(Path::new("logo.SVG")));
        assert!(!is_svg_path(Path::new("logo.png")));
    }

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
