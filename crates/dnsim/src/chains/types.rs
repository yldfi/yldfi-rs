//! Types for supported chains

use serde::{Deserialize, Serialize};

/// Chains response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainsResponse {
    /// List of supported chains
    pub chains: Vec<Chain>,
}

/// Chain info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chain {
    /// Chain ID (API returns `chain_id`, not id)
    pub chain_id: i64,
    /// Chain name
    pub name: String,
    /// Tags (default, mainnet, testnet)
    #[serde(default)]
    pub tags: Vec<String>,
    /// Balances endpoint support
    pub balances: Option<EndpointSupport>,
    /// Transactions endpoint support
    pub transactions: Option<EndpointSupport>,
    /// Activity endpoint support
    pub activity: Option<EndpointSupport>,
    /// Token info endpoint support
    pub token_info: Option<EndpointSupport>,
    /// Token holders endpoint support
    pub token_holders: Option<EndpointSupport>,
    /// Collectibles endpoint support
    pub collectibles: Option<EndpointSupport>,
    /// `DeFi` positions endpoint support
    pub defi_positions: Option<EndpointSupport>,
}

/// Endpoint support status
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EndpointSupport {
    /// Whether the endpoint is supported
    pub supported: bool,
}
