//! Admin RPC client for Virtual TestNets
//!
//! This module provides methods for interacting with Tenderly's Admin RPC endpoints
//! on Virtual TestNets. These are JSON-RPC methods that allow manipulating the
//! network state, balances, storage, and time.
//!
//! # Example
//!
//! ```ignore
//! // Get the admin RPC client for a VNet
//! let admin = client.vnets().admin_rpc("vnet-123").await?;
//!
//! // Set balance for an address
//! let hash = admin.set_balance("0x1234...", "1000000000000000000").await?;
//!
//! // Increase time by 1 hour
//! let hash = admin.increase_time(3600).await?;
//!
//! // Create a snapshot
//! let snapshot_id = admin.snapshot().await?;
//!
//! // Revert to snapshot
//! admin.revert(&snapshot_id).await?;
//! ```

use crate::error::{Error, Result};
use reqwest::Client as HttpClient;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// JSON-RPC request structure
#[derive(Debug, Serialize)]
struct JsonRpcRequest<P: Serialize> {
    jsonrpc: &'static str,
    method: &'static str,
    params: P,
    id: u64,
}

impl<P: Serialize> JsonRpcRequest<P> {
    fn new(method: &'static str, params: P, id: u64) -> Self {
        Self {
            jsonrpc: "2.0",
            method,
            params,
            id,
        }
    }
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<T>,
    error: Option<JsonRpcError>,
    #[allow(dead_code)]
    id: u64,
}

/// JSON-RPC error structure
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

/// Admin RPC client for a Virtual TestNet
///
/// Provides methods for manipulating VNet state via JSON-RPC.
pub struct AdminRpc {
    http: HttpClient,
    url: String,
    request_id: AtomicU64,
}

impl AdminRpc {
    /// Create a new Admin RPC client
    ///
    /// # Arguments
    ///
    /// * `url` - The admin RPC URL for the Virtual TestNet
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let http = HttpClient::builder().build().map_err(Error::Http)?;

