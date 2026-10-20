//! Etherscan ABI fetcher
//!
//! The `contract/*` calls go through the [`crate::etherscan::Client`] wrapper
//! around foundry-block-explorers, the same path `ethcli contract` uses. Only
//! the `proxy/eth_getTransactionByHash` lookup is built by hand, because the
//! library has no proxy module.

use crate::config::Chain;
use crate::error::{AbiError, Error, Result};
use crate::etherscan::{ensure_etherscan_supported, Client as EtherscanClient, SignatureCache};
use crate::utils::{get_shared_http_client, urlencoding_encode, TokenMetadata};
use alloy::json_abi::JsonAbi;
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

/// Contract creation info from Etherscan
#[derive(Debug, Clone)]
pub struct ContractCreation {
    /// Block number where contract was created
    pub block_number: u64,
    /// Transaction hash of contract creation
    pub tx_hash: String,
    /// Contract creator address
    pub creator: String,
}

/// Contract metadata from Etherscan
#[derive(Debug, Clone)]
pub struct ContractMetadata {
    /// Contract name (from source code verification)
    pub name: Option<String>,
    /// Is the contract verified on Etherscan
    pub is_verified: bool,
    /// Is this a proxy contract
    pub is_proxy: bool,
    /// Implementation address (if proxy)
    pub implementation: Option<String>,
}

/// ABI fetcher from Etherscan and local files
pub struct AbiFetcher {
    /// HTTP client
    client: reqwest::Client,
    /// Etherscan API key (optional)
    api_key: Option<String>,
    /// Signature cache for function/event lookups
    cache: Arc<SignatureCache>,
}

impl AbiFetcher {
    /// Create a new ABI fetcher
    ///
    /// Uses a shared HTTP client (PERF-003 fix) to avoid duplicate connection pools.
    /// Returns an error if the HTTP client cannot be initialized (rare, usually
    /// indicates TLS backend issues).
    pub fn new(api_key: Option<String>) -> Result<Self> {
        // Use shared HTTP client to avoid duplicate connection pools
        let client = get_shared_http_client()
            .map_err(AbiError::HttpClientInit)?
            .clone();

        Ok(Self {
            client,
            api_key,
            cache: Arc::new(SignatureCache::new()),
        })
    }

    /// Create a new ABI fetcher with a custom cache
    ///
    /// Uses a shared HTTP client (PERF-003 fix) to avoid duplicate connection pools.
    pub fn with_cache(api_key: Option<String>, cache: Arc<SignatureCache>) -> Result<Self> {
        // Use shared HTTP client to avoid duplicate connection pools
        let client = get_shared_http_client()
            .map_err(AbiError::HttpClientInit)?
            .clone();

        Ok(Self {
            client,
            api_key,
            cache,
        })
    }

    /// Etherscan client for `chain`, sharing this fetcher's API key and cache
    fn etherscan(&self, chain: Chain) -> Result<EtherscanClient> {
        EtherscanClient::with_cache(chain, self.api_key.clone(), Arc::clone(&self.cache))
    }

    /// Fetch ABI from Etherscan
    ///
    /// Uses local cache to avoid repeated fetches.
    pub async fn fetch_from_etherscan(&self, chain: Chain, address: &str) -> Result<JsonAbi> {
        let client = self.etherscan(chain)?;
        if let Some(abi) = client.cached_abi(address) {
            tracing::debug!(
                "Using cached ABI for {} on chain {}",
                address,
                chain.chain_id()
            );
            return Ok(abi);
        }

        tracing::debug!(
            "Fetching ABI from Etherscan for {} on chain {}",
            address,
            chain.chain_id()
        );
        client.fetch_abi(address).await
    }

    /// Load ABI from a local file
    pub fn load_from_file(&self, path: &Path) -> Result<JsonAbi> {
        if !path.exists() {
            return Err(AbiError::FileNotFound(path.display().to_string()).into());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| AbiError::FileNotFound(format!("{}: {}", path.display(), e)))?;

        // Try to parse as JSON ABI
        let abi: JsonAbi = serde_json::from_str(&content)
            .map_err(|e| AbiError::ParseError(format!("Invalid ABI JSON: {}", e)))?;

        Ok(abi)
    }

