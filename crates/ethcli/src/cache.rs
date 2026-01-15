//! Signature cache for event and function signatures
//!
//! Provides a hybrid caching system:
//! 1. Hardcoded signatures for common DeFi events/functions (instant)
//! 2. Local JSON file cache for previously looked up signatures (~1ms)
//! 3. Remote fetch from 4byte.directory for unknown signatures (~100-500ms)

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Maximum number of entries before triggering cleanup
const MAX_CACHE_ENTRIES: usize = 10_000;
/// Cleanup interval in number of writes
const CLEANUP_INTERVAL: u64 = 100;

/// Cache entry with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The signature string (e.g., "Transfer(address,address,uint256)")
    pub signature: String,
    /// When this entry was added (Unix timestamp)
    pub timestamp: u64,
}

/// Signature cache data structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheData {
    /// Event signatures by topic0 hash (keccak256)
    pub events: HashMap<String, CacheEntry>,
    /// Function signatures by 4-byte selector
    pub functions: HashMap<String, CacheEntry>,
}

/// Signature cache manager
pub struct SignatureCache {
    /// Cache file path
    path: PathBuf,
    /// In-memory cache data protected by RwLock
    data: RwLock<CacheData>,
    /// Cache TTL (time-to-live)
    ttl: Duration,
    /// Write counter for periodic cleanup
    write_count: AtomicU64,
}

impl SignatureCache {
    /// Create a new cache with default path (~/.cache/eth-log-fetch/signatures.json)
    pub fn new() -> Self {
        let path = Self::default_cache_path();
        Self::with_path(path)
    }

    /// Create a cache with a custom path
    pub fn with_path(path: PathBuf) -> Self {
        let data = Self::load_from_file(&path).unwrap_or_default();
        Self {
            path,
            data: RwLock::new(data),
            ttl: Duration::from_secs(30 * 24 * 60 * 60), // 30 days default
            write_count: AtomicU64::new(0),
        }
    }

    /// Get the default cache directory path
    pub fn default_cache_path() -> PathBuf {
        // Try XDG cache dir first, then fallback to home dir
        if let Some(cache_dir) = dirs::cache_dir() {
            cache_dir.join("eth-log-fetch").join("signatures.json")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".cache")
                .join("eth-log-fetch")
                .join("signatures.json")
        } else {
            // Fallback to current directory
            PathBuf::from(".eth-log-fetch-cache.json")
        }
    }

    /// Load cache from file
    fn load_from_file(path: &PathBuf) -> Option<CacheData> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save cache to file with periodic cleanup
    fn save_to_file(&self) -> Result<(), std::io::Error> {
        // Create parent directories if needed
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Increment write counter and check if cleanup is needed
        let count = self.write_count.fetch_add(1, Ordering::Relaxed);

        // Check if cleanup is needed (every CLEANUP_INTERVAL writes or if cache is too large)
        let needs_cleanup = {
            let data = self.data.read();
            count.is_multiple_of(CLEANUP_INTERVAL)
                || data.events.len() + data.functions.len() > MAX_CACHE_ENTRIES
        };

        if needs_cleanup {
            self.cleanup_internal();
        }

        let data = self.data.read();
        let content = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.path, content)
    }

    /// Internal cleanup without saving (to avoid recursion)
    fn cleanup_internal(&self) {
        let mut data = self.data.write();
        let now = Self::now();
        let ttl_secs = self.ttl.as_secs();

        data.events
            .retain(|_, e| now.saturating_sub(e.timestamp) <= ttl_secs);
        data.functions
            .retain(|_, e| now.saturating_sub(e.timestamp) <= ttl_secs);
    }

    /// Get current Unix timestamp
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Check if an entry is expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        let now = Self::now();
        let age = now.saturating_sub(entry.timestamp);
        age > self.ttl.as_secs()
    }

    /// Get event signature by topic0 hash
    pub fn get_event(&self, topic0: &str) -> Option<String> {
        let data = self.data.read();
        let topic = topic0.to_lowercase();

        if let Some(entry) = data.events.get(&topic) {
            if !self.is_expired(entry) {
                return Some(entry.signature.clone());
            }
        }
        None
    }

    /// Set event signature
    pub fn set_event(&self, topic0: &str, signature: &str) {
        {
            let mut data = self.data.write();
            let topic = topic0.to_lowercase();
            data.events.insert(
                topic,
                CacheEntry {
                    signature: signature.to_string(),
                    timestamp: Self::now(),
                },
            );
        }
        // Best effort save - don't block on I/O errors
        let _ = self.save_to_file();
    }

    /// Get function signature by 4-byte selector
    pub fn get_function(&self, selector: &str) -> Option<String> {
        let data = self.data.read();
        let sel = selector.to_lowercase();

        if let Some(entry) = data.functions.get(&sel) {
            if !self.is_expired(entry) {
                return Some(entry.signature.clone());
            }
        }
        None
    }

    /// Set function signature
    pub fn set_function(&self, selector: &str, signature: &str) {
        {
            let mut data = self.data.write();
            let sel = selector.to_lowercase();
            data.functions.insert(
                sel,
                CacheEntry {
                    signature: signature.to_string(),
                    timestamp: Self::now(),
                },
            );
        }
        // Best effort save - don't block on I/O errors
        let _ = self.save_to_file();
    }

    /// Batch set multiple event signatures
    pub fn set_events_batch(&self, entries: &[(String, String)]) {
        {
            let mut data = self.data.write();
            let now = Self::now();
            for (topic, signature) in entries {
                data.events.insert(
                    topic.to_lowercase(),
                    CacheEntry {
                        signature: signature.clone(),
                        timestamp: now,
                    },
                );
            }
        }
        let _ = self.save_to_file();
    }

    /// Batch set multiple function signatures
    pub fn set_functions_batch(&self, entries: &[(String, String)]) {
        {
            let mut data = self.data.write();
            let now = Self::now();
            for (selector, signature) in entries {
                data.functions.insert(
                    selector.to_lowercase(),
                    CacheEntry {
                        signature: signature.clone(),
                        timestamp: now,
                    },
                );
            }
        }
        let _ = self.save_to_file();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let data = self.data.read();
        let now = Self::now();
        let ttl_secs = self.ttl.as_secs();

        let valid_events = data
            .events
            .values()
            .filter(|e| now.saturating_sub(e.timestamp) <= ttl_secs)
            .count();
        let valid_functions = data
            .functions
            .values()
            .filter(|e| now.saturating_sub(e.timestamp) <= ttl_secs)
            .count();

        CacheStats {
            total_events: data.events.len(),
            valid_events,
            total_functions: data.functions.len(),
            valid_functions,
            cache_path: self.path.clone(),
        }
    }

    /// Clear expired entries
    pub fn cleanup(&self) {
        self.cleanup_internal();
        // Save directly to disk without triggering cleanup again
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data = self.data.read();
        let _ = serde_json::to_string_pretty(&*data)
            .ok()
            .and_then(|content| fs::write(&self.path, content).ok());
    }

    /// Clear all cached data
    pub fn clear(&self) {
        {
            let mut data = self.data.write();
            data.events.clear();
            data.functions.clear();
        }
        let _ = self.save_to_file();
    }
}

