//! Types for the Lending API

use serde::{Deserialize, Serialize};

/// Response for lending vaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingVaultsResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Lending vault data
    pub data: LendingVaultsData,
}

/// Lending vaults data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingVaultsData {
    /// List of lending vaults
    pub lending_vault_data: Vec<LendingVault>,
}

/// A lending vault
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingVault {
    /// Vault ID
    pub id: String,
    /// Vault name
    pub name: Option<String>,
    /// Vault address
    pub address: String,
    /// Controller address
    pub controller: Option<String>,
    /// Collateral token
    pub collateral_token: Option<Token>,
    /// Borrowed token
    pub borrowed_token: Option<Token>,
    /// Total assets in vault
    pub total_assets: Option<String>,
    /// Total debt
    pub total_debt: Option<String>,
    /// Utilization rate
    pub utilization: Option<f64>,
    /// Borrow APY
    pub borrow_apy: Option<f64>,
    /// Lend APY
    pub lend_apy: Option<f64>,
}

/// Token info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token address
    pub address: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
}
