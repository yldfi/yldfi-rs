//! HTTP client for the `DefiLlama` API
//!
//! This client uses common utilities from `yldfi-common` for HTTP operations
//! while supporting `DefiLlama`'s multi-URL API structure.

use reqwest::Client as HttpClient;
use std::time::Duration;
use url::Url;
use yldfi_common::api::{extract_retry_after, ApiConfig, SecretApiKey};

use crate::error::{Error, Result};

/// Base URLs for `DefiLlama` APIs
pub mod base_urls {
    /// Main API (TVL, protocols, fees, volumes) - Free tier
    pub const MAIN: &str = "https://api.llama.fi";
    /// Pro API (includes all endpoints)
    pub const PRO: &str = "https://pro-api.llama.fi";
    /// Coins/prices API
    pub const COINS: &str = "https://coins.llama.fi";
    /// Stablecoins API
    pub const STABLECOINS: &str = "https://stablecoins.llama.fi";
    /// Yields API (free tier)
    pub const YIELDS: &str = "https://yields.llama.fi";
}

/// Convert a transport error without its URL.
///
/// Pro requests carry the API key in the URL path, and `reqwest::Error`'s
/// `Display` includes the request URL, so it must be stripped before the
/// error can be logged or shown.
fn strip_url(e: reqwest::Error) -> Error {
    Error::from(e.without_url())
}

/// Configuration for the `DefiLlama` API client
///
/// Built on top of [`ApiConfig`] from `yldfi-common` for consistent
/// configuration patterns across all API clients.
#[derive(Debug, Clone)]
pub struct Config {
    /// Pro API key (optional, enables Pro endpoints)
    pub api_key: Option<SecretApiKey>,
    /// HTTP client configuration
    inner: ApiConfig,
    /// Send free-API requests to their Pro mirrors when a key is set
    pro_routes: bool,
}

impl Config {
    /// Create a new free-tier config
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_key: None,
            inner: ApiConfig::new(base_urls::MAIN),
            pro_routes: false,
        }
    }

    /// Create a new Pro config with API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(SecretApiKey::new(api_key)),
            inner: ApiConfig::new(base_urls::MAIN),
            pro_routes: false,
        }
    }

    /// Set a custom timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner.http.timeout = timeout;
        self
    }

    /// Set a custom timeout (alias for `timeout` for consistency)
    #[must_use]
    pub fn with_timeout(self, timeout: Duration) -> Self {
        self.timeout(timeout)
    }

    /// Set a proxy URL
    #[must_use]
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.inner.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.inner.http.proxy = proxy;
        self
    }

    /// Send the free API's endpoints to their Pro mirrors (off by default)
    ///
    /// With an API key set, the TVL, fees, volumes, coins and stablecoins
    /// methods that normally call `api.llama.fi`, `coins.llama.fi` and
    /// `stablecoins.llama.fi` call the same endpoints on
    /// `pro-api.llama.fi` instead, under `/api/...`, `/coins/...` and
    /// `/stablecoins/...`, which have the Pro plan's rate limits and use
    /// its credits. The responses are the same. Without an API key this has
    /// no effect.
    ///
    /// ```no_run
    /// # fn example() -> dllma::error::Result<()> {
    /// let config = dllma::Config::with_api_key("your-api-key").pro_routes(true);
    /// let client = dllma::Client::with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn pro_routes(mut self, enabled: bool) -> Self {
        self.pro_routes = enabled;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// `DefiLlama` API client
///
/// Supports multiple `DefiLlama` API endpoints (main, coins, stablecoins, yields, pro).
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    main_url: Url,
    coins_url: Url,
    stablecoins_url: Url,
    yields_url: Url,
    pro_url: Url,
    /// Pro API key (if provided, enables Pro endpoints)
    api_key: Option<SecretApiKey>,
    /// Send free-API requests to their Pro mirrors (only with a key)
    pro_routes: bool,
}

impl Client {
    /// Create a new `DefiLlama` client (free tier)
    pub fn new() -> Result<Self> {
        Self::with_config(Config::new())
    }

