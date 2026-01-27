//! TVL (Total Value Locked) queries from Kong API

use crate::client::Client;
use crate::error::Result;
use crate::types::Tvl;
use serde::Deserialize;

/// TVL period for historical queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvlPeriod {
    /// Daily data points
    Day,
    /// Weekly data points
    Week,
    /// Monthly data points
    Month,
}

impl TvlPeriod {
    fn as_str(&self) -> &'static str {
        match self {
            TvlPeriod::Day => "day",
            TvlPeriod::Week => "week",
            TvlPeriod::Month => "month",
        }
    }
}

/// TVLs API
pub struct TvlsApi<'a> {
    client: &'a Client,
}

impl<'a> TvlsApi<'a> {
    /// Create a new TVLs API instance
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get TVL history for a vault or strategy
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::{Client, TvlPeriod};
    ///
    /// let client = Client::new()?;
    /// let tvls = client.tvls().history(1, "0x...", TvlPeriod::Day, 30).await?;
    /// for tvl in tvls {
    ///     println!("TVL: ${:.2}", tvl.value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn history(
        &self,
        chain_id: u64,
        address: &str,
        period: TvlPeriod,
        limit: u32,
    ) -> Result<Vec<Tvl>> {
        let query = format!(
            r#"{{
                tvls(chainId: {}, address: "{}", period: "{}", limit: {}) {{
                    chainId
                    address
                    value
                    priceUsd
                    priceSource
                    period
                    blockNumber
                    time
                }}
            }}"#,
            chain_id,
            address,
            period.as_str(),
            limit
        );

        #[derive(Deserialize)]
        struct Response {
            tvls: Vec<Tvl>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.tvls)
    }

    /// Get TVL history starting from a specific timestamp
    pub async fn history_from(
        &self,
        chain_id: u64,
        address: &str,
        period: TvlPeriod,
        limit: u32,
        timestamp: u64,
    ) -> Result<Vec<Tvl>> {
        let query = format!(
            r#"{{
                tvls(chainId: {}, address: "{}", period: "{}", limit: {}, timestamp: {}) {{
                    chainId
                    address
                    value
                    priceUsd
                    priceSource
                    period
                    blockNumber
                    time
                }}
            }}"#,
            chain_id,
            address,
            period.as_str(),
            limit,
            timestamp
        );

        #[derive(Deserialize)]
        struct Response {
            tvls: Vec<Tvl>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.tvls)
    }

    /// Get daily TVL history (convenience method)
    pub async fn daily(&self, chain_id: u64, address: &str, days: u32) -> Result<Vec<Tvl>> {
        self.history(chain_id, address, TvlPeriod::Day, days).await
    }

    /// Get weekly TVL history (convenience method)
    pub async fn weekly(&self, chain_id: u64, address: &str, weeks: u32) -> Result<Vec<Tvl>> {
        self.history(chain_id, address, TvlPeriod::Week, weeks)
            .await
    }

    /// Get monthly TVL history (convenience method)
    pub async fn monthly(&self, chain_id: u64, address: &str, months: u32) -> Result<Vec<Tvl>> {
        self.history(chain_id, address, TvlPeriod::Month, months)
            .await
    }

    /// Get the latest TVL value
    pub async fn current(&self, chain_id: u64, address: &str) -> Result<Option<Tvl>> {
        let tvls = self.history(chain_id, address, TvlPeriod::Day, 1).await?;
        Ok(tvls.into_iter().next())
    }
}
