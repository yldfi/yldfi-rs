//! Moralis API client

use crate::analytics::AnalyticsApi;
use crate::block::BlockApi;
use crate::defi::DefiApi;
use crate::discovery::DiscoveryApi;
use crate::entities::EntitiesApi;
use crate::error::{self, Error, Result};
use crate::market::MarketApi;
use crate::nft::NftApi;
use crate::resolve::ResolveApi;
use crate::token::TokenApi;
use crate::transaction::TransactionApi;
use crate::utils::UtilsApi;
use crate::volume::VolumeApi;
use crate::wallet::WalletApi;
use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::time::Duration;
use yldfi_common::http::HttpClientConfig;

const BASE_URL: &str = "https://deep-index.moralis.io/api/v2.2";

/// Configuration for the Moralis client
#[derive(Clone)]
pub struct Config {
    /// API key for authentication
    pub api_key: SecretString,
    /// Base URL for the API
    pub base_url: String,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl Config {
    /// Create a new configuration with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::from(api_key.into()),
            base_url: BASE_URL.to_string(),
            http: HttpClientConfig::default(),
        }
    }

    /// Set a custom base URL
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the request timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http.timeout = timeout;
        self
    }

    /// Set the request timeout (alias for `timeout` for consistency)
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

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("api_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("http", &self.http)
            .finish()
    }
}

/// Client for the Moralis Web3 API
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    api_key: SecretString,
    base_url: String,
}

impl Client {
    /// Create a new client with the given API key
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::new(api_key))
    }

    /// Create a new client from environment variable MORALIS_API_KEY
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MORALIS_API_KEY").map_err(|_| error::missing_api_key())?;
        Self::new(api_key)
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        if config.api_key.expose_secret().is_empty() {
            return Err(error::missing_api_key());
        }

        let http = yldfi_common::build_client(&config.http)?;

        Ok(Self {
            http,
            api_key: config.api_key,
            base_url: config.base_url,
        })
    }

    /// Extract retry-after header value in seconds
    fn get_retry_after(response: &reqwest::Response) -> Option<u64> {
        response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
    }

    /// Handle error response and convert to appropriate Error type
    async fn handle_error_response(response: reqwest::Response) -> Error {
        let status = response.status().as_u16();
        let retry_after = Self::get_retry_after(&response);
        let body = response.text().await.unwrap_or_default();
        Error::from_response(status, &body, retry_after)
    }

    /// Make a GET request to the API
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("X-API-Key", self.api_key.expose_secret())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Make a GET request with query parameters
    pub(crate) async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("X-API-Key", self.api_key.expose_secret())
            .query(query)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Make a POST request with JSON body
    pub(crate) async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(&url)
            .header("X-API-Key", self.api_key.expose_secret())
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Make a POST request with JSON body and query parameters
    pub(crate) async fn post_with_query<
        T: DeserializeOwned,
        B: serde::Serialize,
        Q: serde::Serialize,
    >(
        &self,
        path: &str,
        body: &B,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(&url)
            .header("X-API-Key", self.api_key.expose_secret())
            .query(query)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Make a PUT request with JSON body
    pub(crate) async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .put(&url)
            .header("X-API-Key", self.api_key.expose_secret())
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Access the Wallet API
    pub fn wallet(&self) -> WalletApi<'_> {
        WalletApi::new(self)
    }

    /// Access the Token API
    pub fn token(&self) -> TokenApi<'_> {
        TokenApi::new(self)
    }

    /// Access the NFT API
    pub fn nft(&self) -> NftApi<'_> {
        NftApi::new(self)
    }

    /// Access the Block API
    pub fn block(&self) -> BlockApi<'_> {
        BlockApi::new(self)
    }

    /// Access the Transaction API
    pub fn transaction(&self) -> TransactionApi<'_> {
        TransactionApi::new(self)
    }

    /// Access the DeFi API
    pub fn defi(&self) -> DefiApi<'_> {
        DefiApi::new(self)
    }

    /// Access the Resolve API (ENS, domains)
    pub fn resolve(&self) -> ResolveApi<'_> {
        ResolveApi::new(self)
    }

    /// Access the Market Data API
    pub fn market(&self) -> MarketApi<'_> {
        MarketApi::new(self)
    }

    /// Access the Discovery API
    pub fn discovery(&self) -> DiscoveryApi<'_> {
        DiscoveryApi::new(self)
    }

    /// Access the Entities API
    pub fn entities(&self) -> EntitiesApi<'_> {
        EntitiesApi::new(self)
    }

    /// Access the Utils/Contract API
    pub fn utils(&self) -> UtilsApi<'_> {
        UtilsApi::new(self)
    }

    /// Access the Volume API
    pub fn volume(&self) -> VolumeApi<'_> {
        VolumeApi::new(self)
    }

    /// Access the Analytics API
    pub fn analytics(&self) -> AnalyticsApi<'_> {
        AnalyticsApi::new(self)
    }
}
