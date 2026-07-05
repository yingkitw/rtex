//! WebAssembly bindings for browser-side TeX-to-PDF conversion.
//!
//! Build with:
//! ```text
//! cargo build --target wasm32-unknown-unknown --features wasm --release
//! ```
//!
//! Then use `wasm-bindgen` to generate JavaScript glue code.

use wasm_bindgen::prelude::*;

/// Convert a LaTeX source string to PDF bytes.
///
/// Returns the raw PDF file contents on success, or a JavaScript error string
/// on failure. Suitable for zero-infrastructure preview UIs in the browser.
#[wasm_bindgen(js_name = convertTexToPdf)]
pub fn convert_tex_to_pdf_wasm(tex: &str) -> Result<Vec<u8>, JsValue> {
    crate::convert_tex_string_to_pdf_bytes(tex).map_err(|e| JsValue::from_str(&e.to_string()))
}
