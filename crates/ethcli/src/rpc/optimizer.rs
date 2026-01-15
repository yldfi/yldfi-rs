//! RPC endpoint optimization and capability detection
//!
//! This module provides functions to test and detect RPC endpoint capabilities:
//! - Archive node detection
//! - Debug namespace support
//! - Block range limits
//! - Max logs limits

use crate::config::{Chain, EndpointConfig, NodeType, DEFAULT_MAX_BLOCK_RANGE, DEFAULT_MAX_LOGS};
use crate::error::Result;
use alloy::primitives::{address, Address};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::Filter;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Type alias for the HTTP provider (same as endpoint.rs)
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

/// Result of optimizing an endpoint
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Updated endpoint configuration
    pub config: EndpointConfig,
    /// Detected chain ID
    pub chain_id: u64,
    /// Current block number
    pub current_block: u64,
    /// Whether connectivity test passed
    pub connectivity_ok: bool,
    /// Whether archive detection was performed
    pub archive_tested: bool,
    /// Whether debug namespace was tested
    pub debug_tested: bool,
    /// Whether trace namespace was tested
    pub trace_tested: bool,
    /// Error message if optimization failed
    pub error: Option<String>,
}

/// USDC contract address on Ethereum mainnet (high-activity contract for testing)
const USDC_ADDRESS: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

/// Block 46147 - first contract deployment on Ethereum (used for archive detection)
const FIRST_CONTRACT_BLOCK: u64 = 46147;

/// Address with balance at block 46147 for archive testing
const ARCHIVE_TEST_ADDRESS: Address = address!("5e97870f263700f46aa00d967821199b9bc5a120");

/// Get current timestamp as Unix epoch seconds string.
/// Using Unix timestamp for simplicity and reliability - avoids complex
/// manual date calculations that could have edge case bugs.
fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

/// Optimize an endpoint by detecting its capabilities
pub async fn optimize_endpoint(
    url: &str,
    expected_chain: Option<Chain>,
    timeout_secs: u64,
) -> Result<OptimizationResult> {
    let timeout = Duration::from_secs(timeout_secs);

    // Parse URL
    let parsed_url: reqwest::Url = url
        .parse()
        .map_err(|e| crate::error::RpcError::Provider(format!("Invalid URL: {}", e)))?;

    // Create provider
    let provider: HttpProvider = ProviderBuilder::new().connect_http(parsed_url);

    // Start with a basic config
    let mut config = EndpointConfig::new(url);
    let mut result = OptimizationResult {
        config: config.clone(),
        chain_id: 0,
        current_block: 0,
        connectivity_ok: false,
        archive_tested: false,
        debug_tested: false,
        trace_tested: false,
        error: None,
    };

    // Test 1: Connectivity and chain ID
    let chain_id = match tokio::time::timeout(timeout, provider.get_chain_id()).await {
        Ok(Ok(id)) => id,
        Ok(Err(e)) => {
            result.error = Some(format!("Failed to get chain ID: {}", e));
            return Ok(result);
        }
        Err(_) => {
            result.error = Some("Timeout getting chain ID".to_string());
            return Ok(result);
        }
    };
    result.chain_id = chain_id;
    result.connectivity_ok = true;

    // Detect chain from chain ID
    let detected_chain = Chain::from_chain_id(chain_id);
    config = config.with_chain(detected_chain);

    // Validate chain if expected
    if let Some(expected) = expected_chain {
        if expected.chain_id() != chain_id {
            result.error = Some(format!(
                "Chain mismatch: expected {} ({}), got {} ({})",
                expected.name(),
                expected.chain_id(),
                detected_chain.name(),
                chain_id
            ));
            result.config = config;
            return Ok(result);
        }
    }

    // Test 2: Get current block
    let current_block = match tokio::time::timeout(timeout, provider.get_block_number()).await {
        Ok(Ok(block)) => block,
        Ok(Err(e)) => {
            result.error = Some(format!("Failed to get block number: {}", e));
            result.config = config;
            return Ok(result);
        }
        Err(_) => {
            result.error = Some("Timeout getting block number".to_string());
            result.config = config;
            return Ok(result);
        }
    };
    result.current_block = current_block;

    // Test 3: Archive detection (only for Ethereum mainnet for now)
    if chain_id == 1 {
        match detect_archive_support(&provider, timeout).await {
            Ok((is_archive, from_block)) => {
                result.archive_tested = true;
                if is_archive {
                    config = config.with_node_type(NodeType::Archive);
                    if let Some(block) = from_block {
                        config = config.with_archive_from_block(block);
                    }
                } else {
                    config = config.with_node_type(NodeType::Full);
                }
            }
            Err(e) => {
                // Archive detection failed, mark as unknown
                eprintln!("  Archive detection error: {}", e);
                config = config.with_node_type(NodeType::Unknown);
            }
        }
    }

    // Test 4: Debug namespace detection (debug_traceCall)
    match detect_debug_support(url, timeout).await {
        Ok(has_debug) => {
            result.debug_tested = true;
            config = config.with_debug(has_debug);
        }
        Err(e) => {
            eprintln!("  Debug detection error: {}", e);
        }
    }

    // Test 5: Trace namespace detection (trace_call)
    match detect_trace_support(url, timeout).await {
        Ok(has_trace) => {
            result.trace_tested = true;
            config = config.with_trace(has_trace);
        }
        Err(e) => {
            eprintln!("  Trace detection error: {}", e);
        }
    }

    // Test 6: Block range limit detection
    match detect_block_range_limit(&provider, current_block, timeout).await {
        Ok(max_range) => {
            config.max_block_range = max_range;
        }
        Err(e) => {
            eprintln!("  Block range detection error: {}", e);
        }
    }

    // Test 7: Max logs detection (only for Ethereum mainnet)
    if chain_id == 1 {
        match detect_max_logs_limit(&provider, current_block, timeout).await {
            Ok(max_logs) => {
                config.max_logs = max_logs;
            }
            Err(e) => {
                eprintln!("  Max logs detection error: {}", e);
            }
        }
    }

    // Set last tested timestamp
    config = config.with_last_tested(current_timestamp());

    result.config = config;
    Ok(result)
}

