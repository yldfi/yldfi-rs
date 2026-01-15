//! Smart RPC endpoint selection
//!
//! Provides intelligent endpoint selection based on:
//! - Priority (higher priority endpoints preferred)
//! - Random distribution among same-priority endpoints (load balancing)
//! - Chain filtering

use crate::config::{Chain, ConfigFile};
use crate::rpc::Endpoint;
use rand::seq::SliceRandom;

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

/// Get an RPC endpoint from a pre-loaded config
pub fn get_rpc_endpoint_from_config(config: &ConfigFile, chain: Chain) -> anyhow::Result<Endpoint> {
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

    Ok(selected.url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EndpointConfig;

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
}
