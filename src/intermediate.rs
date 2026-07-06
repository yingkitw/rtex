//! Intermediate artifact persistence for debugging and inspection.
//!
//! When `keep_intermediate` is enabled, writes sibling files next to the
//! final output:
//! - `{stem}.expanded.tex` — source after macro expansion
//! - `{stem}.ast.json` — serialized parsed element tree
//! - `{stem}.meta.json` — conversion metadata summary

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::LatexError;
use crate::output::{DocumentMeta, OutputFormat, common::extract_metadata};
use crate::parser::TexElement;

/// Metadata summary written alongside intermediate artifacts.
#[derive(Debug, Serialize, Deserialize)]
pub struct IntermediateMeta {
    pub format: String,
    pub element_count: usize,
    pub title: Option<String>,
    pub author: Option<String>,
}

/// Return the path for a sibling artifact next to `output`.
pub fn sibling_artifact(output: &Path, extension: &str) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    parent.join(format!("{stem}.{extension}"))
}

/// Write intermediate artifacts next to `output`.
pub fn write_intermediates(
    output: &Path,
    expanded_tex: &str,
    elements: &[TexElement],
    format: OutputFormat,
) -> Result<(), LatexError> {
    let expanded_path = sibling_artifact(output, "expanded.tex");
    std::fs::write(&expanded_path, expanded_tex).map_err(|source| LatexError::IoError {
        path: expanded_path,
        source,
    })?;

    let ast_path = sibling_artifact(output, "ast.json");
    let ast_json = serde_json::to_string_pretty(elements).map_err(|e| LatexError::ConfigError {
        message: format!("Failed to serialize AST: {e}"),
    })?;
    std::fs::write(&ast_path, ast_json).map_err(|source| LatexError::IoError {
        path: ast_path,
        source,
    })?;

    let meta = build_meta(elements, format);
    let meta_path = sibling_artifact(output, "meta.json");
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| LatexError::ConfigError {
        message: format!("Failed to serialize metadata: {e}"),
    })?;
    std::fs::write(&meta_path, meta_json).map_err(|source| LatexError::IoError {
        path: meta_path,
        source,
    })?;

    Ok(())
}

fn build_meta(elements: &[TexElement], format: OutputFormat) -> IntermediateMeta {
    let doc_meta: DocumentMeta = extract_metadata(elements);
    IntermediateMeta {
        format: format.extension().to_string(),
        element_count: elements.len(),
        title: doc_meta.title,
        author: doc_meta.author,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::TexParser;

    #[test]
    fn sibling_artifact_uses_output_stem() {
        let path = Path::new("output/report.pdf");
        assert_eq!(
            sibling_artifact(path, "ast.json"),
            Path::new("output/report.ast.json")
        );
    }

    #[test]
    fn write_intermediates_creates_files() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("doc.pdf");
        let tex = r#"\documentclass{article}
\title{Test}
\begin{document}
Hello
\end{document}"#;
        let expanded = crate::macros::expand_document(tex);
        let mut parser = TexParser::new(tex.to_string());
        let elements = parser.parse();

        write_intermediates(&output, &expanded, &elements, OutputFormat::Pdf).unwrap();

        assert!(dir.path().join("doc.expanded.tex").exists());
        assert!(dir.path().join("doc.ast.json").exists());
        assert!(dir.path().join("doc.meta.json").exists());

        let meta: IntermediateMeta =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join("doc.meta.json")).unwrap())
                .unwrap();
        assert_eq!(meta.format, "pdf");
        assert!(meta.element_count > 0);
        assert_eq!(meta.title.as_deref(), Some("Test"));
    }
}
