//! Smart RPC endpoint selection
//!
//! Provides intelligent endpoint selection based on:
//! - Priority (higher priority endpoints preferred)
//! - Node classification (tested endpoints preferred over unknown)
//! - General-purpose RPC support (restricted relay RPCs are fallback-only)
//! - Random distribution among same-priority endpoints (load balancing)
//! - Chain filtering
//! - Archive node filtering for historical queries
//! - Failover across ranked candidates on rate limits / transport errors

use crate::config::{Chain, ConfigFile, EndpointConfig, NodeType};
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

/// Rank all usable endpoint configs for `chain`, best first.
///
/// Ranking strategy:
/// 1. Filter endpoints by chain and enabled status
/// 2. If target_block or require_archive is set, filter by archive capability
/// 3. Prefer tested full/archive endpoints over unknown endpoints when available
/// 4. Treat restricted relay RPCs as fallback-only
/// 5. Sort by priority (higher first). Full and archive nodes are treated
///    equally for latest-state reads, so a single FULL node cannot outrank
///    healthy ARCHIVE nodes of equal or higher priority.
/// 6. Shuffle within each priority tier (distributes load across equivalent
///    endpoints, and means a rate-limited node is not always tried first)
pub fn ranked_endpoint_configs(
    config: &ConfigFile,
    chain: Chain,
    options: &SelectionOptions,
) -> anyhow::Result<Vec<EndpointConfig>> {
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
        } else if let Some(target_block) = options.target_block {
            // Warn user that we're falling back - historical queries may fail
            eprintln!(
                "Warning: No archive nodes configured for {}. Historical query at block {} may fail.",
                chain.display_name(),
                target_block
            );
        }
    }

    // Unknown endpoints have not been classified by endpoint health/optimization.
    // Do not let an untested high-priority URL beat known-good full/archive nodes.
    let known_endpoints: Vec<_> = chain_endpoints
        .iter()
        .filter(|e| e.node_type != NodeType::Unknown)
        .cloned()
        .collect();
    if !known_endpoints.is_empty() {
        chain_endpoints = known_endpoints;
    }

    // Relay-oriented RPCs may handle basic methods but reject eth_call or other
    // general-purpose reads. Keep them only if there is no better option.
    let unrestricted_endpoints: Vec<_> = chain_endpoints
        .iter()
        .filter(|e| !is_restricted_rpc_url(&e.url))
        .cloned()
        .collect();
    if !unrestricted_endpoints.is_empty() {
        chain_endpoints = unrestricted_endpoints;
    }

    // Shuffle first, then stable-sort by priority: equal-priority endpoints
    // end up in random order within their tier.
    chain_endpoints.shuffle(&mut rand::thread_rng());
    chain_endpoints.sort_by_key(|endpoint| std::cmp::Reverse(endpoint.priority));

    Ok(chain_endpoints)
}

/// Select the best endpoint config from a pre-loaded config
fn select_endpoint_config_with_options(
    config: &ConfigFile,
    chain: Chain,
    options: &SelectionOptions,
) -> anyhow::Result<EndpointConfig> {
    ranked_endpoint_configs(config, chain, options)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No RPC endpoints available"))
}

/// Maximum number of endpoints tried by [`with_failover`].
pub const MAX_FAILOVER_ATTEMPTS: usize = 5;

/// Whether an RPC error is worth retrying on a different endpoint
/// (rate limits, capacity errors, transport/timeouts, gateway errors).
pub fn is_failover_error(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    [
        "429",
        "-32005", // limit exceeded (Infura/Alchemy style)
        "-32029", // too many requests (OnFinality)
        "-32090", // rate limited (some providers)
        "rate limit",
        "rate-limit",
        "too many requests",
        "exceeded",
        "capacity",
        "timeout",
        "timed out",
        "error sending request",
        "connection",
        "dns error",
        "tls",
        "502",
        "503",
        "504",
        "not whitelisted",
        "method not allowed",
        "unauthorized",
        "403",
        "401",
    ]
    .iter()
    .any(|needle| m.contains(needle))
}

