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
        let http = HttpClient::builder().timeout(DEFAULT_TIMEOUT).build()?;

        Ok(Self {
            http,
            base_url: PRICES_BASE_URL.to_string(),
        })
    }

    /// Create with custom base URL
    pub fn with_base_url(url: impl Into<String>) -> Result<Self> {
        let http = HttpClient::builder().timeout(DEFAULT_TIMEOUT).build()?;

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
        let path = format!("/usd_price/{chain}");
        self.get(&path).await
    }

    /// Get USD price for a specific token
    pub async fn get_usd_price(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/usd_price/{chain}/{address}");
        self.get(&path).await
    }

    /// Get price history for a token
    ///
    /// # Arguments
    /// * `chain` - Chain name (e.g., "ethereum")
    /// * `address` - Token contract address
    /// * `start` - Optional start timestamp (unix seconds)
    /// * `end` - Optional end timestamp (unix seconds)
    pub async fn get_price_history(
        &self,
        chain: &str,
        address: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<serde_json::Value> {
        let mut path = format!("/usd_price/{chain}/{address}/history");
        let mut params = Vec::new();
        if let Some(s) = start {
            params.push(format!("start={s}"));
        }
        if let Some(e) = end {
            params.push(format!("end={e}"));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        self.get(&path).await
    }

    // === Pools ===

    /// Get pool data (TVL, volume, fees)
    pub async fn get_pool(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/pools/{chain}/{address}");
        self.get(&path).await
    }

    /// Get pool metadata
    pub async fn get_pool_metadata(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/pools/{chain}/{address}/metadata");
        self.get(&path).await
    }

    // === OHLC ===

    /// Get OHLC data for a pool
    ///
    /// # Arguments
    /// * `chain` - Chain name (e.g., "ethereum")
    /// * `address` - Pool contract address
    /// * `start` - Optional start timestamp (unix seconds)
    /// * `end` - Optional end timestamp (unix seconds)
    pub async fn get_ohlc(
        &self,
        chain: &str,
        address: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<serde_json::Value> {
        let mut path = format!("/ohlc/{chain}/{address}");
        let mut params = Vec::new();
        if let Some(s) = start {
            params.push(format!("start={s}"));
        }
        if let Some(e) = end {
            params.push(format!("end={e}"));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        self.get(&path).await
    }

    /// Get LP token OHLC data
    ///
    /// # Arguments
    /// * `chain` - Chain name (e.g., "ethereum")
    /// * `address` - LP token contract address
    /// * `start` - Optional start timestamp (unix seconds)
    /// * `end` - Optional end timestamp (unix seconds)
    pub async fn get_lp_ohlc(
        &self,
        chain: &str,
        address: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<serde_json::Value> {
        let mut path = format!("/lp_ohlc/{chain}/{address}");
        let mut params = Vec::new();
        if let Some(s) = start {
            params.push(format!("start={s}"));
        }
        if let Some(e) = end {
            params.push(format!("end={e}"));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        self.get(&path).await
    }

    // === Trades ===

    /// Get trades for a contract
    pub async fn get_trades(&self, chain: &str, address: &str) -> Result<serde_json::Value> {
        let path = format!("/trades/{chain}/{address}");
        self.get(&path).await
    }

    // === Volume ===

    /// Get daily USD volume for a chain over the trailing 30 days
    ///
    /// Convenience wrapper around [`Self::get_chain_volume_range`]; the
    /// endpoint requires explicit `start`/`end` bounds.
    pub async fn get_chain_volume(&self, chain: &str) -> Result<serde_json::Value> {
        let end = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        let start = end.saturating_sub(30 * 86_400);
        self.get_chain_volume_range(chain, start, end, Some(VolumeInterval::Day))
            .await
    }

    /// Get USD-denominated aggregated volume for a chain
    /// (`GET /v1/volume/{chain}?start=&end=&interval=`)
    ///
    /// `start`/`end` are unix timestamps (seconds). The API only allows a
    /// time window of up to 300x the aggregation interval (e.g. 300 days for
    /// `Day`); split larger ranges into several requests. `interval`
    /// defaults to `Day` server-side.
    pub async fn get_chain_volume_range(
        &self,
        chain: &str,
        start: u64,
        end: u64,
        interval: Option<VolumeInterval>,
    ) -> Result<serde_json::Value> {
        let path = chain_volume_path(chain, start, end, interval);
        self.get(&path).await
    }

    /// Get top tokens by volume
    ///
    /// `/v1/volume/tokens/top` does not exist in the Curve Prices API (the
    /// path is matched by the pool pair-volume route and always fails with
    /// 422). This method returns an error without making a request.
    #[deprecated(
        since = "0.1.7",
        note = "`/v1/volume/tokens/top` is not a Curve Prices API route; use `get_chain_volume_range` for chain volume"
    )]
    pub async fn get_top_volume_tokens(&self) -> Result<serde_json::Value> {
        Err(Error::api(
            404,
            "/v1/volume/tokens/top is not provided by the Curve Prices API; use get_chain_volume_range",
        ))
    }

    // === crvUSD ===

    /// Get all crvUSD markets
    pub async fn get_crvusd_markets(&self) -> Result<serde_json::Value> {
        self.get("/crvusd/markets").await
    }

    /// Get crvUSD markets on a chain
    pub async fn get_crvusd_markets_on_chain(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/crvusd/markets/{chain}");
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
        let path = format!("/lending/markets/{chain}");
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
        let path = format!("/dao/lockers/{top}");
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

/// Aggregation interval for Curve Prices volume queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VolumeInterval {
    /// Hourly buckets
    Hour,
    /// Daily buckets (API default)
    #[default]
    Day,
    /// Weekly buckets
    Week,
}

impl VolumeInterval {
    /// Query-string value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
        }
    }
}

impl std::str::FromStr for VolumeInterval {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            other => Err(Error::api(
                400,
                format!("invalid volume interval '{other}' (expected hour, day or week)"),
            )),
        }
    }
}

fn chain_volume_path(
    chain: &str,
    start: u64,
    end: u64,
    interval: Option<VolumeInterval>,
) -> String {
    let mut path = format!("/volume/{chain}?start={start}&end={end}");
    if let Some(interval) = interval {
        path.push_str("&interval=");
        path.push_str(interval.as_str());
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_volume_path_includes_required_bounds() {
        assert_eq!(
            chain_volume_path("ethereum", 1, 2, None),
            "/volume/ethereum?start=1&end=2"
        );
        assert_eq!(
            chain_volume_path("arbitrum", 10, 20, Some(VolumeInterval::Week)),
            "/volume/arbitrum?start=10&end=20&interval=week"
        );
    }

    #[test]
    fn volume_interval_parses() {
        assert_eq!(
            "Hour".parse::<VolumeInterval>().unwrap(),
            VolumeInterval::Hour
        );
        assert!("month".parse::<VolumeInterval>().is_err());
    }
}
