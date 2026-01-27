//! HTTP client for the `CoinGecko` API
//!
//! This client uses common utilities from `yldfi-common` for HTTP operations.

use reqwest::Client as HttpClient;
use std::time::Duration;
use url::Url;
use yldfi_common::api::{extract_retry_after, ApiConfig, SecretApiKey};

use crate::error::{Error, Result};

/// Base URLs
pub mod base_urls {
    /// Pro API
    pub const PRO: &str = "https://pro-api.coingecko.com/api/v3";
    /// Demo/Public API
    pub const DEMO: &str = "https://api.coingecko.com/api/v3";
}

/// Configuration for the `CoinGecko` API client
///
/// Built on top of [`ApiConfig`] from `yldfi-common` for consistent
/// configuration patterns across all API clients.
#[derive(Debug, Clone)]
pub struct Config {
    /// API key (optional for demo, required for pro)
    pub api_key: Option<SecretApiKey>,
    /// Whether to use the Pro API
    pub is_pro: bool,
    /// Inner API configuration
    inner: ApiConfig,
}

impl Config {
    /// Create a new demo config (no API key)
    #[must_use]
    pub fn demo() -> Self {
        Self {
            api_key: None,
            is_pro: false,
            inner: ApiConfig::new(base_urls::DEMO),
        }
    }

    /// Create a new demo config with API key
    pub fn demo_with_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(SecretApiKey::new(api_key)),
            is_pro: false,
            inner: ApiConfig::new(base_urls::DEMO),
        }
    }

    /// Create a new Pro config
    pub fn pro(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(SecretApiKey::new(api_key)),
            is_pro: true,
            inner: ApiConfig::new(base_urls::PRO),
        }
    }

    /// Set a custom timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.inner.http.timeout = timeout;
        self
    }

    /// Set a proxy URL
    #[must_use]
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.inner.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn with_optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.inner.http.proxy = proxy;
        self
    }
}

/// `CoinGecko` API client
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    base_url: Url,
    api_key: Option<SecretApiKey>,
    is_pro: bool,
}

impl Client {
    /// Create a demo/public API client (limited rate)
    pub fn new() -> Result<Self> {
        Self::with_config(Config::demo())
    }

    /// Create a demo API client with optional API key
    pub fn demo(api_key: Option<String>) -> Result<Self> {
        let config = match api_key {
            Some(key) => Config::demo_with_key(key),
            None => Config::demo(),
        };
        Self::with_config(config)
    }

    /// Create a Pro API client
    pub fn pro(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::pro(api_key))
    }

    /// Create a client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = config.inner.build_client()?;
        let base_url = if config.is_pro {
            Url::parse(base_urls::PRO)?
        } else {
            Url::parse(base_urls::DEMO)?
        };

        Ok(Self {
            http,
            base_url,
            api_key: config.api_key,
            is_pro: config.is_pro,
        })
    }

    /// Create from environment variables
    /// Uses `COINGECKO_API_KEY` and `COINGECKO_PRO=true` for Pro API
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("COINGECKO_API_KEY").ok();
        let is_pro = std::env::var("COINGECKO_PRO")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        if is_pro {
            Self::pro(api_key.unwrap_or_default())
        } else {
            Self::demo(api_key)
        }
    }

    #[must_use]
    pub fn is_pro(&self) -> bool {
        self.is_pro
    }

    pub(crate) async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.get(&url);

        if let Some(ref key) = self.api_key {
            let header = if self.is_pro {
                "x-cg-pro-api-key"
            } else {
                "x-cg-demo-api-key"
            };
            req = req.header(header, key.expose());
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        if !response.status().is_success() {
            // Use common extract_retry_after utility
            let retry_after = extract_retry_after(response.headers());
            let body = response.text().await.unwrap_or_default();
            return Err(Error::from_response(status, &body, retry_after));
        }

        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|e| Error::api(status, format!("Parse error: {e}")))
    }
}