/// Detect if endpoint supports archive queries
async fn detect_archive_support(
    provider: &HttpProvider,
    timeout: Duration,
) -> Result<(bool, Option<u64>)> {
    // Try to get balance at block 46147 (first contract deployment)
    let block_id =
        alloy::eips::BlockId::Number(alloy::eips::BlockNumberOrTag::Number(FIRST_CONTRACT_BLOCK));

    match tokio::time::timeout(
        timeout,
        provider
            .get_balance(ARCHIVE_TEST_ADDRESS)
            .block_id(block_id),
    )
    .await
    {
        Ok(Ok(_balance)) => {
            // Success - this is an archive node with full history
            Ok((true, Some(0)))
        }
        Ok(Err(e)) => {
            let error_str = e.to_string().to_lowercase();
            if error_str.contains("missing trie node")
                || error_str.contains("pruned")
                || error_str.contains("not available")
                || error_str.contains("state not available")
            {
                // This is a pruned/full node
                Ok((false, None))
            } else {
                // Some other error - could still be archive
                Err(
                    crate::error::RpcError::Provider(format!("Archive detection error: {}", e))
                        .into(),
                )
            }
        }
        Err(_) => {
            // Timeout - assume it might be archive but slow
            Ok((true, None))
        }
    }
}

/// Detect if endpoint supports debug namespace (debug_traceCall)
async fn detect_debug_support(url: &str, timeout: Duration) -> Result<bool> {
    let client = reqwest::Client::new();

    // Try a minimal debug_traceCall
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "debug_traceCall",
        "params": [
            {
                "to": "0x0000000000000000000000000000000000000000",
                "data": "0x"
            },
            "latest",
            {"tracer": "callTracer"}
        ],
        "id": 1
    });

    match tokio::time::timeout(
        timeout,
        client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send(),
    )
    .await
    {
        Ok(Ok(response)) => {
            let result: serde_json::Value = response.json().await.unwrap_or_default();

            // Check if there's an error indicating method not found
            if let Some(error) = result.get("error") {
                let error_str = error.to_string().to_lowercase();
                if error_str.contains("method not found")
                    || error_str.contains("not supported")
                    || error_str.contains("unknown method")
                {
                    return Ok(false);
                }
            }

            // If we got a result or a different kind of error, debug is likely supported
            Ok(result.get("result").is_some() || result.get("error").is_some())
        }
        Ok(Err(_)) => Ok(false),
        Err(_) => Ok(false), // Timeout - assume no debug
    }
}

/// Detect if endpoint supports trace namespace (trace_call - Parity/Erigon style)
async fn detect_trace_support(url: &str, timeout: Duration) -> Result<bool> {
    let client = reqwest::Client::new();

    // Try a minimal trace_call
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "trace_call",
        "params": [
            {
                "to": "0x0000000000000000000000000000000000000000",
                "data": "0x"
            },
            ["trace"],
            "latest"
        ],
        "id": 1
    });

    match tokio::time::timeout(
        timeout,
        client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send(),
    )
    .await
    {
        Ok(Ok(response)) => {
            let result: serde_json::Value = response.json().await.unwrap_or_default();

            // Check if there's an error indicating method not found
            if let Some(error) = result.get("error") {
                let error_str = error.to_string().to_lowercase();
                if error_str.contains("method not found")
                    || error_str.contains("not supported")
                    || error_str.contains("unknown method")
                    || error_str.contains("does not exist")
                {
                    return Ok(false);
                }
            }

            // If we got a result or a different kind of error, trace is likely supported
            Ok(result.get("result").is_some() || result.get("error").is_some())
        }
        Ok(Err(_)) => Ok(false),
        Err(_) => Ok(false), // Timeout - assume no trace
    }
}

