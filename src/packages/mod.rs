//! On-demand LaTeX package fetching (Tectonic-style).
//!
//! Scans the document preamble for `\documentclass` and `\usepackage`
//! declarations, then downloads missing `.cls`/`.sty` files from CTAN
//! mirrors into a local cache.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::LatexError;

/// A LaTeX package or class file required by a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageRequest {
    pub name: String,
    pub file_name: String,
}

/// Local cache and download settings.
#[derive(Debug, Clone)]
pub struct PackageFetcher {
    cache_dir: PathBuf,
    mirror_base: String,
    offline: bool,
}

impl PackageFetcher {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            mirror_base: "https://mirrors.ctan.org".to_string(),
            offline: false,
        }
    }

    pub fn with_mirror(mut self, mirror_base: impl Into<String>) -> Self {
        self.mirror_base = mirror_base.into();
        self
    }

    pub fn offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Scan preamble for `\documentclass` and `\usepackage` dependencies.
    pub fn scan_dependencies(tex: &str) -> Vec<PackageRequest> {
        let preamble = tex.split("\\begin{document}").next().unwrap_or(tex);
        let mut requests = Vec::new();
        let mut seen = HashSet::new();

        for class in extract_braced_command(preamble, "\\documentclass") {
            push_request(&mut requests, &mut seen, &class, &format!("{class}.cls"));
        }

        for package_list in extract_braced_command(preamble, "\\usepackage") {
            for package in package_list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let pkg = package.split('[').next().unwrap_or(package).trim();
                push_request(&mut requests, &mut seen, pkg, &format!("{pkg}.sty"));
            }
        }

        for style in extract_braced_command(preamble, "\\RequirePackage") {
            for package in style.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                let pkg = package.split('[').next().unwrap_or(package).trim();
                push_request(&mut requests, &mut seen, pkg, &format!("{pkg}.sty"));
            }
        }

        requests
    }

    /// Ensure all declared packages exist in the cache, downloading when needed.
    pub fn ensure_packages(&self, requests: &[PackageRequest]) -> Result<Vec<PathBuf>, LatexError> {
        std::fs::create_dir_all(&self.cache_dir).map_err(|source| LatexError::IoError {
            path: self.cache_dir.clone(),
            source,
        })?;

        let mut resolved = Vec::new();
        for request in requests {
            let local = self.cache_dir.join(&request.file_name);
            if local.exists() {
                resolved.push(local);
                continue;
            }
            if self.offline {
                return Err(LatexError::UnsupportedFeature {
                    feature: format!("Missing package '{}'", request.name),
                    suggestion: Some(format!(
                        "Download {} into {}",
                        request.file_name,
                        self.cache_dir.display()
                    )),
                });
            }
            self.download_package(request)?;
            resolved.push(local);
        }
        Ok(resolved)
    }

    /// Convenience: scan and fetch all preamble dependencies.
    pub fn prepare_source(&self, tex: &str) -> Result<Vec<PathBuf>, LatexError> {
        let requests = Self::scan_dependencies(tex);
        self.ensure_packages(&requests)
    }

    /// Search paths for `\input` resolution: cache dir first.
    pub fn search_paths(&self) -> Vec<PathBuf> {
        vec![self.cache_dir.clone()]
    }

    fn download_package(&self, request: &PackageRequest) -> Result<(), LatexError> {
        let urls = ctan_candidate_urls(&self.mirror_base, &request.name, &request.file_name);
        let mut last_error = String::from("no URLs attempted");

        for url in urls {
            match fetch_url(&url) {
                Ok(bytes) if !bytes.is_empty() => {
                    let dest = self.cache_dir.join(&request.file_name);
                    std::fs::write(&dest, bytes).map_err(|source| LatexError::IoError {
                        path: dest,
                        source,
                    })?;
                    return Ok(());
                }
                Ok(_) => last_error = format!("empty response from {url}"),
                Err(err) => last_error = err,
            }
        }

        Err(LatexError::UnsupportedFeature {
            feature: format!("Could not fetch '{}'", request.file_name),
            suggestion: Some(format!("Last error: {last_error}")),
        })
    }
}

fn push_request(
    requests: &mut Vec<PackageRequest>,
    seen: &mut HashSet<(String, String)>,
    name: &str,
    file_name: &str,
) {
    let key = (name.to_string(), file_name.to_string());
    if seen.insert(key.clone()) {
        requests.push(PackageRequest {
            name: key.0,
            file_name: key.1,
        });
    }
}

fn extract_braced_command(source: &str, command: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find(command) {
        let start = search_from + idx + command.len();
        let rest = source[start..].trim_start();
        let rest = rest
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(_, after)| after)
            .unwrap_or(rest)
            .trim_start();
        if let Some(brace) = rest.find('{') {
            if let Some((inner, _)) = crate::utils::extract_braced(rest, brace) {
                values.push(inner);
            }
        }
        search_from = start + 1;
    }
    values
}

fn ctan_candidate_urls(mirror: &str, package: &str, file_name: &str) -> Vec<String> {
    vec![
        format!("{mirror}/macros/latex/base/{file_name}"),
        format!("{mirror}/macros/latex/required/{package}/{file_name}"),
        format!("{mirror}/macros/latex/contrib/{package}/{file_name}"),
        format!("{mirror}/macros/latex/contrib/{package}/{package}.sty"),
    ]
}

fn fetch_url(url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(url).call().map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response.into_body().read_to_vec().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_documentclass_and_usepackage() {
        let tex = r#"\documentclass{article}
\usepackage[utf8]{inputenc}
\usepackage{amsmath,graphicx}
\RequirePackage{hyperref}
\begin{document}
Hi
\end{document}"#;
        let deps = PackageFetcher::scan_dependencies(tex);
        let names: HashSet<_> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains("article"));
        assert!(names.contains("inputenc"));
        assert!(names.contains("amsmath"));
        assert!(names.contains("graphicx"));
        assert!(names.contains("hyperref"));
    }

    #[test]
    fn ensure_packages_uses_existing_cache() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = PackageFetcher::new(dir.path()).offline(true);
        let request = PackageRequest {
            name: "article".into(),
            file_name: "article.cls".into(),
        };
        std::fs::write(dir.path().join("article.cls"), b"\\ProvidesClass{article}").unwrap();
        let paths = fetcher.ensure_packages(&[request]).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].exists());
    }

    #[test]
    fn ctan_urls_include_base_and_contrib() {
        let urls = ctan_candidate_urls("https://mirrors.ctan.org", "amsmath", "amsmath.sty");
        assert!(urls.iter().any(|u| u.contains("/macros/latex/required/amsmath/")));
        assert!(urls.iter().any(|u| u.contains("/macros/latex/contrib/amsmath/")));
    }
}
