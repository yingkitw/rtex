//! Professional TeX compatibility primitives.
//!
//! Provides the lexical foundation that a full TeX engine requires:
//! category codes, tokenization, dimension parsing, glue, boxes, and
//! Knuth–Plass line breaking.
pub mod boxes;
pub mod catcodes;
pub mod dimensions;
pub mod glue;
pub mod linebreak;
pub mod tokens;

pub use boxes::{BoxDirection, TeXBox};
pub use catcodes::CatCode;
pub use dimensions::Dimension;
pub use glue::{Glue, InfiniteUnit, Stretch};
pub use linebreak::{BrokenLine, LineBreaker, LineItem, TokenizedParagraph, line_badness};
pub use tokens::{TexLexer, Token};