/// Detect maximum block range for eth_getLogs
async fn detect_block_range_limit(
    provider: &HttpProvider,
    current_block: u64,
    timeout: Duration,
) -> Result<u64> {
    // Binary search for the maximum working block range
    let mut low: u64 = 100;
    let mut high: u64 = 2_000_000;
    let mut best: u64 = DEFAULT_MAX_BLOCK_RANGE; // Default fallback

    // Use an address that won't return many logs to test range limits
    let empty_address = address!("0000000000000000000000000000000000000001");

    while low <= high {
        let mid = low + (high - low) / 2;

        // Calculate block range ending at current block
        let from_block = current_block.saturating_sub(mid);

        let filter = Filter::new()
            .address(empty_address)
            .from_block(from_block)
            .to_block(current_block);

        match tokio::time::timeout(timeout, provider.get_logs(&filter)).await {
            Ok(Ok(_)) => {
                // This range works
                best = mid;
                low = mid + 1;
            }
            Ok(Err(e)) => {
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("block range")
                    || error_str.contains("exceed")
                    || error_str.contains("too large")
                    || error_str.contains("max")
                {
                    // Range too large, try smaller
                    high = mid - 1;
                } else if error_str.contains("rate") || error_str.contains("429") {
                    // Rate limited, return current best
                    break;
                } else {
                    // Some other error, try smaller range
                    high = mid - 1;
                }
            }
            Err(_) => {
                // Timeout, try smaller range
                high = mid - 1;
            }
        }

        // Stop if ranges are too close
        if high.saturating_sub(low) < 1000 {
            break;
        }
    }

    Ok(best)
}

/// Detect maximum logs limit
async fn detect_max_logs_limit(
    provider: &HttpProvider,
    current_block: u64,
    timeout: Duration,
) -> Result<usize> {
    // Try to get logs from USDC (high activity) to find the limit
    // Start with a small range and increase

    let mut test_range: u64 = 100;
    let mut last_log_count: usize = 0;
    let mut max_logs: usize = DEFAULT_MAX_LOGS; // Default

    while test_range <= DEFAULT_MAX_BLOCK_RANGE {
        let from_block = current_block.saturating_sub(test_range);

        let filter = Filter::new()
            .address(USDC_ADDRESS)
            .from_block(from_block)
            .to_block(current_block);

        match tokio::time::timeout(timeout, provider.get_logs(&filter)).await {
            Ok(Ok(logs)) => {
                let count = logs.len();

                // If we're getting fewer logs than before with larger range,
                // we might be hitting a limit
                if count < last_log_count && last_log_count > 0 {
                    max_logs = last_log_count;
                    break;
                }

                last_log_count = count;

                // If we get DEFAULT_MAX_LOGS+ logs, that's likely the default limit
                if count >= DEFAULT_MAX_LOGS {
                    max_logs = count;
                    break;
                }

                test_range *= 2;
            }
            Ok(Err(e)) => {
                let error_str = e.to_string().to_lowercase();
                if error_str.contains("10000") || error_str.contains("too many") {
                    // Hit the limit
                    max_logs = DEFAULT_MAX_LOGS;
                    break;
                } else if error_str.contains("response size") {
                    // Response too large
                    max_logs = last_log_count.max(DEFAULT_MAX_LOGS);
                    break;
                }
                break;
            }
            Err(_) => {
                // Timeout
                break;
            }
        }
    }

    Ok(max_logs)
}

/// Quick connectivity test for an endpoint
pub async fn test_connectivity(url: &str, timeout_secs: u64) -> Result<(u64, u64)> {
    let timeout = Duration::from_secs(timeout_secs);

    let parsed_url: reqwest::Url = url
        .parse()
        .map_err(|e| crate::error::RpcError::Provider(format!("Invalid URL: {}", e)))?;

    let provider: HttpProvider = ProviderBuilder::new().connect_http(parsed_url);

    let timeout_ms = timeout.as_millis() as u64;

    let chain_id = tokio::time::timeout(timeout, provider.get_chain_id())
        .await
        .map_err(|_| crate::error::RpcError::Timeout(timeout_ms))?
        .map_err(|e| crate::error::RpcError::Provider(e.to_string()))?;

    let block_number = tokio::time::timeout(timeout, provider.get_block_number())
        .await
        .map_err(|_| crate::error::RpcError::Timeout(timeout_ms))?
        .map_err(|e| crate::error::RpcError::Provider(e.to_string()))?;

    Ok((chain_id, block_number))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_detection() {
        assert_eq!(Chain::from_chain_id(1), Chain::Ethereum);
        assert_eq!(Chain::from_chain_id(137), Chain::Polygon);
        assert_eq!(Chain::from_chain_id(42161), Chain::Arbitrum);
    }

    #[test]
    fn test_timestamp_format() {
        let ts = current_timestamp();
        // Should be Unix epoch seconds (numeric string)
        let epoch: u64 = ts.parse().expect("timestamp should be numeric");
        // Should be a reasonable timestamp (after 2020-01-01)
        assert!(epoch > 1577836800);
    }
}
