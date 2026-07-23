//! Embedded font data for PDF generation.
//!
//! DejaVu Sans is compiled into the binary via [`include_bytes!`] so conversion
//! works on native targets, WASM, and other environments without runtime
//! filesystem access to `fonts/DejaVuSans.ttf`.

/// DejaVu Sans TrueType font used for Unicode math and text rendering.
pub static DEJAVU_SANS: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/DejaVuSans.ttf"));
