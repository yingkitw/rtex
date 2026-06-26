//! Document cache with TTL and LRU eviction.
//!
//! Caches parsed documents (`Vec<TexElement>`) and rendered PDF paths
//! keyed by the content hash of the source, avoiding re-parsing and
//! re-rendering when a file hasn't changed.
//!
//! Ported and adapted from `latex-rust/src/cache.rs`.

use crate::parser::TexElement;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::time::{Duration, Instant};

/// A single cache entry with access tracking.
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn access(&mut self) -> &T {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.value
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Cache configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_parsed_entries: usize,
    pub max_output_entries: usize,
    pub ttl: Duration,
    pub enable_lru: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_parsed_entries: 100,
            max_output_entries: 50,
            ttl: Duration::from_secs(3600),
            enable_lru: true,
        }
    }
}

/// Statistics for cache performance monitoring.
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub parsed_hits: u64,
    pub parsed_misses: u64,
    pub output_hits: u64,
    pub output_misses: u64,
    pub evictions: u64,
    pub expired_entries: u64,
}

impl CacheStats {
    pub fn parsed_hit_rate(&self) -> f64 {
        let total = self.parsed_hits + self.parsed_misses;
        if total == 0 { 0.0 } else { self.parsed_hits as f64 / total as f64 }
    }

    pub fn output_hit_rate(&self) -> f64 {
        let total = self.output_hits + self.output_misses;
        if total == 0 { 0.0 } else { self.output_hits as f64 / total as f64 }
    }

    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.parsed_hits + self.output_hits;
        let total = total_hits + self.parsed_misses + self.output_misses;
        if total == 0 { 0.0 } else { total_hits as f64 / total as f64 }
    }
}

/// Live cache for parsed documents and rendered output.
#[derive(Debug)]
pub struct DocumentCache {
    parsed: HashMap<u64, CacheEntry<Vec<TexElement>>>,
    output: HashMap<u64, CacheEntry<String>>,
    config: CacheConfig,
    stats: CacheStats,
}

impl DocumentCache {
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            parsed: HashMap::new(),
            output: HashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }

    fn key(input: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    /// Look up a parsed document by source content.
    pub fn get_parsed(&mut self, source: &str) -> Option<Vec<TexElement>> {
        let k = Self::key(source);
        if let Some(e) = self.parsed.get_mut(&k) {
            if e.is_expired(self.config.ttl) {
                self.parsed.remove(&k);
                self.stats.expired_entries += 1;
                self.stats.parsed_misses += 1;
                None
            } else {
                self.stats.parsed_hits += 1;
                Some(e.access().clone())
            }
        } else {
            self.stats.parsed_misses += 1;
            None
        }
    }

    /// Store a parsed document.
    pub fn put_parsed(&mut self, source: &str, elements: Vec<TexElement>) {
        let k = Self::key(source);
        if self.parsed.len() >= self.config.max_parsed_entries {
            self.evict_parsed();
        }
        self.parsed.insert(k, CacheEntry::new(elements));
    }

    /// Look up a rendered output path by source content.
    pub fn get_output(&mut self, source: &str) -> Option<String> {
        let k = Self::key(source);
        if let Some(e) = self.output.get_mut(&k) {
            if e.is_expired(self.config.ttl) {
                self.output.remove(&k);
                self.stats.expired_entries += 1;
                self.stats.output_misses += 1;
                None
            } else {
                self.stats.output_hits += 1;
                Some(e.access().clone())
            }
        } else {
            self.stats.output_misses += 1;
            None
        }
    }

    /// Store a rendered output path.
    pub fn put_output(&mut self, source: &str, path: String) {
        let k = Self::key(source);
        if self.output.len() >= self.config.max_output_entries {
            self.evict_output();
        }
        self.output.insert(k, CacheEntry::new(path));
    }

    fn evict_parsed(&mut self) {
        let selector = if self.config.enable_lru {
            |(_, e): &(_, &CacheEntry<Vec<TexElement>>)| e.last_accessed
        } else {
            |(_, e): &(_, &CacheEntry<Vec<TexElement>>)| e.created_at
        };
        if let Some((&k, _)) = self.parsed.iter().min_by_key(selector) {
            self.parsed.remove(&k);
            self.stats.evictions += 1;
        }
    }

    fn evict_output(&mut self) {
        let selector = if self.config.enable_lru {
            |(_, e): &(_, &CacheEntry<String>)| e.last_accessed
        } else {
            |(_, e): &(_, &CacheEntry<String>)| e.created_at
        };
        if let Some((&k, _)) = self.output.iter().min_by_key(selector) {
            self.output.remove(&k);
            self.stats.evictions += 1;
        }
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.parsed.clear();
        self.output.clear();
        self.stats = CacheStats::default();
    }

    /// Remove expired entries.
    pub fn clear_expired(&mut self) {
        let ttl = self.config.ttl;
        self.parsed.retain(|_, e| {
            let expired = e.is_expired(ttl);
            if expired { self.stats.expired_entries += 1; }
            !expired
        });
        self.output.retain(|_, e| {
            let expired = e.is_expired(ttl);
            if expired { self.stats.expired_entries += 1; }
            !expired
        });
    }

    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn size_info(&self) -> (usize, usize) {
        (self.parsed.len(), self.output.len())
    }

    pub fn update_config(&mut self, config: CacheConfig) {
        self.config = config;
        while self.parsed.len() > self.config.max_parsed_entries {
            self.evict_parsed();
        }
        while self.output.len() > self.config.max_output_entries {
            self.evict_output();
        }
    }
}

impl Default for DocumentCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_miss_then_hit() {
        let mut c = DocumentCache::new();
        let source = "\\documentclass{article}\n\\begin{document}\nX\n\\end{document}";

        assert!(c.get_parsed(source).is_none());
        assert_eq!(c.stats().parsed_misses, 1);

        c.put_parsed(source, vec![]);
        assert!(c.get_parsed(source).is_some());
        assert_eq!(c.stats().parsed_hits, 1);
    }

    #[test]
    fn cache_eviction_when_full() {
        let mut c = DocumentCache::with_config(CacheConfig {
            max_parsed_entries: 2,
            max_output_entries: 2,
            ttl: Duration::from_secs(3600),
            enable_lru: true,
        });

        c.put_parsed("a", vec![]);
        c.put_parsed("b", vec![]);
        c.put_parsed("c", vec![]); // should evict one

        assert_eq!(c.size_info().0, 2);
        assert_eq!(c.stats().evictions, 1);
    }

    #[test]
    fn cache_clear() {
        let mut c = DocumentCache::new();
        c.put_parsed("x", vec![]);
        c.put_output("x", "/tmp/x.pdf".to_string());
        c.clear();
        assert_eq!(c.size_info(), (0, 0));
    }

    #[test]
    fn cache_stats_hit_rate() {
        let mut c = DocumentCache::new();
        c.put_parsed("s", vec![]);

        // One hit, zero misses for parsed so far
        let _ = c.get_parsed("s");
        assert_eq!(c.stats().parsed_hit_rate(), 1.0);

        // One miss
        let _ = c.get_parsed("other");
        assert_eq!(c.stats().parsed_hit_rate(), 0.5);
    }
}
