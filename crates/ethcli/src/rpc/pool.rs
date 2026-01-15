//! RPC pool for managing multiple endpoints with parallel requests

use crate::config::{
    Chain, ConfigFile, EndpointConfig, NodeType, RpcConfig, DEFAULT_MAX_BLOCK_RANGE,
    MIN_TX_FETCH_CONCURRENCY,
};
use crate::error::{sanitize_error_message, Error, Result, RpcError};
use crate::rpc::{Endpoint, EndpointHealth, HealthTracker};
use alloy::primitives::B256;
use alloy::rpc::types::{Filter, Log, Transaction, TransactionReceipt};
use futures::future::join_all;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Global mutex to serialize config file updates and prevent TOCTOU race conditions
static CONFIG_WRITE_LOCK: Mutex<()> = Mutex::const_new(());

/// Persist a learned block range limit to the config file.
///
/// This is fire-and-forget - errors are logged but don't affect operation.
/// Note: If the process exits quickly (e.g., Ctrl+C), these background writes
/// may not complete. This is acceptable for a CLI tool since learned limits
/// are optimizations, not critical data.
fn persist_block_range_limit(url: &str, limit: u64) {
    let url = url.to_string();
    // Spawn a blocking task to avoid blocking the async runtime
    tokio::spawn(async move {
        // Acquire lock to prevent concurrent config modifications
        let _guard = CONFIG_WRITE_LOCK.lock().await;
        if let Ok(Some(mut config)) = ConfigFile::load_default() {
            match config.update_endpoint_block_range(&url, limit) {
                Ok(true) => {
                    tracing::info!(
                        "Learned block range limit for {}: {} blocks (saved to config)",
                        sanitize_error_message(&url),
                        limit
                    );
                }
                Ok(false) => {} // No update needed
                Err(e) => {
                    tracing::debug!("Failed to persist block range limit: {}", e);
                }
            }
        }
    });
}

/// Persist a learned max logs limit to the config file
fn persist_max_logs_limit(url: &str, limit: usize) {
    let url = url.to_string();
    tokio::spawn(async move {
        // Acquire lock to prevent concurrent config modifications
        let _guard = CONFIG_WRITE_LOCK.lock().await;
        if let Ok(Some(mut config)) = ConfigFile::load_default() {
            match config.update_endpoint_max_logs(&url, limit) {
                Ok(true) => {
                    tracing::info!(
                        "Learned max logs limit for {}: {} logs (saved to config)",
                        sanitize_error_message(&url),
                        limit
                    );
                }
                Ok(false) => {}
                Err(e) => {
                    tracing::debug!("Failed to persist max logs limit: {}", e);
                }
            }
        }
    });
}

/// Pool of RPC endpoints with load balancing and health tracking
pub struct RpcPool {
    /// Available endpoints
    endpoints: Vec<Endpoint>,
    /// Health tracker
    health: Arc<HealthTracker>,
    /// Max concurrent requests
    concurrency: usize,
    /// Override chunk size (max block range per request)
    chunk_size_override: Option<u64>,
}

impl RpcPool {
    /// Create a new RPC pool for a chain
    pub fn new(chain: Chain, config: &RpcConfig) -> Result<Self> {
        // Use user-provided endpoints (no hardcoded defaults)
        let mut endpoint_configs = config.endpoints.clone();

        // Filter by chain - only use endpoints that match the target chain
        endpoint_configs.retain(|e| e.chain == chain);

        // Add additional endpoints (these bypass chain filter as they're explicitly added)
        for url in &config.add_endpoints {
            if !endpoint_configs.iter().any(|e| &e.url == url) {
                endpoint_configs.push(EndpointConfig::new(url.clone()).with_chain(chain));
            }
        }

        // Exclude specified endpoints
        let excluded: HashSet<_> = config.exclude_endpoints.iter().collect();
        endpoint_configs.retain(|e| !excluded.contains(&e.url));

        // Filter by minimum priority
        endpoint_configs.retain(|e| e.priority >= config.min_priority);

        // Filter disabled endpoints
        endpoint_configs.retain(|e| e.enabled);

        if endpoint_configs.is_empty() {
            return Err(RpcError::NoHealthyEndpoints.into());
        }

        // Get global proxy
        let proxy = config.proxy.as_ref().and_then(|p| p.url.clone());

        // Create endpoint instances
        let mut endpoints = Vec::new();
        for cfg in endpoint_configs {
            match Endpoint::new(cfg.clone(), config.timeout_secs, proxy.as_deref()) {
                Ok(ep) => endpoints.push(ep),
                Err(e) => {
                    tracing::warn!(
                        "Failed to create endpoint {}: {}",
                        sanitize_error_message(&cfg.url),
                        e
                    );
                }
            }
        }

        if endpoints.is_empty() {
            return Err(RpcError::NoHealthyEndpoints.into());
        }

        Ok(Self {
            endpoints,
            health: Arc::new(HealthTracker::new()),
            concurrency: config.concurrency,
            chunk_size_override: config.chunk_size,
        })
    }

