//! HTTP client for the GoPlus Security API

use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use url::Url;
use yldfi_common::api::ApiConfig;

use crate::error::{token_not_found, Error, Result};
use crate::types::{
    AddressSecurity, ApprovalSecurity, NftSecurity, Response, TokenSecurity, TokenSecurityResponse,
};

/// Base URL for GoPlus API
pub const BASE_URL: &str = "https://api.gopluslabs.io/api/v1";

/// Rate limit information from API response headers
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    /// Requests remaining in current window
    pub remaining: Option<u32>,
    /// Total requests allowed per window
    pub limit: Option<u32>,
    /// Seconds until rate limit resets
    pub reset_in_secs: Option<u64>,
    /// Timestamp when this info was captured
    pub captured_at: u64,
}

impl RateLimitInfo {
    /// Check if we're close to hitting the rate limit (< 10% remaining)
    pub fn is_near_limit(&self) -> bool {
        match (self.remaining, self.limit) {
            (Some(remaining), Some(limit)) if limit > 0 => {
                (remaining as f64 / limit as f64) < 0.1
            }
            _ => false,
        }
    }

    /// Check if rate limit is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.remaining == Some(0)
    }
}

/// Cached access token with expiry
#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: u64,
}

/// API credentials for authenticated requests
#[derive(Debug, Clone)]
pub struct Credentials {
    /// App key from GoPlus
    pub app_key: String,
    /// App secret from GoPlus (kept secret)
    pub app_secret: SecretString,
}

impl Credentials {
    /// Create new credentials
    pub fn new(app_key: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_key: app_key.into(),
            app_secret: SecretString::new(app_secret.into().into_boxed_str()),
        }
    }

    /// Generate signature for access token request
    fn sign(&self, timestamp: u64) -> String {
        let data = format!(
            "{}{}{}",
            self.app_key,
            timestamp,
            self.app_secret.expose_secret()
        );
        let mut hasher = Sha1::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Configuration for the GoPlus API client
#[derive(Debug, Clone)]
pub struct Config {
    /// API credentials (optional for limited access)
    pub credentials: Option<Credentials>,
    /// Inner API configuration
    inner: ApiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new config without authentication (limited access)
    pub fn new() -> Self {
        Self {
            credentials: None,
            inner: ApiConfig::new(BASE_URL),
        }
    }

    /// Create a config with authentication
    pub fn with_credentials(app_key: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            credentials: Some(Credentials::new(app_key, app_secret)),
            inner: ApiConfig::new(BASE_URL),
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

    /// Set a custom base URL (for testing)
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.inner.base_url = base_url.into();
        self
    }
}

/// GoPlus Security API client
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    base_url: Url,
    credentials: Option<Credentials>,
    /// Cached access token (shared across clones)
    cached_token: Arc<RwLock<Option<CachedToken>>>,
    /// Last observed rate limit info (shared across clones)
    rate_limit: Arc<RwLock<Option<RateLimitInfo>>>,
}

impl Client {
    /// Create a new client without authentication (limited access)
    pub fn new() -> Result<Self> {
        Self::with_config(Config::new())
    }

    /// Create a client with authentication
    pub fn with_credentials(
        app_key: impl Into<String>,
        app_secret: impl Into<String>,
    ) -> Result<Self> {
        Self::with_config(Config::with_credentials(app_key, app_secret))
    }

    /// Create a client from environment variables
    /// Uses `GOPLUS_APP_KEY` and `GOPLUS_APP_SECRET`
    pub fn from_env() -> Result<Self> {
        let app_key = std::env::var("GOPLUS_APP_KEY").ok();
        let app_secret = std::env::var("GOPLUS_APP_SECRET").ok();

        match (app_key, app_secret) {
            (Some(key), Some(secret)) => Self::with_credentials(key, secret),
            _ => Self::new(),
        }
    }

    /// Create a client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = config.inner.build_client()?;
        let base_url = Url::parse(&config.inner.base_url)?;

