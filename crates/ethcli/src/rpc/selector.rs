//! Smart RPC endpoint selection
//!
//! Provides intelligent endpoint selection based on:
//! - Priority (higher priority endpoints preferred)
//! - Random distribution among same-priority endpoints (load balancing)
//! - Chain filtering
//! - Archive node filtering for historical queries

use crate::config::{Chain, ConfigFile, EndpointConfig};
use crate::rpc::Endpoint;
use rand::seq::SliceRandom;

/// Options for endpoint selection
#[derive(Debug, Clone, Default)]
pub struct SelectionOptions {
    /// Target block number for historical queries.
    /// If set, will prefer archive nodes that can serve this block.
    pub target_block: Option<u64>,
    /// Force archive node selection even for recent blocks
    pub require_archive: bool,
}

impl SelectionOptions {
    /// Create options for a historical query at a specific block
    pub fn for_block(block: u64) -> Self {
        Self {
            target_block: Some(block),
            require_archive: false,
        }
    }

    /// Create options that require an archive node
    pub fn archive() -> Self {
        Self {
            target_block: None,
            require_archive: true,
        }
    }
}

/// Select the best endpoint config from a pre-loaded config
///
/// Selection strategy:
/// 1. Filter endpoints by chain and enabled status
/// 2. If target_block or require_archive is set, filter by archive capability
/// 3. Sort by priority (higher first)
/// 4. Among endpoints with the highest priority, randomly select one
///    (distributes load across equivalent endpoints)
fn select_endpoint_config_with_options(
    config: &ConfigFile,
    chain: Chain,
    options: &SelectionOptions,
) -> anyhow::Result<EndpointConfig> {
    let mut chain_endpoints: Vec<_> = config
        .endpoints
        .iter()
        .filter(|e| e.enabled && e.chain == chain)
        .cloned()
        .collect();

    if chain_endpoints.is_empty() {
        return Err(anyhow::anyhow!(
            "No RPC endpoints configured for {}. Add one with: ethcli endpoints add <url>",
            chain.display_name()
        ));
    }

    // Filter by archive capability if needed
    if options.require_archive || options.target_block.is_some() {
        let archive_endpoints: Vec<_> = chain_endpoints
            .iter()
            .filter(|e| {
                if let Some(block) = options.target_block {
                    e.can_serve_block(block)
                } else {
                    e.is_archive()
                }
            })
            .cloned()
            .collect();

        // Only use archive-filtered list if we found any archive nodes
        // Otherwise fall back to all endpoints (better than failing)
        if !archive_endpoints.is_empty() {
            chain_endpoints = archive_endpoints;
        } else if options.require_archive {
            return Err(anyhow::anyhow!(
                "No archive RPC endpoints configured for {}. \
                 Add an archive node with: ethcli endpoints add <url> --node-type archive",
                chain.display_name()
            ));
        } else if options.target_block.is_some() {
            // Warn user that we're falling back - historical queries may fail
            eprintln!(
                "Warning: No archive nodes configured for {}. Historical query at block {} may fail.",
                chain.display_name(),
                options.target_block.unwrap()
            );
        }
    }

    // Sort by priority (higher priority first)
    chain_endpoints.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Get all endpoints with the highest priority
    let top_priority = chain_endpoints[0].priority;
    let top_endpoints: Vec<_> = chain_endpoints
        .into_iter()
        .filter(|e| e.priority == top_priority)
        .collect();

    // Randomly select one to distribute load
    let selected = if top_endpoints.len() > 1 {
        let mut rng = rand::thread_rng();
        top_endpoints
            .choose(&mut rng)
            .cloned()
            .expect("top_endpoints is not empty")
    } else {
        top_endpoints
            .into_iter()
            .next()
            .expect("top_endpoints is not empty")
    };

    Ok(selected)
}

