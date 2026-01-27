//! Prices API implementation

use super::types::{
    HistoricalInterval, HistoricalPriceRequest, HistoricalPriceResponse, TokenAddress,
    TokenPricesByAddressRequest, TokenPricesByAddressResponse, TokenPricesBySymbolResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Prices API for token price data
pub struct PricesApi<'a> {
    client: &'a Client,
}

impl<'a> PricesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get current prices for tokens by their symbols
    ///
    /// # Arguments
    /// * `symbols` - Token symbols (e.g., ["ETH", "BTC", "USDC"])
    ///
    /// # Example
    /// ```ignore
    /// let prices = client.prices().get_prices_by_symbol(&["ETH", "USDC"]).await?;
    /// ```
    pub async fn get_prices_by_symbol(
        &self,
        symbols: &[&str],
    ) -> Result<TokenPricesBySymbolResponse> {
        let query: Vec<(&str, &str)> = symbols.iter().map(|s| ("symbols", *s)).collect();
        self.client.prices_get("tokens/by-symbol", &query).await
    }

    /// Get current prices for tokens by their network and address
    ///
    /// # Arguments
    /// * `addresses` - List of (network, address) tuples
    ///
    /// # Example
    /// ```ignore
    /// let prices = client.prices().get_prices_by_address(&[
    ///     ("eth-mainnet", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
    /// ]).await?;
    /// ```
    pub async fn get_prices_by_address(
        &self,
        addresses: &[(&str, &str)],
    ) -> Result<TokenPricesByAddressResponse> {
        let body = TokenPricesByAddressRequest {
            addresses: addresses
                .iter()
                .map(|(network, address)| TokenAddress {
                    network: network.to_string(),
                    address: address.to_string(),
                })
                .collect(),
        };
        self.client.prices_post("tokens/by-address", &body).await
    }

    /// Get historical prices for a token by symbol
    ///
    /// # Arguments
    /// * `symbol` - Token symbol (e.g., "ETH")
    /// * `start_time` - Start timestamp (ISO 8601 or Unix)
    /// * `end_time` - End timestamp (ISO 8601 or Unix)
    /// * `interval` - Time interval between data points
    pub async fn get_historical_by_symbol(
        &self,
        symbol: &str,
        start_time: &str,
        end_time: &str,
        interval: HistoricalInterval,
    ) -> Result<HistoricalPriceResponse> {
        let body = HistoricalPriceRequest {
            symbol: Some(symbol.to_string()),
            network: None,
            address: None,
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            interval: interval.as_str().to_string(),
        };
        self.client.prices_post("tokens/historical", &body).await
    }

    /// Get historical prices for a token by network and address
    ///
    /// # Arguments
    /// * `network` - Network name (e.g., "eth-mainnet")
    /// * `address` - Token contract address
    /// * `start_time` - Start timestamp (ISO 8601 or Unix)
    /// * `end_time` - End timestamp (ISO 8601 or Unix)
    /// * `interval` - Time interval between data points
    pub async fn get_historical_by_address(
        &self,
        network: &str,
        address: &str,
        start_time: &str,
        end_time: &str,
        interval: HistoricalInterval,
    ) -> Result<HistoricalPriceResponse> {
        let body = HistoricalPriceRequest {
            symbol: None,
            network: Some(network.to_string()),
            address: Some(address.to_string()),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            interval: interval.as_str().to_string(),
        };
        self.client.prices_post("tokens/historical", &body).await
    }
}
