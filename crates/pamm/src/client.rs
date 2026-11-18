//! HTTP client for a builder's prop AMM data endpoint (JSON-RPC over HTTP)

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use yldfi_common::api::{ApiError, BaseClient};

use crate::error::{rpc_error, DomainError, Result};
use crate::types::{amount_to_hex, check_address, BookSnapshot, Quote, Region, VenueSnapshot};
use crate::{config_for_region, default_config, Config};

/// Path of the JSON-RPC endpoint on the data host
const DATA_PATH: &str = "/data";

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcErrorBody>,
}

#[derive(Deserialize)]
struct RpcErrorBody {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
}

/// Client for prop AMM quotes, venues and price levels
///
/// Talks to Titan Builder's public data endpoint by default. Another builder
/// that serves the same JSON-RPC methods can be used by passing its base URL
/// in the [`Config`].
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
}

impl Client {
    /// Create a client for Titan Builder's default (EU) data endpoint
    pub fn new() -> Result<Self> {
        Self::with_config(default_config())
    }

    /// Create a client for Titan Builder's data endpoint in `region`
    pub fn with_region(region: Region) -> Result<Self> {
        Self::with_config(config_for_region(region))
    }

    /// Create a client with a custom configuration (base URL, timeout, proxy)
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

    /// Best quote across every prop AMM in the builder's latest snapshot.
    ///
    /// `amount_in` is in the smallest units of `token_in`, as a decimal
    /// string (or `0x`-prefixed hex). The pairs are quoted in wrapped tokens:
    /// pass WETH, not a native-ETH placeholder.
    ///
    /// # Errors
    ///
    /// [`DomainError::UnknownPair`] when no prop AMM quotes the pair.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), pamm::Error> {
    ///     let client = pamm::Client::new()?;
    ///     let quote = client
    ///         .quote(
    ///             "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    ///             "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///             "1000000000000000000",                        // 1 WETH
    ///         )
    ///         .await?;
    ///     println!("{} USDC units from prop AMM {}", quote.amount_out, quote.pamm);
    ///     Ok(())
    /// }
    /// ```
    pub async fn quote(&self, token_in: &str, token_out: &str, amount_in: &str) -> Result<Quote> {
        check_address("token_in", token_in)?;
        check_address("token_out", token_out)?;
        let amount = amount_to_hex(amount_in)?;
        self.call("titan_getPammQuote", json!([token_in, token_out, amount]))
            .await
    }

    /// Quote from one prop AMM, named by its address
    ///
    /// # Errors
    ///
    /// [`DomainError::UnknownPair`] when that prop AMM does not quote the pair.
    pub async fn quote_venue(
        &self,
        venue: &str,
        token_in: &str,
        token_out: &str,
        amount_in: &str,
    ) -> Result<Quote> {
        check_address("venue", venue)?;
        check_address("token_in", token_in)?;
        check_address("token_out", token_out)?;
        let amount = amount_to_hex(amount_in)?;
        self.call(
            "titan_getPammQuoteVenue",
            json!([venue, token_in, token_out, amount]),
        )
        .await
    }

    /// Every prop AMM venue the caller can see: maker, addresses, pairs and
    /// the current state override for simulating swaps locally
    pub async fn venues(&self) -> Result<VenueSnapshot> {
        self.call("titan_getPammVenues", json!([])).await
    }

    /// Price levels for every maker and market
    pub async fn books(&self) -> Result<BookSnapshot> {
        self.call("titan_getPammBooks", json!([])).await
    }

