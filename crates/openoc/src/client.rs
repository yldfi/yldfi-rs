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
        let mut params = request.to_query_params();
        self.ensure_gas_price(chain, &mut params).await?;
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
        let mut params = request.to_query_params();
        self.ensure_gas_price(chain, &mut params).await?;
        let path = format!("/{}/swap", chain.as_str());
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

    /// Get the current "standard" gas price for a chain, in wei.
    ///
    /// `/quote` and `/swap` require a gas price; [`Client::get_quote`] and
    /// [`Client::get_swap_quote`] call this automatically when the request
    /// doesn't set one.
    pub async fn get_gas_price(&self, chain: Chain) -> Result<String> {
        let path = format!("/{}/gasPrice", chain.as_str());
        let response: serde_json::Value = self
            .base
            .get::<serde_json::Value, _>(&path, &[] as &[(&str, &str)])
            .await?;
        parse_standard_gas_price(&response).ok_or_else(|| {
            error::invalid_param(format!("unexpected gasPrice response: {response}"))
        })
    }

    /// Fill in `gasPriceDecimals` from `/gasPrice` if the caller didn't set it.
    async fn ensure_gas_price(
        &self,
        chain: Chain,
        params: &mut Vec<(&'static str, String)>,
    ) -> Result<()> {
        if !params.iter().any(|(k, _)| *k == "gasPriceDecimals") {
            let gas_price = self.get_gas_price(chain).await?;
            params.push(("gasPriceDecimals", gas_price));
        }
        Ok(())
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
    /// This is for "exact output" swaps: you want to receive `buy_amount` of
    /// `buy_token` and need to know how much `sell_token` to spend.
    ///
    /// Per the v4 docs, `/reverseQuote` is a buy flow whose token parameters
    /// are *reversed* relative to a normal quote: `inTokenAddress` is the
    /// token you want to receive and `outTokenAddress` the token you sell.
    /// This method performs that swap for you. `buy_amount` is human-readable
    /// (e.g. "1" for 1 token; the endpoint only documents the legacy `amount`
    /// parameter). A gas price is fetched automatically - without one the API
    /// returns `{"code":200}` with no data.
    ///
    /// In the returned [`QuoteData`], `in_token`/`in_amount` describe the
    /// token being bought and `reverse_amount` is the required amount of
    /// `sell_token` in its smallest units.
    pub async fn get_reverse_quote(
        &self,
        chain: Chain,
        sell_token: &str,
        buy_token: &str,
        buy_amount: &str,
    ) -> Result<QuoteData> {
        let path = format!("/{}/reverseQuote", chain.as_str());
        let mut params = reverse_quote_params(sell_token, buy_token, buy_amount);
        self.ensure_gas_price(chain, &mut params).await?;
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
}

/// Build `/reverseQuote` params: the API's in/out tokens are reversed
/// (`inTokenAddress` = token to receive).
fn reverse_quote_params(
    sell_token: &str,
    buy_token: &str,
    buy_amount: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("inTokenAddress", buy_token.to_string()),
        ("outTokenAddress", sell_token.to_string()),
        ("amount", buy_amount.to_string()),
    ]
}

/// Extract the "standard" gas price in wei from a `/gasPrice` response.
///
/// EIP-1559 chains return `data.standard` as an object with `legacyGasPrice`;
/// other chains return `data.standard` as a plain number.
fn parse_standard_gas_price(response: &serde_json::Value) -> Option<String> {
    let standard = response.get("data")?.get("standard")?;
    let value = standard.get("legacyGasPrice").unwrap_or(standard);
    match value {
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|v| v.to_string())
            .or_else(|| n.as_f64().map(|v| format!("{v:.0}"))),
        serde_json::Value::String(s) => Some(s.clone()),
        _ => None,
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

    fn param<'a>(params: &'a [(&'static str, String)], key: &str) -> Option<&'a str> {
        params
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn test_quote_params_use_decimals_fields() {
        let params = QuoteRequest::new("0xin", "0xout", "1000000")
            .with_gas_price("30000000000")
            .to_query_params();
        assert_eq!(param(&params, "amountDecimals"), Some("1000000"));
        assert_eq!(param(&params, "gasPriceDecimals"), Some("30000000000"));
        assert_eq!(param(&params, "amount"), None);
        assert_eq!(param(&params, "gasPrice"), None);
    }

    #[test]
    fn test_swap_params_use_decimals_fields() {
        let params = SwapRequest::new("0xin", "0xout", "1000000", "0xacct")
            .with_gas_price("1000000000")
            .to_query_params();
        assert_eq!(param(&params, "amountDecimals"), Some("1000000"));
        assert_eq!(param(&params, "gasPriceDecimals"), Some("1000000000"));
        assert_eq!(param(&params, "account"), Some("0xacct"));
        assert_eq!(param(&params, "amount"), None);
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

    #[test]
    fn test_parse_standard_gas_price_eip1559_shape() {
        let v = serde_json::json!({"code":200,"data":{"base":180676152,"standard":{"legacyGasPrice":180676152,"maxFeePerGas":181676152}}});
        assert_eq!(parse_standard_gas_price(&v), Some("180676152".to_string()));
    }

    #[test]
    fn test_parse_standard_gas_price_legacy_shape() {
        let v = serde_json::json!({"code":200,"data":{"standard":277261410699u64,"fast":277261410699u64}});
        assert_eq!(
            parse_standard_gas_price(&v),
            Some("277261410699".to_string())
        );
    }

    #[test]
    fn test_parse_standard_gas_price_missing() {
        let v = serde_json::json!({"code":400,"error":"bad"});
        assert_eq!(parse_standard_gas_price(&v), None);
    }

    #[test]
    fn test_reverse_quote_params_swap_tokens() {
        // Selling USDC to receive 1 ETH: API wants inToken=ETH, outToken=USDC
        let params = reverse_quote_params("0xUSDC", "0xETH", "1");
        assert_eq!(params[0], ("inTokenAddress", "0xETH".to_string()));
        assert_eq!(params[1], ("outTokenAddress", "0xUSDC".to_string()));
        assert_eq!(params[2], ("amount", "1".to_string()));
    }

    #[test]
    fn test_swap_data_accepts_numeric_estimated_gas() {
        let json = r#"{"inToken":{"address":"0xe","decimals":18,"symbol":"ETH"},
            "outToken":{"address":"0xa","decimals":6,"symbol":"USDC"},
            "inAmount":"1","outAmount":"2","minOutAmount":"1","estimatedGas":690816,
            "to":"0x6352","data":"0x","value":"1"}"#;
        let data: crate::types::SwapData = serde_json::from_str(json).unwrap();
        assert_eq!(data.estimated_gas, "690816");
    }

    #[test]
    fn test_quote_data_reverse_amount() {
        let json = r#"{"inToken":{"address":"0xe","decimals":18,"symbol":"ETH"},
            "outToken":{"address":"0xa","decimals":6,"symbol":"USDC"},
            "inAmount":"1000000000000000000","outAmount":"2691463997",
            "reverseAmount":"2773005623","estimatedGas":"334077"}"#;
        let data: QuoteData = serde_json::from_str(json).unwrap();
        assert_eq!(data.reverse_amount.as_deref(), Some("2773005623"));
    }
}
