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

    /// Get the circulating supply of **CRV** (not crvUSD).
    ///
    /// Calls `/getCrvCircSupply`. The Curve API has no crvUSD circulating
    /// supply endpoint; use [`Self::get_total_supply`] for crvUSD.
    pub async fn get_crv_circulating_supply(&self) -> Result<serde_json::Value> {
        self.client.get("/getCrvCircSupply").await
    }

    /// Deprecated: this returns the **CRV** circulating supply, not crvUSD.
    #[deprecated(
        since = "0.1.5",
        note = "returns CRV (not crvUSD) circulating supply; use get_crv_circulating_supply"
    )]
    pub async fn get_circulating_supply(&self) -> Result<serde_json::Value> {
        self.get_crv_circulating_supply().await
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
