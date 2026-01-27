//! crvUSD API client

use crate::client::Client;
use crate::error::Result;

/// API for crvUSD data
pub struct CrvUsdApi<'a> {
    client: &'a Client,
}

impl<'a> CrvUsdApi<'a> {
    /// Create a new crvUSD API client
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get circulating supply of crvUSD
    pub async fn get_circulating_supply(&self) -> Result<serde_json::Value> {
        self.client.get("/getCrvCircSupply").await
    }

    /// Get total supply of crvUSD
    pub async fn get_total_supply(&self) -> Result<serde_json::Value> {
        self.client.get("/getCrvusdTotalSupply").await
    }

    /// Get total supply as a number
    pub async fn get_total_supply_number(&self) -> Result<f64> {
        self.client.get("/getCrvusdTotalSupplyNumber").await
    }

    /// Get total scrvUSD supply as a number
    pub async fn get_scrvusd_supply_number(&self) -> Result<f64> {
        self.client.get("/getScrvusdTotalSupplyNumber").await
    }

    /// Get scrvUSD supply as JSON
    pub async fn get_scrvusd_supply(&self) -> Result<serde_json::Value> {
        self.client.get("/getScrvusdTotalSupplyResult").await
    }
}
