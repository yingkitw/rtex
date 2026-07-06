//! Math formatting submodules.
//!
//! - [`symbols`] — 566+ LaTeX-to-Unicode mappings (single-pass scanner)
//! - [`scripts`] — superscript/subscript Unicode conversion
//! - [`radicals`] — `\sqrt{...}` formatting
//! - [`fractions`] — `\frac{n}{d}` with Unicode fallbacks
//!
//! Orchestrated by [`MathFormatter`](crate::math_formatter::MathFormatter).

pub mod symbols;
pub mod scripts;
pub mod radicals;
pub mod fractions;
