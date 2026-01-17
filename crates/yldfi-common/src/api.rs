//! Generic API client infrastructure
//!
//! This module provides shared types for API clients, reducing boilerplate
//! across the DEX aggregator crates.
//!
//! # Error Types
//!
//! ```
//! use yldfi_common::api::{ApiError, ApiResult};
//!
//! // Use with no domain-specific errors
//! type MyResult<T> = ApiResult<T>;
//!
//! // Or with domain-specific errors
//! #[derive(Debug, thiserror::Error)]
//! enum MyDomainError {
//!     #[error("No route found")]
//!     NoRouteFound,
//! }
//!
//! type MyError = ApiError<MyDomainError>;
//! ```
//!
//! # Config Types
//!
//! ```no_run
//! use yldfi_common::api::ApiConfig;
//!
//! let config = ApiConfig::new("https://api.example.com")
//!     .api_key("your-key")
//!     .with_timeout_secs(30);
//! ```

use crate::http::{HttpClientConfig, HttpError};
use crate::RetryableError;
use reqwest::Client;
use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// Marker type for API errors with no domain-specific variants
///
/// This type is used as the default for `ApiError<E>` when no domain-specific
/// errors are needed. It can never be constructed, so the `Domain` variant
/// is effectively unused.
#[derive(Debug, Clone, Copy)]
pub enum NoDomainError {}

impl fmt::Display for NoDomainError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl std::error::Error for NoDomainError {}

/// Generic API error type with support for domain-specific errors
///
/// This error type covers the common error cases across all API clients:
/// - HTTP transport errors
/// - JSON parsing errors
/// - API response errors (4xx)
/// - Rate limiting (429)
/// - Server errors (5xx)
///
/// Domain-specific errors can be added via the generic parameter `E`.
/// Use `ApiError` (without a type parameter) when no domain-specific errors
/// are needed.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ApiError<E: std::error::Error = NoDomainError> {
    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP client build error
    #[error("HTTP client error: {0}")]
    HttpBuild(#[source] HttpError),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response (4xx, excluding 429)
    #[error("API error: {status} - {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message from API
        message: String,
    },

    /// Rate limit exceeded (429)
    #[error("Rate limited{}", .retry_after.map(|s| format!(" (retry after {}s)", s)).unwrap_or_default())]
    RateLimited {
        /// Seconds to wait before retrying
        retry_after: Option<u64>,
    },

    /// Server error (5xx)
    #[error("Server error ({status}): {message}")]
    ServerError {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
    },

    /// URL parsing error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Domain-specific error
    #[error(transparent)]
    Domain(E),
}

// Manual From impl for HttpError since we can't use #[from] with the generic
impl<E: std::error::Error> From<HttpError> for ApiError<E> {
    fn from(e: HttpError) -> Self {
        ApiError::HttpBuild(e)
    }
}

impl<E: std::error::Error> ApiError<E> {
    /// Create an API error
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create a rate limited error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        Self::RateLimited { retry_after }
    }

    /// Create a server error
    pub fn server_error(status: u16, message: impl Into<String>) -> Self {
        Self::ServerError {
            status,
            message: message.into(),
        }
    }

    /// Create a domain-specific error
    pub fn domain(error: E) -> Self {
        Self::Domain(error)
    }

    /// Create from HTTP response status and body
    ///
    /// Automatically categorizes the error based on status code:
    /// - 429 -> RateLimited
    /// - 500-599 -> ServerError
    /// - Other -> Api
    pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Self {
        match status {
            429 => Self::RateLimited { retry_after },
            500..=599 => Self::ServerError {
                status,
                message: body.to_string(),
            },
            _ => Self::Api {
                status,
                message: body.to_string(),
            },
        }
    }

    /// Check if this error is retryable
    ///
    /// Returns true for:
    /// - Rate limited errors
    /// - Server errors (5xx)
    /// - HTTP transport errors
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::ServerError { .. } | Self::Http(_)
        )
    }

    /// Get retry-after duration if available
    pub fn retry_after(&self) -> Option<Duration> {
        if let Self::RateLimited {
            retry_after: Some(secs),
        } = self
        {
            Some(Duration::from_secs(*secs))
        } else {
            None
        }
    }

    /// Get the HTTP status code if this is an API or server error
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            Self::ServerError { status, .. } => Some(*status),
            Self::RateLimited { .. } => Some(429),
            _ => None,
        }
    }
}

