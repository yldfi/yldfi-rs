//! HTTP client for the Curve API

use crate::error::{Error, Result};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;
use yldfi_common::http::HttpClientConfig;

const DEFAULT_BASE_URL: &str = "https://api.curve.finance/v1";

/// Configuration for the Curve API client
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL for the API
    pub base_url: String,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            http: HttpClientConfig::default(),
        }
    }
}

impl Config {
    /// Create a new config with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

/// Client for the Curve Finance API
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    base_url: String,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = yldfi_common::build_client(&config.http)?;

        Ok(Self {
            http,
            base_url: config.base_url,
        })
    }

    /// Make a GET request to the API
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(&url).send().await?;

        let status = response.status().as_u16();

        // Handle rate limiting (429)
        if status == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            return Err(Error::rate_limited(retry_after));
        }

        if !response.status().is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(Error::api(status, message));
        }

        let data = response.json().await?;
        Ok(data)
    }
}
