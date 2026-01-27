//! HTTP client for Yearn's Kong GraphQL API

use reqwest::Client as HttpClient;
use std::sync::Arc;
use std::time::Duration;
use yldfi_common::api::{extract_retry_after, ApiConfig};
use yldfi_common::rate_limit::RateLimiter;

use crate::error::{from_response, graphql_error, Error, Result};
use crate::types::GraphQLResponse;

/// Base URL for Kong API
pub const BASE_URL: &str = "https://kong.yearn.farm/api/gql";

/// Configuration for the Kong API client
///
/// Use the builder pattern to customize timeouts, proxy, and rate limiting.
///
/// # Example
///
/// ```no_run
/// use ykong::Config;
/// use std::time::Duration;
///
/// let config = Config::new()
///     .with_timeout(Duration::from_secs(60))
///     .with_proxy("http://proxy.example.com:8080")
///     .with_rate_limit(10, Duration::from_secs(1)); // 10 req/sec
/// ```
#[derive(Debug, Clone)]
#[must_use = "Config must be passed to Client::with_config() to take effect"]
pub struct Config {
    /// HTTP client configuration
    inner: ApiConfig,
    /// Optional rate limiter
    rate_limiter: Option<RateLimiter>,
}

impl Config {
    /// Create a new config with defaults (no rate limiting)
    pub fn new() -> Self {
        Self {
            inner: ApiConfig::new(BASE_URL),
            rate_limiter: None,
        }
    }

    /// Set a custom timeout for HTTP requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.inner.http.timeout = timeout;
        self
    }

    /// Set a proxy URL for HTTP requests
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.inner.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL (None to disable)
    pub fn with_optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.inner.http.proxy = proxy;
        self
    }

    /// Enable rate limiting with specified requests per window
    ///
    /// # Arguments
    ///
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window` - Time window duration (e.g., 1 second)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ykong::Config;
    /// use std::time::Duration;
    ///
    /// // Allow 10 requests per second
    /// let config = Config::new()
    ///     .with_rate_limit(10, Duration::from_secs(1));
    ///
    /// // Allow 100 requests per minute
    /// let config = Config::new()
    ///     .with_rate_limit(100, Duration::from_secs(60));
    /// ```
    pub fn with_rate_limit(mut self, max_requests: u32, window: Duration) -> Self {
        self.rate_limiter = Some(RateLimiter::new(max_requests, window));
        self
    }

    /// Set a custom rate limiter
    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(limiter);
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Kong GraphQL API client
///
/// Provides access to Yearn vault, strategy, and price data via Kong's GraphQL API.
///
/// The client is cheaply cloneable (uses `Arc` internally for the HTTP client
/// and rate limiter) and can be shared across threads.
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client wrapped in Arc for cheap cloning
    http: Arc<HttpClient>,
    base_url: String,
    /// Optional rate limiter (already Clone via Arc internally)
    rate_limiter: Option<RateLimiter>,
}

impl Client {
    /// Create a new Kong client with default configuration (no rate limiting)
    pub fn new() -> Result<Self> {
        Self::with_config(Config::new())
    }

    /// Create a client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = config.inner.build_client()?;
        Ok(Self {
            http: Arc::new(http),
            base_url: BASE_URL.to_string(),
            rate_limiter: config.rate_limiter,
        })
    }

    /// Create a new client with a custom HTTP client (no rate limiting)
    ///
    /// Note: The client will be wrapped in Arc internally for cheap cloning.
    #[must_use]
    pub fn with_http_client(http: HttpClient) -> Self {
        Self {
            http: Arc::new(http),
            base_url: BASE_URL.to_string(),
            rate_limiter: None,
        }
    }

    /// Get the underlying HTTP client
    #[must_use]
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the rate limiter if configured
    #[must_use]
    pub fn rate_limiter(&self) -> Option<&RateLimiter> {
        self.rate_limiter.as_ref()
    }

    /// Execute a GraphQL query
    pub async fn query<T>(&self, query: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.query_with_variables(query, serde_json::Value::Null)
            .await
    }

    /// Execute a GraphQL query with variables
    pub async fn query_with_variables<T>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Acquire rate limit permit if configured
        if let Some(limiter) = &self.rate_limiter {
            limiter.acquire().await;
        }

        let body = if variables.is_null() {
            serde_json::json!({ "query": query })
        } else {
            serde_json::json!({
                "query": query,
                "variables": variables
            })
        };

        // Extract a preview of the query for error context
        let query_preview: String = query.chars().take(100).collect();

        let response = self
            .http
            .post(&self.base_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Api {
                status: 0,
                message: format!("HTTP request failed for query '{query_preview}...': {e}"),
            })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let retry_after = extract_retry_after(response.headers());
            let body = response.text().await.unwrap_or_default();
            return Err(from_response(status, &body, retry_after));
        }

        let body = response.text().await.map_err(|e| Error::Api {
            status,
            message: format!("Failed to read response body: {e}"),
        })?;

        let gql_response: GraphQLResponse<T> =
            serde_json::from_str(&body).map_err(|e| Error::Api {
                status,
                message: format!(
                    "Failed to parse GraphQL response for query '{query_preview}...': {e}"
                ),
            })?;

        // Check for GraphQL errors - preserve ALL error messages
        if let Some(errors) = gql_response.errors {
            if !errors.is_empty() {
                let error_messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
                let combined = if error_messages.len() == 1 {
                    error_messages[0].to_string()
                } else {
                    format!(
                        "{} errors: {}",
                        error_messages.len(),
                        error_messages.join("; ")
                    )
                };
                return Err(graphql_error(combined));
            }
        }

        gql_response
            .data
            .ok_or_else(|| graphql_error("No data in GraphQL response"))
    }
}
