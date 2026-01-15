//! Curve Prices API client
//!
//! Client for the Curve Prices API at `prices.curve.finance`
//! Provides detailed pricing, OHLC, trades, and DAO data.

use crate::error::{Error, Result};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;

const PRICES_BASE_URL: &str = "https://prices.curve.finance/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Client for the Curve Prices API
#[derive(Debug, Clone)]
pub struct PricesClient {
    http: HttpClient,
    base_url: String,
}

impl PricesClient {
    /// Create a new Prices API client
    pub fn new() -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()?;

        Ok(Self {
            http,
            base_url: PRICES_BASE_URL.to_string(),
        })
    }

    /// Create with custom base URL
    pub fn with_base_url(url: impl Into<String>) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()?;

        Ok(Self {
            http,
            base_url: url.into(),
        })
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::api(status, message));
        }

        let data = response.json().await?;
        Ok(data)
    }

    // === Chains ===

    /// Get all supported chains
    pub async fn get_chains(&self) -> Result<serde_json::Value> {
        self.get("/chains/").await
    }

    /// Get chain stats
    pub async fn get_chain_stats(&self) -> Result<serde_json::Value> {
        self.get("/chains/stats").await
    }

    // === Prices ===

    /// Get USD prices for all tokens on a chain
    pub async fn get_usd_prices(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/usd_price/{}", chain);
        self.get(&path).await
    }

    /// Get USD price for a specific token
    pub async fn get_usd_price(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/usd_price/{}/{}", chain, address);
        self.get(&path).await
    }

    /// Get price history for a token
    pub async fn get_price_history(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/usd_price/{}/{}/history", chain, address);
        self.get(&path).await
    }

    // === Pools ===

    /// Get pool data (TVL, volume, fees)
    pub async fn get_pool(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/pools/{}/{}", chain, address);
        self.get(&path).await
    }

    /// Get pool metadata
    pub async fn get_pool_metadata(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/pools/{}/{}/metadata", chain, address);
        self.get(&path).await
    }

    // === OHLC ===

    /// Get OHLC data for a pool
    pub async fn get_ohlc(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/ohlc/{}/{}", chain, address);
        self.get(&path).await
    }

    /// Get LP token OHLC data
    pub async fn get_lp_ohlc(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/lp_ohlc/{}/{}", chain, address);
        self.get(&path).await
    }

    // === Trades ===

    /// Get trades for a contract
    pub async fn get_trades(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/trades/{}/{}", chain, address);
        self.get(&path).await
    }

    // === Volume ===

    /// Get volume for a chain
    pub async fn get_chain_volume(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/volume/{}", chain);
        self.get(&path).await
    }

    /// Get top tokens by volume
    pub async fn get_top_volume_tokens(&self) -> Result<serde_json::Value> {
        self.get("/volume/tokens/top").await
    }

    // === crvUSD ===

    /// Get all crvUSD markets
    pub async fn get_crvusd_markets(&self) -> Result<serde_json::Value> {
        self.get("/crvusd/markets").await
    }

    /// Get crvUSD markets on a chain
    pub async fn get_crvusd_markets_on_chain(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/crvusd/markets/{}", chain);
        self.get(&path).await
    }

    /// Get crvUSD savings stats
    pub async fn get_crvusd_savings_stats(&self) -> Result<serde_json::Value> {
        self.get("/crvusd/savings/statistics").await
    }

    // === Lending ===

    /// Get all lending markets
    pub async fn get_lending_markets(&self) -> Result<serde_json::Value> {
        self.get("/lending/markets").await
    }

    /// Get lending markets on a chain
    pub async fn get_lending_markets_on_chain(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/lending/markets/{}", chain);
        self.get(&path).await
    }

    // === DAO ===

    /// Get gauge overview
    pub async fn get_gauges_overview(&self) -> Result<serde_json::Value> {
        self.get("/dao/gauges/overview").await
    }

    /// Get DAO proposals
    pub async fn get_proposals(&self) -> Result<serde_json::Value> {
        self.get("/dao/proposals").await
    }

    /// Get top CRV lockers
    pub async fn get_top_lockers(&self, top: u32) -> Result<serde_json::Value> {
        let path = format!("/dao/lockers/{}", top);
        self.get(&path).await
    }

    // === Health ===

    /// Ping the API
    pub async fn ping(&self) -> Result<serde_json::Value> {
        // Note: ping is at root, not /v1
        let url = "https://prices.curve.finance/ping";
        let response = self.http.get(url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }
}

impl Default for PricesClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default PricesClient")
    }
}
