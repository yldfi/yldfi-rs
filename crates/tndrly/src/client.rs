//! Core Tenderly API client

use crate::error::{self, Error, Result};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use yldfi_common::http::HttpClientConfig;

/// URL-encode a path segment to prevent injection
pub fn encode_path_segment(segment: &str) -> String {
    utf8_percent_encode(segment, NON_ALPHANUMERIC).to_string()
}

/// Base URL for the Tenderly API
pub const API_BASE_URL: &str = "https://api.tenderly.co/api/v1";

/// Default request timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default connect timeout in seconds
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// Configuration for the Tenderly client
#[derive(Clone)]
pub struct Config {
    /// API access key
    pub access_key: SecretString,
    /// Account slug (username or organization)
    pub account: String,
    /// Project slug
    pub project: String,
    /// Optional custom base URL (for testing)
    pub base_url: Option<String>,
    /// Request timeout
    pub timeout: Duration,
    /// Connect timeout
    pub connect_timeout: Duration,
    /// Optional proxy URL
    ///
    /// Supports HTTP/HTTPS proxies with optional authentication:
    /// - `http://proxy.example.com:8080`
    /// - `http://user:password@proxy.example.com:8080`
    ///
    /// **Security Note**: Embedded credentials in the URL are supported but
    /// will be redacted in Debug output. Avoid logging proxy URLs directly.
    pub proxy: Option<String>,
}

impl Config {
    /// Create a new configuration
    pub fn new(
        access_key: impl Into<String>,
        account: impl Into<String>,
        project: impl Into<String>,
    ) -> Self {
        Self {
            access_key: SecretString::from(access_key.into()),
            account: account.into(),
            project: project.into(),
            base_url: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            proxy: None,
        }
    }

    /// Create configuration from environment variables
    ///
    /// Reads:
    /// - `TENDERLY_ACCESS_KEY` (required)
    /// - `TENDERLY_ACCOUNT` (required)
    /// - `TENDERLY_PROJECT` (required)
    pub fn from_env() -> Result<Self> {
        let access_key = std::env::var("TENDERLY_ACCESS_KEY")
            .map_err(|_| error::auth("TENDERLY_ACCESS_KEY environment variable not set"))?;
        let account = std::env::var("TENDERLY_ACCOUNT")
            .map_err(|_| error::auth("TENDERLY_ACCOUNT environment variable not set"))?;
        let project = std::env::var("TENDERLY_PROJECT")
            .map_err(|_| error::auth("TENDERLY_PROJECT environment variable not set"))?;

        Ok(Self::new(access_key, account, project))
    }