    /// Get events from an ABI
    pub fn get_events(abi: &JsonAbi) -> Vec<&alloy::json_abi::Event> {
        abi.events().collect()
    }

    /// Find an event by name in an ABI (case-insensitive)
    pub fn find_event<'a>(abi: &'a JsonAbi, name: &str) -> Option<&'a alloy::json_abi::Event> {
        let name_lower = name.to_lowercase();
        abi.events().find(|e| e.name.to_lowercase() == name_lower)
    }

    /// Get the event selector (topic0) for an event
    pub fn event_selector(event: &alloy::json_abi::Event) -> alloy::primitives::B256 {
        event.selector()
    }

    /// Get the full event signature string from an ABI event
    /// e.g., "Transfer(address,address,uint256)"
    pub fn event_signature_string(event: &alloy::json_abi::Event) -> String {
        let param_types: Vec<String> = event.inputs.iter().map(|p| p.ty.to_string()).collect();
        format!("{}({})", event.name, param_types.join(","))
    }

    /// Resolve an event name to its full signature using the contract ABI
    /// Returns the full signature string like "TokenExchange(address,uint256,uint256,uint256,uint256)"
    pub async fn resolve_event_name(
        &self,
        chain: Chain,
        contract: &str,
        event_name: &str,
    ) -> Result<String> {
        // 1. The contract's own ABI
        let own_abi = self.fetch_from_etherscan(chain, contract).await;
        if let Ok(abi) = &own_abi {
            if let Some(event) = Self::find_event(abi, event_name) {
                return Ok(Self::event_signature_string(event));
            }
        }

        // 2. Proxy: follow Etherscan's `Implementation` field
        if let Ok(meta) = self.get_contract_metadata(chain, contract).await {
            if let Some(implementation) = meta.implementation {
                if !implementation.eq_ignore_ascii_case(contract) {
                    tracing::debug!(
                        "Event '{}' not in proxy ABI; checking implementation {}",
                        event_name,
                        implementation
                    );
                    if let Ok(impl_abi) = self.fetch_from_etherscan(chain, &implementation).await {
                        if let Some(event) = Self::find_event(&impl_abi, event_name) {
                            return Ok(Self::event_signature_string(event));
                        }
                    }
                }
            }
        }

        // 3. Well-known standard events
        if let Some(sig) = well_known_event_signature(event_name) {
            tracing::debug!("Using well-known signature for '{}': {}", event_name, sig);
            return Ok(sig.to_string());
        }

        match own_abi {
            Err(e) => Err(e),
            Ok(_) => Err(crate::error::AbiError::EventNotFound(format!(
                "Event '{}' not found in contract (or implementation) ABI. \
                 Pass the full signature, e.g. -e \"{}(address,uint256)\"",
                event_name, event_name
            ))
            .into()),
        }
    }

    /// Get contract creation info from Etherscan API v2
    pub async fn get_contract_creation(
        &self,
        chain: Chain,
        contract: &str,
    ) -> Result<ContractCreation> {
        tracing::debug!(
            "Fetching contract creation for {} on chain {}",
            contract,
            chain.chain_id()
        );

        let data = self
            .etherscan(chain)?
            .creation_data(contract)
            .await
            .map_err(|e| match e {
                Error::Abi(AbiError::EtherscanFetch(msg)) => {
                    AbiError::EtherscanFetch(format!("Failed to get contract creation: {}", msg))
                        .into()
                }
                other => other,
            })?;

        let tx_hash = format!("{:#x}", data.transaction_hash);
        let creator = format!("{:#x}", data.contract_creator);

        // We need to fetch the transaction to get the block number
        // Use eth_getTransactionByHash via a simple RPC call
        let block_number = self.get_tx_block_number(chain, &tx_hash).await.unwrap_or(0);

        Ok(ContractCreation {
            block_number,
            tx_hash,
            creator,
        })
    }

    /// Get contract metadata (name, verification status) from Etherscan
    pub async fn get_contract_metadata(
        &self,
        chain: Chain,
        address: &str,
    ) -> Result<ContractMetadata> {
        tracing::debug!(
            "Fetching contract metadata for {} on chain {}",
            address,
            chain.chain_id()
        );

        let unverified = ContractMetadata {
            name: None,
            is_verified: false,
            is_proxy: false,
            implementation: None,
        };

        let source = match self.etherscan(chain)?.source_code(address).await {
            Ok(source) => source,
            Err(Error::Abi(AbiError::ContractNotVerified(_))) => return Ok(unverified),
            Err(e) => return Err(e),
        };

        let Some(first) = source.items.first() else {
            return Ok(unverified);
        };

        // Check if verified by looking at ABI field
        let is_verified = !first.abi.is_empty() && !first.abi.starts_with("Contract");

        let name = Some(first.contract_name.clone()).filter(|s| !s.is_empty());

        Ok(ContractMetadata {
            name,
            is_verified,
            is_proxy: first.proxy == 1,
            implementation: first.implementation.map(|a| format!("{:#x}", a)),
        })
    }

    /// Get token metadata via Etherscan's `eth_call` proxy (ERC20 name/symbol/decimals)
    pub async fn get_token_metadata_rpc(
        &self,
        chain: Chain,
        address: &str,
    ) -> Result<TokenMetadata> {
        self.etherscan(chain)?.get_token_metadata(address).await
    }

    /// Get transaction block number from Etherscan
    async fn get_tx_block_number(&self, chain: Chain, tx_hash: &str) -> Result<u64> {
        let chain_id = chain.chain_id();
        ensure_etherscan_supported(chain_id)?;
        let encoded_hash: Cow<str> = urlencoding_encode(tx_hash);

        let base_url = format!(
            "https://api.etherscan.io/v2/api?chainid={}&module=proxy&action=eth_getTransactionByHash&txhash={}",
            chain_id, encoded_hash
        );

        let url = if let Some(key) = &self.api_key {
            let encoded_key: Cow<str> = urlencoding_encode(key);
            format!("{}&apikey={}", base_url, encoded_key)
        } else {
            base_url
        };

        let response = self.client.get(&url).send().await.map_err(|e| {
            AbiError::EtherscanFetch(format!(
                "Request failed: {}",
                crate::error::sanitize_error_message(&e.to_string())
            ))
        })?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AbiError::EtherscanFetch(format!("Failed to parse response: {}", e)))?;

        // Extract block number from result.blockNumber (hex string)
        let block_hex = json["result"]["blockNumber"]
            .as_str()
            .ok_or_else(|| AbiError::ParseError("Missing blockNumber".to_string()))?;

        // Parse hex block number (0x...)
        let block_number = u64::from_str_radix(block_hex.trim_start_matches("0x"), 16)
            .map_err(|_| AbiError::ParseError(format!("Invalid block number: {}", block_hex)))?;

        Ok(block_number)
    }
}