impl<E: std::error::Error> RetryableError for ApiError<E> {
    fn is_retryable(&self) -> bool {
        ApiError::is_retryable(self)
    }

    fn retry_after(&self) -> Option<Duration> {
        ApiError::retry_after(self)
    }
}

/// Result type alias for API operations
pub type ApiResult<T, E = NoDomainError> = std::result::Result<T, ApiError<E>>;

// ============================================================================
// Secret API Key Wrapper
// ============================================================================

/// A wrapper for API keys that redacts the value in Debug output.
///
/// This prevents accidental logging of API keys in debug output.
///
/// # Example
///
/// ```
/// use yldfi_common::api::SecretApiKey;
///
/// let key = SecretApiKey::new("sk-secret-key-12345");
/// let debug_str = format!("{:?}", key);
/// assert!(debug_str.contains("REDACTED"));
/// assert!(!debug_str.contains("sk-secret"));
/// assert_eq!(key.expose(), "sk-secret-key-12345");
/// ```
#[derive(Clone)]
pub struct SecretApiKey(String);

impl SecretApiKey {
    /// Create a new secret API key.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Expose the secret value.
    ///
    /// Use this when you need to include the key in an HTTP header.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Check if the key is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SecretApiKey").field(&"[REDACTED]").finish()
    }
}

impl From<String> for SecretApiKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretApiKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// API Configuration
// ============================================================================

/// Generic API configuration
///
/// Provides common configuration options for all API clients:
/// - Base URL (validated to be HTTPS in production)
/// - API key (optional, redacted in Debug)
/// - HTTP client settings (timeout, proxy, user-agent)
///
/// # Security
///
/// - API keys are wrapped in `SecretApiKey` to prevent accidental logging
/// - Use `validate()` to check that the base URL uses HTTPS
#[derive(Clone)]
pub struct ApiConfig {
    /// Base URL for the API
    pub base_url: String,
    /// API key for authentication (optional, redacted in Debug)
    pub api_key: Option<SecretApiKey>,
    /// HTTP client configuration
    pub http: HttpClientConfig,
}

impl fmt::Debug for ApiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key)
            .field("http", &self.http)
            .finish()
    }
}

impl ApiConfig {
    /// Create a new config with a base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            http: HttpClientConfig::default(),
        }
    }

    /// Create a new config with base URL and API key
    pub fn with_api_key(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: Some(SecretApiKey::new(api_key)),
            http: HttpClientConfig::default(),
        }
    }

    /// Set the API key
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(SecretApiKey::new(key));
        self
    }

    /// Set optional API key
    #[must_use]
    pub fn optional_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key.map(SecretApiKey::new);
        self
    }

    /// Set request timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http.timeout = timeout;
        self
    }

    /// Set request timeout in seconds
    #[must_use]
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.http.timeout = Duration::from_secs(secs);
        self
    }

    /// Set proxy URL
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

    /// Build an HTTP client from this configuration
    pub fn build_client(&self) -> Result<Client, HttpError> {
        crate::http::build_client(&self.http)
    }

    /// Validate the configuration for security.
    ///
    /// Returns an error if:
    /// - The base URL uses HTTP instead of HTTPS (security risk)
    /// - The base URL is malformed
    ///
    /// # Example
    ///
    /// ```
    /// use yldfi_common::api::ApiConfig;
    ///
    /// // HTTPS URLs are valid
    /// let config = ApiConfig::new("https://api.example.com");
    /// assert!(config.validate().is_ok());
    ///
    /// // HTTP URLs are rejected
    /// let config = ApiConfig::new("http://api.example.com");
    /// assert!(config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // Parse the URL
        let url = url::Url::parse(&self.base_url)
            .map_err(|e| ConfigValidationError::InvalidUrl(e.to_string()))?;

        // Check scheme
        match url.scheme() {
            "https" => Ok(()),
            "http" => {
                // Allow localhost for development
                if let Some(host) = url.host_str() {
                    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                        return Ok(());
                    }
                }
                Err(ConfigValidationError::InsecureScheme)
            }
            scheme => Err(ConfigValidationError::InvalidUrl(format!(
                "Unsupported URL scheme: {}",
                scheme
            ))),
        }
    }

    /// Check if the base URL uses HTTPS.
    #[must_use]
    pub fn is_https(&self) -> bool {
        self.base_url.starts_with("https://")
    }

    /// Get the exposed API key, if set.
    #[must_use]
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_ref().map(|k| k.expose())
    }
}

