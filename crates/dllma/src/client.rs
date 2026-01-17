//! HTTP client for the DefiLlama API

use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use url::Url;
use yldfi_common::http::HttpClientConfig;

use crate::error::{Error, Result};

/// Base URLs for DefiLlama APIs
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

/// Configuration for the DefiLlama API client
#[derive(Debug, Clone)]
pub struct Config {
    /// Pro API key (optional, enables Pro endpoints)
    pub api_key: Option<SecretString>,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl Config {
    /// Create a new free-tier config
    pub fn new() -> Self {
        Self {
            api_key: None,
            http: HttpClientConfig::default(),
        }
    }

    /// Create a new Pro config with API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(SecretString::from(api_key.into())),
            http: HttpClientConfig::default(),
        }
    }

    /// Set a custom timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http.timeout = timeout;
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
        self.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.http.proxy = proxy;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// DefiLlama API client
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    main_url: Url,
    coins_url: Url,
    stablecoins_url: Url,
    yields_url: Url,
    /// Pro API key (if provided, enables Pro endpoints)
    api_key: Option<SecretString>,
}

impl Client {
    /// Create a new DefiLlama client (free tier)
    pub fn new() -> Result<Self> {
        Self::with_config(Config::new())
    }

    /// Create a new DefiLlama client with Pro API key
    ///
    /// Pro API keys unlock additional endpoints like yields, bridges,
    /// emissions, and more. Get your key at <https://defillama.com/subscription>
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::with_api_key(api_key))
    }

    /// Create a client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = yldfi_common::build_client(&config.http)?;

        Ok(Self {
            http,
            main_url: Url::parse(base_urls::MAIN)?,
            coins_url: Url::parse(base_urls::COINS)?,
            stablecoins_url: Url::parse(base_urls::STABLECOINS)?,
            yields_url: Url::parse(base_urls::YIELDS)?,
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
            api_key: None,
        })
    }

    /// Check if this client has a Pro API key configured
    pub fn has_pro_access(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get the underlying HTTP client
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the main API base URL
    pub fn main_url(&self) -> &Url {
        &self.main_url
    }

    /// Get the coins API base URL
    pub fn coins_url(&self) -> &Url {
        &self.coins_url
    }

    /// Get the stablecoins API base URL
    pub fn stablecoins_url(&self) -> &Url {
        &self.stablecoins_url
    }

    /// Get the yields API base URL
    pub fn yields_url(&self) -> &Url {
        &self.yields_url
    }

    /// Make a GET request to the main API (free endpoints)
    pub(crate) async fn get_main<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
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

        let url = Url::parse(&format!(
            "{}/{}{}",
            base_urls::PRO,
            api_key.expose_secret(),
            path
        ))?;
        self.get(&url).await
    }

    /// Make a GET request to the coins API
    pub(crate) async fn get_coins<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.coins_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request to the stablecoins API
    pub(crate) async fn get_stablecoins<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T> {
        let url = self.stablecoins_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request to the yields API (free tier)
    pub(crate) async fn get_yields<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.yields_url.join(path)?;
        self.get(&url).await
    }

    /// Make a GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &Url) -> Result<T> {
        let response = self.http.get(url.as_str()).send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());

            let body = response.text().await.unwrap_or_default();
            return Err(Error::from_response(status, &body, retry_after));
        }

        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|e| Error::Api {
            status,
            message: format!("Failed to parse response: {e}"),
        })
    }

    /// Make a POST request to the coins API
    pub(crate) async fn post_coins<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = self.coins_url.join(path)?;
        let response = self.http.post(url.as_str()).json(body).send().await?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());

            let body = response.text().await.unwrap_or_default();
            return Err(Error::from_response(status, &body, retry_after));
        }

        let body = response.text().await?;
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
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let usage = client.usage().await?;
    /// if let Some(left) = usage.credits_left {
    ///     println!("Credits remaining: {}", left);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn usage(&self) -> Result<ApiUsage> {
        self.get_pro("/usage/APIKEY").await
    }
}
