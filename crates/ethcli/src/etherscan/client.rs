//! Extended Etherscan client
//!
//! Wraps `foundry-block-explorers::Client` with additional functionality:
//! - Signature cache for function/event lookups
//! - 4byte.directory integration
//! - Token metadata via eth_call proxy

use crate::config::Chain;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Well-known function signatures that should be prioritized over hash collisions
/// Format: selector -> canonical signature
static KNOWN_FUNCTION_SIGNATURES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            // ERC20
            ("0xa9059cbb", "transfer(address,uint256)"),
            ("0x23b872dd", "transferFrom(address,address,uint256)"),
            ("0x095ea7b3", "approve(address,uint256)"),
            ("0x70a08231", "balanceOf(address)"),
            ("0xdd62ed3e", "allowance(address,address)"),
            ("0x18160ddd", "totalSupply()"),
            ("0x06fdde03", "name()"),
            ("0x95d89b41", "symbol()"),
            ("0x313ce567", "decimals()"),
            // ERC721
            ("0x6352211e", "ownerOf(uint256)"),
            ("0x42842e0e", "safeTransferFrom(address,address,uint256)"),
            (
                "0xb88d4fde",
                "safeTransferFrom(address,address,uint256,bytes)",
            ),
            ("0xe985e9c5", "isApprovedForAll(address,address)"),
            ("0xa22cb465", "setApprovalForAll(address,bool)"),
            ("0x081812fc", "getApproved(uint256)"),
            // Common
            ("0x8da5cb5b", "owner()"),
            ("0x715018a6", "renounceOwnership()"),
            ("0xf2fde38b", "transferOwnership(address)"),
            ("0x5c975abb", "paused()"),
            ("0x8456cb59", "pause()"),
            ("0x3f4ba83a", "unpause()"),
            // Uniswap/DEX
            (
                "0x38ed1739",
                "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)",
            ),
            (
                "0x7ff36ab5",
                "swapExactETHForTokens(uint256,address[],address,uint256)",
            ),
            (
                "0x18cbafe5",
                "swapExactTokensForETH(uint256,uint256,address[],address,uint256)",
            ),
            ("0x022c0d9f", "swap(uint256,uint256,address,bytes)"),
            // Proxy patterns
            ("0x3659cfe6", "upgradeTo(address)"),
            ("0x4f1ef286", "upgradeToAndCall(address,bytes)"),
            ("0x5c60da1b", "implementation()"),
            // Multicall
            ("0xac9650d8", "multicall(bytes[])"),
            ("0x5ae401dc", "multicall(uint256,bytes[])"),
        ])
    });

/// Well-known event signatures that should be prioritized over hash collisions
/// Format: topic0 -> canonical signature
static KNOWN_EVENT_SIGNATURES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            // ERC20
            (
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                "Transfer(address,address,uint256)",
            ),
            (
                "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
                "Approval(address,address,uint256)",
            ),
            // Common
            (
                "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0",
                "OwnershipTransferred(address,address)",
            ),
            (
                "0x62e78cea01bee320cd4e420270b5ea74000d11b0c9f74754ebdbfc544b05a258",
                "Paused(address)",
            ),
            (
                "0x5db9ee0a495bf2e6ff9c91a7834c1ba4fdd244a5e8aa4e537bd38aeae4b073aa",
                "Unpaused(address)",
            ),
            // Uniswap
            (
                "0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822",
                "Swap(address,uint256,uint256,uint256,uint256,address)",
            ),
            (
                "0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1",
                "Sync(uint112,uint112)",
            ),
            (
                "0x4c209b5fc8ad50758f13e2e1088ba56a560dff690a1c6fef26394f4c03821c4f",
                "Mint(address,uint256,uint256)",
            ),
            (
                "0xdccd412f0b1252819cb1fd330b93224ca42612892bb3f4f789976e6d81936496",
                "Burn(address,uint256,uint256,address)",
            ),
        ])
    });

/// Reorder signatures to prioritize known canonical signatures
fn prioritize_known_signatures(
    selector: &str,
    mut signatures: Vec<String>,
    known: &HashMap<&'static str, &'static str>,
) -> Vec<String> {
    if let Some(&canonical) = known.get(selector) {
        // Find and move the canonical signature to the front
        if let Some(pos) = signatures.iter().position(|s| s == canonical) {
            let sig = signatures.remove(pos);
            signatures.insert(0, sig);
        } else {
            // Canonical signature not in results, add it at the front
            signatures.insert(0, canonical.to_string());
        }
    }
    signatures
}
use crate::error::{AbiError, Result};
use crate::etherscan::SignatureCache;
use crate::utils::{
    decode_string_from_hex, decode_uint8_from_hex, urlencoding_encode, TokenMetadata,
};
use alloy_chains::Chain as AlloyChain;
use foundry_block_explorers::Client as EtherscanClient;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

