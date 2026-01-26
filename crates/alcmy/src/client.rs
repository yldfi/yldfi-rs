//! HTTP client for the Alchemy API
//!
//! This client uses common utilities from `yldfi-common` for HTTP operations.

use crate::error::{self, Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use yldfi_common::api::{extract_retry_after, ApiConfig, SecretApiKey};

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    // Ethereum
    EthMainnet,
    EthSepolia,
    EthHolesky,
    // Polygon
    PolygonMainnet,
    PolygonAmoy,
    // Arbitrum
    ArbitrumMainnet,
    ArbitrumSepolia,
    // Optimism
    OptMainnet,
    OptSepolia,
    // Base
    BaseMainnet,
    BaseSepolia,
    // zkSync
    ZksyncMainnet,
    ZksyncSepolia,
    // Solana
    SolanaMainnet,
    SolanaDevnet,
    // Other L2s
    LineaMainnet,
    ScrollMainnet,
    BlastMainnet,
    MantleMainnet,
    ZoraMainnet,
    WorldchainMainnet,
    ShapeMainnet,
    PolygonZkevmMainnet,
    Bnb,
    Avalanche,
    Fantom,
    Gnosis,
}

impl Network {
    /// Get the network slug used in URLs
    pub fn slug(&self) -> &'static str {
        match self {
            // Ethereum
            Network::EthMainnet => "eth-mainnet",
            Network::EthSepolia => "eth-sepolia",
            Network::EthHolesky => "eth-holesky",
            // Polygon
            Network::PolygonMainnet => "polygon-mainnet",
            Network::PolygonAmoy => "polygon-amoy",
            // Arbitrum
            Network::ArbitrumMainnet => "arb-mainnet",
            Network::ArbitrumSepolia => "arb-sepolia",
            // Optimism
            Network::OptMainnet => "opt-mainnet",
            Network::OptSepolia => "opt-sepolia",
            // Base
            Network::BaseMainnet => "base-mainnet",
            Network::BaseSepolia => "base-sepolia",
            // zkSync
            Network::ZksyncMainnet => "zksync-mainnet",
            Network::ZksyncSepolia => "zksync-sepolia",
            // Solana
            Network::SolanaMainnet => "solana-mainnet",
            Network::SolanaDevnet => "solana-devnet",
            // Other L2s
            Network::LineaMainnet => "linea-mainnet",
            Network::ScrollMainnet => "scroll-mainnet",
            Network::BlastMainnet => "blast-mainnet",
            Network::MantleMainnet => "mantle-mainnet",
            Network::ZoraMainnet => "zora-mainnet",
            Network::WorldchainMainnet => "worldchain-mainnet",
            Network::ShapeMainnet => "shape-mainnet",
            Network::PolygonZkevmMainnet => "polygonzkevm-mainnet",
            Network::Bnb => "bnb-mainnet",
            Network::Avalanche => "avax-mainnet",
            Network::Fantom => "fantom-mainnet",
            Network::Gnosis => "gnosis-mainnet",
        }
    }

    /// Get the network name for the Data/Prices API
    pub fn data_api_name(&self) -> &'static str {
        match self {
            Network::EthMainnet => "eth-mainnet",
            Network::PolygonMainnet => "polygon-mainnet",
            Network::ArbitrumMainnet => "arb-mainnet",
            Network::OptMainnet => "opt-mainnet",
            Network::BaseMainnet => "base-mainnet",
            Network::ZksyncMainnet => "zksync-mainnet",
            Network::SolanaMainnet => "solana-mainnet",
            Network::LineaMainnet => "linea-mainnet",
            Network::ScrollMainnet => "scroll-mainnet",
            Network::BlastMainnet => "blast-mainnet",
            Network::MantleMainnet => "mantle-mainnet",
            Network::ZoraMainnet => "zora-mainnet",
            Network::WorldchainMainnet => "worldchain-mainnet",
            Network::ShapeMainnet => "shape-mainnet",
            Network::PolygonZkevmMainnet => "polygonzkevm-mainnet",
            Network::Bnb => "bnb-mainnet",
            Network::Avalanche => "avax-mainnet",
            Network::Fantom => "fantom-mainnet",
            Network::Gnosis => "gnosis-mainnet",
            // Testnets
            Network::EthSepolia => "eth-sepolia",
            Network::EthHolesky => "eth-holesky",
            Network::PolygonAmoy => "polygon-amoy",
            Network::ArbitrumSepolia => "arb-sepolia",
            Network::OptSepolia => "opt-sepolia",
            Network::BaseSepolia => "base-sepolia",
            Network::ZksyncSepolia => "zksync-sepolia",
            Network::SolanaDevnet => "solana-devnet",
        }
    }
}

