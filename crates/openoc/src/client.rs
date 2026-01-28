//! HTTP client for the `OpenOcean` API

use crate::error::{self, Error, Result};
use crate::types::{
    Chain, DexListResponse, QuoteData, QuoteRequest, QuoteResponse, SwapData, SwapRequest,
    SwapResponse, TokenInfo, TokenListResponse,
};
use crate::{default_config, Config};
use yldfi_common::api::BaseClient;

/// Client for the `OpenOcean` DEX Aggregator API
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(default_config())
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let base = BaseClient::new(config)?;
        Ok(Self { base })
    }

    /// Get the underlying base client
    #[must_use]
    pub fn base(&self) -> &BaseClient {
        &self.base
    }

    /// Get the configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        self.base.config()
    }

    /// Get a swap quote (no transaction data)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use openoc::{Client, Chain, QuoteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), openoc::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let request = QuoteRequest::new(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "1000000000000000000", // 1 ETH
    ///     ).with_slippage(1.0);
    ///
    ///     let quote = client.get_quote(Chain::Eth, &request).await?;
    ///     println!("Output: {} USDC", quote.out_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_quote(&self, chain: Chain, request: &QuoteRequest) -> Result<QuoteData> {
        let mut params: Vec<(&str, String)> = vec![
            ("inTokenAddress", request.in_token_address.clone()),
            ("outTokenAddress", request.out_token_address.clone()),
            ("amount", request.amount.clone()),
        ];

        if let Some(slippage) = request.slippage {
            params.push(("slippage", slippage.to_string()));
        }
        if let Some(ref gas_price) = request.gas_price {
            params.push(("gasPrice", gas_price.clone()));
        }
        if let Some(ref disabled) = request.disabled_dex_ids {
            params.push(("disabledDexIds", disabled.clone()));
        }

        let path = format!("/{}/quote", chain.as_str());
        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: QuoteResponse = self.base.get(&path, &query_refs).await?;

        if response.code != 200 {
            return Err(Error::api(
                response.code as u16,
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        response.data.ok_or_else(error::no_route_found)
    }

    /// Get a swap quote with transaction data ready to execute
    ///
    /// # Example
    ///
    /// ```no_run
    /// use openoc::{Client, Chain, SwapRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), openoc::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let request = SwapRequest::new(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "1000000000000000000", // 1 ETH
    ///         "0xYourWalletAddress",
    ///     ).with_slippage(1.0);
    ///
    ///     let swap = client.get_swap_quote(Chain::Eth, &request).await?;
    ///     println!("Send tx to: {}", swap.to);
    ///     println!("Data: {}", swap.data);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_swap_quote(&self, chain: Chain, request: &SwapRequest) -> Result<SwapData> {
        let mut params: Vec<(&str, String)> = vec![
            ("inTokenAddress", request.in_token_address.clone()),
            ("outTokenAddress", request.out_token_address.clone()),
            ("amount", request.amount.clone()),
            ("account", request.account.clone()),
        ];

        if let Some(slippage) = request.slippage {
            params.push(("slippage", slippage.to_string()));
        }
        if let Some(ref gas_price) = request.gas_price {
            params.push(("gasPrice", gas_price.clone()));
        }
        if let Some(ref referrer) = request.referrer {
            params.push(("referrer", referrer.clone()));
        }

        let path = format!("/{}/swap_quote", chain.as_str());
        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: SwapResponse = self.base.get(&path, &query_refs).await?;

        if response.code != 200 {
            return Err(Error::api(
                response.code as u16,
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        response.data.ok_or_else(error::no_route_found)
    }

    /// Get list of supported tokens on a chain
    ///
    /// # Example
    ///
    /// ```no_run
    /// use openoc::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), openoc::Error> {
    ///     let client = Client::new()?;
    ///     let tokens = client.get_token_list(Chain::Eth).await?;
    ///     println!("Found {} tokens", tokens.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_token_list(&self, chain: Chain) -> Result<Vec<TokenInfo>> {
        let path = format!("/{}/tokenList", chain.as_str());
        let response: TokenListResponse = self
            .base
            .get::<TokenListResponse, _>(&path, &[] as &[(&str, &str)])
            .await?;

        if response.code != 200 {
            return Err(Error::api(response.code as u16, "Failed to get token list"));
        }

        Ok(response.data.unwrap_or_default())
    }

    /// Get list of available DEXs on a chain
    ///
    /// # Example
    ///
    /// ```no_run
    /// use openoc::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), openoc::Error> {
    ///     let client = Client::new()?;
    ///     let dexs = client.get_dex_list(Chain::Eth).await?;
    ///     for dex in &dexs {
    ///         println!("{}: {}", dex.code, dex.name);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_dex_list(&self, chain: Chain) -> Result<Vec<crate::types::DexInfo>> {
        let path = format!("/{}/dexList", chain.as_str());
        let response: DexListResponse = self
            .base
            .get::<DexListResponse, _>(&path, &[] as &[(&str, &str)])
            .await?;

        if response.code != 200 {
            return Err(Error::api(response.code as u16, "Failed to get DEX list"));
        }

        Ok(response.data.unwrap_or_default())
    }

    /// Get a reverse quote (specify output amount, calculate input)
    ///
    /// This is for "exact output" swaps where you want a specific amount of the output token.
    pub async fn get_reverse_quote(
        &self,
        chain: Chain,
        in_token: &str,
        out_token: &str,
        out_amount: &str,
    ) -> Result<QuoteData> {
        let path = format!("/{}/reverseQuote", chain.as_str());
        let query: &[(&str, &str)] = &[
            ("inTokenAddress", in_token),
            ("outTokenAddress", out_token),
            ("amount", out_amount),
        ];
        let response: QuoteResponse = self.base.get(&path, query).await?;

        if response.code != 200 {
            return Err(Error::api(
                response.code as u16,
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        response.data.ok_or_else(error::no_route_found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_chain_parsing() {
        assert_eq!(Chain::try_from_str("eth"), Some(Chain::Eth));
        assert_eq!(Chain::try_from_str("ethereum"), Some(Chain::Eth));
        assert_eq!(Chain::try_from_str("polygon"), Some(Chain::Polygon));
        assert_eq!(Chain::try_from_str("unknown"), None);
    }

    #[test]
    fn test_quote_request_builder() {
        let request = QuoteRequest::new(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
        )
        .with_slippage(0.5)
        .with_gas_price("50000000000");

        assert_eq!(request.slippage, Some(0.5));
        assert_eq!(request.gas_price, Some("50000000000".to_string()));
    }

    #[test]
    fn test_swap_request_builder() {
        let request = SwapRequest::new(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        )
        .with_slippage(1.0)
        .with_referrer("0xReferrer");

        assert_eq!(request.slippage, Some(1.0));
        assert_eq!(request.referrer, Some("0xReferrer".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = crate::default_config();
        assert_eq!(config.base_url, crate::DEFAULT_BASE_URL);
    }
}
