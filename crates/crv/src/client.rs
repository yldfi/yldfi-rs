//! HTTP client for the Curve API

use crate::error::Result;
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;
use yldfi_common::api::handle_error_response;
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

        if !response.status().is_success() {
            return Err(handle_error_response(response).await);
        }

        let data = response.json().await?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn client_returning(response: ResponseTemplate) -> (MockServer, Client) {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(response)
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().with_base_url(server.uri())).unwrap();
        (server, client)
    }

    #[tokio::test]
    async fn server_error_is_retryable() {
        let (_server, client) = client_returning(ResponseTemplate::new(503)).await;
        let err = client.get::<serde_json::Value>("/x").await.unwrap_err();
        assert!(err.is_retryable(), "{err}");
        assert_eq!(err.status_code(), Some(503));
    }

    #[tokio::test]
    async fn retry_after_is_capped() {
        let (_server, client) =
            client_returning(ResponseTemplate::new(429).insert_header("retry-after", "999999"))
                .await;
        let err = client.get::<serde_json::Value>("/x").await.unwrap_err();
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), Some(Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn error_body_is_sanitized() {
        let (_server, client) =
            client_returning(ResponseTemplate::new(400).set_body_string("token=hunter2")).await;
        let err = client.get::<serde_json::Value>("/x").await.unwrap_err();
        assert!(!err.to_string().contains("hunter2"), "{err}");
    }
}
