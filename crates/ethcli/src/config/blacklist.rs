//! Token blacklist for filtering spam/scam tokens from portfolio
//!
//! Allows users to blacklist token addresses they don't want to see.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

/// Token blacklist storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenBlacklist {
    /// Map of lowercase address -> entry
    #[serde(default)]
    pub entries: HashMap<String, BlacklistEntry>,
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
            Self::load(&path).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Load from a specific path
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read blacklist: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("Failed to parse blacklist: {}", e))
    }

    /// Save to default path
    pub fn save_default(&self) -> Result<(), String> {
        self.save(&Self::default_path())
    }

    /// Save to a specific path
    pub fn save(&self, path: &Path) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(path, content).map_err(|e| format!("Failed to write: {}", e))?;

        // Set restrictive permissions (0600) on Unix systems
        #[cfg(unix)]
        {
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, permissions)
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        Ok(())
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
        let key = address.to_lowercase();
        let checksummed = format!("{:?}", parsed);

        self.entries.insert(
            key,
            BlacklistEntry {
                address: checksummed,
                symbol,
                reason,
                chain,
            },
        );

        self.save_default()
    }

    /// Remove a token from the blacklist
    pub fn remove(&mut self, address: &str) -> Result<bool, String> {
        let key = address.to_lowercase();
        let removed = self.entries.remove(&key).is_some();
        if removed {
            self.save_default()?;
        }
        Ok(removed)
    }

    /// Check if a token is blacklisted
    pub fn is_blacklisted(&self, address: &str) -> bool {
        self.entries.contains_key(&address.to_lowercase())
    }

    /// Get a blacklist entry
    pub fn get(&self, address: &str) -> Option<&BlacklistEntry> {
        self.entries.get(&address.to_lowercase())
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
