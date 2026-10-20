//! Single RPC endpoint wrapper.
//!
//! This module provides the [`Endpoint`] type, which wraps an Alloy provider
//! with configuration and timeout handling. Each endpoint represents a single
//! Ethereum RPC connection.
//!
//! # Features
//!
//! - Configured timeout for all requests
//! - Block range limits (learned from errors)
//! - Priority-based endpoint selection support
//! - Node type awareness (full vs archive)
//!
//! # Usage
//!
//! Endpoints are typically managed by [`RpcPool`](super::RpcPool) rather than
//! used directly. The pool handles health tracking, load balancing, and failover.

use crate::config::EndpointConfig;
use crate::error::{Result, RpcError};
use alloy::primitives::B256;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Block, Filter, Log, Transaction, TransactionReceipt};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Type alias for the HTTP provider
type HttpProvider = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::GasFiller,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::BlobGasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::NonceFiller,
                    alloy::providers::fillers::ChainIdFiller,
                >,
            >,
        >,
    >,
    alloy::providers::RootProvider,
>;

/// A single RPC endpoint with its configuration and provider
#[derive(Clone)]
pub struct Endpoint {
    /// Endpoint configuration
    pub config: EndpointConfig,
    /// Alloy provider
    provider: Arc<HttpProvider>,
    /// Request timeout
    timeout: Duration,
}

impl Endpoint {
    /// Create a new endpoint from config
    ///
    /// Note: The `proxy` parameter is currently not implemented. Proxy support would
    /// require using a custom reqwest client with the alloy provider.
    ///
    /// # Errors
    /// Returns `RpcError::ProxyNotSupported` if a proxy URL is provided, to prevent
    /// users from having a false sense of privacy/security when traffic would actually
    /// go direct.
    pub fn new(config: EndpointConfig, timeout_secs: u64, proxy: Option<&str>) -> Result<Self> {
        let timeout = Duration::from_secs(timeout_secs);

        // Fail fast if proxy is configured - don't silently ignore it
        if let Some(proxy_url) = proxy {
            return Err(RpcError::ProxyNotSupported(format!(
                "Proxy '{}' was configured but proxy support is not yet implemented. \
                 Remove the proxy configuration or traffic will fail. \
                 See: https://github.com/alloy-rs/alloy/issues/... for proxy support status.",
                crate::utils::url::redact_url(proxy_url)
            ))
            .into());
        }

        // Parse URL
        let url: reqwest::Url = config.url.parse().map_err(|e| {
            RpcError::ConnectionFailed(format!(
                "Invalid URL {}: {}",
                crate::utils::url::redact_url(&config.url),
                e
            ))
        })?;

        // Create provider (proxy support would require lower-level transport with custom reqwest client)
        let provider = ProviderBuilder::new().connect_http(url);

        Ok(Self {
            config,
            provider: Arc::new(provider),
            timeout,
        })
    }

    /// Get the endpoint URL
    pub fn url(&self) -> &str {
        &self.config.url
    }

    /// Get the priority
    pub fn priority(&self) -> u8 {
        self.config.priority
    }

    /// Get max block range
    pub fn max_block_range(&self) -> u64 {
        self.config.max_block_range
    }

