//! Math formatting submodules.
//!
//! - [`symbols`] — 566+ LaTeX-to-Unicode mappings (single-pass scanner)
//! - [`scripts`] — superscript/subscript Unicode conversion
//! - [`radicals`] — `\sqrt{...}` formatting
//! - [`fractions`] — `\frac{n}{d}` with Unicode fallbacks
//! - [`mathml`] — LaTeX math to presentation MathML for HTML output
//!
//! Orchestrated by [`MathFormatter`](crate::math_formatter::MathFormatter).

pub mod fractions;
pub mod mathml;
pub mod radicals;
pub mod scripts;
pub mod symbols;
