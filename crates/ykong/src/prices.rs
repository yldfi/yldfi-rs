//! Token price queries from Kong API

use crate::client::Client;
use crate::error::Result;
use crate::types::Price;
use serde::Deserialize;

/// Prices API
pub struct PricesApi<'a> {
    client: &'a Client,
}

impl<'a> PricesApi<'a> {
    /// Create a new prices API instance
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get current price for a token
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// // Get USDC price on Ethereum
    /// let prices = client.prices().get(1, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").await?;
    /// if let Some(price) = prices.first() {
    ///     println!("USDC: ${:.4}", price.price_usd);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, chain_id: u64, address: &str) -> Result<Vec<Price>> {
        let query = format!(
            r#"{{
                prices(chainId: {chain_id}, address: "{address}") {{
                    chainId
                    address
                    priceUsd
                    priceSource
                    blockNumber
                    timestamp
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Response {
            prices: Vec<Price>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.prices)
    }

    /// Get historical price at a specific timestamp
    ///
    /// # Arguments
    ///
    /// * `chain_id` - Chain ID
    /// * `address` - Token address
    /// * `timestamp` - Unix timestamp
    pub async fn at_timestamp(
        &self,
        chain_id: u64,
        address: &str,
        timestamp: u64,
    ) -> Result<Vec<Price>> {
        let query = format!(
            r#"{{
                prices(chainId: {chain_id}, address: "{address}", timestamp: {timestamp}) {{
                    chainId
                    address
                    priceUsd
                    priceSource
                    blockNumber
                    timestamp
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Response {
            prices: Vec<Price>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.prices)
    }

    /// Get current price for a token, returning the first result
    pub async fn current(&self, chain_id: u64, address: &str) -> Result<Option<Price>> {
        let prices = self.get(chain_id, address).await?;
        Ok(prices.into_iter().next())
    }

    /// Get current price in USD, returning just the value
    pub async fn usd(&self, chain_id: u64, address: &str) -> Result<Option<f64>> {
        let price = self.current(chain_id, address).await?;
        Ok(price.map(|p| p.price_usd))
    }
}
