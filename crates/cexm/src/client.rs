//! The [`Client`]: one entry point for every exchange

use crate::error::{Error, Result};
use crate::exchanges;
use crate::types::{Candle, Exchange, Market, OrderBook, Symbol, Ticker, Timeframe, Trade};
use std::time::Duration;
use yldfi_common::http::{build_client, HttpClientConfig};

/// Default request timeout
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default API base URL for each exchange
pub fn default_base_url(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Binance => "https://data-api.binance.vision",
        Exchange::Bitget => "https://api.bitget.com",
        Exchange::Okx => "https://www.okx.com",
        Exchange::Hyperliquid => "https://api.hyperliquid.xyz",
        Exchange::Lighter => "https://mainnet.zklighter.elliot.ai",
        Exchange::Bybit => "https://api.bybit.com",
        Exchange::Coinbase => "https://api.exchange.coinbase.com",
        Exchange::Kraken => "https://api.kraken.com",
    }
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct Config {
    http: HttpClientConfig,
    base_urls: Vec<(Exchange, String)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http: HttpClientConfig::default().with_timeout(DEFAULT_TIMEOUT),
            base_urls: Vec::new(),
        }
    }
}

impl Config {
    /// Override one exchange's base URL (e.g. a mock server in tests)
    #[must_use]
    pub fn with_base_url(mut self, exchange: Exchange, url: impl Into<String>) -> Self {
        self.base_urls.retain(|(e, _)| *e != exchange);
        self.base_urls.push((exchange, url.into()));
        self
    }

    /// Override every exchange's base URL
    #[must_use]
    pub fn with_all_base_urls(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.base_urls = Exchange::ALL.iter().map(|e| (*e, url.clone())).collect();
        self
    }

    /// Request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.http = self.http.with_timeout(timeout);
        self
    }

    /// Send requests through an HTTP(S) proxy, e.g. for an exchange that
    /// refuses the caller's region
    #[must_use]
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.http = self.http.with_proxy(proxy);
        self
    }
}

/// Public market data client for all supported exchanges
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_urls: Vec<(Exchange, String)>,
}

/// What an exchange module needs for one call
pub(crate) struct Ctx<'a> {
    pub http: &'a reqwest::Client,
    pub base: &'a str,
    pub exchange: Exchange,
}

impl Ctx<'_> {
    /// `base` + `path`
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base.trim_end_matches('/'))
    }
}

impl Client {
    /// Client with default settings
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    /// Client with custom settings
    pub fn with_config(config: Config) -> Result<Self> {
        let http = build_client(&config.http).map_err(|e| Error::Build(e.to_string()))?;
        Ok(Self {
            http,
            base_urls: config.base_urls,
        })
    }

    fn ctx(&self, exchange: Exchange) -> Ctx<'_> {
        let base = self
            .base_urls
            .iter()
            .find(|(e, _)| *e == exchange)
            .map(|(_, url)| url.as_str())
            .unwrap_or_else(|| default_base_url(exchange));
        Ctx {
            http: &self.http,
            base,
            exchange,
        }
    }

    /// 24h ticker for a symbol such as `BTC/USDT`
    pub async fn ticker(&self, exchange: Exchange, symbol: &str) -> Result<Ticker> {
        let symbol = parse_symbol(exchange, symbol)?;
        exchanges::ticker(&self.ctx(exchange), &symbol).await
    }

    /// Order book with up to `limit` levels per side (capped at the
    /// exchange's maximum)
    pub async fn order_book(
        &self,
        exchange: Exchange,
        symbol: &str,
        limit: u32,
    ) -> Result<OrderBook> {
        let symbol = parse_symbol(exchange, symbol)?;
        exchanges::order_book(&self.ctx(exchange), &symbol, limit.max(1)).await
    }

    /// The most recent `limit` candles, oldest first (capped at the
    /// exchange's maximum per request)
    pub async fn candles(
        &self,
        exchange: Exchange,
        symbol: &str,
        timeframe: Timeframe,
        limit: u32,
    ) -> Result<Vec<Candle>> {
        let symbol = parse_symbol(exchange, symbol)?;
        let mut candles =
            exchanges::candles(&self.ctx(exchange), &symbol, timeframe, limit.max(1)).await?;
        candles.sort_by_key(|c| c.timestamp);
        let excess = candles.len().saturating_sub(limit.max(1) as usize);
        candles.drain(..excess);
        Ok(candles)
    }

    /// The most recent `limit` public trades, newest first (capped at the
    /// exchange's maximum)
    pub async fn trades(&self, exchange: Exchange, symbol: &str, limit: u32) -> Result<Vec<Trade>> {
        let symbol = parse_symbol(exchange, symbol)?;
        let mut trades = exchanges::trades(&self.ctx(exchange), &symbol, limit.max(1)).await?;
        trades.sort_by_key(|t| std::cmp::Reverse(t.timestamp));
        trades.truncate(limit.max(1) as usize);
        Ok(trades)
    }

    /// Every market the exchange lists
    pub async fn markets(&self, exchange: Exchange) -> Result<Vec<Market>> {
        exchanges::markets(&self.ctx(exchange)).await
    }
}

fn parse_symbol(exchange: Exchange, symbol: &str) -> Result<Symbol> {
    Symbol::parse(symbol).ok_or_else(|| Error::InvalidSymbol {
        exchange,
        symbol: symbol.to_string(),
        reason: "expected BASE/QUOTE, e.g. BTC/USDT (perpetuals: BASE/QUOTE:SETTLE)".into(),
    })
}