        Ok(Self {
            http,
            url: url.into(),
            request_id: AtomicU64::new(1),
        })
    }

    /// Get the next request ID
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Make a JSON-RPC call
    async fn call<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &'static str,
        params: P,
    ) -> Result<R> {
        let request = JsonRpcRequest::new(method, params, self.next_id());

        let response = self.http.post(&self.url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::api(status.as_u16(), message));
        }

        let rpc_response: JsonRpcResponse<R> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(Error::api(
                error.code as u16,
                format!("RPC error: {}", error.message),
            ));
        }

        rpc_response
            .result
            .ok_or_else(|| Error::api(0, "No result in RPC response"))
    }

    // =========================================================================
    // Time Manipulation
    // =========================================================================

    /// Advance the blockchain time by a specified number of seconds
    ///
    /// Creates an empty block with the new timestamp.
    ///
    /// # Arguments
    ///
    /// * `seconds` - Number of seconds to advance time
    ///
    /// # Returns
    ///
    /// Block hash of the newly generated block
    pub async fn increase_time(&self, seconds: u64) -> Result<String> {
        let hex_seconds = format!("0x{:x}", seconds);
        self.call("evm_increaseTime", [hex_seconds]).await
    }

    /// Set the timestamp for the next block and create an empty block
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix epoch timestamp in seconds
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn set_next_block_timestamp(&self, timestamp: u64) -> Result<String> {
        let hex_timestamp = format!("0x{:x}", timestamp);
        self.call("evm_setNextBlockTimestamp", [hex_timestamp])
            .await
    }

    /// Set the timestamp for the next block without creating an empty block
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix epoch timestamp in seconds
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn set_next_block_timestamp_no_mine(&self, timestamp: u64) -> Result<String> {
        let hex_timestamp = format!("0x{:x}", timestamp);
        self.call("tenderly_setNextBlockTimestamp", [hex_timestamp])
            .await
    }

    /// Skip a number of blocks and generate a new block
    ///
    /// # Arguments
    ///
    /// * `blocks` - Number of blocks to skip
    ///
    /// # Returns
    ///
    /// Block hash of the newly generated block
    pub async fn increase_blocks(&self, blocks: u64) -> Result<String> {
        let hex_blocks = format!("0x{:x}", blocks);
        self.call("evm_increaseBlocks", [hex_blocks]).await
    }

    // =========================================================================
    // Balance Management
    // =========================================================================

    /// Set the ETH balance of an account
    ///
    /// # Arguments
    ///
    /// * `address` - The account address
    /// * `amount` - The balance in wei (as hex string or decimal)
    ///
    /// # Returns
    ///
    /// Block hash of the state-changing transaction
    pub async fn set_balance(&self, address: &str, amount: &str) -> Result<String> {
        let hex_amount = to_hex_wei(amount);
        self.call("tenderly_setBalance", (address, hex_amount))
            .await
    }

    /// Set the ETH balance of multiple accounts
    ///
    /// # Arguments
    ///
    /// * `addresses` - List of account addresses
    /// * `amount` - The balance in wei (as hex string or decimal)
    ///
    /// # Returns
    ///
    /// Block hash of the state-changing transaction
    pub async fn set_balances(&self, addresses: &[&str], amount: &str) -> Result<String> {
        let hex_amount = to_hex_wei(amount);
        self.call("tenderly_setBalance", (addresses, hex_amount))
            .await
    }

    /// Add to the ETH balance of an account
    ///
    /// # Arguments
    ///
    /// * `address` - The account address
    /// * `amount` - The amount to add in wei
    ///
    /// # Returns
    ///
    /// Block hash of the state-changing transaction
    pub async fn add_balance(&self, address: &str, amount: &str) -> Result<String> {
        let hex_amount = to_hex_wei(amount);
        self.call("tenderly_addBalance", (address, hex_amount))
            .await
    }

    /// Add to the ETH balance of multiple accounts
    ///
    /// # Arguments
    ///
    /// * `addresses` - List of account addresses
    /// * `amount` - The amount to add in wei
    ///
    /// # Returns
    ///
    /// Block hash of the state-changing transaction
    pub async fn add_balances(&self, addresses: &[&str], amount: &str) -> Result<String> {
        let hex_amount = to_hex_wei(amount);
        self.call("tenderly_addBalance", (addresses, hex_amount))
            .await
    }

    /// Set the ERC20 token balance for a wallet
    ///
    /// # Arguments
    ///
    /// * `token_address` - The ERC20 token contract address
    /// * `wallet` - The wallet address
    /// * `amount` - The token balance (in smallest unit, e.g., wei for 18 decimals)
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn set_erc20_balance(
        &self,
        token_address: &str,
        wallet: &str,
        amount: &str,
    ) -> Result<String> {
        let hex_amount = to_hex_wei(amount);
        self.call(
            "tenderly_setErc20Balance",
            (token_address, wallet, hex_amount),
        )
        .await
    }

    /// Set the maximum possible ERC20 token balance for a wallet
    ///
    /// Tops up the wallet with the maximum possible token balance.
    ///
    /// # Arguments
    ///
    /// * `token_address` - The ERC20 token contract address
    /// * `wallet` - The wallet address
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn set_max_erc20_balance(&self, token_address: &str, wallet: &str) -> Result<String> {
        self.call("tenderly_setMaxErc20Balance", (token_address, wallet))
            .await
    }

    // =========================================================================
    // Storage Manipulation
    // =========================================================================

    /// Set storage at a specific slot for a contract
    ///
    /// # Arguments
    ///
    /// * `address` - The contract address
    /// * `slot` - The storage slot (hex or decimal, will be padded to 32 bytes)
    /// * `value` - The value to set (hex or decimal, will be padded to 32 bytes)
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn set_storage_at(&self, address: &str, slot: &str, value: &str) -> Result<String> {
        let padded_slot = to_hex_32_bytes(slot);
        let padded_value = to_hex_32_bytes(value);
        self.call(
            "tenderly_setStorageAt",
            (address, padded_slot, padded_value),
        )
        .await
    }

    /// Set the bytecode at an address
    ///
    /// # Arguments
    ///
    /// * `address` - The address to set code at
    /// * `bytecode` - The bytecode to deploy (hex string)
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn set_code(&self, address: &str, bytecode: &str) -> Result<String> {
        self.call("tenderly_setCode", (address, bytecode)).await
    }

    // =========================================================================
    // State Management
    // =========================================================================

    /// Create a state snapshot
    ///
    /// Returns a snapshot ID that can be used to revert the state later.
    ///
    /// # Returns
    ///
    /// Snapshot ID (32-byte hash)
    pub async fn snapshot(&self) -> Result<String> {
        self.call::<[(); 0], String>("evm_snapshot", []).await
    }

    /// Revert the state to a previous snapshot
    ///
    /// # Arguments
    ///
    /// * `snapshot_id` - The snapshot ID returned from `snapshot()`
    ///
    /// # Returns
    ///
    /// `true` if successful
    pub async fn revert(&self, snapshot_id: &str) -> Result<bool> {
        self.call("evm_revert", [snapshot_id]).await
    }

    // =========================================================================
    // Transaction Handling
    // =========================================================================

    /// Get the latest block/transaction info on the Virtual TestNet
    ///
    /// # Returns
    ///
    /// Block information including block number, hash, and transaction hash
    pub async fn get_latest(&self) -> Result<LatestBlock> {
        self.call::<[(); 0], LatestBlock>("evm_getLatest", []).await
    }

    /// Send an unsigned transaction
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction parameters
    ///
    /// # Returns
    ///
    /// Transaction hash
    pub async fn send_transaction(&self, tx: &SendTransactionParams) -> Result<String> {
        self.call("eth_sendTransaction", [tx]).await
    }

    /// Create an access list for a transaction
    ///
    /// Returns the access tuples that would be touched by the transaction.
    ///
    /// # Arguments
    ///
    /// * `tx` - The transaction parameters
    /// * `block` - Block number or tag (e.g., "latest")
    ///
    /// # Returns
    ///
    /// Access list result
    pub async fn create_access_list(
        &self,
        tx: &SendTransactionParams,
        block: &str,
    ) -> Result<AccessListResult> {
        self.call("eth_createAccessList", (tx, block)).await
    }
}