        Ok(Self {
            http,
            base_url,
            credentials: config.credentials,
            cached_token: Arc::new(RwLock::new(None)),
            rate_limit: Arc::new(RwLock::new(None)),
        })
    }

    /// Check if client has authentication configured
    pub fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    /// Build a full URL from a path (handles trailing slash issue)
    fn build_url(&self, path: &str) -> String {
        let base = self.base_url.as_str().trim_end_matches('/');
        format!("{}{}", base, path)
    }

    /// Clear the cached access token
    ///
    /// Call this if you receive authentication errors to force a token refresh
    pub async fn clear_token_cache(&self) {
        let mut cached = self.cached_token.write().await;
        *cached = None;
    }

    /// Get the last observed rate limit information
    ///
    /// Returns None if no API calls have been made yet
    pub async fn rate_limit_info(&self) -> Option<RateLimitInfo> {
        self.rate_limit.read().await.clone()
    }

    /// Check if we're near the rate limit
    pub async fn is_near_rate_limit(&self) -> bool {
        self.rate_limit
            .read()
            .await
            .as_ref()
            .is_some_and(|r| r.is_near_limit())
    }

    /// Get or refresh access token
    async fn get_access_token(&self) -> Result<Option<String>> {
        let Some(creds) = &self.credentials else {
            return Ok(None);
        };

        // Check cached token
        {
            let cached = self.cached_token.read().await;
            if let Some(token) = &*cached {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // Refresh 60 seconds before expiry
                if token.expires_at > now + 60 {
                    return Ok(Some(token.token.clone()));
                }
            }
        }

        // Request new token
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let sign = creds.sign(timestamp);

        let url = self.build_url("/token");
        let response = self
            .http
            .post(&url)
            .json(&serde_json::json!({
                "app_key": creds.app_key,
                "time": timestamp,
                "sign": sign,
            }))
            .send()
            .await?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(status, body));
        }

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            result: Option<TokenData>,
        }
        #[derive(serde::Deserialize)]
        struct TokenData {
            access_token: String,
            expires_in: u64,
        }

        let body: TokenResponse = response.json().await?;
        let data = body.result.ok_or_else(|| Error::api(401, "No token in response"))?;

        // Cache the token
        {
            let mut cached = self.cached_token.write().await;
            *cached = Some(CachedToken {
                token: data.access_token.clone(),
                expires_at: timestamp + data.expires_in,
            });
        }

        Ok(Some(data.access_token))
    }

    /// Extract rate limit info from response headers
    fn extract_rate_limit(headers: &reqwest::header::HeaderMap) -> Option<RateLimitInfo> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // GoPlus uses standard rate limit headers
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        let reset_in_secs = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(|reset_at| reset_at.saturating_sub(now));

        // Only return if we got at least one header
        if remaining.is_some() || limit.is_some() || reset_in_secs.is_some() {
            Some(RateLimitInfo {
                remaining,
                limit,
                reset_in_secs,
                captured_at: now,
            })
        } else {
            None
        }
    }

    /// Make a GET request with optional authentication and automatic retry on 401
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.get_inner(path, true).await
    }

    /// Internal GET implementation
    async fn get_inner<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        allow_retry: bool,
    ) -> Result<T> {
        let url = self.build_url(path);
        let mut req = self.http.get(&url);

        // Add auth header if we have credentials
        if let Ok(Some(token)) = self.get_access_token().await {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        // Extract and store rate limit info
        if let Some(rate_info) = Self::extract_rate_limit(response.headers()) {
            let mut rate_limit = self.rate_limit.write().await;
            *rate_limit = Some(rate_info);
        }

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();

            // On 401 Unauthorized, clear token cache and retry once
            if status == 401 && allow_retry && self.credentials.is_some() {
                self.clear_token_cache().await;
                // Non-recursive retry: just make a fresh request
                return self.get_fresh(path).await;
            }

            return Err(Error::api(status, body));
        }

        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|e| Error::api(status, format!("Parse error: {e}")))
    }

    /// Make a fresh GET request (no retry on failure)
    async fn get_fresh<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path);
        let mut req = self.http.get(&url);

        // Get fresh token after cache clear
        if let Ok(Some(token)) = self.get_access_token().await {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await?;
        let status = response.status().as_u16();

        // Extract and store rate limit info
        if let Some(rate_info) = Self::extract_rate_limit(response.headers()) {
            let mut rate_limit = self.rate_limit.write().await;
            *rate_limit = Some(rate_info);
        }

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(status, body));
        }

        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|e| Error::api(status, format!("Parse error: {e}")))
    }

    // ==================== Token Security ====================

    /// Get token security information
    ///
    /// # Arguments
    /// * `chain_id` - The chain ID (1 for Ethereum, 56 for BSC, etc.)
    /// * `address` - The token contract address
    pub async fn token_security(&self, chain_id: u64, address: &str) -> Result<TokenSecurity> {
        let address = address.to_lowercase();
        let path = format!("/token_security/{}?contract_addresses={}", chain_id, address);

        let body: Response<TokenSecurityResponse> = self.get(&path).await?;

        if !body.is_success() {
            return Err(Error::api(400, body.message));
        }

        body.result
            .and_then(|map| map.get(&address).cloned())
            .ok_or_else(|| token_not_found(&address))
    }

    /// Get token security for multiple addresses (requires authentication for >1 token)
    ///
    /// # Arguments
    /// * `chain_id` - The chain ID
    /// * `addresses` - List of token contract addresses
    pub async fn token_security_batch(
        &self,
        chain_id: u64,
        addresses: &[&str],
    ) -> Result<TokenSecurityResponse> {
        if addresses.is_empty() {
            return Ok(TokenSecurityResponse::new());
        }

        let addresses_str = addresses
            .iter()
            .map(|a| a.to_lowercase())
            .collect::<Vec<_>>()
            .join(",");

        let path = format!(
            "/token_security/{}?contract_addresses={}",
            chain_id, addresses_str
        );

        let body: Response<TokenSecurityResponse> = self.get(&path).await?;

        if !body.is_success() {
            return Err(Error::api(400, body.message));
        }

        Ok(body.result.unwrap_or_default())
    }

    // ==================== Address Security ====================

    /// Check if an address is malicious
    ///
    /// # Arguments
    /// * `chain_id` - The chain ID (1=Ethereum, 56=BSC, etc.)
    /// * `address` - The address to check
    pub async fn address_security(&self, chain_id: u64, address: &str) -> Result<AddressSecurity> {
        let address = address.to_lowercase();
        let path = format!("/address_security/{}?chain_id={}", address, chain_id);

        let body: Response<AddressSecurity> = self.get(&path).await?;

        if !body.is_success() {
            return Err(Error::api(400, body.message));
        }

        body.result.ok_or_else(|| token_not_found(&address))
    }

    // ==================== NFT Security ====================

    /// Get NFT collection security information
    ///
    /// # Arguments
    /// * `chain_id` - The chain ID
    /// * `address` - The NFT contract address
    pub async fn nft_security(&self, chain_id: u64, address: &str) -> Result<NftSecurity> {
        let address = address.to_lowercase();
        let path = format!("/nft_security/{}?contract_addresses={}", chain_id, address);

        let body: Response<NftSecurity> = self.get(&path).await?;

        if !body.is_success() {
            return Err(Error::api(400, body.message));
        }

        body.result.ok_or_else(|| token_not_found(&address))
    }

    // ==================== Approval Security ====================

    /// Get approval security information for a token
    ///
    /// # Arguments
    /// * `chain_id` - The chain ID
    /// * `address` - The contract address to check approvals for
    pub async fn approval_security(&self, chain_id: u64, address: &str) -> Result<ApprovalSecurity> {
        let address = address.to_lowercase();
        let path = format!(
            "/approval_security/{}?contract_addresses={}",
            chain_id, address
        );

        let body: Response<ApprovalSecurity> = self.get(&path).await?;

        if !body.is_success() {
            return Err(Error::api(400, body.message));
        }

        body.result.ok_or_else(|| token_not_found(&address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let client = Client::with_config(
            Config::new().with_base_url("http://127.0.0.1:63762"),
        )
        .unwrap();
        let path = "/token_security/1?contract_addresses=0xtest";
        let url = client.build_url(path);
        println!("URL: {}", url);
        assert_eq!(url, "http://127.0.0.1:63762/token_security/1?contract_addresses=0xtest");
        assert!(!url.contains("//token"), "URL has double slash: {}", url);
    }

    #[test]
    fn test_url_construction_with_trailing_slash() {
        let client = Client::with_config(
            Config::new().with_base_url("http://127.0.0.1:63762/"),
        )
        .unwrap();
        let url = client.build_url("/token_security/1");
        assert!(!url.contains("//token"), "URL has double slash: {}", url);
        assert_eq!(url, "http://127.0.0.1:63762/token_security/1");
    }
}
