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
                convert_fn(inp, out)?;
                println!("  [recompiled]  -> {}", out.display());
                last_mtimes.insert(inp.clone(), mtime);
            }
        }
    }
}

/// Discover `\input{...}` dependencies in a `.tex` file.
pub(crate) fn discover_inputs(path: &Path) -> Vec<PathBuf> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut deps = Vec::new();
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));

    // Simple regex-free scan for \input{...} and \input ...
    let mut pos = 0;
    while let Some(idx) = content[pos..].find("\\input") {
        let start = pos + idx + "\\input".len();
        let rest = &content[start..];
        let trimmed = rest.trim_start();

        let filename = if let Some(stripped) = trimmed.strip_prefix('{') {
            // Braced form: \input{file}
            if let Some(end) = stripped.find('}') {
                stripped[..end].to_string()
            } else {
                continue;
            }
        } else {
            // Unbraced form: \input file (ends at whitespace or end of line)
            let end = trimmed.find(|c: char| c.is_whitespace()).unwrap_or(trimmed.len());
            trimmed[..end].to_string()
        };

        if filename.is_empty() {
            continue;
        }

        // Append .tex extension if missing
        let filename = if filename.ends_with(".tex") {
            filename
        } else {
            format!("{}.tex", filename)
        };

        let dep_path = base_dir.join(&filename);
        if dep_path.exists() {
            deps.push(dep_path);
        }

        pos = start;
    }

    deps
}

/// Convenience: watch a single input/output pair, including `\input`
/// dependencies.  When the root file or any dependency changes, the
/// root document is rebuilt.
pub fn watch_single<F, E>(input: &Path, output: &Path, convert_fn: F) -> Result<(), E>
where
    F: Fn(&Path, &Path) -> Result<(), E>,
{
    let mut last_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

    // Initial build + capture mtimes of root + deps
    let mut deps = discover_inputs(input);
    let mut watched: Vec<PathBuf> = vec![input.to_path_buf()];
    watched.extend(deps.iter().cloned());

    for path in &watched {
        if let Ok(meta) = fs::metadata(path) {
            last_mtimes.insert(path.clone(), meta.modified().unwrap_or(SystemTime::UNIX_EPOCH));
        }
    }
    convert_fn(input, output)?;

    println!(
        "Watching {} file(s) for changes... (Ctrl+C to stop)",
        watched.len()
    );

    loop {
        thread::sleep(Duration::from_secs(1));

        let mut changed = false;
        for path in &watched {
            let Ok(meta) = fs::metadata(path) else { continue };
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let prev = last_mtimes.get(path).copied().unwrap_or(SystemTime::UNIX_EPOCH);

            if mtime > prev {
                println!("  [change detected] {}", path.display());
                last_mtimes.insert(path.clone(), mtime);
                changed = true;
            }
        }

        if changed {
            convert_fn(input, output)?;
            println!("  [recompiled]  -> {}", output.display());

            // Re-discover dependencies in case \input list changed
            let new_deps = discover_inputs(input);
            if new_deps != deps {
                deps = new_deps;
                watched = vec![input.to_path_buf()];
                watched.extend(deps.iter().cloned());
                // Prime mtimes for any newly-added deps
                for path in &watched {
                    last_mtimes.entry(path.clone()).or_insert_with(|| {
                        fs::metadata(path)
                            .and_then(|m| m.modified())
                            .unwrap_or(SystemTime::UNIX_EPOCH)
                    });
                }
                println!(
                    "  [deps updated] now watching {} file(s)",
                    watched.len()
                );
            }
        }
    }
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

    #[test]
    fn watch_rebuilds_root_when_dependency_changes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("main.tex");
        let dep = tmp.path().join("chapter.tex");
        let dst = tmp.path().join("main.pdf");

        {
            let mut f = fs::File::create(&root).unwrap();
            f.write_all(b"\\input{chapter}").unwrap();
        }
        {
            let mut f = fs::File::create(&dep).unwrap();
            f.write_all(b"v1").unwrap();
        }

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let root2 = root.clone();
        let dst2 = dst.clone();
        let handle = thread::spawn(move || {
            let result = watch_single(&root2, &dst2, |_inp, _out| {
                let c = count_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if c >= 2 {
                    return Err("stop");
                }
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "stop");
        });

        // Initial build
        thread::sleep(Duration::from_millis(300));
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Modify the dependency file, not the root
        {
            let mut f = fs::File::create(&dep).unwrap();
            f.write_all(b"v2").unwrap();
        }
        thread::sleep(Duration::from_millis(1200));

        // Watcher should have rebuilt because dependency changed
        assert_eq!(count.load(Ordering::SeqCst), 2);

        let _ = handle.join();
    }
}
