//! Professional TeX compatibility primitives.
//!
//! Provides the lexical foundation that a full TeX engine requires:
//! category codes, tokenization, and dimension parsing.
//! Glue, boxes, and the Knuth–Plass line-breaking algorithm are
//! reserved for future work.

pub mod catcodes;
pub mod tokens;
pub mod dimensions;

pub use catcodes::CatCode;
pub use tokens::{Token, TexLexer};
pub use dimensions::Dimension;
