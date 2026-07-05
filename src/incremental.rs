//! Incremental / cached compilation.
//!
//! Tracks source-file hashes and intermediate artifacts so that
//! unchanged documents are skipped entirely.  This complements the
//! fine-grained `DocumentCache` (which caches parsed ASTs) with
//! file-level change detection and dependency tracking.
//!
//! Inspired by `latex-rust/src/incremental.rs`, adapted for the
//! `TexElement`-based pipeline.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Tracks build state for incremental compilation.
#[derive(Debug, Clone)]
pub struct IncrementalCompiler {
    /// Map from source path → last known hash.
    source_hashes: HashMap<PathBuf, u64>,
    /// Map from source path → list of output paths it produced.
    outputs: HashMap<PathBuf, Vec<PathBuf>>,
    /// Map from source path → list of auxiliary file dependencies.
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    /// Whether incremental compilation is enabled.
    enabled: bool,
    /// Number of skipped builds (cache hits).
    skips: u64,
    /// Number of full builds (cache misses).
    builds: u64,
}

impl IncrementalCompiler {
    pub fn new() -> Self {
        Self {
            source_hashes: HashMap::new(),
            outputs: HashMap::new(),
            dependencies: HashMap::new(),
            enabled: true,
            skips: 0,
            builds: 0,
        }
    }

    pub fn disabled() -> Self {
        let mut c = Self::new();
        c.enabled = false;
        c
    }

    /// Check whether `source` has changed since the last successful
    /// build.  Returns `true` if a rebuild is required.
    pub fn needs_rebuild(&mut self, source: &Path) -> bool {
        if !self.enabled {
            return true;
        }
        let current = match file_hash(source) {
            Some(h) => h,
            None => return true, // can't read → force rebuild
        };
        match self.source_hashes.get(source) {
            Some(&prev) if prev == current => {
                // Hash unchanged — also check dependencies.
                if self.dependencies_changed(source) {
                    self.builds += 1;
                    return true;
                }
                self.skips += 1;
                false
            }
            _ => {
                self.builds += 1;
                true
            }
        }
    }

    /// Register a successful build so that future calls to
    /// `needs_rebuild` return `false` until the file changes.
    pub fn mark_built(&mut self, source: &Path, outputs: Vec<PathBuf>, deps: Vec<PathBuf>) {
        if let Some(hash) = file_hash(source) {
            self.source_hashes.insert(source.to_path_buf(), hash);
        }
        self.outputs.insert(source.to_path_buf(), outputs);
        self.dependencies.insert(source.to_path_buf(), deps);
    }

    /// Invalidate all cached state for `source`.
    pub fn invalidate(&mut self, source: &Path) {
        self.source_hashes.remove(source);
        self.outputs.remove(source);
        self.dependencies.remove(source);
    }

    /// Clear every tracked entry.
    pub fn clear(&mut self) {
        self.source_hashes.clear();
        self.outputs.clear();
        self.dependencies.clear();
        self.skips = 0;
        self.builds = 0;
    }

    /// True if any dependency of `source` has a newer mtime than
    /// the oldest known output.
    fn dependencies_changed(&self, source: &Path) -> bool {
        let Some(deps) = self.dependencies.get(source) else {
            return false;
        };
        let Some(outs) = self.outputs.get(source) else {
            return false;
        };
        let oldest_out = outs.iter().filter_map(|p| mtime(p)).min();
        let newest_dep = deps.iter().filter_map(|p| mtime(p)).max();
        match (oldest_out, newest_dep) {
            (Some(out), Some(dep)) => dep > out,
            _ => false,
        }
    }

