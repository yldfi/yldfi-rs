//! Types for the crvUSD API

use serde::{Deserialize, Serialize};

/// Response for crvUSD supply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrvUsdSupplyResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Supply data
    pub data: CrvUsdSupply,
}

/// crvUSD supply data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrvUsdSupply {
    /// Circulating supply
    pub circulating_supply: Option<f64>,
    /// Total supply
    pub total_supply: Option<f64>,
}

/// Simple number response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberResponse {
    /// Whether the request was successful
    pub success: bool,
    /// The number value
    pub data: f64,
}