/// Select the best endpoint config (without archive filtering)
fn select_endpoint_config(config: &ConfigFile, chain: Chain) -> anyhow::Result<EndpointConfig> {
    select_endpoint_config_with_options(config, chain, &SelectionOptions::default())
}

/// Get an RPC endpoint with smart selection
///
/// Selection strategy:
/// 1. Filter endpoints by chain and enabled status
/// 2. Sort by priority (higher first)
/// 3. Among endpoints with the highest priority, randomly select one
///    (distributes load across equivalent endpoints)
///
/// # Arguments
/// * `chain` - The chain to get an endpoint for
///
/// # Returns
/// * `Ok(Endpoint)` - A selected endpoint
/// * `Err` - If no endpoints are configured for the chain
pub fn get_rpc_endpoint(chain: Chain) -> anyhow::Result<Endpoint> {
    let config = ConfigFile::load_default()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
        .unwrap_or_default();

    get_rpc_endpoint_from_config(&config, chain)
}

/// Get an RPC endpoint with archive node preference for historical queries
///
/// When `target_block` is provided, prefers archive nodes that can serve that block.
/// Falls back to regular nodes if no archive nodes are available.
pub fn get_rpc_endpoint_for_block(chain: Chain, target_block: u64) -> anyhow::Result<Endpoint> {
    let config = ConfigFile::load_default()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
        .unwrap_or_default();

    get_rpc_endpoint_from_config_with_options(&config, chain, &SelectionOptions::for_block(target_block))
}

/// Get an RPC endpoint that is guaranteed to be an archive node
///
/// Returns an error if no archive nodes are configured.
pub fn get_archive_endpoint(chain: Chain) -> anyhow::Result<Endpoint> {
    let config = ConfigFile::load_default()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
        .unwrap_or_default();

    get_rpc_endpoint_from_config_with_options(&config, chain, &SelectionOptions::archive())
}

/// Get an RPC endpoint from a pre-loaded config
pub fn get_rpc_endpoint_from_config(config: &ConfigFile, chain: Chain) -> anyhow::Result<Endpoint> {
    let selected = select_endpoint_config(config, chain)?;
    Endpoint::new(selected, 30, None)
        .map_err(|e| anyhow::anyhow!("Failed to create endpoint: {}", e))
}

/// Get an RPC endpoint from a pre-loaded config with selection options
pub fn get_rpc_endpoint_from_config_with_options(
    config: &ConfigFile,
    chain: Chain,
    options: &SelectionOptions,
) -> anyhow::Result<Endpoint> {
    let selected = select_endpoint_config_with_options(config, chain, options)?;
    Endpoint::new(selected, 30, None)
        .map_err(|e| anyhow::anyhow!("Failed to create endpoint: {}", e))
}

/// Get the URL of a smart-selected RPC endpoint
///
/// Useful for commands that need a URL string rather than an Endpoint object
pub fn get_rpc_url(chain: Chain) -> anyhow::Result<String> {
    let config = ConfigFile::load_default()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
        .unwrap_or_default();

    get_rpc_url_from_config(&config, chain)
}

/// Get an RPC URL from a pre-loaded config
pub fn get_rpc_url_from_config(config: &ConfigFile, chain: Chain) -> anyhow::Result<String> {
    let selected = select_endpoint_config(config, chain)?;
    Ok(selected.url)
}