    /// Get max logs
    pub fn max_logs(&self) -> usize {
        self.config.max_logs
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the inner provider for direct RPC calls
    pub fn provider(&self) -> &HttpProvider {
        &self.provider
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let result = tokio::time::timeout(self.timeout, self.provider.get_block_number()).await;

        match result {
            Ok(Ok(block)) => Ok(block),
            Ok(Err(e)) => Err(RpcError::Provider(
                crate::error::sanitize_error_message(&e.to_string()).into_owned(),
            )
            .into()),
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Fetch logs with a filter
    pub async fn get_logs(&self, filter: &Filter) -> Result<(Vec<Log>, Duration)> {
        let start = Instant::now();

        let result = tokio::time::timeout(self.timeout, self.provider.get_logs(filter)).await;

        let elapsed = start.elapsed();

        match result {
            Ok(Ok(logs)) => Ok((logs, elapsed)),
            Ok(Err(e)) => {
                let message = e.to_string();
                // Blocks in the request, inclusive
                let requested = filter
                    .get_from_block()
                    .and_then(|from| {
                        filter
                            .get_to_block()
                            .map(|to| to.saturating_sub(from).saturating_add(1))
                    })
                    .unwrap_or(0);
                Err(classify_logs_error(&message, requested)
                    .unwrap_or_else(|| {
                        RpcError::Provider(
                            crate::error::sanitize_error_message(&message).into_owned(),
                        )
                    })
                    .with_rate_limit_source(&self.config.url)
                    .into())
            }
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Get a transaction by hash
    pub async fn get_transaction(&self, hash: B256) -> Result<Option<Transaction>> {
        let result =
            tokio::time::timeout(self.timeout, self.provider.get_transaction_by_hash(hash)).await;

        match result {
            Ok(Ok(tx)) => Ok(tx),
            Ok(Err(e)) => Err(RpcError::Provider(
                crate::error::sanitize_error_message(&e.to_string()).into_owned(),
            )
            .into()),
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Execute a read-only `eth_call` against the latest block
    pub async fn call(
        &self,
        to: alloy::primitives::Address,
        data: alloy::primitives::Bytes,
    ) -> Result<alloy::primitives::Bytes> {
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(to)
            .input(data.into());
        let result = tokio::time::timeout(self.timeout, self.provider.call(tx)).await;

        match result {
            Ok(Ok(out)) => Ok(out),
            Ok(Err(e)) => Err(RpcError::Provider(
                crate::error::sanitize_error_message(&e.to_string()).into_owned(),
            )
            .into()),
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Get a transaction receipt by hash
    pub async fn get_transaction_receipt(&self, hash: B256) -> Result<Option<TransactionReceipt>> {
        let result =
            tokio::time::timeout(self.timeout, self.provider.get_transaction_receipt(hash)).await;

        match result {
            Ok(Ok(receipt)) => Ok(receipt),
            Ok(Err(e)) => Err(RpcError::Provider(
                crate::error::sanitize_error_message(&e.to_string()).into_owned(),
            )
            .into()),
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Get a block by number with full transaction objects
    pub async fn get_block_with_txs(&self, block_number: u64) -> Result<Option<Block>> {
        let result = tokio::time::timeout(
            self.timeout,
            self.provider
                .get_block_by_number(block_number.into())
                .full(),
        )
        .await;

        match result {
            Ok(Ok(block)) => Ok(block),
            Ok(Err(e)) => Err(RpcError::Provider(
                crate::error::sanitize_error_message(&e.to_string()).into_owned(),
            )
            .into()),
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Check if endpoint supports archive queries (test with an old block)
    pub async fn test_archive_support(&self) -> Result<bool> {
        use alloy::primitives::address;

        // Test with a known early block (block 46147 - first contract deployment)
        let test_address = address!("5e97870f263700f46aa00d967821199b9bc5a120");
        let test_block = 46147u64;

        let result = tokio::time::timeout(
            Duration::from_secs(10),
            self.provider
                .get_balance(test_address)
                .block_id(test_block.into()),
        )
        .await;

        match result {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(e)) => {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("missing trie node") || err_str.contains("pruned") {
                    Ok(false)
                } else {
                    // Other error, might still be archive
                    Ok(false)
                }
            }
            Err(_) => Ok(false), // Timeout
        }
    }
}

/// Classify an `eth_getLogs` failure from the provider's message.
///
/// Matches known provider phrases only; a loose substring such as "exceed" or
/// "too many" mislabels unrelated errors (e.g. "too many logs" is not a rate
/// limit). Returns `None` for anything not recognised.
///
/// For `BlockRangeTooLarge`, `max` is the limit stated in the message, or 0 when
/// the provider didn't state one. Only a stated limit should ever be persisted.
pub(crate) fn classify_logs_error(message: &str, requested: u64) -> Option<RpcError> {
    let msg = message.to_lowercase();

    // Result-size limits. Checked first: some of these messages also mention a
    // block range (Alchemy's "Log response size exceeded ... 2K block range").
    const RESPONSE_TOO_LARGE: &[&str] = &[
        "query returned more than",
        "log response size exceeded",
        "response size exceeded",
        "response is too big",
        "response too large",
        "too many logs",
        "logs matched by query exceeds limit",
        "query exceeds max results",
        "result window is too large",
    ];
    if RESPONSE_TOO_LARGE.iter().any(|p| msg.contains(p)) {
        let count = number_after(&msg, &["more than ", "exceeds limit of ", "limit of "])
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(0);
        return Some(RpcError::ResponseTooLarge(count));
    }

    const RANGE_TOO_LARGE: &[&str] = &[
        "block range",
        "range too large",
        "range is too large",
        "range is too wide",
        "exceed maximum block range",
        "exceeds max block range",
        "max block range",
        "maximum block range",
        "eth_getlogs is limited to",
        "ranges over",
    ];
    if RANGE_TOO_LARGE.iter().any(|p| msg.contains(p))
        && !msg.contains("invalid block range")
        && !msg.contains("fromblock")
    {
        let max = number_after(
            &msg,
            &[
                "max block range ",
                "max block range: ",
                "maximum block range: ",
                "maximum block range ",
                "limited to a ",
                "limited to ",
                "ranges over ",
                "block range limit ",
                "block range limit: ",
                "up to a ",
            ],
        )
        .unwrap_or(0);
        return Some(RpcError::BlockRangeTooLarge { max, requested });
    }

    const RATE_LIMITED: &[&str] = &[
        "rate limit",
        "rate-limit",
        "ratelimit",
        "too many requests",
        "http error 429",
        "status code 429",
        "429 too many",
        "request limit reached",
        "exceeded the quota",
        "compute units per second",
    ];
    if RATE_LIMITED.iter().any(|p| msg.contains(p)) {
        return Some(RpcError::RateLimited(String::new()));
    }

    None
}

/// Parse the integer that follows the first matching prefix ("10,000", "2k", "10000 blocks").
fn number_after(msg: &str, prefixes: &[&str]) -> Option<u64> {
    prefixes.iter().find_map(|prefix| {
        let rest = &msg[msg.find(prefix)? + prefix.len()..];
        let token: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == 'k')
            .collect();
        let (digits, multiplier) = match token.strip_suffix('k') {
            Some(d) => (d, 1000),
            None => (token.as_str(), 1),
        };
        let digits: String = digits.chars().filter(|c| c.is_ascii_digit()).collect();
        digits
            .parse::<u64>()
            .ok()
            .filter(|n| *n > 0)
            .map(|n| n * multiplier)
    })
}

impl RpcError {
    /// Fill in the endpoint for a `RateLimited` error built without one
    fn with_rate_limit_source(self, url: &str) -> Self {
        match self {
            RpcError::RateLimited(s) if s.is_empty() => {
                RpcError::RateLimited(crate::utils::url::redact_url(url))
            }
            other => other,
        }
    }
}

impl std::fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Endpoint")
            .field("url", &self.config.url)
            .field("priority", &self.config.priority)
            .field("max_block_range", &self.config.max_block_range)
            .field("max_logs", &self.config.max_logs)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(msg: &str) -> Option<RpcError> {
        classify_logs_error(msg, 50_000)
    }

    #[test]
    fn classifies_range_limits_and_parses_stated_max() {
        for (msg, max) in [
            ("query exceeds max block range 10000", 10_000),
            ("exceed maximum block range: 50000", 50_000),
            ("eth_getLogs is limited to a 10,000 range", 10_000),
            ("block range is too wide", 0),
            ("Block range too large", 0),
        ] {
            match classify(msg) {
                Some(RpcError::BlockRangeTooLarge { max: m, requested }) => {
                    assert_eq!((m, requested), (max, 50_000), "{msg}");
                }
                other => panic!("{msg}: {other:?}"),
            }
        }
    }

    #[test]
    fn classifies_result_size_limits() {
        for msg in [
            "query returned more than 10000 results",
            "Log response size exceeded. You can make eth_getLogs requests with up to a 2K block range",
            "too many logs",
        ] {
            assert!(
                matches!(classify(msg), Some(RpcError::ResponseTooLarge(_))),
                "{msg}"
            );
        }
        assert!(matches!(
            classify("query returned more than 10000 results"),
            Some(RpcError::ResponseTooLarge(10_000))
        ));
    }

    #[test]
    fn too_many_logs_is_not_a_rate_limit() {
        assert!(!matches!(
            classify("too many logs"),
            Some(RpcError::RateLimited(_))
        ));
        assert!(matches!(
            classify("HTTP error 429 with body: Too Many Requests"),
            Some(RpcError::RateLimited(_))
        ));
    }

    #[test]
    fn unrelated_errors_are_not_classified() {
        for msg in [
            "execution reverted: amount exceeds balance",
            "header not found",
            "invalid block range params: fromBlock > toBlock",
            "gas limit exceeded",
        ] {
            assert!(classify(msg).is_none(), "{msg}");
        }
    }

    #[test]
    fn test_endpoint_creation() {
        let config = EndpointConfig::new("https://eth.llamarpc.com");
        // Can't test actual creation without network, but we can test config
        assert_eq!(config.url, "https://eth.llamarpc.com");
    }
}