impl std::fmt::Debug for AdminRpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminRpc").field("url", &self.url).finish()
    }
}

/// Parameters for sending a transaction via Admin RPC
#[derive(Debug, Clone, Default, Serialize)]
pub struct SendTransactionParams {
    /// Sender address
    pub from: String,

    /// Recipient address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,

    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,

    /// Gas price
    #[serde(rename = "gasPrice", skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,

    /// Value in wei
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Transaction data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl SendTransactionParams {
    /// Create new transaction parameters
    pub fn new(from: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            ..Default::default()
        }
    }

    /// Set the recipient address
    #[must_use]
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Set the gas limit
    #[must_use]
    pub fn gas(mut self, gas: impl Into<String>) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Set the gas price
    #[must_use]
    pub fn gas_price(mut self, price: impl Into<String>) -> Self {
        self.gas_price = Some(price.into());
        self
    }

    /// Set the value in wei (accepts hex or decimal, auto-converts to hex)
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(to_hex_wei(&value.into()));
        self
    }

    /// Set the transaction data
    #[must_use]
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }
}

/// Result from creating an access list
#[derive(Debug, Clone, Deserialize)]
pub struct AccessListResult {
    /// The access list
    #[serde(rename = "accessList")]
    pub access_list: Vec<AccessListEntry>,

    /// Estimated gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: String,
}

/// Entry in an access list
#[derive(Debug, Clone, Deserialize)]
pub struct AccessListEntry {
    /// Address being accessed
    pub address: String,

    /// Storage keys being accessed
    #[serde(rename = "storageKeys")]
    pub storage_keys: Vec<String>,
}