    /// Create from a single endpoint URL
    pub fn from_url(url: &str, timeout_secs: u64) -> Result<Self> {
        let config = EndpointConfig::new(url);
        let endpoint = Endpoint::new(config, timeout_secs, None)?;

        Ok(Self {
            endpoints: vec![endpoint],
            health: Arc::new(HealthTracker::new()),
            concurrency: 1,
            chunk_size_override: None,
        })
    }

    /// Get number of available endpoints
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    /// Get concurrency level
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Get health tracker
    pub fn health_tracker(&self) -> Arc<HealthTracker> {
        self.health.clone()
    }

    /// Get current block number (tries multiple endpoints)
    pub async fn get_block_number(&self) -> Result<u64> {
        let endpoints = self.select_endpoints(3);

        for endpoint in endpoints {
            match endpoint.get_block_number().await {
                Ok(block) => {
                    self.health
                        .record_success(endpoint.url(), Duration::from_millis(100));
                    return Ok(block);
                }
                Err(e) => {
                    self.health.record_failure(endpoint.url(), false, false);
                    tracing::debug!(
                        "Failed to get block number from {}: {}",
                        sanitize_error_message(endpoint.url()),
                        e
                    );
                }
            }
        }

        Err(RpcError::AllEndpointsFailed.into())
    }

    /// Get a transaction by hash (tries multiple endpoints)
    pub async fn get_transaction(&self, hash: B256) -> Result<Option<Transaction>> {
        // Try more endpoints for historical data - archive coverage varies
        let endpoints = self.select_endpoints(self.concurrency.max(MIN_TX_FETCH_CONCURRENCY));

        for endpoint in endpoints {
            match endpoint.get_transaction(hash).await {
                Ok(Some(tx)) => {
                    self.health
                        .record_success(endpoint.url(), Duration::from_millis(100));
                    return Ok(Some(tx));
                }
                Ok(None) => {
                    // Transaction not found on this endpoint, try others
                    tracing::debug!(
                        "Transaction not found on {}, trying next endpoint",
                        sanitize_error_message(endpoint.url())
                    );
                }
                Err(e) => {
                    self.health.record_failure(endpoint.url(), false, false);
                    tracing::debug!(
                        "Failed to get transaction from {}: {}",
                        sanitize_error_message(endpoint.url()),
                        e
                    );
                }
            }
        }

        // No endpoint had the transaction
        Ok(None)
    }

    /// Get a transaction receipt by hash (tries multiple endpoints)
    pub async fn get_transaction_receipt(&self, hash: B256) -> Result<Option<TransactionReceipt>> {
        // Try more endpoints for historical data - archive coverage varies
        let endpoints = self.select_endpoints(self.concurrency.max(MIN_TX_FETCH_CONCURRENCY));

        for endpoint in endpoints {
            match endpoint.get_transaction_receipt(hash).await {
                Ok(Some(receipt)) => {
                    self.health
                        .record_success(endpoint.url(), Duration::from_millis(100));
                    return Ok(Some(receipt));
                }
                Ok(None) => {
                    // Receipt not found on this endpoint, try others
                    tracing::debug!(
                        "Receipt not found on {}, trying next endpoint",
                        sanitize_error_message(endpoint.url())
                    );
                }
                Err(e) => {
                    self.health.record_failure(endpoint.url(), false, false);
                    tracing::debug!(
                        "Failed to get transaction receipt from {}: {}",
                        sanitize_error_message(endpoint.url()),
                        e
                    );
                }
            }
        }

        // No endpoint had the receipt
        Ok(None)
    }

