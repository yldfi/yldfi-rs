//! RPC endpoint configuration

use super::Chain;
use serde::{Deserialize, Serialize};

// ============================================================================
// Default constants for RPC endpoints
// ============================================================================

/// Default maximum block range for getLogs queries
pub const DEFAULT_MAX_BLOCK_RANGE: u64 = 10_000;

/// Default maximum number of logs in a response
pub const DEFAULT_MAX_LOGS: usize = 10_000;

/// Default endpoint priority
pub const DEFAULT_PRIORITY: u8 = 5;

/// Minimum concurrency for transaction/receipt fetching
pub const MIN_TX_FETCH_CONCURRENCY: usize = 5;

/// Node type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// Full archive node - has complete historical state
    Archive,
    /// Full node - only recent blocks (typically 128)
    Full,
    /// Not yet tested/unknown
    #[default]
    Unknown,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Archive => write!(f, "archive"),
            NodeType::Full => write!(f, "full"),
            NodeType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Configuration for a single RPC endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// RPC URL
    pub url: String,
    /// Maximum block range for getLogs
    #[serde(default = "default_max_block_range")]
    pub max_block_range: u64,
    /// Maximum number of logs in response
    #[serde(default = "default_max_logs")]
    pub max_logs: usize,
    /// Priority (higher = preferred)
    #[serde(default = "default_priority")]
    pub priority: u8,
    /// Optional note about the endpoint
    #[serde(default)]
    pub note: Option<String>,
    /// Optional endpoint-specific proxy
    #[serde(default)]
    pub proxy: Option<String>,
    /// Whether this endpoint is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    // === New fields for unified RPC management ===
    /// Node type (archive, full, or unknown)
    #[serde(default)]
    pub node_type: NodeType,
    /// Earliest accessible block (for partial archives or to detect pruning)
    #[serde(default)]
    pub archive_from_block: Option<u64>,
    /// Whether this endpoint supports debug namespace (debug_traceCall, etc.)
    #[serde(default)]
    pub has_debug: bool,
    /// Whether this endpoint supports trace namespace (trace_call, trace_transaction, etc.)
    #[serde(default)]
    pub has_trace: bool,
    /// Which chain this endpoint serves
    #[serde(default)]
    pub chain: Chain,
    /// ISO timestamp of last optimization/test
    #[serde(default)]
    pub last_tested: Option<String>,
}

fn default_max_block_range() -> u64 {
    DEFAULT_MAX_BLOCK_RANGE
}

fn default_max_logs() -> usize {
    DEFAULT_MAX_LOGS
}

fn default_priority() -> u8 {
    DEFAULT_PRIORITY
}

fn default_enabled() -> bool {
    true
}

