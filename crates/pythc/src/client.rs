//! HTTP client for the Pyth Hermes API
//!
//! This client uses the common `BaseClient` from `yldfi-common` for HTTP operations,
//! providing consistent error handling, retry logic, and configuration patterns.

use std::borrow::Cow;
use std::time::Duration;
use url::Url;
use yldfi_common::api::{ApiConfig, BaseClient, SecretApiKey};
use yldfi_common::{with_retry, RetryConfig};

use crate::error::{DomainError, Error, Result};
use crate::types::{LatestPriceResponse, ParsedPriceFeed, PriceFeedId};

/// Maximum length of error body to include in error messages
const MAX_ERROR_BODY_LEN: usize = 500;

/// Environment variable read by [`Client::from_env`] and [`Config::from_env`].
pub const API_KEY_ENV_VAR: &str = "PYTH_API_KEY";

/// Where to obtain a Pyth API key (Pyth Terminal).
pub const API_KEY_URL: &str = "https://pythdata.app";

/// Base URLs for Pyth Hermes
///
/// Since the Pyth Core upgrade (2026-08-26), every Hermes endpoint requires an
/// API key sent as `Authorization: Bearer <key>`. Keys are issued via the Pyth
/// Terminal (<https://pythdata.app>).
pub mod base_urls {
    /// Production Hermes (Pyth Core upgraded endpoint).
    ///
    /// Routes and response shapes are identical to the legacy endpoint.
    pub const MAINNET: &str = "https://pyth.dourolabs.app/hermes";
    /// Legacy production Hermes host.
    ///
    /// Still served, but also requires an API key since the Pyth Core upgrade.
    /// Prefer [`MAINNET`].
    pub const LEGACY: &str = "https://hermes.pyth.network";
    /// Legacy beta/testnet Hermes host.
    ///
    /// Kept for backwards compatibility only. Pyth no longer documents a
    /// separate beta endpoint after the Pyth Core upgrade, so availability and
    /// authentication requirements of this host are unverified.
    pub const TESTNET: &str = "https://hermes-beta.pyth.network";
}

/// Configuration for the Pyth Hermes client
///
/// This is a thin wrapper around [`ApiConfig`] that provides Pyth-specific
/// defaults and convenience methods. The API key (if any) is stored as a
/// [`SecretApiKey`](yldfi_common::api::SecretApiKey) and is redacted from
/// `Debug` output.
///
/// # Example
///
/// ```
/// use pythc::Config;
///
/// let config = Config::mainnet().with_api_key("my-secret-key");
/// assert!(config.has_api_key());
/// assert!(!format!("{config:?}").contains("my-secret-key"));
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    inner: ApiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inner: ApiConfig::new(base_urls::MAINNET),
        }
    }
}

impl Config {
    /// Create a mainnet config
    #[must_use]
    pub fn mainnet() -> Self {
        Self::default()
    }

    /// Create a mainnet config, reading the API key from the `PYTH_API_KEY`
    /// environment variable if it is set and non-empty.
    #[must_use]
    pub fn from_env() -> Self {
        Self::default().with_optional_api_key(std::env::var(API_KEY_ENV_VAR).ok())
    }

    /// Create a config for the legacy `hermes-beta` host.
    ///
    /// **Legacy / unverified:** Pyth does not document a beta endpoint after the
    /// Pyth Core upgrade. See [`base_urls::TESTNET`].
    #[must_use]
    pub fn testnet() -> Self {
        Self {
            inner: ApiConfig::new(base_urls::TESTNET),
        }
    }

