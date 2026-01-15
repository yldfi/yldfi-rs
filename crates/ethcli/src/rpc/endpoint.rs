//! Single RPC endpoint wrapper

use crate::config::EndpointConfig;
use crate::error::{Result, RpcError};
use alloy::primitives::B256;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Log, Transaction, TransactionReceipt};
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
                crate::error::sanitize_error_message(proxy_url)
            ))
            .into());
        }

        // Parse URL
        let url: reqwest::Url = config.url.parse().map_err(|e| {
            RpcError::ConnectionFailed(format!(
                "Invalid URL {}: {}",
                crate::error::sanitize_error_message(&config.url),
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
            Ok(Err(e)) => {
                Err(RpcError::Provider(crate::error::sanitize_error_message(&e.to_string())).into())
            }
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
                let err_str = e.to_string().to_lowercase();

                // Check for rate limiting
                if err_str.contains("rate")
                    || err_str.contains("429")
                    || err_str.contains("too many")
                {
                    return Err(RpcError::RateLimited(self.config.url.clone()).into());
                }

                // Check for block range too large
                if err_str.contains("block range")
                    || err_str.contains("exceed")
                    || err_str.contains("too large")
                {
                    // Try to extract the actual requested range from the filter
                    let requested = filter
                        .get_from_block()
                        .and_then(|from| filter.get_to_block().map(|to| to.saturating_sub(from)))
                        .unwrap_or(0);

                    return Err(RpcError::BlockRangeTooLarge {
                        max: self.config.max_block_range,
                        requested,
                    }
                    .into());
                }

                // Check for response too large
                if err_str.contains("response size")
                    || err_str.contains("too many logs")
                    || err_str.contains("10000")
                {
                    return Err(RpcError::ResponseTooLarge(0).into());
                }

                Err(RpcError::Provider(crate::error::sanitize_error_message(&e.to_string())).into())
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
            Ok(Err(e)) => {
                Err(RpcError::Provider(crate::error::sanitize_error_message(&e.to_string())).into())
            }
            Err(_) => Err(RpcError::Timeout(self.timeout.as_millis() as u64).into()),
        }
    }

    /// Get a transaction receipt by hash
    pub async fn get_transaction_receipt(&self, hash: B256) -> Result<Option<TransactionReceipt>> {
        let result =
            tokio::time::timeout(self.timeout, self.provider.get_transaction_receipt(hash)).await;

        match result {
            Ok(Ok(receipt)) => Ok(receipt),
            Ok(Err(e)) => {
                Err(RpcError::Provider(crate::error::sanitize_error_message(&e.to_string())).into())
            }
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

    #[test]
    fn test_endpoint_creation() {
        let config = EndpointConfig::new("https://eth.llamarpc.com");
        // Can't test actual creation without network, but we can test config
        assert_eq!(config.url, "https://eth.llamarpc.com");
    }
}