impl EndpointConfig {
    /// Create a new endpoint config with defaults
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_block_range: default_max_block_range(),
            max_logs: default_max_logs(),
            priority: default_priority(),
            note: None,
            proxy: None,
            enabled: true,
            node_type: NodeType::Unknown,
            archive_from_block: None,
            has_debug: false,
            has_trace: false,
            chain: Chain::default(),
            last_tested: None,
        }
    }

    /// Builder-style setter for max_block_range
    pub fn with_max_block_range(mut self, range: u64) -> Self {
        self.max_block_range = range;
        self
    }

    /// Builder-style setter for max_logs
    pub fn with_max_logs(mut self, max: usize) -> Self {
        self.max_logs = max;
        self
    }

    /// Builder-style setter for priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Builder-style setter for note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Builder-style setter for proxy
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Builder-style setter for node_type
    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    /// Builder-style setter for archive_from_block
    pub fn with_archive_from_block(mut self, block: u64) -> Self {
        self.archive_from_block = Some(block);
        self
    }

    /// Builder-style setter for has_debug
    pub fn with_debug(mut self, has_debug: bool) -> Self {
        self.has_debug = has_debug;
        self
    }

    /// Builder-style setter for has_trace
    pub fn with_trace(mut self, has_trace: bool) -> Self {
        self.has_trace = has_trace;
        self
    }

    /// Builder-style setter for chain
    pub fn with_chain(mut self, chain: Chain) -> Self {
        self.chain = chain;
        self
    }

    /// Builder-style setter for last_tested
    pub fn with_last_tested(mut self, timestamp: impl Into<String>) -> Self {
        self.last_tested = Some(timestamp.into());
        self
    }

    /// Check if endpoint is suitable for a given block range
    pub fn can_handle_range(&self, range: u64) -> bool {
        self.max_block_range == 0 || range <= self.max_block_range
    }

    /// Check if endpoint is suitable for a given log count
    pub fn can_handle_logs(&self, count: usize) -> bool {
        self.max_logs == 0 || count <= self.max_logs
    }

    /// Check if endpoint can serve historical data for a given block
    pub fn can_serve_block(&self, block: u64) -> bool {
        match self.node_type {
            NodeType::Archive => {
                // If archive_from_block is set, check if block is within range
                self.archive_from_block.is_none_or(|from| block >= from)
            }
            NodeType::Full => false, // Full nodes can't serve historical blocks reliably
            NodeType::Unknown => true, // Unknown - assume it might work
        }
    }

    /// Check if this is an archive node
    pub fn is_archive(&self) -> bool {
        self.node_type == NodeType::Archive
    }

    /// Check if this endpoint supports debug namespace (debug_traceCall, etc.)
    pub fn supports_debug(&self) -> bool {
        self.has_debug
    }

    /// Check if this endpoint supports trace namespace (trace_call, etc.)
    pub fn supports_trace(&self) -> bool {
        self.has_trace
    }

    /// Check if this endpoint supports any tracing API (debug or trace)
    pub fn supports_tracing(&self) -> bool {
        self.has_debug || self.has_trace
    }
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self::new("http://localhost:8545")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_config() {
        let config = EndpointConfig::new("https://example.com/rpc")
            .with_max_block_range(1_000_000)
            .with_max_logs(50_000)
            .with_priority(10);

        assert_eq!(config.url, "https://example.com/rpc");
        assert_eq!(config.max_block_range, 1_000_000);
        assert_eq!(config.max_logs, 50_000);
        assert_eq!(config.priority, 10);
        assert!(config.enabled);
    }

    #[test]
    fn test_can_handle_range() {
        let config = EndpointConfig::new("test").with_max_block_range(10_000);

        assert!(config.can_handle_range(5_000));
        assert!(config.can_handle_range(10_000));
        assert!(!config.can_handle_range(15_000));

        // Unlimited
        let unlimited = EndpointConfig::new("test").with_max_block_range(0);
        assert!(unlimited.can_handle_range(1_000_000_000));
    }

    #[test]
    fn test_node_type_and_archive() {
        use crate::config::Chain;

        // Archive node
        let archive = EndpointConfig::new("test")
            .with_node_type(NodeType::Archive)
            .with_chain(Chain::Ethereum);
        assert!(archive.is_archive());
        assert!(archive.can_serve_block(0)); // Can serve genesis
        assert!(archive.can_serve_block(1_000_000));

        // Archive with partial history
        let partial = EndpointConfig::new("test")
            .with_node_type(NodeType::Archive)
            .with_archive_from_block(1_000_000);
        assert!(partial.is_archive());
        assert!(!partial.can_serve_block(0)); // Can't serve genesis
        assert!(partial.can_serve_block(1_000_000)); // Can serve from this block
        assert!(partial.can_serve_block(2_000_000));

        // Full node
        let full = EndpointConfig::new("test").with_node_type(NodeType::Full);
        assert!(!full.is_archive());
        assert!(!full.can_serve_block(0)); // Full nodes can't serve historical

        // Unknown
        let unknown = EndpointConfig::new("test").with_node_type(NodeType::Unknown);
        assert!(!unknown.is_archive());
        assert!(unknown.can_serve_block(0)); // Assume it might work
    }

    #[test]
    fn test_debug_and_trace_support() {
        // Debug only
        let debug_only = EndpointConfig::new("test")
            .with_debug(true)
            .with_trace(false);
        assert!(debug_only.supports_debug());
        assert!(!debug_only.supports_trace());
        assert!(debug_only.supports_tracing());

        // Trace only
        let trace_only = EndpointConfig::new("test")
            .with_debug(false)
            .with_trace(true);
        assert!(!trace_only.supports_debug());
        assert!(trace_only.supports_trace());
        assert!(trace_only.supports_tracing());

        // Both
        let both = EndpointConfig::new("test")
            .with_debug(true)
            .with_trace(true);
        assert!(both.supports_debug());
        assert!(both.supports_trace());
        assert!(both.supports_tracing());

        // Neither
        let neither = EndpointConfig::new("test")
            .with_debug(false)
            .with_trace(false);
        assert!(!neither.supports_debug());
        assert!(!neither.supports_trace());
        assert!(!neither.supports_tracing());
    }

    #[test]
    fn test_can_handle_logs() {
        let config = EndpointConfig::new("test").with_max_logs(10_000);

        assert!(config.can_handle_logs(5_000));
        assert!(config.can_handle_logs(10_000));
        assert!(!config.can_handle_logs(15_000));

        // Unlimited
        let unlimited = EndpointConfig::new("test").with_max_logs(0);
        assert!(unlimited.can_handle_logs(1_000_000_000));
    }

    #[test]
    fn test_builder_chain() {
        use crate::config::Chain;

        let eth = EndpointConfig::new("test").with_chain(Chain::Ethereum);
        assert_eq!(eth.chain, Chain::Ethereum);

        let polygon = EndpointConfig::new("test").with_chain(Chain::Polygon);
        assert_eq!(polygon.chain, Chain::Polygon);

        let arb = EndpointConfig::new("test").with_chain(Chain::Arbitrum);
        assert_eq!(arb.chain, Chain::Arbitrum);
    }

    #[test]
    fn test_last_tested() {
        let config = EndpointConfig::new("test").with_last_tested("2024-12-24T10:00:00Z");
        assert_eq!(config.last_tested, Some("2024-12-24T10:00:00Z".to_string()));
    }
}
