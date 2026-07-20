//! Professional TeX compatibility primitives.
//!
//! Provides the lexical foundation that a full TeX engine requires:
//! category codes, tokenization, dimension parsing, glue, boxes, and
//! Knuth–Plass line breaking.
pub mod catcodes;
pub mod tokens;
pub mod dimensions;
pub mod glue;
pub mod boxes;
pub mod linebreak;

pub use catcodes::CatCode;
pub use tokens::{Token, TexLexer};
pub use dimensions::Dimension;
pub use glue::{Glue, Stretch, InfiniteUnit};
pub use boxes::{TeXBox, BoxDirection};
pub use linebreak::{LineItem, BrokenLine, TokenizedParagraph, LineBreaker, line_badness};
