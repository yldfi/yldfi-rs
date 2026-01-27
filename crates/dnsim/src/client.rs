//! HTTP client for the Dune SIM API

use crate::activity::ActivityApi;
use crate::balances::BalancesApi;
use crate::chains::ChainsApi;
use crate::collectibles::CollectiblesApi;
use crate::defi::DefiApi;
use crate::error::{self, Error, Result};
use crate::holders::HoldersApi;
use crate::tokens::TokensApi;
use crate::transactions::TransactionsApi;
use crate::webhooks::WebhooksApi;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::fmt;
use std::time::Duration;
use url::Url;
use yldfi_common::http::HttpClientConfig;

const DEFAULT_BASE_URL: &str = "https://api.sim.dune.com";
const API_KEY_HEADER: &str = "X-Sim-Api-Key";

/// Configuration for the Dune SIM API client
#[derive(Clone)]
pub struct Config {
    /// API key for authentication (redacted in Debug output)
    pub api_key: SecretString,
    /// Base URL for the API
    pub base_url: String,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("api_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("http", &self.http)
            .finish()
    }
}

impl Config {
    /// Create a new configuration with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::from(api_key.into()),
            base_url: DEFAULT_BASE_URL.to_string(),
            http: HttpClientConfig::default(),
        }
    }

    /// Set a custom base URL
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set a custom timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.http.timeout = timeout;
        self
    }

    /// Set a proxy URL
    #[must_use]
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn with_optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.http.proxy = proxy;
        self
    }
}

/// Dune SIM API client
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: Url,
}

impl Client {
    /// Create a new client with an API key
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_config(Config::new(api_key))
    }

    /// Create a new client with a custom base URL
    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self> {
        Self::with_config(Config::new(api_key).with_base_url(base_url))
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            API_KEY_HEADER,
            HeaderValue::from_str(config.api_key.expose_secret())
                .map_err(|_| error::bad_request("Invalid API key format"))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Build the HTTP client with proxy support and custom headers
        let mut builder = reqwest::Client::builder()
            .timeout(config.http.timeout)
            .user_agent(&config.http.user_agent)
            .default_headers(headers);

        if let Some(ref proxy_url) = config.http.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| error::bad_request(format!("Invalid proxy URL: {e}")))?;
            builder = builder.proxy(proxy);
        }

        let http = builder.build().map_err(Error::Http)?;
        let base_url = Url::parse(&config.base_url)?;

        Ok(Self { http, base_url })
    }

    /// Create a new client from environment variable
    ///
    /// Uses `DUNE_SIM_API_KEY` environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DUNE_SIM_API_KEY")
            .map_err(|_| error::unauthorized("DUNE_SIM_API_KEY environment variable not set"))?;
        Self::new(&api_key)
    }

    // API accessor methods

    /// Access wallet activity endpoints
    #[must_use]
    pub fn activity(&self) -> ActivityApi<'_> {
        ActivityApi::new(self)
    }

    /// Access token balances endpoints
    #[must_use]
    pub fn balances(&self) -> BalancesApi<'_> {
        BalancesApi::new(self)
    }

    /// Access supported chains endpoints
    #[must_use]
    pub fn chains(&self) -> ChainsApi<'_> {
        ChainsApi::new(self)
    }

    /// Access collectibles (NFTs) endpoints
    #[must_use]
    pub fn collectibles(&self) -> CollectiblesApi<'_> {
        CollectiblesApi::new(self)
    }

    /// Access `DeFi` positions endpoints (Beta)
    #[must_use]
    pub fn defi(&self) -> DefiApi<'_> {
        DefiApi::new(self)
    }

    /// Access token holders endpoints
    #[must_use]
    pub fn holders(&self) -> HoldersApi<'_> {
        HoldersApi::new(self)
    }

    /// Access token info endpoints
    #[must_use]
    pub fn tokens(&self) -> TokensApi<'_> {
        TokensApi::new(self)
    }

    /// Access transactions endpoints
    #[must_use]
    pub fn transactions(&self) -> TransactionsApi<'_> {
        TransactionsApi::new(self)
    }

    /// Access webhooks subscription endpoints (Beta)
    #[must_use]
    pub fn webhooks(&self) -> WebhooksApi<'_> {
        WebhooksApi::new(self)
    }

    // HTTP methods

    /// Make a GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.base_url.join(path)?;
        let response = self.http.get(url).send().await?;
        self.handle_response(response).await
    }

    /// Make a POST request with a JSON body
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.base_url.join(path)?;
        let response = self.http.post(url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// Make a PUT request with a JSON body
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.base_url.join(path)?;
        let response = self.http.put(url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// Make a PUT request with no response body
    pub async fn put_no_content<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.base_url.join(path)?;
        let response = self.http.put(url).json(body).send().await?;
        self.handle_no_content_response(response).await
    }

    /// Make a PATCH request with a JSON body
    pub async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.base_url.join(path)?;
        let response = self.http.patch(url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// Make a PATCH request with no response body
    pub async fn patch_no_content<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.base_url.join(path)?;
        let response = self.http.patch(url).json(body).send().await?;
        self.handle_no_content_response(response).await
    }

    /// Make a DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.base_url.join(path)?;
        let response = self.http.delete(url).send().await?;
        self.handle_response(response).await
    }

    /// Make a DELETE request with no response body
    pub async fn delete_no_content(&self, path: &str) -> Result<()> {
        let url = self.base_url.join(path)?;
        let response = self.http.delete(url).send().await?;
        self.handle_no_content_response(response).await
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        if (200..300).contains(&status) {
            let body = response.text().await?;
            serde_json::from_str(&body).map_err(Error::Json)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(error::from_response(status, &body, retry_after))
        }
    }

    async fn handle_no_content_response(&self, response: reqwest::Response) -> Result<()> {
        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            Ok(())
        } else {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            let body = response.text().await.unwrap_or_default();
            Err(error::from_response(status, &body, retry_after))
        }
    }
}
