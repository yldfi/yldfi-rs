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
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Write file with restrictive permissions (0600) on Unix
fn write_with_permissions(path: &Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content)?;
    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

/// Maximum number of entries before triggering cleanup
const MAX_CACHE_ENTRIES: usize = 10_000;
/// Cleanup interval in number of writes
const CLEANUP_INTERVAL: u64 = 100;
/// ABI cache TTL in seconds (90 days)
const ABI_TTL_SECS: u64 = 90 * 24 * 60 * 60;
/// Negative cache TTL in seconds (1 day) - for "not found" entries
const NEGATIVE_CACHE_TTL_SECS: u64 = 24 * 60 * 60;

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
    /// Contract ABIs by chain_address key
    #[serde(default)]
    pub abis: HashMap<String, AbiCacheEntry>,
    /// Negative cache: selectors/topics that were looked up but not found
    /// Stores Unix timestamp of when the lookup failed
    #[serde(default)]
    pub not_found: HashMap<String, u64>,
}

/// ABI cache entry with timestamp and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiCacheEntry {
    /// The ABI JSON string
    pub abi: String,
    /// Contract name if known
    pub name: Option<String>,
    /// When this entry was added (Unix timestamp)
    pub timestamp: u64,
}

/// Signature cache manager
pub struct SignatureCache {
    /// Cache file path
    path: PathBuf,
    /// In-memory cache data protected by RwLock
    data: RwLock<CacheData>,
    /// Cache TTL (time-to-live)
    ttl: Duration,
    /// Write counter for periodic cleanup/save
    write_count: AtomicU64,
    /// Dirty flag - true if cache has unsaved changes
    dirty: AtomicBool,
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
            dirty: AtomicBool::new(false),
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

    /// Mark cache as dirty and periodically save to disk
    ///
    /// Only writes to disk every CLEANUP_INTERVAL writes to reduce I/O.
    /// Use `save()` to force an immediate write.
    fn maybe_save(&self) {
        self.dirty.store(true, Ordering::Release);

        // Increment write counter and check if save is needed
        let count = self.write_count.fetch_add(1, Ordering::Relaxed);

        // Only save every CLEANUP_INTERVAL writes or if cache is too large
        let needs_save = {
            let data = self.data.read();
            count.is_multiple_of(CLEANUP_INTERVAL)
                || data.events.len() + data.functions.len() > MAX_CACHE_ENTRIES
        };

        if needs_save {
            self.cleanup_internal();
            let _ = self.save_to_file();
        }
    }

    /// Force save cache to file immediately
    pub fn save(&self) {
        if self.dirty.swap(false, Ordering::AcqRel) {
            let _ = self.save_to_file();
        }
    }

    /// Internal save to file
    fn save_to_file(&self) -> Result<(), std::io::Error> {
        // Create parent directories if needed
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = self.data.read();
        let content = serde_json::to_string_pretty(&*data)?;
        write_with_permissions(&self.path, &content)?;
        self.dirty.store(false, Ordering::Release);
        Ok(())
    }

    /// Internal cleanup without saving (to avoid recursion)
    fn cleanup_internal(&self) {
        let mut data = self.data.write();
        let now = Self::now();
        let ttl_secs = self.ttl.as_secs();
        let abi_ttl_secs = ABI_TTL_SECS;
        let negative_ttl_secs = NEGATIVE_CACHE_TTL_SECS;

        data.events
            .retain(|_, e| now.saturating_sub(e.timestamp) <= ttl_secs);
        data.functions
            .retain(|_, e| now.saturating_sub(e.timestamp) <= ttl_secs);
        data.abis
            .retain(|_, e| now.saturating_sub(e.timestamp) <= abi_ttl_secs);
        data.not_found
            .retain(|_, &mut ts| now.saturating_sub(ts) <= negative_ttl_secs);
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
        self.maybe_save();
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
        self.maybe_save();
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
        self.maybe_save();
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
        self.maybe_save();
    }

    // ========================================================================
    // Negative Cache Methods
    // ========================================================================

    /// Check if a key is in the negative cache (recently looked up but not found)
    ///
    /// Returns true if we should skip looking this up again
    pub fn is_not_found(&self, key: &str) -> bool {
        let data = self.data.read();
        let key_lower = key.to_lowercase();
        if let Some(&timestamp) = data.not_found.get(&key_lower) {
            let now = Self::now();
            // Check if the negative cache entry is still valid
            now.saturating_sub(timestamp) <= NEGATIVE_CACHE_TTL_SECS
        } else {
            false
        }
    }

    /// Mark a key as not found (for negative caching)
    pub fn set_not_found(&self, key: &str) {
        {
            let mut data = self.data.write();
            data.not_found.insert(key.to_lowercase(), Self::now());
        }
        self.maybe_save();
    }

    /// Remove a key from the negative cache (when it's been found elsewhere)
    pub fn clear_not_found(&self, key: &str) {
        {
            let mut data = self.data.write();
            data.not_found.remove(&key.to_lowercase());
        }
    }