    async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
        let response: RpcResponse<T> = self.base.post_json(DATA_PATH, &body).await?;
        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (None, Some(error)) => Err(rpc_error(error.code, error.message)),
            (None, None) => Err(ApiError::domain(DomainError::EmptyResponse)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yldfi_common::api::ApiConfig;

    /// A one-shot HTTP server on localhost that answers 200 with `body` and
    /// hands back the request it received
    fn serve_once(body: String) -> (String, std::sync::mpsc::Receiver<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                let _ = tx.send(String::from_utf8_lossy(&buf[..n]).into_owned());
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
            }
        });
        (format!("http://{addr}"), rx)
    }

    const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    const TEMPEST: &str = "0x00000003f1ec2379e79f58e12ec6c4f51ee92149";

    fn quote_body() -> String {
        json!({"jsonrpc": "2.0", "id": 1, "result": {
            "tokenIn": WETH.to_lowercase(), "tokenOut": USDC.to_lowercase(),
            "amountIn": "0xde0b6b3a7640000", "amountOut": "0x6be1c920",
            "pamm": TEMPEST, "router": "0x4ddf368080cd7946db5b459ad591c350158175e1",
            "blockNumber": 26_441_191u64, "slot": 15_680_986u64,
            "timestamp": 1_794_995_849_854_286_566u64
        }})
        .to_string()
    }

    fn sent_json(request: &str) -> serde_json::Value {
        let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
        serde_json::from_str(body).unwrap()
    }

    #[tokio::test]
    async fn test_quote_sends_hex_amount_and_parses_result() {
        let (url, rx) = serve_once(quote_body());
        let client = Client::with_config(ApiConfig::new(url)).unwrap();

        let quote = client
            .quote(WETH, USDC, "1000000000000000000")
            .await
            .unwrap();
        assert_eq!(quote.amount_out, "1809959200");
        assert_eq!(quote.pamm, TEMPEST);

        let request = rx.recv().unwrap();
        assert!(request.starts_with("POST /data "), "{request}");
        let sent = sent_json(&request);
        assert_eq!(sent["method"], "titan_getPammQuote");
        assert_eq!(sent["params"], json!([WETH, USDC, "0xde0b6b3a7640000"]));
        assert_eq!(sent["jsonrpc"], "2.0");
    }

    #[tokio::test]
    async fn test_quote_venue_puts_the_venue_first() {
        let (url, rx) = serve_once(quote_body());
        let client = Client::with_config(ApiConfig::new(url)).unwrap();

        client
            .quote_venue(TEMPEST, WETH, USDC, "0xde0b6b3a7640000")
            .await
            .unwrap();
        let sent = sent_json(&rx.recv().unwrap());
        assert_eq!(sent["method"], "titan_getPammQuoteVenue");
        assert_eq!(
            sent["params"],
            json!([TEMPEST, WETH, USDC, "0xde0b6b3a7640000"])
        );
    }

    #[tokio::test]
    async fn test_unknown_pair_error() {
        let body = json!({"jsonrpc": "2.0", "id": 1,
            "error": {"code": -32000, "message": "unknown pair"}})
        .to_string();
        let (url, _rx) = serve_once(body);
        let client = Client::with_config(ApiConfig::new(url)).unwrap();

        let err = client.quote(WETH, USDC, "1").await.unwrap_err();
        assert!(matches!(err, ApiError::Domain(DomainError::UnknownPair)));
    }

    #[tokio::test]
    async fn test_other_rpc_error_and_empty_response() {
        let body = json!({"jsonrpc": "2.0", "id": 1,
            "error": {"code": -32601, "message": "Method not found"}})
        .to_string();
        let (url, _rx) = serve_once(body);
        let client = Client::with_config(ApiConfig::new(url)).unwrap();
        let err = client.venues().await.unwrap_err();
        assert!(matches!(
            err,
            ApiError::Domain(DomainError::Rpc { code: -32601, .. })
        ));

        let (url, _rx) = serve_once(json!({"jsonrpc": "2.0", "id": 1}).to_string());
        let client = Client::with_config(ApiConfig::new(url)).unwrap();
        let err = client.books().await.unwrap_err();
        assert!(matches!(err, ApiError::Domain(DomainError::EmptyResponse)));
    }

    #[tokio::test]
    async fn test_venues_and_books_methods() {
        let body = json!({"jsonrpc": "2.0", "id": 1, "result": {"venues": [
            {"maker": "Tempest", "quoteAddress": TEMPEST}]}})
        .to_string();
        let (url, rx) = serve_once(body);
        let client = Client::with_config(ApiConfig::new(url)).unwrap();
        let venues = client.venues().await.unwrap();
        assert_eq!(venues.maker_of(TEMPEST), Some("Tempest"));
        let sent = sent_json(&rx.recv().unwrap());
        assert_eq!(sent["method"], "titan_getPammVenues");
        assert_eq!(sent["params"], json!([]));

        let body = json!({"jsonrpc": "2.0", "id": 1, "result": {"slot": 1, "block": 2, "ts": 3,
            "books": []}})
        .to_string();
        let (url, rx) = serve_once(body);
        let client = Client::with_config(ApiConfig::new(url)).unwrap();
        let books = client.books().await.unwrap();
        assert_eq!(books.block, Some(2));
        assert_eq!(
            sent_json(&rx.recv().unwrap())["method"],
            "titan_getPammBooks"
        );
    }

    #[tokio::test]
    async fn test_bad_arguments_fail_before_any_request() {
        // Nothing listens on this address: a request would be a network error
        let client = Client::with_config(ApiConfig::new("http://127.0.0.1:1")).unwrap();
        for (token_in, token_out, amount) in [
            ("WETH", USDC, "1"),
            (WETH, "0x1234", "1"),
            (WETH, USDC, "1.5"),
            (WETH, USDC, "0"),
        ] {
            let err = client.quote(token_in, token_out, amount).await.unwrap_err();
            assert!(
                matches!(err, ApiError::Domain(DomainError::InvalidParam(_))),
                "{token_in} {token_out} {amount}: {err}"
            );
        }
        let err = client
            .quote_venue("tempest", WETH, USDC, "1")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ApiError::Domain(DomainError::InvalidParam(_))
        ));
    }

    #[test]
    fn test_default_and_regional_clients() {
        let client = Client::new().unwrap();
        assert_eq!(client.config().base_url, crate::DEFAULT_BASE_URL);
        let client = Client::with_region(Region::Us).unwrap();
        assert_eq!(client.config().base_url, "https://us.data.titanbuilder.xyz");
    }
}
