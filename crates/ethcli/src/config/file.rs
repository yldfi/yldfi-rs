//! Configuration file handling

use super::{EndpointConfig, ProxyConfig};
use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Configuration file structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Global settings
    #[serde(default)]
    pub settings: Settings,

    /// Custom endpoints
    #[serde(default)]
    pub endpoints: Vec<EndpointConfig>,

    /// Disabled endpoints
    #[serde(default)]
    pub disabled_endpoints: DisabledEndpoints,

    /// Proxy configuration
    #[serde(default)]
    pub proxy: Option<ProxyFileConfig>,

    /// Etherscan API key
    #[serde(default)]
    pub etherscan_api_key: Option<String>,

    /// Tenderly configuration
    #[serde(default)]
    pub tenderly: Option<TenderlyConfig>,

    /// Debug-capable RPC endpoints (for debug_traceCall, etc.)
    #[serde(default)]
    pub debug_rpc_urls: Vec<String>,
}

/// Tenderly API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenderlyConfig {
    /// Tenderly access key
    pub access_key: String,
    /// Tenderly account slug
    pub account: String,
    /// Tenderly project slug
    pub project: String,
}

/// Global settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Default concurrency
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Max retry attempts
    #[serde(default = "default_retries")]
    pub retry_attempts: u32,

    /// Checkpoint save interval (blocks)
    #[serde(default = "default_checkpoint_interval")]
    pub checkpoint_interval: u64,
}

fn default_concurrency() -> usize {
    5
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_checkpoint_interval() -> u64 {
    1000
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            timeout_seconds: default_timeout(),
            retry_attempts: default_retries(),
            checkpoint_interval: default_checkpoint_interval(),
        }
    }
}

/// Disabled endpoints configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisabledEndpoints {
    /// List of URLs to disable
    #[serde(default)]
    pub urls: Vec<String>,
}

/// Proxy configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyFileConfig {
    /// Default proxy URL
    #[serde(default)]
    pub default: Option<String>,

    /// Rotate between proxies
    #[serde(default)]
    pub rotate: bool,

    /// File containing proxy URLs
    #[serde(default)]
    pub file: Option<PathBuf>,
}

impl ConfigFile {
    /// Get the default config file path
    ///
    /// Can be overridden by setting the `ETHCLI_CONFIG_DIR` environment variable.
    pub fn default_path() -> PathBuf {
        if let Ok(config_dir) = std::env::var("ETHCLI_CONFIG_DIR") {
            return PathBuf::from(config_dir).join("config.toml");
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ethcli")
            .join("config.toml")
    }

    /// Load from default path
    pub fn load_default() -> Result<Option<Self>> {
        let path = Self::default_path();
        if path.exists() {
            Ok(Some(Self::load(&path)?))
        } else {
            Ok(None)
        }
    }

    /// Load from a specific path
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidFile(format!("{}: {}", path.display(), e)))?;