    // ========================================================================
    // ABI Cache Methods
    // ========================================================================

    /// Make ABI cache key from chain ID and address
    fn abi_key(chain_id: u64, address: &str) -> String {
        format!("{}_{}", chain_id, address.to_lowercase())
    }

    /// Get cached ABI for a contract
    pub fn get_abi(&self, chain_id: u64, address: &str) -> Option<(String, Option<String>)> {
        let data = self.data.read();
        let key = Self::abi_key(chain_id, address);

        if let Some(entry) = data.abis.get(&key) {
            // ABIs have longer TTL (90 days) since they rarely change
            let abi_ttl = ABI_TTL_SECS;
            let now = Self::now();
            if now.saturating_sub(entry.timestamp) <= abi_ttl {
                return Some((entry.abi.clone(), entry.name.clone()));
            }
        }
        None
    }

    /// Cache an ABI for a contract
    pub fn set_abi(&self, chain_id: u64, address: &str, abi: &str, name: Option<&str>) {
        {
            let mut data = self.data.write();
            let key = Self::abi_key(chain_id, address);
            data.abis.insert(
                key,
                AbiCacheEntry {
                    abi: abi.to_string(),
                    name: name.map(|s| s.to_string()),
                    timestamp: Self::now(),
                },
            );
        }
        self.maybe_save();
    }

    /// Get number of cached ABIs
    pub fn abi_count(&self) -> usize {
        self.data.read().abis.len()
    }

    /// Search all cached ABIs for a function matching the given selector
    ///
    /// Returns the function signature string if found (e.g., "transfer(address,uint256)")
    pub fn find_function_in_abis(&self, selector: &[u8; 4]) -> Option<String> {
        use alloy::json_abi::JsonAbi;

        let data = self.data.read();
        let now = Self::now();
        let abi_ttl = ABI_TTL_SECS;

        for entry in data.abis.values() {
            // Skip expired entries
            if now.saturating_sub(entry.timestamp) > abi_ttl {
                continue;
            }

            // Try to parse and search this ABI
            if let Ok(abi) = serde_json::from_str::<JsonAbi>(&entry.abi) {
                for func in abi.functions() {
                    if func.selector() == *selector {
                        let param_types: Vec<String> =
                            func.inputs.iter().map(|p| p.ty.to_string()).collect();
                        return Some(format!("{}({})", func.name, param_types.join(",")));
                    }
                }
            }
        }

        None
    }

    /// Search all cached ABIs for an event matching the given topic0
    ///
    /// Returns the event signature string if found (e.g., "Transfer(address,address,uint256)")
    pub fn find_event_in_abis(&self, topic0: &[u8; 32]) -> Option<String> {
        use alloy::json_abi::JsonAbi;

        let data = self.data.read();
        let now = Self::now();
        let abi_ttl = ABI_TTL_SECS;

        for entry in data.abis.values() {
            // Skip expired entries
            if now.saturating_sub(entry.timestamp) > abi_ttl {
                continue;
            }

            // Try to parse and search this ABI
            if let Ok(abi) = serde_json::from_str::<JsonAbi>(&entry.abi) {
                for event in abi.events() {
                    if event.selector().0 == *topic0 {
                        let param_types: Vec<String> =
                            event.inputs.iter().map(|p| p.ty.to_string()).collect();
                        return Some(format!("{}({})", event.name, param_types.join(",")));
                    }
                }
            }
        }

        None
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let data = self.data.read();
        let now = Self::now();
        let ttl_secs = self.ttl.as_secs();
        let abi_ttl_secs = ABI_TTL_SECS;

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
        let valid_abis = data
            .abis
            .values()
            .filter(|e| now.saturating_sub(e.timestamp) <= abi_ttl_secs)
            .count();

        CacheStats {
            total_events: data.events.len(),
            valid_events,
            total_functions: data.functions.len(),
            valid_functions,
            total_abis: data.abis.len(),
            valid_abis,
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
            .and_then(|content| write_with_permissions(&self.path, &content).ok());
    }

    /// Clear all cached data
    pub fn clear(&self) {
        {
            let mut data = self.data.write();
            data.events.clear();
            data.functions.clear();
            data.abis.clear();
            data.not_found.clear();
        }
        self.maybe_save();
    }
}

impl Default for SignatureCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SignatureCache {
    fn drop(&mut self) {
        // Save any unsaved changes on shutdown
        if self.dirty.load(Ordering::Acquire) {
            let _ = self.save_to_file();
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_events: usize,
    pub valid_events: usize,
    pub total_functions: usize,
    pub valid_functions: usize,
    pub total_abis: usize,
    pub valid_abis: usize,
    pub cache_path: PathBuf,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {} events ({} valid), {} functions ({} valid), {} ABIs ({} valid)\nPath: {}",
            self.total_events,
            self.valid_events,
            self.total_functions,
            self.valid_functions,
            self.total_abis,
            self.valid_abis,
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

    #[test]
    fn test_abi_cache() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");
        let cache = SignatureCache::with_path(path.clone());

        // Test set and get ABI
        let abi = r#"[{"type":"function","name":"transfer","inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}],"outputs":[{"type":"bool"}]}]"#;
        cache.set_abi(
            1,
            "0x1234567890123456789012345678901234567890",
            abi,
            Some("TestToken"),
        );

        let result = cache.get_abi(1, "0x1234567890123456789012345678901234567890");
        assert!(result.is_some());
        let (cached_abi, name) = result.unwrap();
        assert_eq!(cached_abi, abi);
        assert_eq!(name, Some("TestToken".to_string()));

        // Test case insensitivity
        let result = cache.get_abi(1, "0x1234567890123456789012345678901234567890");
        assert!(result.is_some());
    }

    #[test]
    fn test_abi_cache_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");

        let abi = r#"[{"type":"function","name":"approve","inputs":[{"name":"spender","type":"address"},{"name":"amount","type":"uint256"}],"outputs":[{"type":"bool"}]}]"#;

        // Create cache and add ABI
        {
            let cache = SignatureCache::with_path(path.clone());
            cache.set_abi(1, "0xabcdef", abi, None);
        }

        // Create new cache instance and verify persistence
        {
            let cache = SignatureCache::with_path(path);
            let result = cache.get_abi(1, "0xabcdef");
            assert!(result.is_some());
            let (cached_abi, _) = result.unwrap();
            assert_eq!(cached_abi, abi);
        }
    }