    /// Set a custom base URL
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.inner.base_url = url.into();
        self
    }

    /// Set the Pyth API key, sent as `Authorization: Bearer <key>`.
    ///
    /// Required by all Hermes endpoints since the Pyth Core upgrade.
    /// Surrounding whitespace is trimmed; an empty key clears the setting.
    #[must_use]
    pub fn with_api_key(self, api_key: impl Into<String>) -> Self {
        self.with_optional_api_key(Some(api_key.into()))
    }

    /// Set the Pyth API key if `Some` and non-empty, otherwise clear it.
    #[must_use]
    pub fn with_optional_api_key(mut self, api_key: Option<String>) -> Self {
        self.inner.api_key = api_key
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .map(SecretApiKey::new);
        self
    }

    /// Whether an API key is configured.
    #[must_use]
    pub fn has_api_key(&self) -> bool {
        self.inner.api_key.is_some()
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

    /// Validate the configuration.
    ///
    /// Checks that the base URL is valid and uses HTTPS (except for localhost).
    /// This is called automatically by `Client::with_config()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is malformed
    /// - The URL uses HTTP for non-localhost addresses
    ///
    /// # Example
    ///
    /// ```
    /// use pythc::Config;
    ///
    /// // HTTPS URLs are valid
    /// let config = Config::mainnet();
    /// assert!(config.validate().is_ok());
    ///
    /// // HTTP is rejected for non-localhost
    /// let config = Config::default().with_base_url("http://example.com");
    /// assert!(config.validate().is_err());
    ///
    /// // HTTP localhost is allowed for development
    /// let config = Config::default().with_base_url("http://localhost:8080");
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<()> {
        self.inner.validate().map_err(|e| match e {
            yldfi_common::api::ConfigValidationError::InsecureScheme => {
                crate::error::insecure_scheme()
            }
            yldfi_common::api::ConfigValidationError::InvalidUrl(msg) => {
                crate::error::invalid_url(msg)
            }
        })
    }
}

/// Pyth Hermes API client
///
/// Built on top of [`BaseClient`] from `yldfi-common` for consistent
/// HTTP handling across all API clients.
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
    retry_config: RetryConfig,
}

impl Client {
    /// Create a new mainnet client **without** an API key.
    ///
    /// Since the Pyth Core upgrade, Hermes rejects unauthenticated requests
    /// with `401 Unauthorized`. Prefer [`Client::with_api_key`] or
    /// [`Client::from_env`].
    pub fn new() -> Result<Self> {
        Self::with_config(Config::mainnet())
    }

    /// Create a mainnet client authenticated with the given Pyth API key.
    ///
    /// # Example
    ///
    /// ```
    /// let client = pythc::Client::with_api_key("my-pyth-api-key")?;
    /// assert!(client.has_api_key());
    /// # Ok::<(), pythc::Error>(())
    /// ```
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::mainnet().with_api_key(api_key))
    }

    /// Create a mainnet client using the `PYTH_API_KEY` environment variable
    /// (if set). See [`Config::from_env`].
    pub fn from_env() -> Result<Self> {
        Self::with_config(Config::from_env())
    }

    /// Create a client for the legacy `hermes-beta` host (unverified, see
    /// [`base_urls::TESTNET`]).
    pub fn testnet() -> Result<Self> {
        Self::with_config(Config::testnet())
    }

    /// Create a client with custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid (e.g., malformed URL
    /// or insecure HTTP scheme for non-localhost addresses).
    pub fn with_config(config: Config) -> Result<Self> {
        // Validate URL format and security (HTTPS required for non-localhost)
        config.validate()?;

        let base = BaseClient::new(config.inner)
            .map_err(yldfi_common::api::ApiError::<DomainError>::HttpBuild)?;

        Ok(Self {
            base,
            retry_config: RetryConfig::new(3)
                .with_initial_delay(Duration::from_millis(100))
                .with_max_delay(Duration::from_secs(5)),
        })
    }

    /// Whether this client sends an API key.
    #[must_use]
    pub fn has_api_key(&self) -> bool {
        self.base.config().api_key.is_some()
    }

    /// Get the latest price for one or more feed IDs
    ///
    /// # Arguments
    /// * `feed_ids` - Hex-encoded feed IDs (with or without 0x prefix)
    ///
    /// # Example
    /// ```ignore
    /// let feeds = client.get_latest_prices(&[
    ///     "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace", // ETH/USD
    /// ]).await?;
    /// ```
    pub async fn get_latest_prices(&self, feed_ids: &[&str]) -> Result<Vec<ParsedPriceFeed>> {
        if feed_ids.is_empty() {
            return Ok(vec![]);
        }

        // Validate all feed IDs before making the request
        for id in feed_ids {
            if !validate_feed_id(id) {
                return Err(crate::error::invalid_feed_id(*id));
            }
        }

        // Build URL with proper encoding
        let mut url = Url::parse(&self.base.url("/v2/updates/price/latest"))?;
        {
            let mut pairs = url.query_pairs_mut();
            for id in feed_ids {
                pairs.append_pair("ids[]", &normalize_feed_id(id));
            }
        }

        let response: LatestPriceResponse = self.get_url(&url).await?;

        Ok(response.parsed)
    }

    /// Get the latest price for a single feed ID
    pub async fn get_latest_price(&self, feed_id: &str) -> Result<Option<ParsedPriceFeed>> {
        let feeds = self.get_latest_prices(&[feed_id]).await?;
        Ok(feeds.into_iter().next())
    }

    /// Get all available price feed IDs
    pub async fn get_price_feed_ids(&self) -> Result<Vec<PriceFeedId>> {
        self.get("/v2/price_feeds").await
    }

    /// Search for price feeds by query (symbol, base, quote, etc.)
    pub async fn search_feeds(&self, query: &str) -> Result<Vec<PriceFeedId>> {
        let mut url = Url::parse(&self.base.url("/v2/price_feeds"))?;
        url.query_pairs_mut()
            .append_pair("query", query)
            .append_pair("asset_type", "crypto");
        self.get_url(&url).await
    }

    /// Get price feeds filtered by asset type
    pub async fn get_feeds_by_asset_type(&self, asset_type: &str) -> Result<Vec<PriceFeedId>> {
        let mut url = Url::parse(&self.base.url("/v2/price_feeds"))?;
        url.query_pairs_mut().append_pair("asset_type", asset_type);
        self.get_url(&url).await
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = Url::parse(&self.base.url(path))?;
        self.get_url(&url).await
    }

    async fn get_url<T: serde::de::DeserializeOwned>(&self, url: &Url) -> Result<T> {
        let url_str = url.to_string();
        let http = self.base.http().clone();
        // Includes `Authorization: Bearer <key>` when an API key is configured
        let headers = self.base.default_headers();
        let has_api_key = self.has_api_key();

        // Use with_retry for automatic retry with backoff
        let result = with_retry(&self.retry_config, || {
            let http = http.clone();
            let url_str = url_str.clone();
            let headers = headers.clone();
            async move {
                let response = http.get(&url_str).headers(headers).send().await?;
                let status = response.status().as_u16();

                if matches!(status, 401 | 403) {
                    return Err(crate::error::unauthorized(status, has_api_key));
                }

                if response.status().is_success() {
                    response
                        .json()
                        .await
                        .map_err(|e| Error::api(status, format!("Parse error: {e}")))
                } else {
                    let retry_after = yldfi_common::api::extract_retry_after(response.headers());
                    let body = response.text().await.unwrap_or_default();

                    // Truncate error body to prevent huge error messages
                    let body_truncated = if body.len() > MAX_ERROR_BODY_LEN {
                        let mut end = MAX_ERROR_BODY_LEN;
                        while !body.is_char_boundary(end) {
                            end -= 1;
                        }
                        format!("{}...(truncated)", &body[..end])
                    } else {
                        body
                    };

                    Err(Error::from_response(status, &body_truncated, retry_after))
                }
            }
        })
        .await;

        result.map_err(yldfi_common::RetryError::into_inner)
    }
}