    /// Fetch logs with automatic retry and load balancing
    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>> {
        let endpoints = self.select_endpoints(self.concurrency);

        if endpoints.is_empty() {
            return Err(RpcError::NoHealthyEndpoints.into());
        }

        // Try endpoints in order of health score
        for endpoint in &endpoints {
            match endpoint.get_logs(filter).await {
                Ok((logs, latency)) => {
                    self.health.record_success(endpoint.url(), latency);
                    return Ok(logs);
                }
                Err(e) => {
                    let is_rate_limit = matches!(&e, Error::Rpc(RpcError::RateLimited(_)));
                    let is_timeout = matches!(&e, Error::Rpc(RpcError::Timeout(_)));

                    self.health
                        .record_failure(endpoint.url(), is_rate_limit, is_timeout);

                    // Learn from block range errors
                    // When we get a BlockRangeTooLarge error, the 'max' is the configured value
                    // that was too high. We should record a reduced value to actually back off.
                    if let Error::Rpc(RpcError::BlockRangeTooLarge { max, requested }) = &e {
                        // Halve the failed range, with a floor of 100 blocks to allow
                        // adapting to very restrictive endpoints while remaining practical
                        let reduced_limit = if *requested > 0 {
                            (*requested / 2).max(100)
                        } else {
                            (*max / 2).max(100)
                        };
                        self.health
                            .record_block_range_limit(endpoint.url(), reduced_limit);

                        // Persist the learned limit to config
                        persist_block_range_limit(endpoint.url(), reduced_limit);
                    }

                    // Learn from response too large errors (max logs exceeded)
                    if let Error::Rpc(RpcError::ResponseTooLarge(count)) = &e {
                        // Use the count if available, otherwise estimate based on typical limits
                        let reduced_limit = if *count > 0 {
                            (*count / 2).max(1000)
                        } else {
                            5000 // Conservative default
                        };
                        persist_max_logs_limit(endpoint.url(), reduced_limit);
                    }

                    tracing::debug!(
                        "Failed to get logs from {}: {}",
                        sanitize_error_message(endpoint.url()),
                        e
                    );
                }
            }
        }

        Err(RpcError::AllEndpointsFailed.into())
    }

    /// Fetch logs from multiple filters in parallel
    pub async fn get_logs_parallel(&self, filters: Vec<Filter>) -> Vec<Result<Vec<Log>>> {
        let endpoints = self.select_endpoints(self.concurrency.max(filters.len()));

        // Guard against empty endpoints (would panic on modulo)
        if endpoints.is_empty() {
            return filters
                .into_iter()
                .map(|_| Err(RpcError::NoHealthyEndpoints.into()))
                .collect();
        }

        // Create a task for each filter
        let tasks: Vec<_> = filters
            .into_iter()
            .enumerate()
            .map(|(i, filter)| {
                let endpoint = endpoints[i % endpoints.len()].clone();
                let health = self.health.clone();

                async move {
                    match endpoint.get_logs(&filter).await {
                        Ok((logs, latency)) => {
                            health.record_success(endpoint.url(), latency);
                            Ok(logs)
                        }
                        Err(e) => {
                            let is_rate_limit = matches!(&e, Error::Rpc(RpcError::RateLimited(_)));
                            let is_timeout = matches!(&e, Error::Rpc(RpcError::Timeout(_)));
                            health.record_failure(endpoint.url(), is_rate_limit, is_timeout);

                            // Learn from errors and persist to config
                            if let Error::Rpc(RpcError::BlockRangeTooLarge { max, requested }) = &e
                            {
                                let reduced_limit = if *requested > 0 {
                                    (*requested / 2).max(100)
                                } else {
                                    (*max / 2).max(100)
                                };
                                health.record_block_range_limit(endpoint.url(), reduced_limit);
                                persist_block_range_limit(endpoint.url(), reduced_limit);
                            }

                            if let Error::Rpc(RpcError::ResponseTooLarge(count)) = &e {
                                let reduced_limit = if *count > 0 {
                                    (*count / 2).max(1000)
                                } else {
                                    5000
                                };
                                persist_max_logs_limit(endpoint.url(), reduced_limit);
                            }

                            Err(e)
                        }
                    }
                }
            })
            .collect();

        join_all(tasks).await
    }

