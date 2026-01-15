//! Moralis API client

use crate::block::BlockApi;
use crate::defi::DefiApi;
use crate::discovery::DiscoveryApi;
use crate::entities::EntitiesApi;
use crate::error::{Error, Result};
use crate::market::MarketApi;
use crate::nft::NftApi;
use crate::resolve::ResolveApi;
use crate::token::TokenApi;
use crate::transaction::TransactionApi;
use crate::wallet::WalletApi;
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;

const BASE_URL: &str = "https://deep-index.moralis.io/api/v2.2";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the Moralis client
#[derive(Debug, Clone)]
pub struct Config {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
}

impl Config {
    /// Create a new configuration with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: BASE_URL.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set a custom base URL
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the request timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Client for the Moralis Web3 API
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    api_key: String,
    base_url: String,
}

impl Client {
    /// Create a new client with the given API key
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::new(api_key))
    }

    /// Create a new client from environment variable MORALIS_API_KEY
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MORALIS_API_KEY").map_err(|_| Error::MissingApiKey)?;
        Self::new(api_key)
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }

        let http = HttpClient::builder().timeout(config.timeout).build()?;

        Ok(Self {
            http,
            api_key: config.api_key,
            base_url: config.base_url,
        })
    }

    /// Make a GET request to the API
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::api(status, message));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Make a GET request with query parameters
    pub(crate) async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(query)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::api(status, message));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Access the Wallet API
    pub fn wallet(&self) -> WalletApi<'_> {
        WalletApi::new(self)
    }

    /// Access the Token API
    pub fn token(&self) -> TokenApi<'_> {
        TokenApi::new(self)
    }

    /// Access the NFT API
    pub fn nft(&self) -> NftApi<'_> {
        NftApi::new(self)
    }

    /// Access the Block API
    pub fn block(&self) -> BlockApi<'_> {
        BlockApi::new(self)
    }

    /// Access the Transaction API
    pub fn transaction(&self) -> TransactionApi<'_> {
        TransactionApi::new(self)
    }

    /// Access the DeFi API
    pub fn defi(&self) -> DefiApi<'_> {
        DefiApi::new(self)
    }

    /// Access the Resolve API (ENS, domains)
    pub fn resolve(&self) -> ResolveApi<'_> {
        ResolveApi::new(self)
    }

    /// Access the Market Data API
    pub fn market(&self) -> MarketApi<'_> {
        MarketApi::new(self)
    }

    /// Access the Discovery API
    pub fn discovery(&self) -> DiscoveryApi<'_> {
        DiscoveryApi::new(self)
    }

    /// Access the Entities API
    pub fn entities(&self) -> EntitiesApi<'_> {
        EntitiesApi::new(self)
    }
}
