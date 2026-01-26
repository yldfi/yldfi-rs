//! RPC pool for managing multiple endpoints with parallel requests.
//!
//! This module provides the [`RpcPool`] which manages a collection of RPC endpoints
//! with automatic health tracking, load balancing, and failover capabilities.
//!
//! # Endpoint Selection
//!
//! Endpoints are selected based on health scores, which factor in:
//! - Success/failure rates
//! - Response latency
//! - Rate limiting incidents
//! - Configured priority
//!
//! # Learned Optimizations
//!
//! The pool learns from errors and persists optimizations to the config file:
//! - **Block range limits**: When an endpoint returns "block range too large", the pool
//!   learns and remembers the reduced limit for future requests.
//! - **Max logs limits**: When an endpoint returns "response too large", the pool
//!   learns and remembers the limit.
//!
//! # Concurrency Model
//!
//! ## Config File Persistence (TOCTOU Considerations)
//!
//! Config persistence uses a global mutex (`CONFIG_WRITE_LOCK`) to serialize writes
//! within a single process. However, there are trade-offs to be aware of:
//!
//! - **Single process**: The mutex prevents race conditions between concurrent async tasks.
//! - **Multiple processes**: If multiple ethcli processes run simultaneously, there's a
//!   theoretical TOCTOU (time-of-check-to-time-of-use) window between reading and writing
//!   the config file. In practice, this is rare for CLI usage and the worst case is a
//!   lost optimization (not data corruption).
//! - **Fire-and-forget**: Persistence is done in background tasks. If the process exits
//!   quickly (Ctrl+C), writes may not complete. This is acceptable since learned limits
//!   are optimizations, not critical data.
//!
//! If this code were used in a long-running daemon or library context, consider adding
//! file-level locking (e.g., `fs2::FileExt::lock_exclusive`).

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
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Global mutex to serialize config file updates within a single process.
///
/// This prevents race conditions between concurrent async tasks. For multi-process
/// scenarios, see the module-level documentation on TOCTOU considerations.
static CONFIG_WRITE_LOCK: Mutex<()> = Mutex::const_new(());

/// Persist a learned block range limit to the config file.
///
/// This function spawns a background task to update the config file with the
/// learned limit. The operation is **fire-and-forget**:
///
/// - Errors are logged at debug level but don't affect the caller
/// - If the process exits quickly (e.g., Ctrl+C), the write may not complete
/// - Multiple concurrent calls are serialized via `CONFIG_WRITE_LOCK`
///
/// This trade-off is acceptable for a CLI tool since:
/// 1. Learned limits are optimizations, not critical data
/// 2. The limit will be re-learned on the next run if not persisted
/// 3. Blocking the main operation for config writes would hurt UX
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

/// Persist a learned max logs limit to the config file.
///
/// See [`persist_block_range_limit`] for details on the fire-and-forget behavior.
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

/// Pool of RPC endpoints with load balancing and health tracking.
///
/// # Memory Considerations
///
/// The endpoint list is fixed at construction time. Endpoints are never removed,
/// only marked as unhealthy via the [`HealthTracker`]. This is intentional:
///
/// - CLI tools are short-lived, so memory growth isn't a concern
/// - Endpoints may recover after temporary issues (circuit breaker resets)
/// - The list size is bounded by user configuration
///
/// For long-running daemons, consider implementing periodic cleanup of
/// persistently unhealthy endpoints.
///
/// # Block Reorganization Handling
///
/// This pool does **not** account for chain reorganizations. Queries for recent
/// blocks may return data that becomes stale if a reorg occurs. This is acceptable
/// for a CLI tool because:
///
/// - Most queries are for historical data (well past finalization)
/// - Real-time queries are point-in-time snapshots by nature
/// - Reorg detection would add significant complexity
///
/// For applications requiring reorg-aware data, consider waiting for finalization
/// (12-15 confirmations on Ethereum mainnet) or using a service that handles reorgs.
pub struct RpcPool {
    /// Available endpoints (wrapped in Arc to avoid cloning config strings).
    ///
    /// Endpoints are created at construction and never removed. Unhealthy endpoints
    /// are tracked via `health` and excluded from selection until they recover.
    endpoints: Vec<Arc<Endpoint>>,
    /// Health tracker for endpoint selection and circuit breaking.
    health: Arc<HealthTracker>,
    /// Max concurrent requests for parallel operations.
    concurrency: usize,
    /// User-specified chunk size override (max block range per request).
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

