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
    #[serde(default)]
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
    pub controller_address: Option<String>,
    /// AMM address
    pub amm_address: Option<String>,
    /// Monetary policy address
    pub monetary_policy_address: Option<String>,
    /// Gauge address
    pub gauge_address: Option<String>,
    /// Blockchain ID
    pub blockchain_id: Option<String>,
    /// Registry ID
    pub registry_id: Option<String>,
    /// Lending rates (APR/APY)
    pub rates: Option<LendingRates>,
    /// Assets (borrowed and collateral tokens)
    pub assets: Option<LendingAssets>,
    /// Total supplied to vault
    pub total_supplied: Option<AmountWithUsd>,
    /// Total borrowed from vault
    pub borrowed: Option<AmountWithUsd>,
    /// Available to borrow
    pub available_to_borrow: Option<AmountWithUsd>,
    /// Vault shares info
    pub vault_shares: Option<VaultShares>,
    /// AMM balances
    pub amm_balances: Option<AmmBalances>,
    /// Gauge rewards
    pub gauge_rewards: Option<Vec<GaugeReward>>,
    /// Lending vault URLs
    pub lending_vault_urls: Option<LendingVaultUrls>,
    /// Total USD value
    pub usd_total: Option<f64>,
}

/// Lending rates (APR and APY)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingRates {
    /// Borrow APR (decimal form)
    pub borrow_apr: Option<f64>,
    /// Borrow APY (decimal form)
    pub borrow_apy: Option<f64>,
    /// Borrow APY as percentage
    pub borrow_apy_pcent: Option<f64>,
    /// Lend APR (decimal form)
    pub lend_apr: Option<f64>,
    /// Lend APY (decimal form)
    pub lend_apy: Option<f64>,
    /// Lend APY as percentage
    pub lend_apy_pcent: Option<f64>,
}

/// Lending assets (borrowed and collateral tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingAssets {
    /// Borrowed token info
    pub borrowed: Option<Token>,
    /// Collateral token info
    pub collateral: Option<Token>,
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
    /// Blockchain ID
    pub blockchain_id: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
}

/// Amount with USD equivalent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountWithUsd {
    /// Total amount
    pub total: Option<f64>,
    /// USD equivalent
    pub usd_total: Option<f64>,
}

/// Vault shares info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultShares {
    /// Price per share
    pub price_per_share: Option<f64>,
    /// Total shares
    pub total_shares: Option<f64>,
}

/// AMM balances
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmmBalances {
    /// AMM balance of borrowed token
    pub amm_balance_borrowed: Option<f64>,
    /// AMM balance of borrowed token in USD
    pub amm_balance_borrowed_usd: Option<f64>,
    /// AMM balance of collateral token
    pub amm_balance_collateral: Option<f64>,
    /// AMM balance of collateral token in USD
    pub amm_balance_collateral_usd: Option<f64>,
}

/// Gauge reward info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeReward {
    /// Reward token symbol
    pub symbol: Option<String>,
    /// Reward token address
    pub token_address: Option<String>,
    /// APY from this reward
    pub apy: Option<f64>,
    /// Gauge address
    pub gauge_address: Option<String>,
}

/// Lending vault URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LendingVaultUrls {
    /// Deposit URL
    pub deposit: Option<String>,
    /// Withdraw URL
    pub withdraw: Option<String>,
    /// Borrow URL
    pub borrow: Option<String>,
}