impl AbiFetcher {
    /// Create a new ABI fetcher with default settings (no API key)
    ///
    /// This is a convenience method for tests only.
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be initialized (extremely rare).
    #[cfg(test)]
    pub fn new_default() -> Self {
        Self::new(None).expect("Failed to initialize HTTP client")
    }

    /// Decode function call data using the contract ABI
    ///
    /// Returns `None` if the ABI is unavailable or does not contain the selector
    /// (e.g. a proxy ABI), so callers can fall back to a signature database.
    pub async fn decode_function_call(
        &self,
        chain: Chain,
        contract: &str,
        calldata: &[u8],
    ) -> Option<DecodedFunction> {
        if calldata.len() < 4 {
            return None;
        }

        let selector: [u8; 4] = calldata[..4].try_into().ok()?;
        let selector_hex = format!("0x{}", hex::encode(selector));

        // Try to get ABI from Etherscan
        let abi = match self.fetch_from_etherscan(chain, contract).await {
            Ok(abi) => abi,
            Err(_) => return None,
        };

        // Find the function by selector
        for func in abi.functions() {
            if func.selector() == selector {
                // Build signature
                let param_types: Vec<String> =
                    func.inputs.iter().map(|p| p.ty.to_string()).collect();
                let signature = format!("{}({})", func.name, param_types.join(","));

                // Decode parameters using alloy
                let params = decode_function_params(func, &calldata[4..]);

                return Some(DecodedFunction {
                    selector: selector_hex,
                    name: Some(func.name.clone()),
                    signature: Some(signature),
                    params,
                });
            }
        }

        // Selector not found in ABI
        None
    }

