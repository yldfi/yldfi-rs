//! HTTP client for the Dune API

use crate::error::{self, Error, Result};
use crate::executions::ExecutionsApi;
use crate::matviews::MatviewsApi;
use crate::pipelines::PipelinesApi;
use crate::queries::QueriesApi;
use crate::tables::TablesApi;
use crate::usage::UsageApi;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use std::fmt;
use std::time::Duration;
use yldfi_common::http::HttpClientConfig;

const BASE_URL: &str = "https://api.dune.com/api";

/// Configuration for the Dune API client
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
            base_url: BASE_URL.to_string(),
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

/// Dune Analytics API client
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    /// Create a new Dune client with the given API key
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_config(Config::new(api_key))
    }

    /// Create a client with a custom base URL (useful for testing)
    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self> {
        Self::with_config(Config::new(api_key).with_base_url(base_url))
    }

    /// Create a client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Dune-API-Key",
            HeaderValue::from_str(config.api_key.expose_secret())
                .map_err(|_| error::invalid_api_key())?,
        );

        // Build the HTTP client with proxy support and custom headers
        let mut builder = reqwest::Client::builder()
            .timeout(config.http.timeout)
            .user_agent(&config.http.user_agent)
            .default_headers(headers);

        if let Some(ref proxy_url) = config.http.proxy {
            let proxy = reqwest::Proxy::all(proxy_url).map_err(|e| Error::Api {
                status: 0,
                message: format!("Invalid proxy URL: {e}"),
            })?;
            builder = builder.proxy(proxy);
        }

        let http = builder.build()?;

        Ok(Self {
            http,
            base_url: config.base_url,
        })
    }

    /// Create a new client from environment variable
    ///
    /// Uses `DUNE_API_KEY` environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DUNE_API_KEY").map_err(|_| error::invalid_api_key())?;
        Self::new(&api_key)
    }

    /// Get the HTTP client
    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Get the base URL
    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Access the Queries API
    #[must_use] 
    pub fn queries(&self) -> QueriesApi<'_> {
        QueriesApi::new(self)
    }

    /// Access the Executions API
    #[must_use] 
    pub fn executions(&self) -> ExecutionsApi<'_> {
        ExecutionsApi::new(self)
    }

    /// Access the Tables (uploads) API
    #[must_use] 
    pub fn tables(&self) -> TablesApi<'_> {
        TablesApi::new(self)
    }

    /// Access the Materialized Views API
    #[must_use] 
    pub fn matviews(&self) -> MatviewsApi<'_> {
        MatviewsApi::new(self)
    }

    /// Access the Pipelines API
    #[must_use] 
    pub fn pipelines(&self) -> PipelinesApi<'_> {
        PipelinesApi::new(self)
    }

    /// Access the Usage API
    #[must_use] 
    pub fn usage(&self) -> UsageApi<'_> {
        UsageApi::new(self)
    }
}