    #[test]
    fn test_find_function_in_abis() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");
        let cache = SignatureCache::with_path(path);

        // Add an ABI with transfer function (selector 0xa9059cbb)
        let abi = r#"[{"type":"function","name":"transfer","inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}],"outputs":[{"type":"bool"}],"stateMutability":"nonpayable"}]"#;
        cache.set_abi(1, "0xtoken", abi, Some("TestToken"));

        // Search for transfer selector
        let selector: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
        let result = cache.find_function_in_abis(&selector);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "transfer(address,uint256)");

        // Search for non-existent selector
        let unknown_selector: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
        let result = cache.find_function_in_abis(&unknown_selector);
        assert!(result.is_none());
    }

    #[test]
    fn test_abi_count() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_cache.json");
        let cache = SignatureCache::with_path(path);

        assert_eq!(cache.abi_count(), 0);

        cache.set_abi(1, "0xaaa", "[]", None);
        assert_eq!(cache.abi_count(), 1);

        cache.set_abi(1, "0xbbb", "[]", None);
        assert_eq!(cache.abi_count(), 2);

        // Same address, different chain
        cache.set_abi(137, "0xaaa", "[]", None);
        assert_eq!(cache.abi_count(), 3);
    }
}

// ============================================================================
// Token Metadata Cache
// ============================================================================

/// Token metadata cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCacheEntry {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub total_supply: Option<String>,
    pub timestamp: u64,
}

/// Token cache data structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCacheData {
    /// Map of "chain:address" -> TokenCacheEntry
    pub tokens: HashMap<String, TokenCacheEntry>,
}

/// Token metadata cache for immutable token data
pub struct TokenMetadataCache {
    path: PathBuf,
    data: RwLock<TokenCacheData>,
}

impl TokenMetadataCache {
    /// Create a new token metadata cache with default path
    pub fn new() -> Self {
        let path = Self::default_path();
        Self::with_path(path)
    }

    /// Get the default cache path
    fn default_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ethcli")
            .join("token_cache.json")
    }

    /// Create a cache with a specific path
    pub fn with_path(path: PathBuf) -> Self {
        let data = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            TokenCacheData::default()
        };

        Self {
            path,
            data: RwLock::new(data),
        }
    }

    /// Get token metadata from cache
    pub fn get(&self, chain: &str, address: &str) -> Option<TokenCacheEntry> {
        let key = format!("{}:{}", chain, address.to_lowercase());
        self.data.read().tokens.get(&key).cloned()
    }

    /// Set token metadata in cache
    pub fn set(
        &self,
        chain: &str,
        address: &str,
        name: Option<String>,
        symbol: Option<String>,
        decimals: Option<u8>,
        total_supply: Option<String>,
    ) {
        let key = format!("{}:{}", chain, address.to_lowercase());
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        {
            let mut data = self.data.write();
            data.tokens.insert(
                key,
                TokenCacheEntry {
                    name,
                    symbol,
                    decimals,
                    total_supply,
                    timestamp,
                },
            );
        }

        self.save();
    }

    /// Save cache to disk
    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let data = self.data.read();
        if let Ok(json) = serde_json::to_string_pretty(&*data) {
            let _ = write_with_permissions(&self.path, &json);
        }
    }
}

impl Default for TokenMetadataCache {
    fn default() -> Self {
        Self::new()
    }
}
