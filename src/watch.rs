//! Watch mode for automatic recompilation on file changes.
//!
//! Polls the input file and its dependencies for modifications and
//! triggers recompilation when changes are detected.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::{fs, thread};

/// Watch a set of input files and recompile when any change.
///
/// `inputs` is a list of `(input_path, output_path)` pairs.
/// `convert_fn` is called for each pair that has changed.
/// Runs indefinitely until a conversion returns an error or the process is interrupted.
pub fn watch_batch<F, E>(inputs: &[(PathBuf, PathBuf)], convert_fn: F) -> Result<(), E>
where
    F: Fn(&Path, &Path) -> Result<(), E>,
{
    let mut last_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

    // Initial build + capture mtimes
    for (inp, out) in inputs {
        if let Ok(meta) = fs::metadata(inp) {
            last_mtimes.insert(inp.clone(), meta.modified().unwrap_or(SystemTime::UNIX_EPOCH));
        }
        convert_fn(inp, out)?;
    }

    println!("Watching {} file(s) for changes... (Ctrl+C to stop)", inputs.len());

    loop {
        thread::sleep(Duration::from_secs(1));

        for (inp, out) in inputs {
            let Ok(meta) = fs::metadata(inp) else {
                continue;
            };
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let prev = last_mtimes.get(inp).copied().unwrap_or(SystemTime::UNIX_EPOCH);

            if mtime > prev {
                println!("  [change detected] {}", inp.display());
                if let Err(e) = convert_fn(inp, out) {
                    return Err(e);
                }
                println!("  [recompiled]  -> {}", out.display());
                last_mtimes.insert(inp.clone(), mtime);
            }
        }
    }
}

/// Convenience: watch a single input/output pair.
pub fn watch_single<F, E>(input: &Path, output: &Path, convert_fn: F) -> Result<(), E>
where
    F: Fn(&Path, &Path) -> Result<(), E>,
{
    watch_batch(&[(input.to_path_buf(), output.to_path_buf())], convert_fn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn watch_triggers_on_change() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("doc.tex");
        let dst = tmp.path().join("doc.pdf");

        {
            let mut f = fs::File::create(&src).unwrap();
            f.write_all(b"v1").unwrap();
        }

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        // Spawn watcher in a thread that will exit after 2 conversions
        let src2 = src.clone();
        let dst2 = dst.clone();
        let handle = thread::spawn(move || {
            let local_count = AtomicUsize::new(0);
            let result = watch_single(&src2, &dst2, |_inp, _out| {
                let c = local_count.fetch_add(1, Ordering::SeqCst) + 1;
                count_clone.store(c, Ordering::SeqCst);
                if c >= 2 {
                    return Err("stop");
                }
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "stop");
        });

        // Give the watcher time to do the initial build
        thread::sleep(Duration::from_millis(300));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Modify the file
        {
            let mut f = fs::File::create(&src).unwrap();
            f.write_all(b"v2").unwrap();
        }
        // Ensure the new mtime is strictly greater by waiting a bit
        thread::sleep(Duration::from_millis(1100));

        // Touch the file again to guarantee a newer mtime
        {
            let mut f = fs::OpenOptions::new().append(true).open(&src).unwrap();
            f.write_all(b"\n").unwrap();
        }

        // Wait for the watcher to detect the change
        thread::sleep(Duration::from_millis(1200));
        assert_eq!(count.load(Ordering::SeqCst), 2);

        let _ = handle.join();
    }

    #[test]
    fn watch_batch_initial_build() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("a.tex");
        let dst = tmp.path().join("a.pdf");

        {
            let mut f = fs::File::create(&src).unwrap();
            f.write_all(b"hello").unwrap();
        }

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let src2 = src.clone();
        let dst2 = dst.clone();
        let handle = thread::spawn(move || {
            let result = watch_batch(&[(src2, dst2)], |_inp, _out| {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Err("stop_after_first")
            });
            assert!(result.is_err());
        });

        thread::sleep(Duration::from_millis(200));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        let _ = handle.join();
    }
}
