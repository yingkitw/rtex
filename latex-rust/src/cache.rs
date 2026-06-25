//! Intelligent caching system for LaTeX processing
//!
//! This module provides caching functionality to improve performance
//! by storing parsed ASTs and rendered outputs.

use crate::{
    ast::Document,
    common::{impl_default_config, Stats, Clear},
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::time::{Duration, Instant};

/// Cache entry with timestamp for TTL support
#[derive(Clone, Debug)]
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

/// Cache configuration
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Maximum number of entries in AST cache
    pub max_ast_entries: usize,
    /// Maximum number of entries in HTML cache
    pub max_html_entries: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Enable LRU eviction
    pub enable_lru: bool,
}

impl_default_config!(CacheConfig, {
    max_ast_entries: 100,
    max_html_entries: 50,
    ttl: Duration::from_secs(3600), // 1 hour
    enable_lru: true,
});

/// Intelligent cache for LaTeX processing results
#[derive(Debug)]
pub struct LaTeXCache {
    ast_cache: HashMap<u64, CacheEntry<Document>>,
    html_cache: HashMap<u64, CacheEntry<String>>,
    config: CacheConfig,
    stats: CacheStats,
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub ast_hits: u64,
    pub ast_misses: u64,
    pub html_hits: u64,
    pub html_misses: u64,
    pub evictions: u64,
    pub expired_entries: u64,
}

impl CacheStats {
    /// Get AST cache hit rate
    pub(crate) fn ast_hit_rate(&self) -> f64 {
        if self.ast_hits + self.ast_misses == 0 {
            0.0
        } else {
            self.ast_hits as f64 / (self.ast_hits + self.ast_misses) as f64
        }
    }
    
    /// Get HTML cache hit rate
    pub(crate) fn html_hit_rate(&self) -> f64 {
        if self.html_hits + self.html_misses == 0 {
            0.0
        } else {
            self.html_hits as f64 / (self.html_hits + self.html_misses) as f64
        }
    }
    
    /// Get overall hit rate
    pub(crate) fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.ast_hits + self.html_hits;
        let total_requests = total_hits + self.ast_misses + self.html_misses;
        
        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }
}

impl LaTeXCache {
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }
    
    /// Create a new cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            ast_cache: HashMap::new(),
            html_cache: HashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }
    
    /// Generate cache key from input string
    fn generate_key(input: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get cached AST document
    pub fn get_ast(&mut self, input: &str) -> Option<Document> {
        let key = Self::generate_key(input);
        
        if let Some(entry) = self.ast_cache.get_mut(&key) {
            if entry.is_expired(self.config.ttl) {
                self.ast_cache.remove(&key);
                self.stats.expired_entries += 1;
                self.stats.ast_misses += 1;
                None
            } else {
                self.stats.ast_hits += 1;
                Some(entry.access().clone())
            }
        } else {
            self.stats.ast_misses += 1;
            None
        }
    }
    
    /// Cache AST document
    pub fn put_ast(&mut self, input: &str, document: Document) {
        let key = Self::generate_key(input);
        
        // Check if we need to evict entries
        if self.ast_cache.len() >= self.config.max_ast_entries {
            self.evict_ast_entry();
        }
        
        self.ast_cache.insert(key, CacheEntry::new(document));
    }
    
    /// Get cached HTML output
    pub fn get_html(&mut self, input: &str) -> Option<String> {
        let key = Self::generate_key(input);
        
        if let Some(entry) = self.html_cache.get_mut(&key) {
            if entry.is_expired(self.config.ttl) {
                self.html_cache.remove(&key);
                self.stats.expired_entries += 1;
                self.stats.html_misses += 1;
                None
            } else {
                self.stats.html_hits += 1;
                Some(entry.access().clone())
            }
        } else {
            self.stats.html_misses += 1;
            None
        }
    }
    
    /// Cache HTML output
    pub fn put_html(&mut self, input: &str, html: String) {
        let key = Self::generate_key(input);
        
        // Check if we need to evict entries
        if self.html_cache.len() >= self.config.max_html_entries {
            self.evict_html_entry();
        }
        
        self.html_cache.insert(key, CacheEntry::new(html));
    }
    
    /// Evict least recently used AST entry
    fn evict_ast_entry(&mut self) {
        if self.config.enable_lru {
            if let Some((&key, _)) = self.ast_cache.iter()
                .min_by_key(|(_, entry)| entry.last_accessed) {
                self.ast_cache.remove(&key);
                self.stats.evictions += 1;
            }
        } else {
            // Remove oldest entry
            if let Some((&key, _)) = self.ast_cache.iter()
                .min_by_key(|(_, entry)| entry.created_at) {
                self.ast_cache.remove(&key);
                self.stats.evictions += 1;
            }
        }
    }
    
    /// Evict least recently used HTML entry
    fn evict_html_entry(&mut self) {
        if self.config.enable_lru {
            if let Some((&key, _)) = self.html_cache.iter()
                .min_by_key(|(_, entry)| entry.last_accessed) {
                self.html_cache.remove(&key);
                self.stats.evictions += 1;
            }
        } else {
            // Remove oldest entry
            if let Some((&key, _)) = self.html_cache.iter()
                .min_by_key(|(_, entry)| entry.created_at) {
                self.html_cache.remove(&key);
                self.stats.evictions += 1;
            }
        }
    }
    
    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.ast_cache.clear();
        self.html_cache.clear();
        self.stats = CacheStats::default();
    }
    
    /// Clear expired entries
    pub fn clear_expired(&mut self) {
        let ttl = self.config.ttl;
        
        self.ast_cache.retain(|_, entry| {
            if entry.is_expired(ttl) {
                self.stats.expired_entries += 1;
                false
            } else {
                true
            }
        });
        
        self.html_cache.retain(|_, entry| {
            if entry.is_expired(ttl) {
                self.stats.expired_entries += 1;
                false
            } else {
                true
            }
        });
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
    
    /// Get cache size information
    pub fn size_info(&self) -> CacheSizeInfo {
        CacheSizeInfo {
            ast_entries: self.ast_cache.len(),
            html_entries: self.html_cache.len(),
            max_ast_entries: self.config.max_ast_entries,
            max_html_entries: self.config.max_html_entries,
        }
    }
    
    /// Update cache configuration
    pub fn update_config(&mut self, config: CacheConfig) {
        self.config = config;
        
        // Trim caches if new limits are smaller
        while self.ast_cache.len() > self.config.max_ast_entries {
            self.evict_ast_entry();
        }
        
        while self.html_cache.len() > self.config.max_html_entries {
            self.evict_html_entry();
        }
    }
}

/// Cache size information
#[derive(Debug, Clone)]
pub struct CacheSizeInfo {
    pub ast_entries: usize,
    pub html_entries: usize,
    pub max_ast_entries: usize,
    pub max_html_entries: usize,
}

impl Default for LaTeXCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Stats for LaTeXCache {
    type StatsType = CacheStats;
    
    fn stats(&self) -> &Self::StatsType {
        &self.stats
    }
}

impl Clear for LaTeXCache {
    fn clear(&mut self) {
        self.ast_cache.clear();
        self.html_cache.clear();
        self.stats = CacheStats::default();
    }
}