//! HTTP client utilities with proxy support
//!
//! This module provides shared HTTP client configuration and building
//! functionality that can be used by all API crates in yldfi-rs.
//!
//! # Example
//!
//! ```no_run
//! use yldfi_common::http::{HttpClientConfig, build_client};
//!
//! let config = HttpClientConfig::default()
//!     .with_proxy("http://user:pass@proxy:8080")
//!     .with_timeout_secs(60);
//!
//! let client = build_client(&config).unwrap();
//! ```

use reqwest::Client;
use std::time::Duration;
use thiserror::Error;
use url::Url;

/// Default User-Agent to avoid Cloudflare blocks.
///
/// Many API providers (especially those behind Cloudflare) block requests with
/// default library User-Agent strings. This mimics a real browser to avoid
/// 403 Forbidden responses.
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Default request timeout (30 seconds).
///
/// This balances responsiveness with reliability:
/// - Short enough to fail fast on unresponsive servers
/// - Long enough for slow API responses (e.g., archive node queries, large responses)
/// - Matches common API gateway timeouts (AWS API Gateway: 29s, Cloudflare: 30s)
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Redact credentials from a proxy URL for safe logging (MED-001 fix)
///
/// Replaces username and password with `[REDACTED]` if present.
fn redact_proxy_url(url: &str) -> String {
    if let Ok(mut parsed) = Url::parse(url) {
        if !parsed.username().is_empty() || parsed.password().is_some() {
            let _ = parsed.set_username("[REDACTED]");
            let _ = parsed.set_password(Some("[REDACTED]"));
        }
        parsed.to_string()
    } else {
        "[invalid proxy URL]".to_string()
    }
}

/// HTTP client configuration errors
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Invalid proxy URL: {0}")]
    InvalidProxy(String),

    #[error("Failed to build HTTP client: {0}")]
    BuildError(String),
}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        HttpError::BuildError(e.to_string())
    }
}

/// Default connection pool idle timeout (90 seconds).
///
/// Connections are kept alive for reuse to avoid TCP/TLS handshake overhead.
/// 90 seconds is chosen because:
/// - Most HTTP keep-alive defaults are 60-120 seconds
/// - Long enough to benefit from connection reuse in burst operations
/// - Short enough to release resources reasonably quickly after idle periods
/// - Matches common load balancer idle timeouts (AWS ALB: 60s, nginx: 75s)
pub const DEFAULT_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);

/// Default maximum idle connections per host (10).
///
/// This limits memory usage while allowing good parallelism:
/// - 10 connections supports typical CLI concurrency (1-10 parallel requests)
/// - Higher values waste memory for infrequent hosts
/// - Lower values cause connection churn under parallel load
/// - reqwest's default is 100, which is excessive for CLI tools
///
/// ## Tuning Guidelines (MED-004)
///
/// **Increase `DEFAULT_POOL_MAX_IDLE_PER_HOST` if:**
/// - Running many parallel CLI commands to the same API
/// - Seeing connection setup latency in traces
/// - Batch operations with high parallelism (e.g., 20+ concurrent requests)
///
/// **Decrease if:**
/// - Running on memory-constrained systems
/// - Targeting many different API hosts (memory per host adds up)
/// - Simple sequential operations with low parallelism
///
/// **Adjust `DEFAULT_POOL_IDLE_TIMEOUT` based on target API's keep-alive:**
/// - Lower (30-60s) for APIs with short keep-alive timeouts
/// - Higher (120s+) for APIs with long timeouts or when running long batch operations
/// - Match to the minimum keep-alive of your target APIs to avoid broken connection errors
pub const DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 10;

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Request timeout
    pub timeout: Duration,
    /// User-Agent header
    pub user_agent: String,
    /// Optional proxy URL
    ///
    /// Supports HTTP/HTTPS proxies with optional authentication:
    /// - `http://proxy.example.com:8080`
    /// - `http://user:password@proxy.example.com:8080`
    ///
    /// **Security Note**: Embedded credentials in the URL are supported.
    /// Avoid logging proxy URLs directly as they may contain secrets.
    pub proxy: Option<String>,
    /// Connection pool idle timeout
    pub pool_idle_timeout: Duration,
    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            proxy: None,
            pool_idle_timeout: DEFAULT_POOL_IDLE_TIMEOUT,
            pool_max_idle_per_host: DEFAULT_POOL_MAX_IDLE_PER_HOST,
        }
    }
}

impl HttpClientConfig {
    /// Create a new config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set request timeout in seconds
    #[must_use]
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Set User-Agent header
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set proxy URL
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

    /// Set connection pool idle timeout
    #[must_use]
    pub fn with_pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }

    /// Set maximum idle connections per host
    #[must_use]
    pub fn with_pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }
}

/// Build a reqwest Client with the given configuration
pub fn build_client(config: &HttpClientConfig) -> Result<Client, HttpError> {
    let mut builder = Client::builder()
        .timeout(config.timeout)
        .user_agent(&config.user_agent)
        .pool_idle_timeout(config.pool_idle_timeout)
        .pool_max_idle_per_host(config.pool_max_idle_per_host);

    if let Some(ref proxy_url) = config.proxy {
        let proxy = reqwest::Proxy::all(proxy_url)
            // MED-001 fix: Redact credentials from error message
            .map_err(|e| HttpError::InvalidProxy(format!("{}: {}", redact_proxy_url(proxy_url), e)))?;
        builder = builder.proxy(proxy);
    }

    // MED-003 fix: Provide detailed context on build failure
    builder.build().map_err(|e| {
        HttpError::BuildError(format!(
            "Failed to build HTTP client (timeout: {:?}, pool_idle: {:?}, pool_max_idle: {}, proxy: {}): {}",
            config.timeout,
            config.pool_idle_timeout,
            config.pool_max_idle_per_host,
            config.proxy.as_ref().map(|p| redact_proxy_url(p)).unwrap_or_else(|| "none".to_string()),
            e
        ))
    })
}

/// Build a reqwest Client with default configuration
pub fn build_default_client() -> Result<Client, HttpError> {
    build_client(&HttpClientConfig::default())
}

/// Build a reqwest Client with just a proxy URL
pub fn build_client_with_proxy(proxy: Option<&str>) -> Result<Client, HttpError> {
    let config = HttpClientConfig::default().with_optional_proxy(proxy.map(String::from));
    build_client(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
        assert_eq!(config.user_agent, DEFAULT_USER_AGENT);
        assert!(config.proxy.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = HttpClientConfig::new()
            .with_timeout_secs(60)
            .with_user_agent("CustomAgent/1.0")
            .with_proxy("http://proxy:8080");

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.user_agent, "CustomAgent/1.0");
        assert_eq!(config.proxy, Some("http://proxy:8080".to_string()));
    }

    #[test]
    fn test_build_default_client() {
        let client = build_default_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_client_with_config() {
        let config = HttpClientConfig::new().with_timeout_secs(45);
        let client = build_client(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_proxy_url_format() {
        // Valid proxy URLs work
        let config = HttpClientConfig::new().with_proxy("http://proxy.example.com:8080");
        let result = build_client(&config);
        assert!(result.is_ok());

        // Note: reqwest is lenient with proxy URL formats - most strings are accepted
        // and errors only occur when the proxy is actually used for a connection.
    }
}