/// Configuration validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValidationError {
    /// The URL scheme is HTTP instead of HTTPS
    InsecureScheme,
    /// The URL is malformed
    InvalidUrl(String),
}

impl fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsecureScheme => write!(
                f,
                "Insecure URL scheme: use HTTPS instead of HTTP to protect API keys"
            ),
            Self::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
        }
    }
}

impl std::error::Error for ConfigValidationError {}

// ============================================================================
// Base Client
// ============================================================================

/// A base HTTP client that handles common request/response patterns.
///
/// This client provides reusable methods for making GET and POST requests
/// with proper error handling, reducing boilerplate across API crates.
///
/// # Example
///
/// ```
/// use yldfi_common::api::{ApiConfig, BaseClient};
///
/// // Create a client with configuration
/// let config = ApiConfig::new("https://api.example.com")
///     .api_key("your-api-key");
/// let client = BaseClient::new(config).unwrap();
///
/// // Build URLs
/// assert_eq!(client.url("/quote"), "https://api.example.com/quote");
///
/// // Access config
/// assert_eq!(client.config().get_api_key(), Some("your-api-key"));
/// ```
#[derive(Debug, Clone)]
pub struct BaseClient {
    http: Client,
    config: ApiConfig,
}

impl BaseClient {
    /// Create a new base client from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(config: ApiConfig) -> Result<Self, HttpError> {
        let http = config.build_client()?;
        Ok(Self { http, config })
    }

    /// Get the underlying HTTP client.
    #[must_use]
    pub fn http(&self) -> &Client {
        &self.http
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }

    /// Get the base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Build the full URL for a path.
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
        } else {
            format!("{}/{}", self.config.base_url.trim_end_matches('/'), path)
        }
    }

    /// Build default headers with API key (if present).
    ///
    /// Override this in your client to add custom headers.
    pub fn default_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add Authorization header if API key is set
        if let Some(key) = self.config.get_api_key() {
            if let Ok(value) = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(reqwest::header::AUTHORIZATION, value);
            }
        }

        headers
    }

    /// Make a GET request with query parameters.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The response type (must implement `DeserializeOwned`)
    /// * `E` - Domain-specific error type (default: `NoDomainError`)
    ///
    /// # Arguments
    ///
    /// * `path` - The API path (will be joined with base_url)
    /// * `params` - Query parameters as key-value pairs
    pub async fn get<T, E>(
        &self,
        path: &str,
        params: &[(&str, impl AsRef<str>)],
    ) -> Result<T, ApiError<E>>
    where
        T: serde::de::DeserializeOwned,
        E: std::error::Error,
    {
        self.get_with_headers(path, params, self.default_headers())
            .await
    }

    /// Make a GET request with custom headers.
    ///
    /// Use this when you need to add API-specific headers beyond the default
    /// Authorization header.
    pub async fn get_with_headers<T, E>(
        &self,
        path: &str,
        params: &[(&str, impl AsRef<str>)],
        headers: reqwest::header::HeaderMap,
    ) -> Result<T, ApiError<E>>
    where
        T: serde::de::DeserializeOwned,
        E: std::error::Error,
    {
        let url = self.url(path);
        let query: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_ref())).collect();

        let response = self
            .http
            .get(&url)
            .headers(headers)
            .query(&query)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request with JSON body.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The response type
    /// * `B` - The request body type (must implement `Serialize`)
    /// * `E` - Domain-specific error type
    pub async fn post_json<T, B, E>(&self, path: &str, body: &B) -> Result<T, ApiError<E>>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
        E: std::error::Error,
    {
        self.post_json_with_headers(path, body, self.default_headers())
            .await
    }

    /// Make a POST request with JSON body and custom headers.
    pub async fn post_json_with_headers<T, B, E>(
        &self,
        path: &str,
        body: &B,
        headers: reqwest::header::HeaderMap,
    ) -> Result<T, ApiError<E>>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
        E: std::error::Error,
    {
        let url = self.url(path);

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request with form data.
    pub async fn post_form<T, E>(
        &self,
        path: &str,
        form: &[(&str, impl AsRef<str>)],
    ) -> Result<T, ApiError<E>>
    where
        T: serde::de::DeserializeOwned,
        E: std::error::Error,
    {
        let url = self.url(path);
        let form_data: Vec<(&str, &str)> = form.iter().map(|(k, v)| (*k, v.as_ref())).collect();

        let response = self
            .http
            .post(&url)
            .headers(self.default_headers())
            .form(&form_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle a response, extracting the body or converting to error.
    async fn handle_response<T, E>(&self, response: reqwest::Response) -> Result<T, ApiError<E>>
    where
        T: serde::de::DeserializeOwned,
        E: std::error::Error,
    {
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(handle_error_response(response).await)
        }
    }
}

// ============================================================================
// Response Handling Helper
// ============================================================================

/// Extract retry-after header value from a response
pub fn extract_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
}

