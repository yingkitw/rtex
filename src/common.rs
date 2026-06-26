//! Shared traits and macros to reduce boilerplate.
//!
//! Ported and simplified from `latex-rust/src/common.rs`.

/// Trait for types that provide a `clear` operation.
pub trait Clear {
    fn clear(&mut self);
}

/// Trait for types that expose statistics.
pub trait Stats {
    type StatsType;
    fn stats(&self) -> &Self::StatsType;
}