/// Build up to `max` ranked endpoints for `chain` (see [`ranked_endpoint_configs`]).
pub fn candidate_endpoints(
    chain: Chain,
    options: &SelectionOptions,
    max: usize,
) -> anyhow::Result<Vec<Endpoint>> {
    let config = ConfigFile::load_default()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
        .unwrap_or_default();
    ranked_endpoint_configs(&config, chain, options)?
        .into_iter()
        .take(max)
        .map(|c| {
            Endpoint::new(c, 30, None)
                .map_err(|e| anyhow::anyhow!("Failed to create endpoint: {}", e))
        })
        .collect()
}

/// Run `op` against ranked endpoints until one succeeds.
///
/// Moves on to the next candidate only for [`is_failover_error`] errors; any
/// other error (bad input, revert, ...) is returned immediately.
pub async fn with_failover<T, F, Fut>(candidates: Vec<Endpoint>, mut op: F) -> anyhow::Result<T>
where
    F: FnMut(Endpoint) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let total = candidates.len();
    let mut last_err = None;
    for (i, endpoint) in candidates.into_iter().enumerate() {
        let host = crate::utils::url::redact_url(endpoint.url());
        match op(endpoint).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                let msg = format!("{e:#}");
                if i + 1 < total && is_failover_error(&msg) {
                    tracing::warn!("RPC error on {host}; trying next endpoint");
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No RPC endpoints available")))
}

/// Select the best endpoint config (without archive filtering)
fn select_endpoint_config(config: &ConfigFile, chain: Chain) -> anyhow::Result<EndpointConfig> {
    select_endpoint_config_with_options(config, chain, &SelectionOptions::default())
}

fn is_restricted_rpc_url(url: &str) -> bool {
    let url = url.to_ascii_lowercase();
    url.contains("flashbots.net")
}

/// Get an RPC endpoint with smart selection
///
/// Selection strategy:
/// 1. Filter endpoints by chain and enabled status
/// 2. Prefer tested full/archive endpoints over unknown endpoints when available
/// 3. Treat restricted relay RPCs as fallback-only
/// 4. For latest-state reads, prefer full nodes over archive nodes when both exist
/// 5. Sort by priority (higher first)
/// 6. Among endpoints with the highest priority, randomly select one
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

    get_rpc_endpoint_from_config_with_options(
        &config,
        chain,
        &SelectionOptions::for_block(target_block),
    )
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

    let selected = select_endpoint_config_with_options(
        &config,
        chain,
        &SelectionOptions::for_block(target_block),
    )?;
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
    fn test_known_endpoint_beats_unknown_higher_priority() {
        let mut config = ConfigFile::default();

        let mut unknown = EndpointConfig::new("https://unknown.example.com");
        unknown.priority = 10;
        unknown.chain = Chain::Ethereum;
        unknown.node_type = NodeType::Unknown;

        let mut full = EndpointConfig::new("https://full.example.com");
        full.priority = 5;
        full.chain = Chain::Ethereum;
        full.node_type = NodeType::Full;

        config.endpoints = vec![unknown, full];

        let url = get_rpc_url_from_config(&config, Chain::Ethereum).unwrap();
        assert_eq!(url, "https://full.example.com");
    }

    #[test]
    fn test_lower_priority_full_does_not_outrank_archive() {
        let mut config = ConfigFile::default();

        let mut archive = EndpointConfig::new("https://archive.example.com");
        archive.priority = 10;
        archive.chain = Chain::Ethereum;
        archive.node_type = NodeType::Archive;

        let mut full = EndpointConfig::new("https://full.example.com");
        full.priority = 5;
        full.chain = Chain::Ethereum;
        full.node_type = NodeType::Full;

        config.endpoints = vec![archive, full];

        for _ in 0..10 {
            let url = get_rpc_url_from_config(&config, Chain::Ethereum).unwrap();
            assert_eq!(url, "https://archive.example.com");
        }
    }

    #[test]
    fn test_single_full_node_does_not_always_win_tier() {
        // One FULL and several ARCHIVE nodes at the same priority: the FULL
        // node must not be selected every time (it may be rate limited).
        let mut config = ConfigFile::default();
        let mut full = EndpointConfig::new("https://full.example.com");
        full.chain = Chain::Ethereum;
        full.node_type = NodeType::Full;
        full.priority = 5;
        config.endpoints.push(full);
        for i in 0..3 {
            let mut a = EndpointConfig::new(format!("https://archive{i}.example.com"));
            a.chain = Chain::Ethereum;
            a.node_type = NodeType::Archive;
            a.priority = 5;
            config.endpoints.push(a);
        }
        let picked_archive = (0..200).any(|_| {
            get_rpc_url_from_config(&config, Chain::Ethereum)
                .unwrap()
                .contains("archive")
        });
        assert!(picked_archive);
    }

    #[test]
    fn test_ranked_candidates_ordered_by_priority() {
        let mut config = ConfigFile::default();
        for (url, prio) in [
            ("https://a.io", 1u8),
            ("https://b.io", 9),
            ("https://c.io", 5),
        ] {
            let mut e = EndpointConfig::new(url);
            e.chain = Chain::Ethereum;
            e.node_type = NodeType::Archive;
            e.priority = prio;
            config.endpoints.push(e);
        }
        let ranked: Vec<_> =
            ranked_endpoint_configs(&config, Chain::Ethereum, &SelectionOptions::default())
                .unwrap()
                .into_iter()
                .map(|e| e.url)
                .collect();
        assert_eq!(ranked, vec!["https://b.io", "https://c.io", "https://a.io"]);
    }

    #[test]
    fn test_is_failover_error() {
        assert!(is_failover_error(
            "HTTP error 429 with body: Too Many Requests"
        ));
        assert!(is_failover_error("{\"code\":-32005,\"message\":\"limit\"}"));
        assert!(is_failover_error("error sending request for url"));
        assert!(!is_failover_error("execution reverted"));
        assert!(!is_failover_error("Invalid address"));
    }

    #[tokio::test]
    async fn test_with_failover_moves_on_after_rate_limit() {
        let mk = |u: &str| Endpoint::new(EndpointConfig::new(u), 5, None).unwrap();
        let eps = vec![mk("https://one.example.com"), mk("https://two.example.com")];
        let mut seen = Vec::new();
        let out = with_failover(eps, |ep| {
            seen.push(ep.url().to_string());
            let url = ep.url().to_string();
            async move {
                if url.contains("one") {
                    Err(anyhow::anyhow!("HTTP error 429"))
                } else {
                    Ok(42)
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(out, 42);
        assert_eq!(seen.len(), 2);

        // Non-retryable errors are returned immediately
        let eps = vec![mk("https://one.example.com"), mk("https://two.example.com")];
        let mut calls = 0;
        let err = with_failover(eps, |_| {
            calls += 1;
            async { Err::<(), _>(anyhow::anyhow!("execution reverted")) }
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("reverted"));
        assert_eq!(calls, 1);
    }

    #[test]
    fn test_restricted_rpc_is_fallback_only() {
        let mut config = ConfigFile::default();

        let mut restricted = EndpointConfig::new("https://rpc.flashbots.net");
        restricted.priority = 10;
        restricted.chain = Chain::Ethereum;
        restricted.node_type = NodeType::Full;

        let mut archive = EndpointConfig::new("https://archive.example.com");
        archive.priority = 5;
        archive.chain = Chain::Ethereum;
        archive.node_type = NodeType::Archive;

        config.endpoints = vec![restricted, archive];

        let url = get_rpc_url_from_config(&config, Chain::Ethereum).unwrap();
        assert_eq!(url, "https://archive.example.com");
    }

    #[test]
    fn test_unknown_priority_selection_when_no_known_endpoints() {
        let mut config = ConfigFile::default();

        let mut low_priority = EndpointConfig::new("https://low-unknown.example.com");
        low_priority.priority = 1;
        low_priority.chain = Chain::Ethereum;

        let mut high_priority = EndpointConfig::new("https://high-unknown.example.com");
        high_priority.priority = 10;
        high_priority.chain = Chain::Ethereum;

        config.endpoints = vec![low_priority, high_priority];

        let url = get_rpc_url_from_config(&config, Chain::Ethereum).unwrap();
        assert_eq!(url, "https://high-unknown.example.com");
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
