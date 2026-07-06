//! Language Server Protocol support for `.tex` editors.
//!
//! The analysis layer (`diagnostics`, `completion`, `symbols`, `hover`) is always
//! available. The stdio language server binary requires the `lsp` feature.

pub mod completion;
pub mod diagnostics;
pub mod hover;
pub mod positions;
pub mod symbols;

pub use completion::{TexCompletion, CompletionKind, command_completions, completions_at};
pub use diagnostics::{TexDiagnostic, Severity, analyze_diagnostics};
pub use hover::hover_at;
pub use positions::{TexPosition, TexRange, offset_to_position, range_to_positions};
pub use symbols::{TexSymbol, SymbolKind, document_symbols};

#[cfg(feature = "lsp")]
pub mod server;

/// Run the language server over stdio (requires `lsp` feature).
#[cfg(feature = "lsp")]
pub fn run_stdio_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    server::run()
}
