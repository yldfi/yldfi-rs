//! Token blacklist for filtering spam/scam tokens from portfolio
//!
//! Allows users to blacklist token addresses they don't want to see.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Blacklisted token entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    /// The token contract address (checksummed)
    pub address: String,
    /// Token symbol (for display)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Why this token was blacklisted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Chain where this token exists (e.g., "ethereum", "polygon")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// Blacklist key for an address: trimmed, lowercase, 0x-prefixed
fn normalize_key(address: &str) -> String {
    let lower = address.trim().to_lowercase();
    if lower.starts_with("0x") {
        lower
    } else {
        format!("0x{lower}")
    }
}

/// Token blacklist storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenBlacklist {
    /// Map of lowercase address -> entry
    #[serde(default)]
    pub entries: HashMap<String, BlacklistEntry>,
    /// Set when the file exists but couldn't be read or parsed. Saving is then
    /// refused, so an empty in-memory copy never overwrites the user's file.
    #[serde(skip)]
    load_error: Option<String>,
}

impl TokenBlacklist {
    /// Get the default blacklist file path
    pub fn default_path() -> PathBuf {
        if let Ok(config_dir) = std::env::var("ETHCLI_CONFIG_DIR") {
            return PathBuf::from(config_dir).join("blacklist.toml");
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ethcli")
            .join("blacklist.toml")
    }

    /// Load from default path
    pub fn load_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            match Self::load(&path) {
                Ok(loaded) => loaded,
                Err(e) => {
                    eprintln!("Warning: {e} ({}); it will not be modified", path.display());
                    Self {
                        load_error: Some(e),
                        ..Self::default()
                    }
                }
            }
        } else {
            Self::default()
        }
    }

    /// Load from a specific path
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read blacklist: {}", e))?;

        let mut list: Self =
            toml::from_str(&content).map_err(|e| format!("Failed to parse blacklist: {}", e))?;
        // Hand-edited files may use checksummed or unprefixed addresses
        list.entries = list
            .entries
            .into_iter()
            .map(|(k, v)| (normalize_key(&k), v))
            .collect();
        Ok(list)
    }

    /// Save to default path
    pub fn save_default(&self) -> Result<(), String> {
        self.save(&Self::default_path())
    }

    /// Save to a specific path (locked, atomic). This writes the whole
    /// in-memory copy; to change entries, prefer [`Self::update`], which
    /// applies the change to the file as it is now.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let _lock = crate::utils::locked_file::exclusive_lock(path)
            .map_err(|e| format!("Failed to lock {}: {}", path.display(), e))?;
        self.write_locked(path)
    }

    /// Write the file. The caller holds the exclusive lock.
    fn write_locked(&self, path: &Path) -> Result<(), String> {
        if let Some(e) = &self.load_error {
            return Err(format!(
                "Refusing to overwrite {}: {}. Fix or move the file first",
                path.display(),
                e
            ));
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize: {}", e))?;
        crate::utils::locked_file::atomic_write(path, content.as_bytes())
            .map_err(|e| format!("Failed to write: {}", e))
    }

    /// Load the file under the exclusive lock, apply `f`, and write it back.
    /// Concurrent `ethcli` processes changing the blacklist don't undo each
    /// other's edits. A file that doesn't parse is left alone (error).
    pub fn update<T>(
        path: &Path,
        f: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        let _lock = crate::utils::locked_file::exclusive_lock(path)
            .map_err(|e| format!("Failed to lock {}: {}", path.display(), e))?;
        let mut current = if path.exists() {
            Self::load(path).map_err(|e| {
                format!(
                    "Refusing to overwrite {}: {}. Fix or move the file first",
                    path.display(),
                    e
                )
            })?
        } else {
            Self::default()
        };
        let out = f(&mut current)?;
        current.write_locked(path)?;
        Ok(out)
    }

    /// [`Self::update`] on the default path
    pub fn update_default<T>(f: impl FnOnce(&mut Self) -> Result<T, String>) -> Result<T, String> {
        Self::update(&Self::default_path(), f)
    }

    /// Add a token to the blacklist
    pub fn add(
        &mut self,
        address: &str,
        symbol: Option<String>,
        reason: Option<String>,
        chain: Option<String>,
    ) -> Result<(), String> {
        // Validate and normalize the address
        let parsed = Address::from_str(address)
            .map_err(|_| format!("Invalid Ethereum address: {}", address))?;

        // Store with checksummed address, keyed by lowercase
        let key = normalize_key(address);
        let checksummed = format!("{:?}", parsed);

        let entry = BlacklistEntry {
            address: checksummed,
            symbol,
            reason,
            chain,
        };
        if let Some(e) = &self.load_error {
            return Err(format!("Refusing to change the blacklist: {e}"));
        }
        self.entries.insert(key.clone(), entry.clone());
        Self::update_default(|list| {
            list.entries.insert(key, entry);
            Ok(())
        })
    }

    /// Remove a token from the blacklist
    pub fn remove(&mut self, address: &str) -> Result<bool, String> {
        let key = normalize_key(address);
        if let Some(e) = &self.load_error {
            return Err(format!("Refusing to change the blacklist: {e}"));
        }
        let removed_here = self.entries.remove(&key).is_some();
        if !Self::default_path().exists() {
            return Ok(removed_here);
        }
        let removed_on_disk = Self::update_default(|list| Ok(list.entries.remove(&key).is_some()))?;
        Ok(removed_here || removed_on_disk)
    }

    /// Check if a token is blacklisted
    pub fn is_blacklisted(&self, address: &str) -> bool {
        self.entries.contains_key(&normalize_key(address))
    }

    /// Get a blacklist entry
    pub fn get(&self, address: &str) -> Option<&BlacklistEntry> {
        self.entries.get(&normalize_key(address))
    }

    /// List all blacklisted tokens
    pub fn list(&self) -> Vec<&BlacklistEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| {
            // Sort by symbol if available, otherwise by address
            match (&a.symbol, &b.symbol) {
                (Some(sa), Some(sb)) => sa.to_lowercase().cmp(&sb.to_lowercase()),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.address.cmp(&b.address),
            }
        });
        entries
    }

    /// Get count of blacklisted tokens
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if blacklist is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn keys_are_normalised() {
        assert_eq!(normalize_key(" 0xAbC "), "0xabc");
        assert_eq!(normalize_key("ABC"), "0xabc");
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bl.toml");
        std::fs::write(&path, "[entries.\"0xABC\"]\naddress = \"0xABC\"\n").unwrap();
        let list = TokenBlacklist::load(&path).unwrap();
        assert!(list.is_blacklisted("0xabc"));
        assert!(list.is_blacklisted("ABC"));
    }

    use super::*;

    #[test]
    fn test_add_and_check() {
        let mut blacklist = TokenBlacklist::default();
        blacklist.entries.insert(
            "0x3fc29836e84e471a053d2d9e80494a867d670ead".to_string(),
            BlacklistEntry {
                address: "0x3fC29836E84E471a053D2D9E80494A867D670EAD".to_string(),
                symbol: Some("ETHG".to_string()),
                reason: Some("Spam airdrop".to_string()),
                chain: Some("ethereum".to_string()),
            },
        );

        // Case insensitive check
        assert!(blacklist.is_blacklisted("0x3fC29836E84E471a053D2D9E80494A867D670EAD"));
        assert!(blacklist.is_blacklisted("0x3fc29836e84e471a053d2d9e80494a867d670ead"));
        assert!(!blacklist.is_blacklisted("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));
    }

    #[test]
    fn test_remove() {
        let mut blacklist = TokenBlacklist::default();
        blacklist.entries.insert(
            "0x3fc29836e84e471a053d2d9e80494a867d670ead".to_string(),
            BlacklistEntry {
                address: "0x3fC29836E84E471a053D2D9E80494A867D670EAD".to_string(),
                symbol: Some("ETHG".to_string()),
                reason: None,
                chain: None,
            },
        );

        assert!(blacklist.is_blacklisted("0x3fC29836E84E471a053D2D9E80494A867D670EAD"));
        blacklist
            .entries
            .remove("0x3fc29836e84e471a053d2d9e80494a867d670ead");
        assert!(!blacklist.is_blacklisted("0x3fC29836E84E471a053D2D9E80494A867D670EAD"));
    }
}

#[cfg(test)]
mod load_error_tests {
    use super::*;

    #[test]
    fn unparsable_file_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.toml");
        std::fs::write(&path, "not = [valid").unwrap();
        let err = TokenBlacklist::load(&path).unwrap_err();
        let book = TokenBlacklist {
            load_error: Some(err),
            ..TokenBlacklist::default()
        };
        assert!(book.save(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "not = [valid");
    }
}

#[cfg(test)]
mod locked_update_tests {
    use super::*;

    #[test]
    fn concurrent_updates_keep_every_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blacklist.toml");
        let handles: Vec<_> = (0u8..8)
            .map(|i| {
                let path = path.clone();
                std::thread::spawn(move || {
                    TokenBlacklist::update(&path, |list| {
                        let addr = format!("0x{:040x}", i + 1);
                        list.entries.insert(
                            normalize_key(&addr),
                            BlacklistEntry {
                                address: addr,
                                symbol: None,
                                reason: None,
                                chain: None,
                            },
                        );
                        Ok(())
                    })
                    .unwrap();
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(TokenBlacklist::load(&path).unwrap().entries.len(), 8);
    }
}