/// Normalize feed ID (ensure 0x prefix, lowercase)
///
/// Returns a Cow to avoid allocation when the ID is already normalized.
fn normalize_feed_id(id: &str) -> Cow<'_, str> {
    let id = id.trim();

    // Check if already normalized (lowercase with 0x prefix)
    if id.starts_with("0x")
        && id
            .chars()
            .skip(2)
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return Cow::Borrowed(id);
    }

    // Need to normalize
    let lower = id.to_lowercase();
    if lower.starts_with("0x") {
        Cow::Owned(lower)
    } else {
        Cow::Owned(format!("0x{lower}"))
    }
}

/// Validate that a feed ID has the correct format (64 hex chars with optional 0x prefix)
fn validate_feed_id(id: &str) -> bool {
    let id = id.trim();
    let hex_part = id.strip_prefix("0x").unwrap_or(id);
    hex_part.len() == 64 && hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

/// Well-known Pyth price feed IDs for common assets
///
/// Feed IDs can be found at: <https://pyth.network/developers/price-feed-ids>
pub mod feed_ids {
    /// BTC/USD
    pub const BTC_USD: &str = "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";
    /// ETH/USD
    pub const ETH_USD: &str = "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace";
    /// SOL/USD
    pub const SOL_USD: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    /// USDC/USD
    pub const USDC_USD: &str = "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";
    /// USDT/USD
    pub const USDT_USD: &str = "0x2b89b9dc8fdf9f34709a5b106b472f0f39bb6ca9ce04b0fd7f2e971688e2e53b";
    /// LINK/USD
    pub const LINK_USD: &str = "0x8ac0c70fff57e9aefdf5edf44b51d62c2d433653cbb2cf5cc06bb115af04d221";
    /// ARB/USD
    pub const ARB_USD: &str = "0x3fa4252848f9f0a1480be62745a4629d9eb1322aebab8a791e344b3b9c1adcf5";
    /// OP/USD
    pub const OP_USD: &str = "0x385f64d993f7b77d8182ed5003d97c60aa3361f3cecfe711544d2d59165e9bdf";
    /// AAVE/USD
    pub const AAVE_USD: &str = "0x2b9ab1e972a281585084148ba1389800799bd4be63b957507db1349314e47445";
    /// UNI/USD
    pub const UNI_USD: &str = "0x78d185a741d07edb3412b09008b7c5cfb9bbbd7d568bf00ba737b456ba171501";
    /// CRV/USD
    pub const CRV_USD: &str = "0xa19d04ac696c7a6616d291c7e5d1377cc8be437c327b75adb5dc1bad745fcae8";
    /// CVX/USD
    pub const CVX_USD: &str = "0x6aac625e125ada0d2a6b98316493256ca733a5808cd34ccef79b0e28c64d1e76";
    /// SNX/USD (Synthetix)
    pub const SNX_USD: &str = "0x39d020f60982ed892abbcd4a06a276a9f9b7bfbce003204c110b6e488f502da3";
    /// LDO/USD (Lido DAO)
    pub const LDO_USD: &str = "0xc63e2a7f37a04e5e614c07238bedb25dcc38927fba8fe890597a593c0b2fa4ad";
    /// DAI/USD
    pub const DAI_USD: &str = "0xb0948a5e5313200c632b51bb5ca32f6de0d36e9950a942d19751e833f70dabfd";
    /// DOGE/USD
    pub const DOGE_USD: &str = "0xdcef50dd0a4cd2dcc17e45df1676dcb336a11a61c69df7a0299b0150c672d25c";
    /// AVAX/USD
    pub const AVAX_USD: &str = "0x93da3352f9f1d105fdfe4971cfa80e9dd777bfc5d0f683ebb6e1294b92137bb7";
    /// ATOM/USD (Cosmos)
    pub const ATOM_USD: &str = "0xb00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819";
    /// DOT/USD (Polkadot)
    pub const DOT_USD: &str = "0xca3eed9b267293f6595901c734c7525ce8ef49adafe8284606ceb307afa2ca5b";
}

/// Map common token symbols to Pyth feed IDs
///
/// Returns `None` for unknown symbols. Use `search_feeds()` to find
/// feed IDs for tokens not in this mapping.
#[must_use]
pub fn symbol_to_feed_id(symbol: &str) -> Option<&'static str> {
    match symbol.to_uppercase().as_str() {
        // Major tokens
        "BTC" | "BITCOIN" | "WBTC" => Some(feed_ids::BTC_USD),
        "ETH" | "ETHEREUM" | "WETH" => Some(feed_ids::ETH_USD),
        "SOL" | "SOLANA" => Some(feed_ids::SOL_USD),
        // Stablecoins
        "USDC" => Some(feed_ids::USDC_USD),
        "USDT" | "TETHER" => Some(feed_ids::USDT_USD),
        "DAI" => Some(feed_ids::DAI_USD),
        // DeFi tokens
        "LINK" | "CHAINLINK" => Some(feed_ids::LINK_USD),
        "AAVE" => Some(feed_ids::AAVE_USD),
        "UNI" | "UNISWAP" => Some(feed_ids::UNI_USD),
        "CRV" | "CURVE" => Some(feed_ids::CRV_USD),
        "CVX" | "CONVEX" => Some(feed_ids::CVX_USD),
        "SNX" | "SYNTHETIX" => Some(feed_ids::SNX_USD),
        "LDO" | "LIDO" => Some(feed_ids::LDO_USD),
        // L2 tokens
        "ARB" | "ARBITRUM" => Some(feed_ids::ARB_USD),
        "OP" | "OPTIMISM" => Some(feed_ids::OP_USD),
        // Other popular tokens
        "DOGE" | "DOGECOIN" => Some(feed_ids::DOGE_USD),
        "AVAX" | "AVALANCHE" => Some(feed_ids::AVAX_USD),
        "ATOM" | "COSMOS" => Some(feed_ids::ATOM_USD),
        "DOT" | "POLKADOT" => Some(feed_ids::DOT_USD),
        _ => None,
    }
}