/// Extended Etherscan client with signature caching and 4byte lookups
pub struct Client {
    /// Inner foundry-block-explorers client
    inner: EtherscanClient,
    /// HTTP client for 4byte.directory and eth_call
    http: reqwest::Client,
    /// Signature cache
    cache: Arc<SignatureCache>,
    /// Chain
    chain: Chain,
    /// API key (if any)
    api_key: Option<String>,
}

impl Client {
    /// Create a new client for the given chain
    pub fn new(chain: Chain, api_key: Option<String>) -> Result<Self> {
        // Convert our Chain enum to alloy_chains::Chain for foundry-block-explorers
        let alloy_chain = AlloyChain::from_id(chain.chain_id());

        let inner = EtherscanClient::builder()
            .chain(alloy_chain)
            .map_err(|e| AbiError::EtherscanFetch(format!("Unsupported chain: {}", e)))?
            .with_api_key(api_key.as_deref().unwrap_or_default())
            .build()
            .map_err(|e| AbiError::EtherscanFetch(format!("Failed to build client: {}", e)))?;

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AbiError::HttpClientInit(e.to_string()))?;

        Ok(Self {
            inner,
            http,
            cache: Arc::new(SignatureCache::new()),
            chain,
            api_key,
        })
    }

    /// Create a new client with a shared cache
    pub fn with_cache(
        chain: Chain,
        api_key: Option<String>,
        cache: Arc<SignatureCache>,
    ) -> Result<Self> {
        let mut client = Self::new(chain, api_key)?;
        client.cache = cache;
        Ok(client)
    }

    /// Get the inner foundry-block-explorers client
    pub fn inner(&self) -> &EtherscanClient {
        &self.inner
    }

    /// Get the chain
    pub fn chain(&self) -> Chain {
        self.chain
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::etherscan::CacheStats {
        self.cache.stats()
    }

    /// Get the signature cache
    pub fn cache(&self) -> &SignatureCache {
        &self.cache
    }

    // ========================================================================
    // Signature lookups (our unique value-add)
    // ========================================================================

    /// Lookup function signature by 4-byte selector
    ///
    /// Checks cache first, then queries 4byte.directory.
    /// Returns the first (most popular) result.
    pub async fn lookup_selector(&self, selector: &str) -> Option<String> {
        self.lookup_selector_all(selector)
            .await
            .and_then(|v| v.into_iter().next())
    }

    /// Lookup all function signatures by 4-byte selector
    ///
    /// Returns all matching signatures from 4byte.directory, sorted by popularity.
    /// The first result is typically the most common/correct one.
    /// If 4byte.directory has no results, falls back to searching cached ABIs.
    pub async fn lookup_selector_all(&self, selector: &str) -> Option<Vec<String>> {
        let normalized = format!(
            "0x{}",
            selector
                .strip_prefix("0x")
                .unwrap_or(selector)
                .to_lowercase()
        );

        // Check negative cache first - skip lookup if we recently found nothing
        if self.cache.is_not_found(&normalized) {
            tracing::debug!("Selector {} in negative cache, skipping lookup", normalized);
            return None;
        }

        // Try 4byte.directory first
        let url = format!(
            "https://www.4byte.directory/api/v1/signatures/?hex_signature={}",
            normalized
        );

        if let Ok(response) = self.http.get(&url).send().await {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                // Get all results
                let results: Vec<String> = json["results"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|r| r.get("text_signature")?.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                if !results.is_empty() {
                    // Prioritize known signatures over hash collisions
                    let results = prioritize_known_signatures(
                        &normalized,
                        results,
                        &KNOWN_FUNCTION_SIGNATURES,
                    );
                    // Cache the first (now prioritized) result for future single lookups
                    self.cache.set_function(&normalized, &results[0]);
                    tracing::debug!("Cached selector {} -> {}", normalized, results[0]);
                    return Some(results);
                }
            }
        }

        // Fallback: search cached ABIs for matching function
        let selector_bytes: [u8; 4] =
            hex::decode(normalized.strip_prefix("0x").unwrap_or(&normalized))
                .ok()?
                .try_into()
                .ok()?;

        if let Some(sig) = self.cache.find_function_in_abis(&selector_bytes) {
            self.cache.set_function(&normalized, &sig);
            tracing::debug!("Found selector {} in cached ABI -> {}", normalized, sig);
            return Some(vec![sig]);
        }

        // Nothing found anywhere - add to negative cache to avoid repeated lookups
        self.cache.set_not_found(&normalized);
        tracing::debug!("Selector {} not found, added to negative cache", normalized);
        None
    }

    /// Lookup event signature by topic0 hash
    ///
    /// Checks cache first, then queries 4byte.directory.
    /// Returns the first (most popular) result.
    pub async fn lookup_event(&self, topic0: &str) -> Option<String> {
        self.lookup_event_all(topic0)
            .await
            .and_then(|v| v.into_iter().next())
    }

    /// Lookup all event signatures by topic0 hash
    ///
    /// Returns all matching signatures from 4byte.directory, sorted by popularity.
    /// If 4byte.directory has no results, falls back to searching cached ABIs.
    pub async fn lookup_event_all(&self, topic0: &str) -> Option<Vec<String>> {
        let normalized = format!(
            "0x{}",
            topic0.strip_prefix("0x").unwrap_or(topic0).to_lowercase()
        );

        // Check negative cache first - skip lookup if we recently found nothing
        if self.cache.is_not_found(&normalized) {
            tracing::debug!("Event {} in negative cache, skipping lookup", normalized);
            return None;
        }

        // Try 4byte.directory first
        let url = format!(
            "https://www.4byte.directory/api/v1/event-signatures/?hex_signature={}",
            normalized
        );

        if let Ok(response) = self.http.get(&url).send().await {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                // Get all results
                let results: Vec<String> = json["results"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|r| r.get("text_signature")?.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                if !results.is_empty() {
                    // Prioritize known signatures over hash collisions
                    let results =
                        prioritize_known_signatures(&normalized, results, &KNOWN_EVENT_SIGNATURES);
                    // Cache the first (now prioritized) result for future single lookups
                    self.cache.set_event(&normalized, &results[0]);
                    tracing::debug!("Cached event {} -> {}", normalized, results[0]);
                    return Some(results);
                }
            }
        }

        // Fallback: search cached ABIs for matching event
        let topic_bytes: [u8; 32] =
            hex::decode(normalized.strip_prefix("0x").unwrap_or(&normalized))
                .ok()?
                .try_into()
                .ok()?;

        if let Some(sig) = self.cache.find_event_in_abis(&topic_bytes) {
            self.cache.set_event(&normalized, &sig);
            tracing::debug!("Found event {} in cached ABI -> {}", normalized, sig);
            return Some(vec![sig]);
        }

        // Nothing found anywhere - add to negative cache to avoid repeated lookups
        self.cache.set_not_found(&normalized);
        tracing::debug!("Event {} not found, added to negative cache", normalized);
        None
    }

    // ========================================================================
    // Token metadata via eth_call (our unique value-add)
    // ========================================================================

    /// Get token metadata via RPC calls (ERC20 standard methods)
    ///
    /// Uses Etherscan's eth_call proxy to fetch name(), symbol(), decimals()
    pub async fn get_token_metadata(&self, address: &str) -> Result<TokenMetadata> {
        let chain_id = self.chain.chain_id();

        // name() = 0x06fdde03
        // symbol() = 0x95d89b41
        // decimals() = 0x313ce567

        let name = self
            .eth_call(chain_id, address, "0x06fdde03")
            .await
            .ok()
            .and_then(|data| decode_string_from_hex(&data));

        let symbol = self
            .eth_call(chain_id, address, "0x95d89b41")
            .await
            .ok()
            .and_then(|data| decode_string_from_hex(&data));

        let decimals = self
            .eth_call(chain_id, address, "0x313ce567")
            .await
            .ok()
            .and_then(|data| decode_uint8_from_hex(&data));

        Ok(TokenMetadata {
            name,
            symbol,
            decimals,
        })
    }

    /// Make an eth_call via Etherscan proxy
    async fn eth_call(&self, chain_id: u64, to: &str, data: &str) -> Result<String> {
        let encoded_to: Cow<str> = urlencoding_encode(to);
        let encoded_data: Cow<str> = urlencoding_encode(data);

        let base_url = format!(
            "https://api.etherscan.io/v2/api?chainid={}&module=proxy&action=eth_call&to={}&data={}&tag=latest",
            chain_id, encoded_to, encoded_data
        );

        let url = if let Some(key) = &self.api_key {
            let encoded_key: Cow<str> = urlencoding_encode(key);
            format!("{}&apikey={}", base_url, encoded_key)
        } else {
            base_url
        };

        let response = self.http.get(&url).send().await.map_err(|e| {
            AbiError::EtherscanFetch(format!(
                "eth_call failed: {}",
                crate::error::sanitize_error_message(&e.to_string())
            ))
        })?;

        let json: serde_json::Value = response.json().await.map_err(|e| {
            AbiError::EtherscanFetch(format!("Failed to parse eth_call response: {}", e))
        })?;

        // Result should be a hex string
        let result = json["result"]
            .as_str()
            .ok_or_else(|| AbiError::ParseError("Missing result in eth_call".to_string()))?;

        // Check for empty result (0x is empty, but 0x00 or 0x01 are valid)
        if result == "0x" {
            return Err(AbiError::ParseError("Empty eth_call result".to_string()).into());
        }

        Ok(result.to_string())
    }
}

// Allow accessing inner client methods directly
impl std::ops::Deref for Client {
    type Target = EtherscanClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Tests for decode_* functions are in utils.rs