/// Handle an HTTP response, converting errors appropriately
///
/// This helper extracts the retry-after header and creates the appropriate
/// error type based on the response status code.
pub async fn handle_error_response<E: std::error::Error>(
    response: reqwest::Response,
) -> ApiError<E> {
    let status = response.status().as_u16();
    let retry_after = extract_retry_after(response.headers());
    let body = response.text().await.unwrap_or_default();
    ApiError::from_response(status, &body, retry_after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let err: ApiError = ApiError::api(400, "Bad request");
        assert!(err.to_string().contains("400"));
        assert!(err.to_string().contains("Bad request"));
    }

    #[test]
    fn test_rate_limited_display() {
        let err: ApiError = ApiError::rate_limited(Some(60));
        assert!(err.to_string().contains("60"));
    }

    #[test]
    fn test_rate_limited_no_retry() {
        let err: ApiError = ApiError::rate_limited(None);
        assert!(err.to_string().contains("Rate limited"));
        assert!(!err.to_string().contains("retry"));
    }

    #[test]
    fn test_is_retryable() {
        let err: ApiError = ApiError::rate_limited(Some(10));
        assert!(err.is_retryable());
        let err: ApiError = ApiError::server_error(500, "error");
        assert!(err.is_retryable());
        let err: ApiError = ApiError::api(400, "bad request");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_from_response() {
        let err: ApiError = ApiError::from_response(429, "rate limited", Some(30));
        assert!(matches!(
            err,
            ApiError::RateLimited {
                retry_after: Some(30)
            }
        ));

        let err: ApiError = ApiError::from_response(503, "service unavailable", None);
        assert!(matches!(err, ApiError::ServerError { status: 503, .. }));

        let err: ApiError = ApiError::from_response(400, "bad request", None);
        assert!(matches!(err, ApiError::Api { status: 400, .. }));
    }

    #[test]
    fn test_retry_after() {
        let err: ApiError = ApiError::rate_limited(Some(30));
        assert_eq!(err.retry_after(), Some(Duration::from_secs(30)));

        let err: ApiError = ApiError::api(400, "bad");
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn test_status_code() {
        let err: ApiError = ApiError::api(400, "bad");
        assert_eq!(err.status_code(), Some(400));
        let err: ApiError = ApiError::server_error(503, "down");
        assert_eq!(err.status_code(), Some(503));
        let err: ApiError = ApiError::rate_limited(None);
        assert_eq!(err.status_code(), Some(429));
        let err: ApiError = ApiError::Json(serde_json::from_str::<()>("invalid").unwrap_err());
        assert_eq!(err.status_code(), None);
    }

    #[test]
    fn test_api_config() {
        let config = ApiConfig::new("https://api.example.com")
            .api_key("test-key")
            .with_timeout_secs(60)
            .proxy("http://proxy:8080");

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.get_api_key(), Some("test-key"));
        assert_eq!(config.http.timeout, Duration::from_secs(60));
        assert_eq!(config.http.proxy, Some("http://proxy:8080".to_string()));
    }

    #[test]
    fn test_api_config_build_client() {
        let config = ApiConfig::new("https://api.example.com");
        let client = config.build_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_secret_api_key_redacted() {
        let key = SecretApiKey::new("sk-secret-key-12345");
        let debug_output = format!("{:?}", key);
        assert!(debug_output.contains("REDACTED"));
        assert!(!debug_output.contains("sk-secret"));
        assert_eq!(key.expose(), "sk-secret-key-12345");
    }

    #[test]
    fn test_api_config_debug_redacts_key() {
        let config = ApiConfig::with_api_key("https://api.example.com", "super-secret-key");
        let debug_output = format!("{:?}", config);
        assert!(debug_output.contains("REDACTED"));
        assert!(!debug_output.contains("super-secret-key"));
    }

    #[test]
    fn test_config_validation_https() {
        // HTTPS is valid
        let config = ApiConfig::new("https://api.example.com");
        assert!(config.validate().is_ok());
        assert!(config.is_https());

        // HTTP is rejected
        let config = ApiConfig::new("http://api.example.com");
        assert!(config.validate().is_err());
        assert!(!config.is_https());
        assert_eq!(
            config.validate().unwrap_err(),
            ConfigValidationError::InsecureScheme
        );
    }

    #[test]
    fn test_config_validation_localhost() {
        // HTTP to localhost is allowed for development
        let config = ApiConfig::new("http://localhost:8080");
        assert!(config.validate().is_ok());

        let config = ApiConfig::new("http://127.0.0.1:8080");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let config = ApiConfig::new("not a url");
        let result = config.validate();
        assert!(matches!(result, Err(ConfigValidationError::InvalidUrl(_))));
    }

    // Test with domain-specific errors
    #[derive(Debug, thiserror::Error)]
    enum TestDomainError {
        #[error("No route found")]
        NoRouteFound,
        #[error("Insufficient liquidity")]
        InsufficientLiquidity,
    }

    #[test]
    fn test_domain_error() {
        let err: ApiError<TestDomainError> = ApiError::domain(TestDomainError::NoRouteFound);
        assert!(err.to_string().contains("No route found"));
        assert!(!err.is_retryable());
    }

    // BaseClient tests
    #[test]
    fn test_base_client_creation() {
        let config = ApiConfig::new("https://api.example.com");
        let client = BaseClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_base_client_url_building() {
        let config = ApiConfig::new("https://api.example.com");
        let client = BaseClient::new(config).unwrap();

        // With leading slash
        assert_eq!(client.url("/quote"), "https://api.example.com/quote");

        // Without leading slash
        assert_eq!(client.url("quote"), "https://api.example.com/quote");

        // With path
        assert_eq!(
            client.url("/v1/swap/quote"),
            "https://api.example.com/v1/swap/quote"
        );
    }

    #[test]
    fn test_base_client_url_building_trailing_slash() {
        // Base URL with trailing slash
        let config = ApiConfig::new("https://api.example.com/");
        let client = BaseClient::new(config).unwrap();

        assert_eq!(client.url("/quote"), "https://api.example.com/quote");
        assert_eq!(client.url("quote"), "https://api.example.com/quote");
    }

    #[test]
    fn test_base_client_default_headers_no_key() {
        let config = ApiConfig::new("https://api.example.com");
        let client = BaseClient::new(config).unwrap();
        let headers = client.default_headers();

        // No Authorization header without API key
        assert!(!headers.contains_key(reqwest::header::AUTHORIZATION));
    }

    #[test]
    fn test_base_client_default_headers_with_key() {
        let config = ApiConfig::new("https://api.example.com").api_key("test-key");
        let client = BaseClient::new(config).unwrap();
        let headers = client.default_headers();

        // Authorization header present with Bearer token
        assert!(headers.contains_key(reqwest::header::AUTHORIZATION));
        assert_eq!(
            headers.get(reqwest::header::AUTHORIZATION).unwrap(),
            "Bearer test-key"
        );
    }

    #[test]
    fn test_base_client_accessors() {
        let config = ApiConfig::new("https://api.example.com").api_key("my-key");
        let client = BaseClient::new(config).unwrap();

        assert_eq!(client.base_url(), "https://api.example.com");
        assert_eq!(client.config().get_api_key(), Some("my-key"));
    }
}