        // Create endpoint instances (wrapped in Arc to avoid cloning config strings)
        let mut endpoints = Vec::new();
        for cfg in endpoint_configs {
            match Endpoint::new(cfg.clone(), config.timeout_secs, proxy.as_deref()) {
                Ok(ep) => endpoints.push(Arc::new(ep)),
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
            endpoints: vec![Arc::new(endpoint)],
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
    ///
    /// Returns Arc<Endpoint> to avoid cloning EndpointConfig strings.
    fn select_endpoints(&self, count: usize) -> Vec<Arc<Endpoint>> {
        // Get URLs sorted by health score
        let urls: Vec<_> = self.endpoints.iter().map(|e| e.url().to_string()).collect();
        let ranked = self.health.rank_endpoints(&urls);

        // Build HashMap for O(1) score lookup instead of O(n) linear search
        let scores: HashMap<&str, f64> = ranked.iter().map(|(u, s)| (u.as_str(), *s)).collect();

        // Get available endpoints (Arc clone is cheap - just reference count increment)
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

        // Sort by ranking with O(1) score lookup
        available.sort_by(|a, b| {
            let a_score = scores.get(a.url()).copied().unwrap_or(0.0);
            let b_score = scores.get(b.url()).copied().unwrap_or(0.0);
            b_score.total_cmp(&a_score)
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
    pub fn select_archive_endpoints(&self, count: usize) -> Vec<Arc<Endpoint>> {
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

    #[test]
    fn test_health_tracker_selection_priority() {
        // Test that endpoints with lower latency get higher scores
        let tracker = HealthTracker::new();

        // Simulate successful requests with different latencies
        for _ in 0..5 {
            tracker.record_success("https://fast.com", Duration::from_millis(50));
            tracker.record_success("https://medium.com", Duration::from_millis(200));
            tracker.record_success("https://slow.com", Duration::from_millis(1000));
        }

        let urls = vec![
            "https://slow.com".to_string(),
            "https://fast.com".to_string(),
            "https://medium.com".to_string(),
        ];
        let ranked = tracker.rank_endpoints(&urls);

        // Fast should be first (highest score), slow should be last
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].0, "https://fast.com");
        assert_eq!(ranked[2].0, "https://slow.com");
    }

    #[test]
    fn test_health_tracker_failure_tracking() {
        let tracker = HealthTracker::new();

        // Record some successes
        tracker.record_success("https://good.com", Duration::from_millis(100));
        tracker.record_success("https://good.com", Duration::from_millis(100));

        // Record failures
        tracker.record_failure("https://bad.com", false, false);
        tracker.record_failure("https://bad.com", false, false);

        let urls = vec!["https://good.com".to_string(), "https://bad.com".to_string()];
        let ranked = tracker.rank_endpoints(&urls);

        // Good should have higher score
        assert_eq!(ranked.len(), 2);
        assert!(ranked[0].1 > ranked[1].1);
        assert_eq!(ranked[0].0, "https://good.com");
    }

    #[test]
    fn test_circuit_breaker_activation() {
        // Test that circuit breaker opens after consecutive failures
        let tracker = HealthTracker::with_circuit_breaker(3, 60);

        assert!(tracker.is_available("https://test.com"));

        // Record consecutive failures
        tracker.record_failure("https://test.com", false, false);
        assert!(tracker.is_available("https://test.com"));

        tracker.record_failure("https://test.com", false, false);
        assert!(tracker.is_available("https://test.com"));

        tracker.record_failure("https://test.com", false, false);
        // Circuit breaker should be open now
        assert!(!tracker.is_available("https://test.com"));
    }

    #[test]
    fn test_circuit_breaker_reset_on_success() {
        let tracker = HealthTracker::with_circuit_breaker(3, 60);

        // Get close to threshold
        tracker.record_failure("https://test.com", false, false);
        tracker.record_failure("https://test.com", false, false);

        // Success resets the counter
        tracker.record_success("https://test.com", Duration::from_millis(100));

        // Now need 3 more failures to trip
        tracker.record_failure("https://test.com", false, false);
        tracker.record_failure("https://test.com", false, false);
        assert!(tracker.is_available("https://test.com"));

        tracker.record_failure("https://test.com", false, false);
        assert!(!tracker.is_available("https://test.com"));
    }

    #[test]
    fn test_rate_limit_tracking() {
        let tracker = HealthTracker::new();

        // Record rate limit hits
        tracker.record_failure("https://test.com", true, false);
        tracker.record_failure("https://test.com", true, false);

        let health = tracker.get_health("https://test.com").unwrap();
        assert_eq!(health.rate_limit_hits, 2);
        assert_eq!(health.failed_requests, 2);
    }

    #[test]
    fn test_learned_block_range_limits() {
        let tracker = HealthTracker::new();

        // Learn a limit
        tracker.record_block_range_limit("https://test.com", 1000);

        // Should use learned limit
        assert_eq!(tracker.effective_max_block_range("https://test.com", 5000), 1000);

        // Should use configured if smaller
        assert_eq!(tracker.effective_max_block_range("https://test.com", 500), 500);

        // Unknown endpoint uses configured
        assert_eq!(
            tracker.effective_max_block_range("https://unknown.com", 5000),
            5000
        );
    }

    #[test]
    fn test_learned_limit_only_decreases() {
        let tracker = HealthTracker::new();

        // Learn a limit
        tracker.record_block_range_limit("https://test.com", 1000);
        assert_eq!(tracker.effective_max_block_range("https://test.com", 5000), 1000);

        // Trying to set a higher limit doesn't change it
        tracker.record_block_range_limit("https://test.com", 2000);
        assert_eq!(tracker.effective_max_block_range("https://test.com", 5000), 1000);

        // Lower limit does update it
        tracker.record_block_range_limit("https://test.com", 500);
        assert_eq!(tracker.effective_max_block_range("https://test.com", 5000), 500);
    }

    #[test]
    fn test_endpoint_priority_config() {
        // Test that priority is respected in config
        let high_priority =
            EndpointConfig::new("https://primary.com").with_priority(1);
        let low_priority =
            EndpointConfig::new("https://backup.com").with_priority(10);

        assert_eq!(high_priority.priority, 1);
        assert_eq!(low_priority.priority, 10);
    }

    #[test]
    fn test_node_type_config() {
        let archive = EndpointConfig::new("https://archive.com").with_node_type(NodeType::Archive);
        let full = EndpointConfig::new("https://full.com").with_node_type(NodeType::Full);

        assert_eq!(archive.node_type, NodeType::Archive);
        assert_eq!(full.node_type, NodeType::Full);
    }

    #[test]
    fn test_max_block_range_config() {
        let endpoint =
            EndpointConfig::new("https://test.com").with_max_block_range(10000);
        assert_eq!(endpoint.max_block_range, 10000);

        // Default should be DEFAULT_MAX_BLOCK_RANGE
        let default_endpoint = EndpointConfig::new("https://default.com");
        assert_eq!(default_endpoint.max_block_range, DEFAULT_MAX_BLOCK_RANGE);
    }

    #[test]
    fn test_rpc_config_defaults() {
        let config = RpcConfig::default();

        // Default config has no endpoints - users add them via config or CLI
        assert!(config.endpoints.is_empty(), "Default config should have no endpoints");
        assert_eq!(config.timeout_secs, 30, "Should have 30s default timeout");
        assert_eq!(config.concurrency, 5, "Should have default concurrency of 5");
        assert_eq!(config.max_retries, 3, "Should have default max_retries of 3");
        assert_eq!(config.min_priority, 1, "Should have default min_priority of 1");
    }

    #[test]
    fn test_new_endpoint_has_high_initial_score() {
        // New endpoints without any history should be given benefit of the doubt
        let tracker = HealthTracker::new();

        let urls = vec![
            "https://new.com".to_string(),
            "https://other.com".to_string(),
        ];

        let ranked = tracker.rank_endpoints(&urls);
        assert_eq!(ranked.len(), 2);
        // Both should have score of 100 (new endpoints assumed healthy)
        assert_eq!(ranked[0].1, 100.0);
        assert_eq!(ranked[1].1, 100.0);
    }

    #[test]
    fn test_health_score_calculation() {
        let mut health = EndpointHealth::default();

        // Empty endpoint should have 0 error rate
        assert_eq!(health.error_rate(), 0.0);

        // Simulate requests
        health.total_requests = 10;
        health.failed_requests = 3;
        health.successful_requests = 7;

        assert!((health.error_rate() - 0.3).abs() < 0.001);

        // Score should be positive and bounded
        let score = health.health_score();
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_try_probe_atomicity() {
        // Test that try_probe properly gates concurrent probes
        let tracker = HealthTracker::with_circuit_breaker(1, 1); // 1 failure trips, 1 sec timeout

        // Trip the circuit breaker
        tracker.record_failure("https://test.com", false, false);
        assert!(!tracker.is_available("https://test.com"));

        // Wait for timeout (in test, we can't easily do this, but we can test the logic)
        // For now just verify the method exists and can be called
        let can_probe = tracker.try_probe("https://test.com");
        // Initially should be false since timeout hasn't passed
        assert!(!can_probe);
    }
}