    /// Lookup function selector from cache first, then 4byte.directory
    ///
    /// Uses a 3-tier lookup:
    /// 1. Local cache (instant)
    /// 2. 4byte.directory API (slow, ~100-500ms)
    /// 3. Cache the result for future lookups
    pub async fn lookup_selector(&self, selector: &str) -> Option<String> {
        let selector_normalized = format!(
            "0x{}",
            selector
                .strip_prefix("0x")
                .unwrap_or(selector)
                .to_lowercase()
        );

        // Check cache first
        if let Some(sig) = self.cache.get_function(&selector_normalized) {
            tracing::debug!("Cache hit for selector {}", selector_normalized);
            return Some(sig);
        }

        // Fetch from 4byte.directory
        let url = format!(
            "https://www.4byte.directory/api/v1/signatures/?hex_signature={}",
            selector_normalized
        );

        let response = self.client.get(&url).send().await.ok()?;
        let json: serde_json::Value = response.json().await.ok()?;

        // Get the first (most popular) result
        let signature = json["results"]
            .as_array()?
            .first()?
            .get("text_signature")?
            .as_str()
            .map(String::from)?;

        // Cache the result
        self.cache.set_function(&selector_normalized, &signature);
        tracing::debug!("Cached selector {} -> {}", selector_normalized, signature);

        Some(signature)
    }

    /// Lookup event signature from cache first, then 4byte.directory
    ///
    /// Uses a 3-tier lookup:
    /// 1. Hardcoded signatures (instant, see tx::addresses::events)
    /// 2. Local cache (instant)
    /// 3. 4byte.directory API (slow, ~100-500ms)
    pub async fn lookup_event(&self, topic0: &str) -> Option<String> {
        let topic_normalized = format!(
            "0x{}",
            topic0.strip_prefix("0x").unwrap_or(topic0).to_lowercase()
        );

        // Check cache first
        if let Some(sig) = self.cache.get_event(&topic_normalized) {
            tracing::debug!("Cache hit for event {}", topic_normalized);
            return Some(sig);
        }

        // Fetch from 4byte.directory event signatures
        let url = format!(
            "https://www.4byte.directory/api/v1/event-signatures/?hex_signature={}",
            topic_normalized
        );

        let response = self.client.get(&url).send().await.ok()?;
        let json: serde_json::Value = response.json().await.ok()?;

        // Get the first result
        let signature = json["results"]
            .as_array()?
            .first()?
            .get("text_signature")?
            .as_str()
            .map(String::from)?;

        // Cache the result
        self.cache.set_event(&topic_normalized, &signature);
        tracing::debug!("Cached event {} -> {}", topic_normalized, signature);

        Some(signature)
    }

    /// Get the signature cache for external use
    pub fn cache(&self) -> &SignatureCache {
        &self.cache
    }
}

/// Decoded function call
#[derive(Debug, Clone)]
pub struct DecodedFunction {
    /// Function selector (4 bytes as hex)
    pub selector: String,
    /// Function name (if decoded)
    pub name: Option<String>,
    /// Function signature (if decoded)
    pub signature: Option<String>,
    /// Decoded parameters
    pub params: Vec<(String, String, String)>, // (name, type, value)
}

/// Canonical signatures for widely used standard events (ERC-20/721/1155,
/// WETH, OpenZeppelin Ownable, ERC-1967 proxies), keyed case-insensitively by
/// name. Used as a last resort when an event name cannot be found in the
/// contract (or implementation) ABI.
const WELL_KNOWN_EVENTS: &[(&str, &str)] = &[
    ("transfer", "Transfer(address,address,uint256)"),
    ("approval", "Approval(address,address,uint256)"),
    ("approvalforall", "ApprovalForAll(address,address,bool)"),
    (
        "transfersingle",
        "TransferSingle(address,address,address,uint256,uint256)",
    ),
    (
        "transferbatch",
        "TransferBatch(address,address,address,uint256[],uint256[])",
    ),
    ("deposit", "Deposit(address,uint256)"),
    ("withdrawal", "Withdrawal(address,uint256)"),
    (
        "ownershiptransferred",
        "OwnershipTransferred(address,address)",
    ),
    ("upgraded", "Upgraded(address)"),
    ("adminchanged", "AdminChanged(address,address)"),
];

