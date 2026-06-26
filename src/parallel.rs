//! Parallel batch conversion for multiple LaTeX documents.
//!
//! Distributes independent conversions across a pool of worker threads,
//! collecting results without stopping the whole batch on individual
//! failures.

use crate::error::LatexError;
use crate::{NativeTexConverter, TexConverter};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Result of a single conversion job.
pub type ConversionResult = Result<PathBuf, (PathBuf, LatexError)>;

/// Convert many `(input, output)` pairs in parallel using a fixed
/// number of worker threads.
pub struct ParallelConverter {
    /// Maximum concurrent worker threads.  `None` means one per CPU core.
    workers: Option<usize>,
}

impl Default for ParallelConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelConverter {
    pub fn new() -> Self {
        Self { workers: None }
    }

    pub fn with_workers(n: usize) -> Self {
        Self { workers: Some(n) }
    }

    /// Convert all `jobs` in parallel.
    ///
    /// Each job is `(input_path, output_path)`.
    /// Returns a vector of results in the same order as `jobs`.
    pub fn convert_batch(&self, jobs: &[(PathBuf, PathBuf)]) -> Vec<ConversionResult> {
        if jobs.is_empty() {
            return Vec::new();
        }

        let worker_count = self.workers.unwrap_or_else(num_cpus::get);
        let worker_count = worker_count.min(jobs.len()).max(1);

        // Channel to send work to threads
        let (work_tx, work_rx) = mpsc::channel::<(usize, PathBuf, PathBuf)>();
        let work_rx = Arc::new(Mutex::new(work_rx));

        // Channel to collect results back
        let (res_tx, res_rx) = mpsc::channel::<(usize, ConversionResult)>();

        // Spawn workers
        let mut handles = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let work_rx = Arc::clone(&work_rx);
            let res_tx = res_tx.clone();
            let handle = thread::spawn(move || {
                let converter = NativeTexConverter::new();
                loop {
                    let (idx, input, output) = {
                        let lock = work_rx.lock().unwrap();
                        match lock.recv() {
                            Ok(triple) => triple,
                            Err(_) => break, // channel closed
                        }
                    };
                    let res = converter.convert(&input, &output);
                    let _ = res_tx.send((idx, match res {
                        Ok(()) => Ok(output),
                        Err(e) => Err((input, e)),
                    }));
                }
            });
            handles.push(handle);
        }
        drop(res_tx); // drop our clone so channel closes when all workers finish

        // Queue all jobs
        for (idx, (input, output)) in jobs.iter().enumerate() {
            work_tx.send((idx, input.clone(), output.clone())).unwrap();
        }
        drop(work_tx); // close work channel so workers exit when drained

        // Wait for all workers
        for h in handles {
            let _ = h.join();
        }

        // Collect results and sort by original index
        let mut results: Vec<(usize, ConversionResult)> = res_rx.iter().collect();
        results.sort_by_key(|(idx, _)| *idx);
        results.into_iter().map(|(_, r)| r).collect()
    }
}

/// Convenience: convert every `.tex` file in `inputs` directory to a
/// `.pdf` in `outputs` directory, using the same base name.
pub fn convert_dir(
    inputs: &Path,
    outputs: &Path,
    workers: Option<usize>,
) -> Vec<ConversionResult> {
    let mut jobs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(inputs) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "tex").unwrap_or(false) {
                let stem = path.file_stem().unwrap_or_default();
                let out = outputs.join(format!("{}.pdf", stem.to_string_lossy()));
                jobs.push((path, out));
            }
        }
    }
    ParallelConverter::with_workers(workers.unwrap_or_else(num_cpus::get))
        .convert_batch(&jobs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tex(path: &Path, content: &str) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn empty_batch() {
        let c = ParallelConverter::new();
        let r = c.convert_batch(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn single_file_conversion() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("hello.tex");
        let dst = tmp.path().join("hello.pdf");
        write_tex(
            &src,
            r#"\documentclass{article}
\begin{document}
Hello
\end{document}
"#,
        );

        let c = ParallelConverter::new();
        let results = c.convert_batch(&[(src.clone(), dst.clone())]);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok(), "{:?}", results[0]);
        assert!(dst.exists());
    }

    #[test]
    fn multiple_files_parallel() {
        let tmp = tempfile::tempdir().unwrap();
        let mut jobs = Vec::new();
        for i in 0..4 {
            let src = tmp.path().join(format!("doc{i}.tex"));
            let dst = tmp.path().join(format!("doc{i}.pdf"));
            write_tex(
                &src,
                &format!(
                    r#"\documentclass{{article}}
\begin{{document}}
Doc {i}
\end{{document}}
"#
                ),
            );
            jobs.push((src, dst));
        }

        let c = ParallelConverter::with_workers(2);
        let results = c.convert_batch(&jobs);
        assert_eq!(results.len(), 4);
        for (i, r) in results.iter().enumerate() {
            assert!(r.is_ok(), "doc{i} failed: {:?}", r);
        }
        for (_, dst) in &jobs {
            assert!(dst.exists());
        }
    }

    #[test]
    fn missing_file_reports_error() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("nonexistent.tex");
        let dst = tmp.path().join("bad.pdf");

        let c = ParallelConverter::new();
        let results = c.convert_batch(&[(src.clone(), dst.clone())]);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err(), "expected error for missing file");
    }

    #[test]
    fn convert_dir_finds_tex_files() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        write_tex(
            &tmp.path().join("a.tex"),
            r#"\documentclass{article}\begin{document}A\end{document}"#,
        );
        write_tex(
            &tmp.path().join("b.tex"),
            r#"\documentclass{article}\begin{document}B\end{document}"#,
        );
        // Non-tex file should be ignored
        std::fs::write(tmp.path().join("readme.txt"), "hi").unwrap();

        let results = convert_dir(tmp.path(), out.path(), Some(2));
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
