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
    /// Kong API period value (`"1 day"`, `"1 week"`, `"1 month"`)
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TvlPeriod::Day => "1 day",
            TvlPeriod::Week => "1 week",
            TvlPeriod::Month => "1 month",
        }
    }

    /// Approximate period length in seconds
    #[must_use]
    pub fn seconds(&self) -> u64 {
        match self {
            TvlPeriod::Day => 86_400,
            TvlPeriod::Week => 7 * 86_400,
            TvlPeriod::Month => 31 * 86_400,
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Start timestamp so that `limit` points of `period` end at `now`.
///
/// Kong returns points in ascending time order starting at `timestamp`
/// (or at inception when omitted), so without a start the *oldest* points
/// are returned.
fn history_start(now: u64, period: TvlPeriod, limit: u32) -> u64 {
    now.saturating_sub(period.seconds() * u64::from(limit))
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

    /// Get the most recent `limit` TVL points for a vault or strategy
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
        let start = history_start(now_secs(), period, limit);
        self.history_from(chain_id, address, period, limit, start)
            .await
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
                tvls(chainId: {}, address: "{}", period: "{}", limit: {}, timestamp: "{}") {{
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

    /// Get the latest TVL value (most recent daily point)
    pub async fn current(&self, chain_id: u64, address: &str) -> Result<Option<Tvl>> {
        let tvls = self.history(chain_id, address, TvlPeriod::Day, 3).await?;
        Ok(tvls.into_iter().max_by_key(|t| t.time.unwrap_or(0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periods_use_kong_values() {
        assert_eq!(TvlPeriod::Day.as_str(), "1 day");
        assert_eq!(TvlPeriod::Week.as_str(), "1 week");
        assert_eq!(TvlPeriod::Month.as_str(), "1 month");
    }

    #[test]
    fn history_starts_limit_periods_ago() {
        assert_eq!(
            history_start(1_000_000, TvlPeriod::Day, 3),
            1_000_000 - 3 * 86_400
        );
        assert_eq!(history_start(10, TvlPeriod::Week, 3), 0);
    }
}