    /// Create a new `DefiLlama` client with Pro API key
    ///
    /// Pro API keys unlock additional endpoints like yields, bridges,
    /// emissions, and more. Get your key at <https://defillama.com/subscription>
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::with_api_key(api_key))
    }

    /// Create a client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = config.inner.build_client()?;

        Ok(Self {
            http,
            main_url: Url::parse(base_urls::MAIN)?,
            coins_url: Url::parse(base_urls::COINS)?,
            stablecoins_url: Url::parse(base_urls::STABLECOINS)?,
            yields_url: Url::parse(base_urls::YIELDS)?,
            pro_url: Url::parse(base_urls::PRO)?,
            pro_routes: config.pro_routes && config.api_key.is_some(),
            api_key: config.api_key,
        })
    }

    /// Create a client from the `DEFILLAMA_API_KEY` environment variable
    ///
    /// Returns a free-tier client if the env var is not set.
    pub fn from_env() -> Result<Self> {
        match std::env::var("DEFILLAMA_API_KEY") {
            Ok(key) if !key.is_empty() => Self::with_api_key(key),
            _ => Self::new(),
        }
    }

    /// Create a new client with a custom HTTP client
    pub fn with_http_client(http: HttpClient) -> Result<Self> {
        Ok(Self {
            http,
            main_url: Url::parse(base_urls::MAIN)?,
            coins_url: Url::parse(base_urls::COINS)?,
            stablecoins_url: Url::parse(base_urls::STABLECOINS)?,
            yields_url: Url::parse(base_urls::YIELDS)?,
            pro_url: Url::parse(base_urls::PRO)?,
            api_key: None,
            pro_routes: false,
        })
    }

    /// Check if this client has a Pro API key configured
    #[must_use]
    pub fn has_pro_access(&self) -> bool {
        self.api_key.is_some()
    }

    /// Whether free-API endpoints go to their Pro mirrors
    /// (see [`Config::pro_routes`])
    #[must_use]
    pub fn uses_pro_routes(&self) -> bool {
        self.pro_routes
    }

    /// Get the underlying HTTP client
    #[must_use]
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the main API base URL
    #[must_use]
    pub fn main_url(&self) -> &Url {
        &self.main_url
    }

    /// Get the coins API base URL
    #[must_use]
    pub fn coins_url(&self) -> &Url {
        &self.coins_url
    }

    /// Get the stablecoins API base URL
    #[must_use]
    pub fn stablecoins_url(&self) -> &Url {
        &self.stablecoins_url
    }

    /// Get the yields API base URL
    #[must_use]
    pub fn yields_url(&self) -> &Url {
        &self.yields_url
    }

    /// Make a GET request to the main API (free endpoints)
    pub(crate) async fn get_main<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        if self.pro_routes {
            return self.get_pro(&format!("/api{path}")).await;
        }
        let url = self.main_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request to Pro API endpoints
    ///
    /// Pro API key is inserted into the URL path:
    /// `https://pro-api.llama.fi/{API_KEY}/{endpoint}`
    pub(crate) async fn get_pro<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            Error::api(
                401,
                "Pro API key required. Set DEFILLAMA_API_KEY or use Client::with_api_key()",
            )
        })?;

        let url = Url::parse(&format!("{}{}{}", self.pro_url, api_key.expose(), path))?;
        self.get(&url).await
    }

    /// Make a GET request to the coins API
    pub(crate) async fn get_coins<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        if self.pro_routes {
            return self.get_pro(&format!("/coins{path}")).await;
        }
        let url = self.coins_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request to the stablecoins API
    pub(crate) async fn get_stablecoins<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T> {
        if self.pro_routes {
            return self.get_pro(&format!("/stablecoins{path}")).await;
        }
        let url = self.stablecoins_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request to the yields API (free tier)
    pub(crate) async fn get_yields<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.yields_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request using common error handling
    ///
    /// Pro URLs carry the API key in the path, so transport errors are
    /// stripped of their URL before they reach callers or logs.
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &Url) -> Result<T> {
        let response = self
            .http
            .get(url.as_str())
            .send()
            .await
            .map_err(strip_url)?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            // Use common extract_retry_after utility
            let retry_after = extract_retry_after(response.headers());
            let body = response.text().await.unwrap_or_default();
            return Err(crate::error::from_response(status, &body, retry_after));
        }

        let body = response.text().await.map_err(strip_url)?;
        serde_json::from_str(&body).map_err(|e| Error::Api {
            status,
            message: format!("Failed to parse response: {e}"),
        })
    }
}