    /// Set a custom base URL (useful for testing)
    ///
    /// The URL will be normalized by removing trailing slashes.
    ///
    /// # Warning
    ///
    /// Using non-HTTPS URLs in production is not recommended as API keys
    /// will be transmitted in plain text.
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        // Normalize: remove trailing slashes to prevent double-slash issues
        let normalized = url.trim_end_matches('/').to_string();
        self.base_url = Some(normalized);
        self
    }

    /// Set the request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the connect timeout
    #[must_use]
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set a proxy URL
    #[must_use]
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn with_optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
        self
    }

    /// Get the base URL
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(API_BASE_URL)
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("access_key", &"[REDACTED]")
            .field("account", &self.account)
            .field("project", &self.project)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("proxy", &self.proxy.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// The main Tenderly API client
#[derive(Clone)]
pub struct Client {
    config: Arc<Config>,
    http: reqwest::Client,
    /// Pre-computed URL prefix for API calls (PERF-011 fix)
    url_prefix: String,
}

impl Client {
    /// Create a new Tenderly client with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&HttpClientConfig::default().user_agent);

        if let Some(ref proxy_url) = config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| error::config(format!("Invalid proxy URL: {e}")))?;
            builder = builder.proxy(proxy);
        }

        let http = builder.build().map_err(Error::Http)?;

        // PERF-011 fix: Pre-compute URL prefix to avoid allocation on every API call
        let url_prefix = format!(
            "{}/account/{}/project/{}",
            config.base_url(),
            encode_path_segment(&config.account),
            encode_path_segment(&config.project),
        );

        Ok(Self {
            config: Arc::new(config),
            http,
            url_prefix,
        })
    }

    /// Create a client from environment variables
    pub fn from_env() -> Result<Self> {
        Self::new(Config::from_env()?)
    }

    /// Get the client configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the account slug
    #[must_use]
    pub fn account(&self) -> &str {
        &self.config.account
    }

    /// Get the project slug
    #[must_use]
    pub fn project(&self) -> &str {
        &self.config.project
    }

    /// Build the full URL for an API endpoint
    ///
    /// Uses pre-computed URL prefix for efficiency (PERF-011 fix).
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.url_prefix, path)
    }

    /// Build headers for API requests
    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let access_key = HeaderValue::from_str(self.config.access_key.expose_secret())
            .map_err(|_| error::auth("API access key contains invalid header characters"))?;
        headers.insert("X-Access-Key", access_key);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// Make a GET request to the API
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.url(path);
        let response = self.http.get(&url).headers(self.headers()?).send().await?;

        self.handle_response(response).await
    }

    /// Make a GET request with query parameters
    pub async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = self.url(path);
        let response = self
            .http
            .get(&url)
            .headers(self.headers()?)
            .query(query)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the API
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.url(path);
        let response = self
            .http
            .post(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request without expecting a response body
    pub async fn post_no_response<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.url(path);
        let response = self
            .http
            .post(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Make a DELETE request to the API
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.url(path);
        let response = self
            .http
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .await?;

        self.handle_empty_response(response).await
    }

    /// Make a PUT request to the API
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.url(path);
        let response = self
            .http
            .put(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response and deserialize JSON
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let body = response.json().await?;
            Ok(body)
        } else {
            self.handle_error(status.as_u16(), response).await
        }
    }

    /// Handle API response that doesn't return a body
    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<()> {
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            self.handle_error(status.as_u16(), response).await
        }
    }

    /// Handle error responses
    async fn handle_error<T>(&self, status: u16, response: reqwest::Response) -> Result<T> {
        // Extract rate limit headers before consuming the response
        // Try standard Retry-After first, then Tenderly's X-Tdly-Reset-Timestamp
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| {
                // Tenderly uses X-Tdly-Reset-Timestamp (Unix timestamp)
                // Convert to seconds from now
                response
                    .headers()
                    .get("x-tdly-reset-timestamp")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .and_then(|ts| {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .ok()?
                            .as_secs();
                        ts.checked_sub(now)
                    })
            });

        if status == 429 {
            return Err(Error::rate_limited(retry_after));
        }

        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        match status {
            404 => Err(error::not_found(message)),
            401 | 403 => Err(error::auth(message)),
            400 | 422 => Err(error::invalid_param(message)),
            402 => Err(Error::api(status, format!("Request failed: {message}"))),
            _ => Err(Error::api(status, message)),
        }
    }

    /// Get raw JSON response (for debugging or custom handling)
    pub async fn get_raw(&self, path: &str) -> Result<serde_json::Value> {
        self.get(path).await
    }

    /// Post and get raw JSON response
    pub async fn post_raw<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<serde_json::Value> {
        self.post(path, body).await
    }

    /// Build the full URL for an account-level API endpoint (no project in path)
    #[must_use]
    pub fn account_url(&self, path: &str) -> String {
        format!(
            "{}/account/{}{}",
            self.config.base_url(),
            encode_path_segment(&self.config.account),
            path
        )
    }

    /// Build the full URL for a global API endpoint (no account or project in path)
    #[must_use]
    pub fn global_url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url(), path)
    }

    /// Make a GET request to an account-level endpoint
    pub async fn get_account<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.account_url(path);
        let response = self.http.get(&url).headers(self.headers()?).send().await?;
        self.handle_response(response).await
    }

    /// Make a GET request to a global endpoint (no auth required)
    pub async fn get_global<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.global_url(path);
        let response = self
            .http
            .get(&url)
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a PATCH request to the API
    pub async fn patch<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.url(path);
        let response = self
            .http
            .patch(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a PATCH request without expecting a response body
    pub async fn patch_no_response<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.url(path);
        let response = self
            .http
            .patch(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;
        self.handle_empty_response(response).await
    }

    /// Make a DELETE request with a body
    pub async fn delete_with_body<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = self.url(path);
        let response = self
            .http
            .delete(&url)
            .headers(self.headers()?)
            .json(body)
            .send()
            .await?;
        self.handle_empty_response(response).await
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("key123", "myaccount", "myproject");
        assert_eq!(config.account, "myaccount");
        assert_eq!(config.project, "myproject");
        assert_eq!(config.base_url(), API_BASE_URL);
    }

    #[test]
    fn test_config_with_base_url() {
        let config =
            Config::new("key123", "myaccount", "myproject").with_base_url("https://custom.api.com");
        assert_eq!(config.base_url(), "https://custom.api.com");
    }

    #[test]
    fn test_client_url() {
        let config = Config::new("key123", "myaccount", "myproject");
        let client = Client::new(config).unwrap();
        assert_eq!(
            client.url("/simulate"),
            "https://api.tenderly.co/api/v1/account/myaccount/project/myproject/simulate"
        );
    }

    #[test]
    fn test_config_debug_redacts_key() {
        let config = Config::new("supersecret", "myaccount", "myproject");
        let debug_str = format!("{:?}", config);
        assert!(!debug_str.contains("supersecret"));
        assert!(debug_str.contains("[REDACTED]"));
    }
}