/// Configuration for the Alchemy API client
///
/// Built on top of [`ApiConfig`] from `yldfi-common` for consistent
/// configuration patterns across all API clients.
#[derive(Clone)]
pub struct Config {
    /// API key for authentication
    pub api_key: SecretApiKey,
    /// Target blockchain network
    pub network: Network,
    /// Inner API configuration
    inner: ApiConfig,
}

impl Config {
    /// Create a new configuration
    pub fn new(api_key: impl Into<String>, network: Network) -> Self {
        // Use a placeholder base URL since Alchemy uses dynamic URLs per endpoint
        Self {
            api_key: SecretApiKey::new(api_key),
            network,
            inner: ApiConfig::new("https://api.g.alchemy.com"),
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

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("api_key", &"[REDACTED]")
            .field("network", &self.network)
            .field("inner", &self.inner)
            .finish()
    }
}

/// Alchemy API client
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: SecretApiKey,
    network: Network,
}

impl Client {
    /// Create a new Alchemy client
    ///
    /// # Arguments
    /// * `api_key` - Alchemy API key
    /// * `network` - Target blockchain network
    ///
    /// # Errors
    /// Returns an error if the HTTP client fails to build
    pub fn new(api_key: impl Into<String>, network: Network) -> Result<Self> {
        Self::with_config(Config::new(api_key, network))
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = config.inner.build_client()?;

        Ok(Self {
            http,
            api_key: config.api_key,
            network: config.network,
        })
    }

    /// Create a new client from environment variable
    ///
    /// Uses `ALCHEMY_API_KEY` environment variable
    pub fn from_env(network: Network) -> Result<Self> {
        let api_key = std::env::var("ALCHEMY_API_KEY").map_err(|_| error::invalid_api_key())?;
        Self::new(api_key, network)
    }

    /// Get the API key (exposed for URL construction)
    ///
    /// # Warning
    /// This exposes the secret API key. Only use when necessary (e.g., URL construction).
    pub fn api_key(&self) -> &str {
        self.api_key.expose()
    }

    /// Get the current network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Get the HTTP client
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Get the base URL for JSON-RPC requests
    pub fn rpc_url(&self) -> String {
        format!(
            "https://{}.g.alchemy.com/v2/{}",
            self.network.slug(),
            self.api_key.expose()
        )
    }

    /// Get the base URL for NFT API requests
    pub fn nft_url(&self) -> String {
        format!(
            "https://{}.g.alchemy.com/nft/v3/{}",
            self.network.slug(),
            self.api_key.expose()
        )
    }

    /// Get the base URL for Prices API requests
    pub fn prices_url(&self) -> String {
        format!(
            "https://api.g.alchemy.com/prices/v1/{}",
            self.api_key.expose()
        )
    }

    /// Get the base URL for Data/Portfolio API requests
    pub fn data_url(&self) -> String {
        format!(
            "https://api.g.alchemy.com/data/v1/{}",
            self.api_key.expose()
        )
    }

    /// Make a JSON-RPC request
    pub async fn rpc<P, R>(&self, method: &str, params: P) -> Result<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self.http.post(self.rpc_url()).json(&request).send().await?;

        if response.status() == 429 {
            let retry_after = extract_retry_after(response.headers());
            return Err(Error::rate_limited(retry_after));
        }

        let result: serde_json::Value = response.json().await?;

        if let Some(error) = result.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            return Err(error::rpc(code, message));
        }

        let result = result
            .get("result")
            .ok_or_else(|| error::rpc(-1, "No result in response"))?
            .clone();

        Ok(serde_json::from_value(result)?)
    }

    /// Make a GET request to the NFT API
    pub async fn nft_get<R>(&self, path: &str, query: &[(&str, &str)]) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", self.nft_url(), path);
        let response = self.http.get(&url).query(query).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the NFT API
    pub async fn nft_post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", self.nft_url(), path);
        let response = self.http.post(&url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Make a GET request to the Prices API
    pub async fn prices_get<R>(&self, path: &str, query: &[(&str, &str)]) -> Result<R>
    where
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", self.prices_url(), path);
        let response = self.http.get(&url).query(query).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the Prices API
    pub async fn prices_post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", self.prices_url(), path);
        let response = self.http.post(&url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the Data/Portfolio API
    pub async fn data_post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", self.data_url(), path);
        let response = self.http.post(&url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Handle API response using common utilities
    async fn handle_response<R>(&self, response: reqwest::Response) -> Result<R>
    where
        R: DeserializeOwned,
    {
        if response.status() == 429 {
            let retry_after = extract_retry_after(response.headers());
            return Err(Error::rate_limited(retry_after));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