/// API usage information (Pro)
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsage {
    /// Credits remaining this month
    pub credits_left: Option<u64>,
    /// Total credits per month
    pub credits_total: Option<u64>,
    /// Credits used this month
    pub credits_used: Option<u64>,
    /// API key tier/plan
    pub plan: Option<String>,
    /// Resets on date
    pub resets_on: Option<String>,
}

impl Client {
    /// Check API usage and remaining credits (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// Returns the amount of credits left in your API key.
    /// Credits reset on the 1st of each month.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let usage = client.usage().await?;
    /// if let Some(left) = usage.credits_left {
    ///     println!("Credits remaining: {}", left);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn usage(&self) -> Result<ApiUsage> {
        // Unlike every other Pro route, the key goes at the end of this path:
        // `https://pro-api.llama.fi/usage/{API_KEY}`.
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            Error::api(
                401,
                "Pro API key required. Set DEFILLAMA_API_KEY or use Client::with_api_key()",
            )
        })?;
        let url = Url::parse(&format!("{}usage/{}", self.pro_url, api_key.expose()))?;
        self.get(&url).await
    }
}

#[cfg(test)]
impl Client {
    /// Point every host at `base` (a local test server)
    pub(crate) fn with_test_base(mut self, base: &str) -> Self {
        let url = Url::parse(base).expect("test base URL");
        self.main_url = url.clone();
        self.coins_url = url.clone();
        self.stablecoins_url = url.clone();
        self.yields_url = url.clone();
        self.pro_url = url;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn transport_error_does_not_leak_url_path_key() {
        let http = HttpClient::new();
        let err = http
            .get("http://127.0.0.1:1/FAKEKEY/usage/APIKEY")
            .send()
            .await
            .expect_err("nothing listens on port 1");
        assert!(err.to_string().contains("FAKEKEY"));

        let err = strip_url(err);
        assert!(!err.to_string().contains("FAKEKEY"), "{err}");
        assert!(!format!("{err:?}").contains("FAKEKEY"), "{err:?}");
    }

    fn test_client(config: Config, server: &crate::test_server::TestServer) -> Client {
        Client::with_config(config)
            .unwrap()
            .with_test_base(&server.url)
    }

    #[tokio::test]
    async fn pro_routes_send_free_endpoints_to_pro_mirrors() {
        let config = Config::with_api_key("KEY").pro_routes(true);
        let server = crate::test_server::TestServer::json("12.5");
        let client = test_client(config.clone(), &server);
        assert!(client.uses_pro_routes());
        assert_eq!(client.tvl().protocol_tvl("aave").await.unwrap(), 12.5);
        assert_eq!(server.path(), "/KEY/api/tvl/aave");

        let server = crate::test_server::TestServer::json(r#"{"height":1,"timestamp":2}"#);
        let client = test_client(config.clone(), &server);
        client.coins().block("ethereum", 2).await.unwrap();
        assert_eq!(server.path(), "/KEY/coins/block/ethereum/2");

        let server = crate::test_server::TestServer::json("[]");
        let client = test_client(config, &server);
        client.stablecoins().chains().await.unwrap();
        assert_eq!(server.path(), "/KEY/stablecoins/stablecoinchains");
    }

    #[tokio::test]
    async fn free_endpoints_stay_free_by_default() {
        let server = crate::test_server::TestServer::json("1");
        let client = test_client(Config::with_api_key("KEY"), &server);
        assert!(!client.uses_pro_routes());
        client.tvl().protocol_tvl("aave").await.unwrap();
        assert_eq!(server.path(), "/tvl/aave");

        // Without a key the option has no effect
        let server = crate::test_server::TestServer::json("1");
        let client = test_client(Config::new().pro_routes(true), &server);
        assert!(!client.uses_pro_routes());
        client.tvl().protocol_tvl("aave").await.unwrap();
        assert_eq!(server.path(), "/tvl/aave");
    }

    #[tokio::test]
    async fn usage_puts_key_at_end() {
        let server = crate::test_server::TestServer::json(r#"{"creditsLeft":5}"#);
        let client = test_client(Config::with_api_key("KEY"), &server);
        assert_eq!(client.usage().await.unwrap().credits_left, Some(5));
        assert_eq!(server.path(), "/usage/KEY");
    }
}