/// Look up a well-known event signature by (case-insensitive) name.
pub fn well_known_event_signature(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    WELL_KNOWN_EVENTS
        .iter()
        .find(|(n, _)| *n == lower)
        .map(|(_, sig)| *sig)
}

/// Decode function parameters using the function definition
fn decode_function_params(
    func: &alloy::json_abi::Function,
    data: &[u8],
) -> Vec<(String, String, String)> {
    use alloy::dyn_abi::{DynSolType, DynSolValue};

    let mut params = Vec::new();

    // Build the tuple type for decoding
    let types: Vec<DynSolType> = func
        .inputs
        .iter()
        .filter_map(|p| p.ty.parse::<DynSolType>().ok())
        .collect();

    if types.len() != func.inputs.len() {
        // Failed to parse some types
        return params;
    }

    // Create tuple type and decode
    let tuple_type = DynSolType::Tuple(types);
    let decoded = match tuple_type.abi_decode(data) {
        Ok(DynSolValue::Tuple(values)) => values,
        _ => return params,
    };

    // Format each parameter
    for (i, value) in decoded.into_iter().enumerate() {
        if let Some(input) = func.inputs.get(i) {
            let name = if input.name.is_empty() {
                format!("param{}", i)
            } else {
                input.name.clone()
            };
            let ty = input.ty.to_string();
            let val = format_sol_value(&value);
            params.push((name, ty, val));
        }
    }

    params
}

/// Format a DynSolValue for display
fn format_sol_value(value: &alloy::dyn_abi::DynSolValue) -> String {
    use alloy::dyn_abi::DynSolValue;

    match value {
        DynSolValue::Bool(b) => b.to_string(),
        DynSolValue::Int(i, _) => i.to_string(),
        DynSolValue::Uint(u, _) => u.to_string(),
        DynSolValue::FixedBytes(b, _) => format!("0x{}", hex::encode(b)),
        DynSolValue::Address(a) => format!("{:#x}", a),
        DynSolValue::Function(f) => format!("0x{}", hex::encode(f)),
        DynSolValue::Bytes(b) => {
            if b.len() <= 32 {
                format!("0x{}", hex::encode(b))
            } else {
                format!("0x{}... ({} bytes)", hex::encode(&b[..16]), b.len())
            }
        }
        DynSolValue::String(s) => format!("\"{}\"", s),
        DynSolValue::Array(arr) | DynSolValue::FixedArray(arr) => {
            let items: Vec<String> = arr.iter().map(format_sol_value).collect();
            if items.len() <= 3 {
                format!("[{}]", items.join(", "))
            } else {
                format!("[{}, ... ({} items)]", items[..2].join(", "), items.len())
            }
        }
        DynSolValue::Tuple(t) => {
            let items: Vec<String> = t.iter().map(format_sol_value).collect();
            format!("({})", items.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_known_event_signature() {
        assert_eq!(
            well_known_event_signature("Transfer"),
            Some("Transfer(address,address,uint256)")
        );
        assert_eq!(
            well_known_event_signature("APPROVAL"),
            Some("Approval(address,address,uint256)")
        );
        assert_eq!(well_known_event_signature("TokenExchange"), None);
        // Every entry must parse as a valid event signature
        for (_, sig) in WELL_KNOWN_EVENTS {
            crate::abi::EventSignature::parse(sig).unwrap();
        }
    }

    #[test]
    fn test_fetcher_creation() {
        let fetcher = AbiFetcher::new(Some("test_key".to_string())).unwrap();
        assert!(fetcher.api_key.is_some());

        let fetcher = AbiFetcher::new_default();
        assert!(fetcher.api_key.is_none());
    }

    // Integration test (requires network)
    #[tokio::test]
    #[ignore]
    async fn test_fetch_usdc_abi() {
        let fetcher = AbiFetcher::new_default();
        let result = fetcher
            .fetch_from_etherscan(
                Chain::Ethereum,
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            )
            .await;

        assert!(result.is_ok());
        let abi = result.unwrap();
        let events: Vec<_> = AbiFetcher::get_events(&abi);
        assert!(!events.is_empty());

        // Should have Transfer event
        let transfer = AbiFetcher::find_event(&abi, "Transfer");
        assert!(transfer.is_some());
    }
}
