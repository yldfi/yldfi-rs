//! HTTP client for the Curve API

use crate::error::{Error, Result};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.curve.finance/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the Curve API client
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL for the API
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl Config {
    /// Create a new config with default settings
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
        self.timeout = timeout;
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
        let http = HttpClient::builder().timeout(config.timeout).build()?;

        Ok(Self {
            http,
            base_url: config.base_url,
        })
    }

    /// Make a GET request to the API
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::api(status, message));
        }

        let data = response.json().await?;
        Ok(data)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}
