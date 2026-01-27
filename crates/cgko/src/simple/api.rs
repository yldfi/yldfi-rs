//! Simple price API endpoints

use super::types::{PricesResponse, PriceOptions, TokenPricesResponse, SupportedCurrencies};
use crate::client::Client;
use crate::error::Result;

/// Simple price API
pub struct SimpleApi<'a> {
    client: &'a Client,
}

impl<'a> SimpleApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get price for coins in given currencies
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> cgko::error::Result<()> {
    /// let client = cgko::Client::new()?;
    /// let prices = client.simple().price(&["bitcoin", "ethereum"], &["usd", "eur"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn price(&self, ids: &[&str], vs_currencies: &[&str]) -> Result<PricesResponse> {
        let path = format!(
            "/simple/price?ids={}&vs_currencies={}",
            ids.join(","),
            vs_currencies.join(",")
        );
        self.client.get(&path).await
    }

    /// Get price with options
    pub async fn price_with_options(
        &self,
        ids: &[&str],
        vs_currencies: &[&str],
        options: &PriceOptions,
    ) -> Result<PricesResponse> {
        let path = format!(
            "/simple/price?ids={}&vs_currencies={}{}",
            ids.join(","),
            vs_currencies.join(","),
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get token prices by contract address
    ///
    /// # Arguments
    /// * `platform` - Asset platform (e.g., "ethereum", "polygon-pos")
    /// * `contract_addresses` - Token contract addresses
    /// * `vs_currencies` - Target currencies
    pub async fn token_price(
        &self,
        platform: &str,
        contract_addresses: &[&str],
        vs_currencies: &[&str],
    ) -> Result<TokenPricesResponse> {
        let path = format!(
            "/simple/token_price/{}?contract_addresses={}&vs_currencies={}",
            platform,
            contract_addresses.join(","),
            vs_currencies.join(",")
        );
        self.client.get(&path).await
    }

    /// Get list of supported vs currencies
    pub async fn supported_vs_currencies(&self) -> Result<SupportedCurrencies> {
        self.client.get("/simple/supported_vs_currencies").await
    }
}