/// Latest block/transaction info returned by `evm_getLatest`
#[derive(Debug, Clone, Deserialize)]
pub struct LatestBlock {
    /// Block number (hex)
    #[serde(rename = "blockNumber")]
    pub block_number: Option<String>,

    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: Option<String>,

    /// Transaction hash (if applicable)
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,

    /// Additional fields captured as raw JSON
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

// =========================================================================
// Helper functions
// =========================================================================

/// Parse a hex string to u64
#[allow(dead_code)]
fn parse_hex_u64(s: &str) -> Result<u64> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16)
        .map_err(|e| Error::invalid_param(format!("Invalid hex number: {}", e)))
}

/// Convert a decimal or hex string to hex wei format
fn to_hex_wei(amount: &str) -> String {
    // If already hex, return as-is
    if amount.starts_with("0x") {
        return amount.to_string();
    }
    // Otherwise parse as decimal and convert to hex
    if let Ok(n) = amount.parse::<u128>() {
        format!("0x{:x}", n)
    } else {
        // Return as-is if we can't parse (let the RPC handle the error)
        amount.to_string()
    }
}

/// Convert a decimal or hex string to 32-byte hex format
fn to_hex_32_bytes(amount: &str) -> String {
    // If already hex with 0x prefix, ensure it's 32 bytes (64 chars)
    if let Some(hex_part) = amount.strip_prefix("0x") {
        return format!("0x{:0>64}", hex_part);
    }
    // Parse as decimal and convert to 32-byte hex
    if let Ok(n) = amount.parse::<u128>() {
        format!("0x{:064x}", n)
    } else {
        // Return as-is if we can't parse
        amount.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Helper function tests
    // =========================================================================

    #[test]
    fn test_to_hex_wei() {
        assert_eq!(to_hex_wei("0x1"), "0x1");
        assert_eq!(to_hex_wei("1000000000000000000"), "0xde0b6b3a7640000");
        assert_eq!(to_hex_wei("100"), "0x64");
        assert_eq!(to_hex_wei("0"), "0x0");
    }

    #[test]
    fn test_to_hex_wei_large_numbers() {
        // 100 ETH in wei
        assert_eq!(to_hex_wei("100000000000000000000"), "0x56bc75e2d63100000");
        // Max u128 would overflow, test near-max u64
        assert_eq!(to_hex_wei("18446744073709551615"), "0xffffffffffffffff");
    }

    #[test]
    fn test_to_hex_wei_passthrough() {
        // Already hex values should pass through unchanged
        assert_eq!(to_hex_wei("0xde0b6b3a7640000"), "0xde0b6b3a7640000");
        assert_eq!(to_hex_wei("0x0"), "0x0");
        // Invalid values pass through for RPC to handle
        assert_eq!(to_hex_wei("invalid"), "invalid");
    }

    #[test]
    fn test_to_hex_32_bytes() {
        assert_eq!(
            to_hex_32_bytes("100"),
            "0x0000000000000000000000000000000000000000000000000000000000000064"
        );
        assert_eq!(
            to_hex_32_bytes("0x64"),
            "0x0000000000000000000000000000000000000000000000000000000000000064"
        );
        assert_eq!(
            to_hex_32_bytes("0"),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_to_hex_32_bytes_large_values() {
        // 1 ETH in wei (18 decimals)
        assert_eq!(
            to_hex_32_bytes("1000000000000000000"),
            "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000"
        );
    }

    #[test]
    fn test_to_hex_32_bytes_already_padded() {
        // If already 32 bytes hex, should preserve
        let full = "0x0000000000000000000000000000000000000000000000000000000000000001";
        assert_eq!(to_hex_32_bytes(full), full);
    }

    #[test]
    fn test_parse_hex_u64() {
        assert_eq!(parse_hex_u64("0x64").unwrap(), 100);
        assert_eq!(parse_hex_u64("64").unwrap(), 100);
        assert_eq!(parse_hex_u64("0x0").unwrap(), 0);
        assert_eq!(parse_hex_u64("0xffffffffffffffff").unwrap(), u64::MAX);
    }

    #[test]
    fn test_parse_hex_u64_error() {
        assert!(parse_hex_u64("0xzz").is_err());
        assert!(parse_hex_u64("not_hex").is_err());
        // Overflow
        assert!(parse_hex_u64("0x10000000000000000").is_err());
    }

    // =========================================================================
    // Builder pattern tests
    // =========================================================================

    #[test]
    fn test_send_transaction_params_builder() {
        let params = SendTransactionParams::new("0x1234")
            .to("0x5678")
            .gas("0x5208")
            .value("0x1");

        assert_eq!(params.from, "0x1234");
        assert_eq!(params.to, Some("0x5678".to_string()));
        assert_eq!(params.gas, Some("0x5208".to_string()));
        assert_eq!(params.value, Some("0x1".to_string()));
    }

    #[test]
    fn test_send_transaction_params_value_decimal_conversion() {
        // Value should auto-convert decimal to hex
        let params = SendTransactionParams::new("0x1234").value("1000000000000000000"); // 1 ETH in decimal

        assert_eq!(params.value, Some("0xde0b6b3a7640000".to_string()));
    }

    #[test]
    fn test_send_transaction_params_value_hex_passthrough() {
        // Already hex values should pass through unchanged
        let params = SendTransactionParams::new("0x1234").value("0xde0b6b3a7640000");

        assert_eq!(params.value, Some("0xde0b6b3a7640000".to_string()));
    }

    #[test]
    fn test_send_transaction_params_minimal() {
        let params = SendTransactionParams::new("0xsender");
        assert_eq!(params.from, "0xsender");
        assert!(params.to.is_none());
        assert!(params.gas.is_none());
        assert!(params.value.is_none());
        assert!(params.data.is_none());
    }

    #[test]
    fn test_send_transaction_params_with_data() {
        let params = SendTransactionParams::new("0xsender")
            .to("0xcontract")
            .data("0xa9059cbb000000000000000000000000");

        assert_eq!(
            params.data,
            Some("0xa9059cbb000000000000000000000000".to_string())
        );
    }

    // =========================================================================
    // JSON-RPC request serialization tests
    // =========================================================================

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest::new("evm_snapshot", Vec::<()>::new(), 1);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"evm_snapshot\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_json_rpc_request_with_tuple_params() {
        // Test how set_balance params serialize (address, amount tuple)
        let request = JsonRpcRequest::new(
            "tenderly_setBalance",
            ("0x1234567890abcdef", "0xde0b6b3a7640000"),
            1,
        );
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"method\":\"tenderly_setBalance\""));
        assert!(json.contains("\"params\":[\"0x1234567890abcdef\",\"0xde0b6b3a7640000\"]"));
    }

    #[test]
    fn test_json_rpc_request_with_array_params() {
        // Test how increase_time params serialize (single hex value in array)
        let request = JsonRpcRequest::new("evm_increaseTime", ["0xe10"], 1);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"method\":\"evm_increaseTime\""));
        assert!(json.contains("\"params\":[\"0xe10\"]"));
    }

    #[test]
    fn test_json_rpc_request_with_object_params() {
        // Test transaction object serialization
        let tx = SendTransactionParams::new("0xfrom")
            .to("0xto")
            .value("0x1")
            .gas("0x5208");
        let request = JsonRpcRequest::new("eth_sendTransaction", [&tx], 1);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"method\":\"eth_sendTransaction\""));
        assert!(json.contains("\"from\":\"0xfrom\""));
        assert!(json.contains("\"to\":\"0xto\""));
        assert!(json.contains("\"value\":\"0x1\""));
        assert!(json.contains("\"gas\":\"0x5208\""));
    }

    #[test]
    fn test_send_transaction_params_skips_none_fields() {
        let params = SendTransactionParams::new("0xfrom").to("0xto");
        let json = serde_json::to_string(&params).unwrap();

        assert!(json.contains("\"from\":\"0xfrom\""));
        assert!(json.contains("\"to\":\"0xto\""));
        // None fields should not be serialized
        assert!(!json.contains("\"gas\""));
        assert!(!json.contains("\"value\""));
        assert!(!json.contains("\"data\""));
        assert!(!json.contains("\"gasPrice\""));
    }

    // =========================================================================
    // JSON-RPC response deserialization tests
    // =========================================================================

    #[test]
    fn test_json_rpc_response_success() {
        let json = r#"{"jsonrpc":"2.0","result":"0x1234","id":1}"#;
        let response: JsonRpcResponse<String> = serde_json::from_str(json).unwrap();

        assert_eq!(response.result, Some("0x1234".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid request"},"id":1}"#;
        let response: JsonRpcResponse<String> = serde_json::from_str(json).unwrap();

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
    }

    #[test]
    fn test_json_rpc_response_bool() {
        let json = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
        let response: JsonRpcResponse<bool> = serde_json::from_str(json).unwrap();
        assert_eq!(response.result, Some(true));
    }

    #[test]
    fn test_access_list_result_deserialization() {
        let json = r#"{
            "accessList": [
                {
                    "address": "0x1234567890abcdef1234567890abcdef12345678",
                    "storageKeys": [
                        "0x0000000000000000000000000000000000000000000000000000000000000001",
                        "0x0000000000000000000000000000000000000000000000000000000000000002"
                    ]
                }
            ],
            "gasUsed": "0x5208"
        }"#;

        let result: AccessListResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.access_list.len(), 1);
        assert_eq!(
            result.access_list[0].address,
            "0x1234567890abcdef1234567890abcdef12345678"
        );
        assert_eq!(result.access_list[0].storage_keys.len(), 2);
        assert_eq!(result.gas_used, "0x5208");
    }

    #[test]
    fn test_access_list_result_empty() {
        let json = r#"{"accessList":[],"gasUsed":"0x0"}"#;
        let result: AccessListResult = serde_json::from_str(json).unwrap();
        assert!(result.access_list.is_empty());
        assert_eq!(result.gas_used, "0x0");
    }

    #[test]
    fn test_latest_block_deserialization() {
        let json = r#"{
            "blockNumber": "0x10",
            "blockHash": "0xabc123",
            "transactionHash": "0xdef456",
            "extraField": "extraValue"
        }"#;
        let block: LatestBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_number, Some("0x10".to_string()));
        assert_eq!(block.block_hash, Some("0xabc123".to_string()));
        assert_eq!(block.transaction_hash, Some("0xdef456".to_string()));
        assert!(block.extra.contains_key("extraField"));
    }

    #[test]
    fn test_latest_block_partial_fields() {
        // API may not always return all fields
        let json = r#"{"blockNumber": "0x1"}"#;
        let block: LatestBlock = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_number, Some("0x1".to_string()));
        assert!(block.block_hash.is_none());
        assert!(block.transaction_hash.is_none());
    }

    // =========================================================================
    // Integration-style tests (testing method param construction)
    // =========================================================================

    #[test]
    fn test_increase_time_param_format() {
        // Verify the hex conversion for time increase
        let seconds = 3600u64; // 1 hour
        let hex = format!("0x{:x}", seconds);
        assert_eq!(hex, "0xe10");
    }

    #[test]
    fn test_set_balance_param_format() {
        // 1 ETH in wei
        let amount = "1000000000000000000";
        let hex = to_hex_wei(amount);
        assert_eq!(hex, "0xde0b6b3a7640000");
    }

    #[test]
    fn test_set_erc20_balance_param_format() {
        // 1000 USDC (6 decimals) = 1000000000
        // set_erc20_balance now uses to_hex_wei (unpadded) format
        let amount = "1000000000";
        let hex = to_hex_wei(amount);
        assert_eq!(hex, "0x3b9aca00");
    }

    #[test]
    fn test_storage_slot_format() {
        // Slot 0
        let slot = to_hex_32_bytes("0");
        assert_eq!(
            slot,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );

        // Slot 1
        let slot = to_hex_32_bytes("1");
        assert_eq!(
            slot,
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
    }
}