    /// Hit rate: fraction of `needs_rebuild` calls that returned
    /// `false`.
    pub fn hit_rate(&self) -> f64 {
        let total = self.skips + self.builds;
        if total == 0 {
            0.0
        } else {
            self.skips as f64 / total as f64
        }
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.skips, self.builds)
    }

    /// Return true when `output` is at least as new as `source` and all dependencies.
    pub fn outputs_up_to_date(source: &Path, output: &Path, deps: &[PathBuf]) -> bool {
        if !output.exists() {
            return false;
        }
        let Some(out_mtime) = mtime(output) else {
            return false;
        };
        if mtime(source).is_some_and(|src| src > out_mtime) {
            return false;
        }
        deps.iter()
            .all(|dep| mtime(dep).is_none_or(|dep_mtime| dep_mtime <= out_mtime))
    }
}

impl Default for IncrementalCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a content hash for a file.
fn file_hash(path: &Path) -> Option<u64> {
    let content = std::fs::read(path).ok()?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Some(hasher.finish())
}

/// Get the modification time of a file.
fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(path: &Path, content: &str) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn first_build_always_needed() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        write_file(&src, "\\documentclass{article}\n");

        let mut inc = IncrementalCompiler::new();
        assert!(inc.needs_rebuild(&src));
    }

    #[test]
    fn unchanged_file_skips_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        write_file(&src, "hello");

        let mut inc = IncrementalCompiler::new();
        assert!(inc.needs_rebuild(&src));
        inc.mark_built(&src, vec![], vec![]);

        assert!(!inc.needs_rebuild(&src));
        assert_eq!(inc.stats(), (1, 1));
    }

    #[test]
    fn changed_file_triggers_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        write_file(&src, "hello");

        let mut inc = IncrementalCompiler::new();
        assert!(inc.needs_rebuild(&src));
        inc.mark_built(&src, vec![], vec![]);

        // Modify file
        write_file(&src, "world");
        assert!(inc.needs_rebuild(&src));
    }

    #[test]
    fn disabled_always_rebuilds() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        write_file(&src, "x");

        let mut inc = IncrementalCompiler::disabled();
        assert!(inc.needs_rebuild(&src));
        inc.mark_built(&src, vec![], vec![]);
        assert!(inc.needs_rebuild(&src));
    }

    #[test]
    fn invalidate_forces_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        write_file(&src, "x");

        let mut inc = IncrementalCompiler::new();
        inc.mark_built(&src, vec![], vec![]);
        assert!(!inc.needs_rebuild(&src));

        inc.invalidate(&src);
        assert!(inc.needs_rebuild(&src));
    }

    #[test]
    fn hit_rate_calculation() {
        let mut inc = IncrementalCompiler::new();
        assert_eq!(inc.hit_rate(), 0.0);
        inc.skips = 3;
        inc.builds = 1;
        assert_eq!(inc.hit_rate(), 0.75);
    }

    #[test]
    fn outputs_up_to_date_when_output_is_newer() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let out = tmp.path().join("doc.pdf");
        write_file(&src, "hello");
        write_file(&out, "pdf");
        assert!(IncrementalCompiler::outputs_up_to_date(&src, &out, &[]));
    }

    #[test]
    fn outputs_not_up_to_date_when_source_is_newer() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let out = tmp.path().join("doc.pdf");
        write_file(&out, "pdf");
        std::thread::sleep(std::time::Duration::from_millis(1100));
        write_file(&src, "changed");
        assert!(!IncrementalCompiler::outputs_up_to_date(&src, &out, &[]));
    }

    #[test]
    fn dependency_change_triggers_rebuild() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let out = tmp.path().join("doc.pdf");
        let dep = tmp.path().join("header.tex");
        write_file(&src, "x");
        write_file(&dep, "h");
        // Ensure output is created strictly after dependency.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        write_file(&out, "pdf");

        let mut inc = IncrementalCompiler::new();
        inc.mark_built(&src, vec![out.clone()], vec![dep.clone()]);
        assert!(!inc.needs_rebuild(&src));

        // Touch dependency after build
        std::thread::sleep(std::time::Duration::from_millis(1100));
        write_file(&dep, "changed");

        assert!(inc.needs_rebuild(&src));
    }
}
