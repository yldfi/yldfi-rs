//! Coins/Prices API endpoints

use crate::client::Client;
use crate::error::Result;

use super::types::{
    BatchHistoricalResponse, BlockResponse, ChartResponse, FirstPriceResponse, PercentageResponse,
    PricesResponse, Token,
};

/// Coins/Prices API client
pub struct CoinsApi<'a> {
    client: &'a Client,
}

impl<'a> CoinsApi<'a> {
    /// Create a new Coins API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get current prices for specified coins
    ///
    /// # Arguments
    ///
    /// * `tokens` - List of tokens in `chain:address` format
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![
    ///     Token::coingecko("ethereum"),
    ///     Token::ethereum("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
    /// ];
    /// let prices = client.coins().current(&tokens).await?;
    /// for (id, data) in &prices.coins {
    ///     println!("{}: ${:.2}", id, data.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn current(&self, tokens: &[Token]) -> Result<PricesResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/prices/current/{coins_param}");
        self.client.get_coins(&path).await
    }

    /// Get current prices with search width option
    ///
    /// # Arguments
    ///
    /// * `tokens` - List of tokens in `chain:address` format
    /// * `search_width` - Time range to search for prices (e.g., "4h", "12h")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![Token::coingecko("ethereum")];
    /// let prices = client.coins().current_with_search_width(&tokens, "4h").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn current_with_search_width(
        &self,
        tokens: &[Token],
        search_width: &str,
    ) -> Result<PricesResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/prices/current/{coins_param}?searchWidth={search_width}");
        self.client.get_coins(&path).await
    }

    /// Get historical prices at a specific timestamp
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp
    /// * `tokens` - List of tokens in `chain:address` format
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![Token::coingecko("ethereum")];
    /// let timestamp = 1609459200; // 2021-01-01
    /// let prices = client.coins().historical(timestamp, &tokens).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn historical(&self, timestamp: u64, tokens: &[Token]) -> Result<PricesResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/prices/historical/{timestamp}/{coins_param}");
        self.client.get_coins(&path).await
    }

    /// Get historical prices with options
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp
    /// * `tokens` - List of tokens in `chain:address` format
    /// * `search_width` - Time range to search for prices (e.g., "4h", "12h")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![Token::coingecko("ethereum")];
    /// let timestamp = 1609459200; // 2021-01-01
    /// let prices = client.coins().historical_with_search_width(timestamp, &tokens, "4h").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn historical_with_search_width(
        &self,
        timestamp: u64,
        tokens: &[Token],
        search_width: &str,
    ) -> Result<PricesResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");
        let path =
            format!("/prices/historical/{timestamp}/{coins_param}?searchWidth={search_width}");
        self.client.get_coins(&path).await
    }

    /// Batch query historical prices for multiple timestamps
    ///
    /// # Arguments
    ///
    /// * `queries` - Map of token ID to list of timestamps
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use std::collections::HashMap;
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let mut queries = HashMap::new();
    /// queries.insert(
    ///     Token::coingecko("ethereum").format(),
    ///     vec![1609459200, 1612137600], // Jan 1 and Feb 1 2021
    /// );
    /// let prices = client.coins().batch_historical(&queries).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn batch_historical(
        &self,
        queries: &std::collections::HashMap<String, Vec<u64>>,
    ) -> Result<BatchHistoricalResponse> {
        let path = batch_historical_path(queries)?;
        self.client.get_coins(&path).await
    }

    /// Get price chart data with configurable time span
    ///
    /// # Arguments
    ///
    /// * `tokens` - List of tokens
    /// * `span` - Time span in seconds (optional, defaults to all history)
    /// * `period` - Data point period: "1d", "4h", "1h", etc. (optional)
    /// * `search_width` - Search width for finding prices (optional)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![Token::coingecko("ethereum")];
    /// // Get last 30 days with daily data points
    /// let chart = client.coins().chart(&tokens, Some(30 * 86400), Some("1d"), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chart(
        &self,
        tokens: &[Token],
        span: Option<u64>,
        period: Option<&str>,
        search_width: Option<&str>,
    ) -> Result<ChartResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");

        let mut path = format!("/chart/{coins_param}");
        let mut params = Vec::new();

        if let Some(s) = span {
            params.push(format!("span={s}"));
        }
        if let Some(p) = period {
            params.push(format!("period={p}"));
        }
        if let Some(w) = search_width {
            params.push(format!("searchWidth={w}"));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.client.get_coins(&path).await
    }

    /// Get price percentage changes
    ///
    /// # Arguments
    ///
    /// * `tokens` - List of tokens
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![Token::coingecko("ethereum"), Token::coingecko("bitcoin")];
    /// let changes = client.coins().percentage(&tokens).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn percentage(&self, tokens: &[Token]) -> Result<PercentageResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/percentage/{coins_param}");
        self.client.get_coins(&path).await
    }

    /// Get first recorded price for coins
    ///
    /// # Arguments
    ///
    /// * `tokens` - List of tokens
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    ///
    /// let client = dllma::Client::new()?;
    /// let tokens = vec![Token::coingecko("ethereum")];
    /// let first = client.coins().first_price(&tokens).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn first_price(&self, tokens: &[Token]) -> Result<FirstPriceResponse> {
        let coins_param = tokens
            .iter()
            .map(super::types::Token::format)
            .collect::<Vec<_>>()
            .join(",");
        let path = format!("/prices/first/{coins_param}");
        self.client.get_coins(&path).await
    }

    /// Get block number at a specific timestamp
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "ethereum", "bsc", "polygon")
    /// * `timestamp` - Unix timestamp
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let block = client.coins().block("ethereum", 1609459200).await?;
    /// println!("Block at timestamp: {}", block.height);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn block(&self, chain: &str, timestamp: u64) -> Result<BlockResponse> {
        let path = format!("/block/{chain}/{timestamp}");
        self.client.get_coins(&path).await
    }
}

/// Build `GET /batchHistorical?coins=<json>`, where `coins` is a URL-encoded
/// JSON object mapping each coin to its list of timestamps.
fn batch_historical_path(queries: &std::collections::HashMap<String, Vec<u64>>) -> Result<String> {
    let json = serde_json::to_string(queries).map_err(|e| {
        crate::error::invalid_param(format!("Failed to encode batchHistorical coins: {e}"))
    })?;
    let encoded: String = url::form_urlencoded::byte_serialize(json.as_bytes()).collect();
    Ok(format!("/batchHistorical?coins={encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn batch_historical_uses_get_with_encoded_json_query() {
        let mut queries = HashMap::new();
        queries.insert(
            "coingecko:ethereum".to_string(),
            vec![1_600_000_000, 1_600_086_400],
        );
        let path = batch_historical_path(&queries).unwrap();

        let url = url::Url::parse("https://coins.example")
            .unwrap()
            .join(&path)
            .unwrap();
        assert_eq!(url.path(), "/batchHistorical");
        assert!(!path.contains('{') && !path.contains('"'), "{path}");
        let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "coins");
        let decoded: HashMap<String, Vec<u64>> = serde_json::from_str(&pairs[0].1).unwrap();
        assert_eq!(decoded, queries);
    }
}
