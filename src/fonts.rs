//! Embedded font data for PDF generation.
//!
//! Fonts are compiled into the binary so conversion works in WASM and other
//! environments without filesystem access to `fonts/`.

/// DejaVu Sans TrueType font used for Unicode math and text rendering.
pub static DEJAVU_SANS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fonts/DejaVuSans.ttf"
));