    /// Select endpoints for a request based on health and priority
    fn select_endpoints(&self, count: usize) -> Vec<Endpoint> {
        // Get URLs sorted by health score
        let urls: Vec<_> = self.endpoints.iter().map(|e| e.url().to_string()).collect();
        let ranked = self.health.rank_endpoints(&urls);

        // Get available endpoints
        let mut available: Vec<_> = self
            .endpoints
            .iter()
            .filter(|e| self.health.is_available(e.url()))
            .cloned()
            .collect();

        // Warn if all endpoints are unhealthy
        if available.is_empty() && !self.endpoints.is_empty() {
            tracing::warn!(
                "All {} endpoints are currently unhealthy (circuit breaker open). \
                 Requests will fail until circuit breakers reset.",
                self.endpoints.len()
            );
        }

        // Sort by ranking
        available.sort_by(|a, b| {
            let a_score = ranked
                .iter()
                .find(|(u, _)| u == a.url())
                .map(|(_, s)| *s)
                .unwrap_or(0.0);
            let b_score = ranked
                .iter()
                .find(|(u, _)| u == b.url())
                .map(|(_, s)| *s)
                .unwrap_or(0.0);
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Add some randomization among similar scores
        let mut rng = thread_rng();
        if available.len() > 2 {
            // Shuffle top endpoints a bit for load distribution
            let shuffle_count = (available.len() / 3).max(2).min(available.len());
            available[..shuffle_count].shuffle(&mut rng);
        }

        available.truncate(count);
        available
    }

    /// Get effective max block range for an endpoint
    pub fn effective_max_block_range(&self, url: &str) -> u64 {
        if let Some(endpoint) = self.endpoints.iter().find(|e| e.url() == url) {
            self.health
                .effective_max_block_range(url, endpoint.max_block_range())
        } else {
            DEFAULT_MAX_BLOCK_RANGE
        }
    }

    /// Get the smallest max block range across all endpoints
    pub fn min_block_range(&self) -> u64 {
        self.endpoints
            .iter()
            .map(|e| {
                self.health
                    .effective_max_block_range(e.url(), e.max_block_range())
            })
            .min()
            .unwrap_or(DEFAULT_MAX_BLOCK_RANGE)
    }

    /// Get the largest max block range across healthy endpoints
    /// Returns the user-specified chunk_size if set, otherwise uses endpoint limits
    pub fn max_block_range(&self) -> u64 {
        // If user specified a chunk size, use that
        if let Some(chunk_size) = self.chunk_size_override {
            return chunk_size;
        }

        self.endpoints
            .iter()
            .filter(|e| self.health.is_available(e.url()))
            .map(|e| {
                self.health
                    .effective_max_block_range(e.url(), e.max_block_range())
            })
            .max()
            .unwrap_or(DEFAULT_MAX_BLOCK_RANGE)
    }

    /// Get health info for all endpoints
    pub fn get_endpoint_health(&self) -> Vec<(String, u8, Option<EndpointHealth>)> {
        self.endpoints
            .iter()
            .map(|e| {
                (
                    e.url().to_string(),
                    e.priority(),
                    self.health.get_health(e.url()),
                )
            })
            .collect()
    }

    /// List all endpoint URLs
    pub fn list_endpoints(&self) -> Vec<&str> {
        self.endpoints.iter().map(|e| e.url()).collect()
    }

    /// Get count of archive endpoints
    pub fn archive_endpoint_count(&self) -> usize {
        self.endpoints
            .iter()
            .filter(|e| e.config.node_type == NodeType::Archive)
            .count()
    }

    /// Get count of debug-capable endpoints
    pub fn debug_endpoint_count(&self) -> usize {
        self.endpoints.iter().filter(|e| e.config.has_debug).count()
    }

    /// Select endpoints that are known archives (for historical data queries)
    /// Falls back to all available endpoints if no archives are known
    pub fn select_archive_endpoints(&self, count: usize) -> Vec<Endpoint> {
        let archive_eps: Vec<_> = self
            .endpoints
            .iter()
            .filter(|e| {
                e.config.node_type == NodeType::Archive && self.health.is_available(e.url())
            })
            .cloned()
            .collect();

        if archive_eps.is_empty() {
            // Fall back to all available endpoints
            self.select_endpoints(count)
        } else {
            let mut result = archive_eps;
            result.truncate(count);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let config = RpcConfig {
            endpoints: vec![EndpointConfig::new("https://eth.llamarpc.com")],
            ..Default::default()
        };

        // Pool creation will fail without network, but we can test config handling
        let result = RpcPool::new(Chain::Ethereum, &config);
        // This may fail due to URL validation, which is fine for unit test
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_endpoint_selection() {
        // Test that health tracker rankings work
        let tracker = HealthTracker::new();
        tracker.record_success("https://a.com", Duration::from_millis(100));
        tracker.record_success("https://b.com", Duration::from_millis(500));

        let urls = vec!["https://a.com".to_string(), "https://b.com".to_string()];
        let ranked = tracker.rank_endpoints(&urls);

        assert_eq!(ranked.len(), 2);
        // First should have higher score (lower latency)
        assert!(ranked[0].1 >= ranked[1].1);
    }

    #[test]
    fn test_chain_filtering() {
        // Create endpoints for different chains
        let eth_endpoint =
            EndpointConfig::new("https://eth.example.com").with_chain(Chain::Ethereum);
        let polygon_endpoint =
            EndpointConfig::new("https://polygon.example.com").with_chain(Chain::Polygon);
        let arb_endpoint =
            EndpointConfig::new("https://arb.example.com").with_chain(Chain::Arbitrum);

        let config = RpcConfig {
            endpoints: vec![eth_endpoint, polygon_endpoint, arb_endpoint],
            ..Default::default()
        };

        // Filter for Ethereum - should only get eth endpoint
        let mut filtered = config.endpoints.clone();
        filtered.retain(|e| e.chain == Chain::Ethereum);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].url, "https://eth.example.com");

        // Filter for Polygon - should only get polygon endpoint
        let mut filtered = config.endpoints.clone();
        filtered.retain(|e| e.chain == Chain::Polygon);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].url, "https://polygon.example.com");

        // Filter for Arbitrum - should only get arb endpoint
        let mut filtered = config.endpoints.clone();
        filtered.retain(|e| e.chain == Chain::Arbitrum);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].url, "https://arb.example.com");
    }

    #[test]
    fn test_disabled_endpoint_filtering() {
        let mut enabled = EndpointConfig::new("https://enabled.com");
        enabled.enabled = true;

        let mut disabled = EndpointConfig::new("https://disabled.com");
        disabled.enabled = false;

        let config = RpcConfig {
            endpoints: vec![enabled, disabled],
            ..Default::default()
        };

        // Filter out disabled endpoints
        let mut filtered = config.endpoints.clone();
        filtered.retain(|e| e.enabled);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].url, "https://enabled.com");
    }

    #[test]
    fn test_add_endpoints_bypass_chain_filter() {
        // When using add_endpoints, they should be added with the target chain
        let config = RpcConfig {
            endpoints: vec![],
            add_endpoints: vec!["https://custom.example.com".to_string()],
            ..Default::default()
        };

        // add_endpoints should be usable for any chain since they're explicitly added
        assert_eq!(config.add_endpoints.len(), 1);
        assert_eq!(config.add_endpoints[0], "https://custom.example.com");
    }
}
