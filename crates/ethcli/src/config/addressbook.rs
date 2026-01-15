//! Address book for storing labeled addresses
//!
//! Allows users to save addresses with custom labels and use them in commands.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Address book entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressEntry {
    /// The Ethereum address
    pub address: String,
    /// Optional description/notes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Chain-specific (if address is only valid on certain chains)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// Address book storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AddressBook {
    /// Map of label -> address entry
    #[serde(default)]
    pub entries: HashMap<String, AddressEntry>,
}

impl AddressBook {
    /// Get the default address book file path
    pub fn default_path() -> PathBuf {
        if let Ok(config_dir) = std::env::var("ETHCLI_CONFIG_DIR") {
            return PathBuf::from(config_dir).join("addressbook.toml");
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ethcli")
            .join("addressbook.toml")
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
            .map_err(|e| format!("Failed to read address book: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("Failed to parse address book: {}", e))
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

    /// Add an address entry
    pub fn add(
        &mut self,
        label: &str,
        address: &str,
        description: Option<String>,
        tags: Vec<String>,
        chain: Option<String>,
    ) -> Result<(), String> {
        // Validate the address
        let _ = Address::from_str(address)
            .map_err(|_| format!("Invalid Ethereum address: {}", address))?;

        // Normalize label to lowercase
        let label = label.to_lowercase();

        self.entries.insert(
            label,
            AddressEntry {
                address: address.to_string(),
                description,
                tags,
                chain,
            },
        );

        self.save_default()
    }

    /// Remove an address entry
    pub fn remove(&mut self, label: &str) -> Result<bool, String> {
        let label = label.to_lowercase();
        let removed = self.entries.remove(&label).is_some();
        if removed {
            self.save_default()?;
        }
        Ok(removed)
    }

    /// Get an address by label
    pub fn get(&self, label: &str) -> Option<&AddressEntry> {
        self.entries.get(&label.to_lowercase())
    }

    /// Resolve a label or address to an address
    /// If the input looks like an address (0x...), returns it as-is
    /// Otherwise, looks up the label in the address book
    pub fn resolve(&self, label_or_address: &str) -> Option<String> {
        // If it looks like an address, return as-is
        if label_or_address.starts_with("0x") && label_or_address.len() == 42 {
            return Some(label_or_address.to_string());
        }

        // Try to resolve as ENS name (ends with .eth) - return None to let caller handle ENS
        if label_or_address.ends_with(".eth") {
            return None;
        }

        // Try to look up in address book
        self.get(label_or_address).map(|e| e.address.clone())
    }

    /// List all entries, optionally filtered by tag
    pub fn list(&self, tag_filter: Option<&str>) -> Vec<(&String, &AddressEntry)> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|(_, entry)| {
                if let Some(tag) = tag_filter {
                    entry.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                } else {
                    true
                }
            })
            .collect();

        // Sort by label
        entries.sort_by(|a, b| a.0.cmp(b.0));
        entries
    }

    /// Search entries by partial label match
    pub fn search(&self, query: &str) -> Vec<(&String, &AddressEntry)> {
        let query = query.to_lowercase();
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|(label, entry)| {
                label.contains(&query)
                    || entry.address.to_lowercase().contains(&query)
                    || entry
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || entry.tags.iter().any(|t| t.to_lowercase().contains(&query))
            })
            .collect();

        entries.sort_by(|a, b| a.0.cmp(b.0));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut book = AddressBook::default();
        book.entries.insert(
            "vitalik".to_string(),
            AddressEntry {
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
                description: Some("Vitalik's address".to_string()),
                tags: vec!["whale".to_string()],
                chain: None,
            },
        );

        assert!(book.get("vitalik").is_some());
        assert!(book.get("VITALIK").is_some()); // Case insensitive
        assert!(book.get("unknown").is_none());
    }

    #[test]
    fn test_resolve() {
        let mut book = AddressBook::default();
        book.entries.insert(
            "usdc".to_string(),
            AddressEntry {
                address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                description: None,
                tags: vec!["token".to_string()],
                chain: None,
            },
        );

        // Raw address returns as-is
        assert_eq!(
            book.resolve("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string())
        );

        // Label resolves to address
        assert_eq!(
            book.resolve("usdc"),
            Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string())
        );

        // ENS names return None (for caller to handle)
        assert_eq!(book.resolve("vitalik.eth"), None);

        // Unknown labels return None
        assert_eq!(book.resolve("unknown"), None);
    }

    #[test]
    fn test_list_with_tag_filter() {
        let mut book = AddressBook::default();
        book.entries.insert(
            "usdc".to_string(),
            AddressEntry {
                address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                description: None,
                tags: vec!["token".to_string(), "stablecoin".to_string()],
                chain: None,
            },
        );
        book.entries.insert(
            "vitalik".to_string(),
            AddressEntry {
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
                description: None,
                tags: vec!["whale".to_string()],
                chain: None,
            },
        );

        assert_eq!(book.list(None).len(), 2);
        assert_eq!(book.list(Some("token")).len(), 1);
        assert_eq!(book.list(Some("whale")).len(), 1);
        assert_eq!(book.list(Some("unknown")).len(), 0);
    }
}