impl Default for SignatureCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_events: usize,
    pub valid_events: usize,
    pub total_functions: usize,
    pub valid_functions: usize,
    pub cache_path: PathBuf,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {} events ({} valid), {} functions ({} valid)\nPath: {}",
            self.total_events,
            self.valid_events,
            self.total_functions,
            self.valid_functions,
            self.cache_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_event_cache() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");
        let cache = SignatureCache::with_path(path.clone());

        // Test set and get
        cache.set_event(
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
            "Transfer(address,address,uint256)",
        );

        let sig =
            cache.get_event("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
        assert_eq!(sig, Some("Transfer(address,address,uint256)".to_string()));

        // Test case insensitivity
        let sig =
            cache.get_event("0xDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF");
        assert_eq!(sig, Some("Transfer(address,address,uint256)".to_string()));
    }

    #[test]
    fn test_function_cache() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");
        let cache = SignatureCache::with_path(path);

        cache.set_function("0xa9059cbb", "transfer(address,uint256)");

        let sig = cache.get_function("0xa9059cbb");
        assert_eq!(sig, Some("transfer(address,uint256)".to_string()));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");

        // Create cache and add entry
        {
            let cache = SignatureCache::with_path(path.clone());
            cache.set_event("0xabc123", "TestEvent(uint256)");
        }

        // Create new cache instance and verify persistence
        {
            let cache = SignatureCache::with_path(path);
            let sig = cache.get_event("0xabc123");
            assert_eq!(sig, Some("TestEvent(uint256)".to_string()));
        }
    }

    #[test]
    fn test_batch_operations() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");
        let cache = SignatureCache::with_path(path);

        let events = vec![
            ("0xaaa".to_string(), "EventA(uint256)".to_string()),
            ("0xbbb".to_string(), "EventB(address)".to_string()),
        ];
        cache.set_events_batch(&events);

        assert_eq!(
            cache.get_event("0xaaa"),
            Some("EventA(uint256)".to_string())
        );
        assert_eq!(
            cache.get_event("0xbbb"),
            Some("EventB(address)".to_string())
        );
    }
}