/// Get an RPC URL for a historical block query
pub fn get_rpc_url_for_block(chain: Chain, target_block: u64) -> anyhow::Result<String> {
    let config = ConfigFile::load_default()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
        .unwrap_or_default();

    let selected = select_endpoint_config_with_options(&config, chain, &SelectionOptions::for_block(target_block))?;
    Ok(selected.url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EndpointConfig, NodeType};

    #[test]
    fn test_priority_selection() {
        let mut config = ConfigFile::default();

        let mut low_priority = EndpointConfig::new("https://low.example.com");
        low_priority.priority = 1;
        low_priority.chain = Chain::Ethereum;

        let mut high_priority = EndpointConfig::new("https://high.example.com");
        high_priority.priority = 10;
        high_priority.chain = Chain::Ethereum;

        config.endpoints = vec![low_priority, high_priority];

        // Should always select high priority
        for _ in 0..10 {
            let url = get_rpc_url_from_config(&config, Chain::Ethereum).unwrap();
            assert_eq!(url, "https://high.example.com");
        }
    }

    #[test]
    fn test_archive_selection_for_historical_block() {
        let mut config = ConfigFile::default();

        // Full node (high priority)
        let mut full_node = EndpointConfig::new("https://full.example.com");
        full_node.priority = 10;
        full_node.chain = Chain::Ethereum;
        full_node.node_type = NodeType::Full;

        // Archive node (lower priority)
        let mut archive_node = EndpointConfig::new("https://archive.example.com");
        archive_node.priority = 5;
        archive_node.chain = Chain::Ethereum;
        archive_node.node_type = NodeType::Archive;

        config.endpoints = vec![full_node, archive_node];

        // For recent queries, should select high-priority full node
        let url = get_rpc_url_from_config(&config, Chain::Ethereum).unwrap();
        assert_eq!(url, "https://full.example.com");

        // For historical queries, should select archive node
        let selected = select_endpoint_config_with_options(
            &config,
            Chain::Ethereum,
            &SelectionOptions::for_block(1_000_000),
        )
        .unwrap();
        assert_eq!(selected.url, "https://archive.example.com");
    }

    #[test]
    fn test_archive_selection_fallback() {
        let mut config = ConfigFile::default();

        // Only full nodes available
        let mut full_node = EndpointConfig::new("https://full.example.com");
        full_node.priority = 10;
        full_node.chain = Chain::Ethereum;
        full_node.node_type = NodeType::Full;

        config.endpoints = vec![full_node];

        // For historical queries with no archive nodes, should fall back to full node
        let selected = select_endpoint_config_with_options(
            &config,
            Chain::Ethereum,
            &SelectionOptions::for_block(1_000_000),
        )
        .unwrap();
        assert_eq!(selected.url, "https://full.example.com");
    }

    #[test]
    fn test_require_archive_fails_without_archive() {
        let mut config = ConfigFile::default();

        // Only full nodes available
        let mut full_node = EndpointConfig::new("https://full.example.com");
        full_node.priority = 10;
        full_node.chain = Chain::Ethereum;
        full_node.node_type = NodeType::Full;

        config.endpoints = vec![full_node];

        // require_archive should fail
        let result = select_endpoint_config_with_options(
            &config,
            Chain::Ethereum,
            &SelectionOptions::archive(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("archive"));
    }

    #[test]
    fn test_partial_archive_selection() {
        let mut config = ConfigFile::default();

        // Partial archive (from block 10M)
        let mut partial_archive = EndpointConfig::new("https://partial.example.com");
        partial_archive.priority = 10;
        partial_archive.chain = Chain::Ethereum;
        partial_archive.node_type = NodeType::Archive;
        partial_archive.archive_from_block = Some(10_000_000);

        // Full archive
        let mut full_archive = EndpointConfig::new("https://full-archive.example.com");
        full_archive.priority = 5;
        full_archive.chain = Chain::Ethereum;
        full_archive.node_type = NodeType::Archive;

        config.endpoints = vec![partial_archive.clone(), full_archive.clone()];

        // For block 5M (before partial archive range), should select full archive
        let selected = select_endpoint_config_with_options(
            &config,
            Chain::Ethereum,
            &SelectionOptions::for_block(5_000_000),
        )
        .unwrap();
        assert_eq!(selected.url, "https://full-archive.example.com");

        // For block 15M (within partial archive range), should select partial (higher priority)
        let selected = select_endpoint_config_with_options(
            &config,
            Chain::Ethereum,
            &SelectionOptions::for_block(15_000_000),
        )
        .unwrap();
        assert_eq!(selected.url, "https://partial.example.com");
    }
}