        let config: Self = toml::from_str(&content).map_err(ConfigError::from)?;
        Ok(config)
    }

    /// Save to a specific path
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigError::InvalidFile(format!("Failed to create directory: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::InvalidFile(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| ConfigError::InvalidFile(format!("Failed to write config: {}", e)))?;

        // Set restrictive permissions (0600) on Unix systems since config may contain API keys
        #[cfg(unix)]
        {
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(path, permissions).map_err(|e| {
                ConfigError::InvalidFile(format!("Failed to set config permissions: {}", e))
            })?;
        }

        Ok(())
    }

    /// Save to default path
    pub fn save_default(&self) -> Result<()> {
        self.save(&Self::default_path())
    }

    /// Check if an endpoint URL is disabled
    pub fn is_endpoint_disabled(&self, url: &str) -> bool {
        self.disabled_endpoints.urls.iter().any(|u| u == url)
    }

    /// Convert proxy config to runtime ProxyConfig
    pub fn proxy_config(&self) -> Option<ProxyConfig> {
        self.proxy.as_ref().map(|p| ProxyConfig {
            url: p.default.clone(),
            file: p.file.clone(),
            rotate_per_request: p.rotate,
        })
    }

    /// Set the Etherscan API key and save
    pub fn set_etherscan_key(&mut self, key: String) -> Result<()> {
        self.etherscan_api_key = Some(key);
        self.save_default()
    }

    /// Set Tenderly credentials and save
    pub fn set_tenderly(
        &mut self,
        access_key: String,
        account: String,
        project: String,
    ) -> Result<()> {
        self.tenderly = Some(TenderlyConfig {
            access_key,
            account,
            project,
        });
        self.save_default()
    }

    /// Add a debug RPC URL and save
    pub fn add_debug_rpc(&mut self, url: String) -> Result<()> {
        if !self.debug_rpc_urls.contains(&url) {
            self.debug_rpc_urls.push(url);
        }
        self.save_default()
    }

    /// Remove a debug RPC URL and save
    pub fn remove_debug_rpc(&mut self, url: &str) -> Result<()> {
        self.debug_rpc_urls.retain(|u| u != url);
        self.save_default()
    }

    /// Update an endpoint's max_block_range (runtime learning)
    pub fn update_endpoint_block_range(&mut self, url: &str, max_block_range: u64) -> Result<bool> {
        if let Some(ep) = self.endpoints.iter_mut().find(|e| e.url == url) {
            // Only update if the new limit is lower (more restrictive)
            if max_block_range < ep.max_block_range {
                ep.max_block_range = max_block_range;
                self.save_default()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Update an endpoint's max_logs (runtime learning)
    pub fn update_endpoint_max_logs(&mut self, url: &str, max_logs: usize) -> Result<bool> {
        if let Some(ep) = self.endpoints.iter_mut().find(|e| e.url == url) {
            // Only update if the new limit is lower (more restrictive)
            if max_logs < ep.max_logs {
                ep.max_logs = max_logs;
                self.save_default()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Disable an endpoint (runtime learning - endpoint is broken)
    pub fn disable_endpoint(&mut self, url: &str) -> Result<bool> {
        if let Some(ep) = self.endpoints.iter_mut().find(|e| e.url == url) {
            if ep.enabled {
                ep.enabled = false;
                self.save_default()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Update block range in memory only (for testing)
    #[cfg(test)]
    pub fn update_endpoint_block_range_in_memory(
        &mut self,
        url: &str,
        max_block_range: u64,
    ) -> bool {
        if let Some(ep) = self.endpoints.iter_mut().find(|e| e.url == url) {
            if max_block_range < ep.max_block_range {
                ep.max_block_range = max_block_range;
                return true;
            }
        }
        false
    }

    /// Update max logs in memory only (for testing)
    #[cfg(test)]
    pub fn update_endpoint_max_logs_in_memory(&mut self, url: &str, max_logs: usize) -> bool {
        if let Some(ep) = self.endpoints.iter_mut().find(|e| e.url == url) {
            if max_logs < ep.max_logs {
                ep.max_logs = max_logs;
                return true;
            }
        }
        false
    }

    /// Disable endpoint in memory only (for testing)
    #[cfg(test)]
    pub fn disable_endpoint_in_memory(&mut self, url: &str) -> bool {
        if let Some(ep) = self.endpoints.iter_mut().find(|e| e.url == url) {
            if ep.enabled {
                ep.enabled = false;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml = r#"
etherscan_api_key = "test_key"

[settings]
concurrency = 10
timeout_seconds = 60

[[endpoints]]
url = "https://example.com/rpc"
max_block_range = 1000000
priority = 10

[disabled_endpoints]
urls = ["https://disabled.com/rpc"]
"#;

        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.settings.concurrency, 10);
        assert_eq!(config.endpoints.len(), 1);
        assert_eq!(config.endpoints[0].url, "https://example.com/rpc");
        assert!(config.is_endpoint_disabled("https://disabled.com/rpc"));
        assert_eq!(config.etherscan_api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_default_path() {
        let path = ConfigFile::default_path();
        assert!(path.to_string_lossy().contains("ethcli"));
    }

    #[test]
    fn test_parse_config_with_new_fields() {
        use crate::config::{Chain, NodeType};

        let toml = r#"
[[endpoints]]
url = "https://example.com/rpc"
max_block_range = 100000
max_logs = 50000
priority = 10
node_type = "archive"
has_debug = true
has_trace = false
chain = "ethereum"
last_tested = "2024-12-24T10:00:00Z"

[[endpoints]]
url = "https://polygon.example.com/rpc"
max_block_range = 50000
priority = 8
node_type = "full"
has_debug = false
has_trace = true
chain = "polygon"
"#;

        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.endpoints.len(), 2);

        // First endpoint - Ethereum archive with debug
        let ep1 = &config.endpoints[0];
        assert_eq!(ep1.chain, Chain::Ethereum);
        assert_eq!(ep1.node_type, NodeType::Archive);
        assert!(ep1.has_debug);
        assert!(!ep1.has_trace);
        assert_eq!(ep1.last_tested, Some("2024-12-24T10:00:00Z".to_string()));

        // Second endpoint - Polygon full node with trace
        let ep2 = &config.endpoints[1];
        assert_eq!(ep2.chain, Chain::Polygon);
        assert_eq!(ep2.node_type, NodeType::Full);
        assert!(!ep2.has_debug);
        assert!(ep2.has_trace);
    }

    #[test]
    fn test_runtime_learning_block_range() {
        let mut config = ConfigFile::default();
        config
            .endpoints
            .push(EndpointConfig::new("https://test.com/rpc"));
        config.endpoints[0].max_block_range = 100000;

        // Should update when new limit is lower
        let updated = config.update_endpoint_block_range_in_memory("https://test.com/rpc", 50000);
        assert!(updated);
        assert_eq!(config.endpoints[0].max_block_range, 50000);

        // Should NOT update when new limit is higher
        let updated = config.update_endpoint_block_range_in_memory("https://test.com/rpc", 75000);
        assert!(!updated);
        assert_eq!(config.endpoints[0].max_block_range, 50000);

        // Should NOT update for unknown endpoint
        let updated = config.update_endpoint_block_range_in_memory("https://unknown.com/rpc", 1000);
        assert!(!updated);
    }

    #[test]
    fn test_runtime_learning_max_logs() {
        let mut config = ConfigFile::default();
        config
            .endpoints
            .push(EndpointConfig::new("https://test.com/rpc"));
        config.endpoints[0].max_logs = 100000;

        // Should update when new limit is lower
        let updated = config.update_endpoint_max_logs_in_memory("https://test.com/rpc", 50000);
        assert!(updated);
        assert_eq!(config.endpoints[0].max_logs, 50000);

        // Should NOT update when new limit is higher
        let updated = config.update_endpoint_max_logs_in_memory("https://test.com/rpc", 75000);
        assert!(!updated);
        assert_eq!(config.endpoints[0].max_logs, 50000);
    }

    #[test]
    fn test_disable_endpoint_in_memory() {
        let mut config = ConfigFile::default();
        config
            .endpoints
            .push(EndpointConfig::new("https://test.com/rpc"));
        assert!(config.endpoints[0].enabled);

        // Should disable
        let disabled = config.disable_endpoint_in_memory("https://test.com/rpc");
        assert!(disabled);
        assert!(!config.endpoints[0].enabled);

        // Should not disable again
        let disabled = config.disable_endpoint_in_memory("https://test.com/rpc");
        assert!(!disabled);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = ConfigFile::load(Path::new("/nonexistent/path/config.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("nonexistent"));
    }

    #[test]
    fn test_load_invalid_toml_syntax() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "this is not valid { toml [syntax").unwrap();

        let result = ConfigFile::load(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_field_types() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        // concurrency should be a number, not a string
        std::fs::write(
            &config_path,
            r#"
[settings]
concurrency = "not_a_number"
"#,
        )
        .unwrap();

        let result = ConfigFile::load(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_enum_value() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        // "invalid_chain" is not a valid Chain variant
        std::fs::write(
            &config_path,
            r#"
[[endpoints]]
url = "https://example.com/rpc"
chain = "invalid_chain"
"#,
        )
        .unwrap();

        let result = ConfigFile::load(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").unwrap();

        // Empty file should parse successfully with defaults
        let result = ConfigFile::load(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.endpoints.is_empty());
        assert_eq!(config.settings.concurrency, 5); // default
    }

    #[test]
    fn test_load_partial_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        // Only specify some settings, others should use defaults
        std::fs::write(
            &config_path,
            r#"
[settings]
concurrency = 20
"#,
        )
        .unwrap();

        let result = ConfigFile::load(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.settings.concurrency, 20);
        assert_eq!(config.settings.timeout_seconds, 30); // default
        assert_eq!(config.settings.retry_attempts, 3); // default
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = ConfigFile::default();
        config.etherscan_api_key = Some("test_key_123".to_string());
        config.settings.concurrency = 15;
        config
            .endpoints
            .push(EndpointConfig::new("https://test.example.com/rpc"));

        // Save
        config.save(&config_path).unwrap();

        // Load and verify
        let loaded = ConfigFile::load(&config_path).unwrap();
        assert_eq!(loaded.etherscan_api_key, Some("test_key_123".to_string()));
        assert_eq!(loaded.settings.concurrency, 15);
        assert_eq!(loaded.endpoints.len(), 1);
        assert_eq!(loaded.endpoints[0].url, "https://test.example.com/rpc");
    }

    #[test]
    fn test_save_creates_parent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("config.toml");

        let config = ConfigFile::default();
        let result = config.save(&config_path);
        assert!(result.is_ok());
        assert!(config_path.exists());
    }
}
