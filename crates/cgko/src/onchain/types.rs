//! Types for onchain/GeckoTerminal endpoints

use serde::{Deserialize, Serialize};

/// Network data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Network {
    pub id: String,
    #[serde(rename = "type")]
    pub network_type: Option<String>,
    pub attributes: Option<NetworkAttributes>,
}

/// Network attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkAttributes {
    pub name: Option<String>,
    pub coingecko_asset_platform_id: Option<String>,
}

/// Networks response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworksResponse {
    pub data: Vec<Network>,
}

/// DEX data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dex {
    pub id: String,
    #[serde(rename = "type")]
    pub dex_type: Option<String>,
    pub attributes: Option<DexAttributes>,
}

/// DEX attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexAttributes {
    pub name: Option<String>,
}

/// DEXes response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexesResponse {
    pub data: Vec<Dex>,
}

/// Pool data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pool {
    pub id: String,
    #[serde(rename = "type")]
    pub pool_type: Option<String>,
    pub attributes: Option<PoolAttributes>,
    pub relationships: Option<serde_json::Value>,
}

/// Pool attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolAttributes {
    pub name: Option<String>,
    pub address: Option<String>,
    pub base_token_price_usd: Option<String>,
    pub quote_token_price_usd: Option<String>,
    pub base_token_price_native_currency: Option<String>,
    pub quote_token_price_native_currency: Option<String>,
    pub pool_created_at: Option<String>,
    pub reserve_in_usd: Option<String>,
    pub fdv_usd: Option<String>,
    pub market_cap_usd: Option<String>,
    pub price_change_percentage: Option<PriceChangePercentage>,
    pub transactions: Option<Transactions>,
    pub volume_usd: Option<VolumeUsd>,
}

/// Price change percentage
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceChangePercentage {
    pub m5: Option<String>,
    pub h1: Option<String>,
    pub h6: Option<String>,
    pub h24: Option<String>,
}

/// Transaction counts
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transactions {
    pub m5: Option<TransactionCount>,
    pub h1: Option<TransactionCount>,
    pub h6: Option<TransactionCount>,
    pub h24: Option<TransactionCount>,
}

/// Transaction count
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionCount {
    pub buys: Option<u64>,
    pub sells: Option<u64>,
}

/// Volume USD
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VolumeUsd {
    pub m5: Option<String>,
    pub h1: Option<String>,
    pub h6: Option<String>,
    pub h24: Option<String>,
}

/// Pools response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolsResponse {
    pub data: Vec<Pool>,
}

/// Single pool response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolResponse {
    pub data: Pool,
    pub included: Option<Vec<serde_json::Value>>,
}

/// Token data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Token {
    pub id: String,
    #[serde(rename = "type")]
    pub token_type: Option<String>,
    pub attributes: Option<TokenAttributes>,
}

/// Token attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenAttributes {
    pub name: Option<String>,
    pub address: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub total_supply: Option<String>,
    pub coingecko_coin_id: Option<String>,
    pub price_usd: Option<String>,
    pub fdv_usd: Option<String>,
    pub total_reserve_in_usd: Option<String>,
    pub volume_usd: Option<VolumeUsd>,
    pub market_cap_usd: Option<String>,
}

/// Token response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenResponse {
    pub data: Token,
    pub included: Option<Vec<serde_json::Value>>,
}

/// Token price response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenPriceResponse {
    pub data: TokenPriceData,
}

/// Token price data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenPriceData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<TokenPriceAttributes>,
}

/// Token price attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenPriceAttributes {
    pub token_prices: Option<serde_json::Value>,
}

/// OHLCV response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OhlcvResponse {
    pub data: OhlcvData,
    /// Base and quote token metadata
    #[serde(default)]
    pub meta: Option<OhlcvMeta>,
}

/// OHLCV metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OhlcvMeta {
    pub base: Option<OhlcvToken>,
    pub quote: Option<OhlcvToken>,
}

/// Token metadata in an OHLCV response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OhlcvToken {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub coingecko_coin_id: Option<String>,
    pub address: Option<String>,
}

/// OHLCV data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OhlcvData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<OhlcvAttributes>,
}

/// OHLCV attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OhlcvAttributes {
    pub ohlcv_list: Option<Vec<[f64; 6]>>,
}

/// Trades response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradesResponse {
    pub data: Vec<Trade>,
    /// Pagination: pass `next_cursor` back via [`TradesOptions::cursor`]
    #[serde(default)]
    pub meta: Option<TradesMeta>,
}

/// Trades pagination metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradesMeta {
    /// Cursor for the next page; `None` on the last page
    pub next_cursor: Option<String>,
}

/// Trade data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trade {
    pub id: String,
    #[serde(rename = "type")]
    pub trade_type: Option<String>,
    pub attributes: Option<TradeAttributes>,
}

/// Trade attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeAttributes {
    pub block_number: Option<u64>,
    pub tx_hash: Option<String>,
    pub tx_from_address: Option<String>,
    pub from_token_amount: Option<String>,
    pub to_token_amount: Option<String>,
    pub price_from_in_currency_token: Option<String>,
    pub price_to_in_currency_token: Option<String>,
    pub price_from_in_usd: Option<String>,
    pub price_to_in_usd: Option<String>,
    pub block_timestamp: Option<String>,
    pub kind: Option<String>,
    pub volume_in_usd: Option<String>,
    /// Whether `CoinGecko` flagged the trade as anomalous (only returned
    /// when anomalous trades are requested)
    #[serde(default)]
    pub is_anomalous: Option<bool>,
}

/// Liquidity events response (`/pools/{pool_address}/liquidity_events`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityEventsResponse {
    pub data: Vec<LiquidityEvent>,
    pub meta: Option<LiquidityEventsMeta>,
}

/// Liquidity event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub attributes: LiquidityEventAttributes,
}

/// Liquidity event attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityEventAttributes {
    pub block_number: u64,
    pub tx_hash: String,
    pub tx_from_address: String,
    pub block_timestamp: String,
    /// `add` or `remove`
    pub kind: String,
    pub base_token_address: String,
    /// Always positive; `kind` carries the direction
    pub base_token_amount: String,
    pub quote_token_address: String,
    /// Always positive; `kind` carries the direction
    pub quote_token_amount: String,
    pub base_token_price_in_usd: String,
    pub quote_token_price_in_usd: String,
    /// Combined USD value of both tokens in the event
    pub volume_in_usd: String,
}

/// Liquidity events pagination metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityEventsMeta {
    /// Pass back unchanged as `cursor` for the next page; `None` on the last
    pub next_cursor: Option<String>,
}

/// Options for liquidity events by pool address
#[derive(Debug, Clone, Default)]
pub struct LiquidityEventsOptions {
    /// `add` or `remove` (omit for both)
    pub kind: Option<String>,
    /// Only events worth more than this many USD
    pub volume_in_usd_greater_than: Option<f64>,
    /// `meta.next_cursor` from the previous page
    pub cursor: Option<String>,
    /// Results per page, 1 to 50 (API default: 20)
    pub per_page: Option<u32>,
}

impl LiquidityEventsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by kind (`add` or `remove`)
    #[must_use]
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Only events worth more than `usd`
    #[must_use]
    pub fn volume_in_usd_greater_than(mut self, usd: f64) -> Self {
        self.volume_in_usd_greater_than = Some(usd);
        self
    }

    /// Continue from a previous page's `meta.next_cursor`
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Set the number of results per page (1 to 50)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref k) = self.kind {
            params.push(format!("kind={k}"));
        }
        if let Some(v) = self.volume_in_usd_greater_than {
            params.push(format!("volume_in_usd_greater_than={v}"));
        }
        if let Some(ref c) = self.cursor {
            params.push(format!(
                "cursor={}",
                url::form_urlencoded::byte_serialize(c.as_bytes()).collect::<String>()
            ));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Multi-token response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokensResponse {
    pub data: Vec<Token>,
}

/// Token info attributes (detailed)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfoAttributes {
    pub name: Option<String>,
    pub address: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub total_supply: Option<String>,
    pub coingecko_coin_id: Option<String>,
    pub price_usd: Option<String>,
    pub fdv_usd: Option<String>,
    pub total_reserve_in_usd: Option<String>,
    pub volume_usd: Option<VolumeUsd>,
    pub market_cap_usd: Option<String>,
    pub description: Option<String>,
    pub gt_score: Option<f64>,
    pub discord_url: Option<String>,
    pub telegram_handle: Option<String>,
    pub twitter_handle: Option<String>,
    pub websites: Option<Vec<String>>,
    pub image_url: Option<String>,
    /// Whether the token's name or symbol matches the `GeckoTerminal` NSFW
    /// keyword list
    #[serde(default)]
    pub is_nsfw: Option<bool>,
}

/// Token info response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfoResponse {
    pub data: TokenInfo,
}

/// Token info data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub token_type: Option<String>,
    pub attributes: Option<TokenInfoAttributes>,
    pub relationships: Option<serde_json::Value>,
}

/// Pool info response
///
/// `data` holds one token-info entry per pool token (base and quote); the
/// pool itself is referenced from each entry's `relationships.pool`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolInfoResponse {
    pub data: Vec<TokenInfo>,
    /// Included resources (e.g. the pool) when requested via `include`
    #[serde(default)]
    pub included: Option<Vec<serde_json::Value>>,
}

/// Top token holders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHoldersResponse {
    pub data: TokenHoldersData,
}

/// Top token holders data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHoldersData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<TokenHoldersAttributes>,
}

/// Top token holders attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHoldersAttributes {
    pub last_updated_at: Option<String>,
    #[serde(default)]
    pub holders: Vec<TokenHolder>,
}

/// Token holder (amounts are decimal strings)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHolder {
    pub rank: Option<u64>,
    pub address: String,
    pub label: Option<String>,
    pub amount: Option<String>,
    /// Percentage of total supply held
    pub percentage: Option<String>,
    /// Value of holdings in USD
    pub value: Option<String>,
    pub average_buy_price_usd: Option<String>,
    pub total_buy_count: Option<u64>,
    pub total_sell_count: Option<u64>,
    pub unrealized_pnl_usd: Option<String>,
    pub unrealized_pnl_percentage: Option<String>,
    pub realized_pnl_usd: Option<String>,
    pub realized_pnl_percentage: Option<String>,
    pub explorer_url: Option<String>,
}

/// Top token traders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTradersResponse {
    pub data: TokenTradersData,
}

/// Top token traders data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTradersData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<TokenTradersAttributes>,
}

/// Top token traders attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTradersAttributes {
    #[serde(default)]
    pub traders: Vec<TokenTrader>,
}

/// Token trader (amounts are decimal strings)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTrader {
    pub address: String,
    pub name: Option<String>,
    pub label: Option<String>,
    /// Address type
    #[serde(rename = "type")]
    pub trader_type: Option<String>,
    pub realized_pnl_usd: Option<String>,
    pub unrealized_pnl_usd: Option<String>,
    pub token_balance: Option<String>,
    pub average_buy_price_usd: Option<String>,
    pub average_sell_price_usd: Option<String>,
    pub total_buy_count: Option<u64>,
    pub total_sell_count: Option<u64>,
    pub total_buy_token_amount: Option<String>,
    pub total_sell_token_amount: Option<String>,
    pub total_buy_usd: Option<String>,
    pub total_sell_usd: Option<String>,
    pub explorer_url: Option<String>,
}

/// Holders chart response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldersChartResponse {
    pub data: HoldersChartData,
    #[serde(default)]
    pub meta: Option<HoldersChartMeta>,
}

/// Holders chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldersChartData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<HoldersChartAttributes>,
}

/// Holders chart attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldersChartAttributes {
    /// Vec of [ISO 8601 timestamp, `holder_count`]
    #[serde(default)]
    pub token_holders_list: Vec<(String, u64)>,
}

/// Holders chart metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldersChartMeta {
    pub token: Option<HoldersChartToken>,
}

/// Token described by a holders chart
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldersChartToken {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub coingecko_coin_id: Option<String>,
    pub address: Option<String>,
}

/// Token OHLCV response (for token, not pool)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenOhlcvResponse {
    pub data: TokenOhlcvData,
    /// Base and quote token metadata
    #[serde(default)]
    pub meta: Option<OhlcvMeta>,
}

/// Token OHLCV data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenOhlcvData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<TokenOhlcvAttributes>,
}

/// Token OHLCV attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenOhlcvAttributes {
    pub ohlcv_list: Option<Vec<[f64; 6]>>,
}

/// Token trades response (trades for a token across all pools)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTradesResponse {
    pub data: Vec<Trade>,
    /// Pagination: pass `next_cursor` back via [`TradesOptions::cursor`]
    #[serde(default)]
    pub meta: Option<TradesMeta>,
}

fn encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn query_string(params: &[String]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    }
}

/// Options for pool and token trades
#[derive(Debug, Clone, Default)]
pub struct TradesOptions {
    /// Only trades worth more than this many USD (API default: 0)
    pub trade_volume_in_usd_greater_than: Option<f64>,
    /// Pool trades only: `base`, `quote` or a token address (API default:
    /// `base`); not sent for token trades
    pub token: Option<String>,
    /// Lookback period: `1d` (API default), `7d` or `30d` (Analyst plan and
    /// above)
    pub trading_period: Option<String>,
    /// Include trades flagged as anomalous (API default: false)
    pub include_anomalous: Option<bool>,
    /// `meta.next_cursor` from the previous page (Analyst plan and above)
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (API default: 300; Analyst plan and above)
    pub per_page: Option<u32>,
}

impl TradesOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only trades worth more than `usd`
    #[must_use]
    pub fn trade_volume_in_usd_greater_than(mut self, usd: f64) -> Self {
        self.trade_volume_in_usd_greater_than = Some(usd);
        self
    }

    /// Pool trades only: `base`, `quote` or a token address
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Lookback period: `1d`, `7d` or `30d`
    #[must_use]
    pub fn trading_period(mut self, period: impl Into<String>) -> Self {
        self.trading_period = Some(period.into());
        self
    }

    /// Include trades flagged as anomalous
    #[must_use]
    pub fn include_anomalous(mut self, include: bool) -> Self {
        self.include_anomalous = Some(include);
        self
    }

    /// Continue from a previous page's `meta.next_cursor`
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Set the number of results per page (1 to 300)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(v) = self.trade_volume_in_usd_greater_than {
            params.push(format!("trade_volume_in_usd_greater_than={v}"));
        }
        if let Some(ref t) = self.token {
            params.push(format!("token={}", encode(t)));
        }
        if let Some(ref p) = self.trading_period {
            params.push(format!("trading_period={p}"));
        }
        if let Some(a) = self.include_anomalous {
            params.push(format!("include_anomalous={a}"));
        }
        if let Some(ref c) = self.cursor {
            params.push(format!("cursor={}", encode(c)));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        query_string(&params)
    }
}

/// Options for pool and token OHLCV
#[derive(Debug, Clone, Default)]
pub struct OhlcvOptions {
    /// Candle size in timeframe units: day `1`; hour `1`, `4`, `12`;
    /// minute `1`, `5`, `15`; second `1`, `15`, `30` (API default: 1)
    pub aggregate: Option<u32>,
    /// Return candles up to and including the one containing this Unix
    /// timestamp (seconds)
    pub before_timestamp: Option<u64>,
    /// Number of candles, up to 1000 (API default: 100)
    pub limit: Option<u32>,
    /// `usd` (API default) or `token` (quote token)
    pub currency: Option<String>,
    /// Pool OHLCV only: `base`, `quote` or a token address (API default:
    /// `base`); not sent for token OHLCV
    pub token: Option<String>,
    /// Include intervals with no trades (API default: false)
    pub include_empty_intervals: Option<bool>,
    /// Token OHLCV only: include data from inactive pools (API default:
    /// false); not sent for pool OHLCV
    pub include_inactive_source: Option<bool>,
}

impl OhlcvOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the candle size in timeframe units
    #[must_use]
    pub fn aggregate(mut self, aggregate: u32) -> Self {
        self.aggregate = Some(aggregate);
        self
    }

    /// Only candles up to the one containing this Unix timestamp (seconds).
    /// Free plans only serve the last 180 days.
    #[must_use]
    pub fn before_timestamp(mut self, timestamp: u64) -> Self {
        self.before_timestamp = Some(timestamp);
        self
    }

    /// Set the number of candles (up to 1000)
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// `usd` or `token`
    #[must_use]
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }

    /// Pool OHLCV only: `base`, `quote` or a token address
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Include intervals with no trades
    #[must_use]
    pub fn include_empty_intervals(mut self, include: bool) -> Self {
        self.include_empty_intervals = Some(include);
        self
    }

    /// Token OHLCV only: include data from inactive pools
    #[must_use]
    pub fn include_inactive_source(mut self, include: bool) -> Self {
        self.include_inactive_source = Some(include);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(a) = self.aggregate {
            params.push(format!("aggregate={a}"));
        }
        if let Some(t) = self.before_timestamp {
            params.push(format!("before_timestamp={t}"));
        }
        if let Some(l) = self.limit {
            params.push(format!("limit={l}"));
        }
        if let Some(ref c) = self.currency {
            params.push(format!("currency={c}"));
        }
        if let Some(ref t) = self.token {
            params.push(format!("token={}", encode(t)));
        }
        if let Some(e) = self.include_empty_intervals {
            params.push(format!("include_empty_intervals={e}"));
        }
        if let Some(i) = self.include_inactive_source {
            params.push(format!("include_inactive_source={i}"));
        }
        query_string(&params)
    }
}

/// Onchain category
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnchainCategory {
    pub id: String,
    #[serde(rename = "type")]
    pub category_type: Option<String>,
    pub attributes: Option<OnchainCategoryAttributes>,
}

/// Onchain category attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnchainCategoryAttributes {
    pub name: Option<String>,
    pub description: Option<String>,
    pub volume_change_percentage: Option<serde_json::Value>,
}

/// Onchain categories response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnchainCategoriesResponse {
    pub data: Vec<OnchainCategory>,
}

/// Megafilter options for pool search
#[derive(Debug, Clone, Default)]
pub struct MegafilterOptions {
    pub networks: Option<Vec<String>>,
    pub dexes: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub page: Option<u32>,
    /// Sort order, e.g. `h24_volume_usd_desc` (API default: `h6_trending`)
    pub sort: Option<String>,
}

impl MegafilterOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sort order (e.g. `h24_volume_usd_desc`, `reserve_in_usd_desc`)
    #[must_use]
    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref networks) = self.networks {
            params.push(format!("networks={}", networks.join(",")));
        }
        if let Some(ref dexes) = self.dexes {
            params.push(format!("dexes={}", dexes.join(",")));
        }
        if let Some(ref include) = self.include {
            params.push(format!("include={}", include.join(",")));
        }
        if let Some(page) = self.page {
            params.push(format!("page={page}"));
        }
        if let Some(ref sort) = self.sort {
            params.push(format!("sort={sort}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trades_options_query() {
        assert_eq!(TradesOptions::new().to_query_string(), "");
        let q = TradesOptions::new()
            .trade_volume_in_usd_greater_than(100.0)
            .token("quote")
            .trading_period("7d")
            .include_anomalous(true)
            .cursor("a+b=")
            .per_page(50)
            .to_query_string();
        assert_eq!(
            q,
            "?trade_volume_in_usd_greater_than=100&token=quote&trading_period=7d&include_anomalous=true&cursor=a%2Bb%3D&per_page=50"
        );
    }

    #[test]
    fn ohlcv_options_query() {
        assert_eq!(OhlcvOptions::new().to_query_string(), "");
        let q = OhlcvOptions::new()
            .aggregate(4)
            .before_timestamp(1_700_000_000)
            .limit(1000)
            .currency("token")
            .token("quote")
            .include_empty_intervals(true)
            .include_inactive_source(false)
            .to_query_string();
        assert_eq!(
            q,
            "?aggregate=4&before_timestamp=1700000000&limit=1000&currency=token&token=quote&include_empty_intervals=true&include_inactive_source=false"
        );
    }

    #[test]
    fn trades_and_ohlcv_read_meta() {
        let t: TradesResponse =
            serde_json::from_str(r#"{"data":[],"meta":{"next_cursor":"abc"}}"#).unwrap();
        assert_eq!(t.meta.unwrap().next_cursor.as_deref(), Some("abc"));
        let t: TradesResponse = serde_json::from_str(r#"{"data":[]}"#).unwrap();
        assert!(t.meta.is_none());
        let o: OhlcvResponse = serde_json::from_str(
            r#"{"data":{"id":"x","attributes":{"ohlcv_list":[[1,2,3,4,5,6]]}},
                "meta":{"base":{"symbol":"AAA"},"quote":{"symbol":"BBB"}}}"#,
        )
        .unwrap();
        assert_eq!(
            o.meta.unwrap().quote.unwrap().symbol.as_deref(),
            Some("BBB")
        );
    }

    #[test]
    fn liquidity_events_match_spec() {
        let json = r#"{"data":[{"id":"solana_1_abc_20_1792148006","type":"liquidity_event",
            "attributes":{"block_number":457611113,"tx_hash":"abc","tx_from_address":"sender",
            "block_timestamp":"2026-10-16T10:53:26Z","kind":"remove",
            "base_token_address":"base","base_token_amount":"131587980.65415",
            "quote_token_address":"quote","quote_token_amount":"187.258906",
            "base_token_price_in_usd":"0.0001417","quote_token_price_in_usd":"106.496866",
            "volume_in_usd":"38593.231589"}}],"meta":{"next_cursor":null}}"#;
        let r: LiquidityEventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data[0].attributes.kind, "remove");
        assert_eq!(r.data[0].attributes.block_number, 457_611_113);
        assert!(r.meta.unwrap().next_cursor.is_none());
    }

    #[test]
    fn liquidity_events_options_query() {
        assert_eq!(LiquidityEventsOptions::new().to_query_string(), "");
        let q = LiquidityEventsOptions::new()
            .kind("add")
            .volume_in_usd_greater_than(1000.5)
            .cursor("a+b/c=")
            .per_page(50)
            .to_query_string();
        assert_eq!(
            q,
            "?kind=add&volume_in_usd_greater_than=1000.5&cursor=a%2Bb%2Fc%3D&per_page=50"
        );
    }

    #[test]
    fn token_info_reads_is_nsfw() {
        let t: TokenInfoAttributes = serde_json::from_str(r#"{"is_nsfw":false}"#).unwrap();
        assert_eq!(t.is_nsfw, Some(false));
        let t: TokenInfoAttributes = serde_json::from_str(r#"{}"#).unwrap();
        assert!(t.is_nsfw.is_none());
    }

    #[test]
    fn top_holders_data_is_an_object_with_string_amounts() {
        let json = r#"{"data": {"id": "eth_0xtoken", "type": "top_holder", "attributes": {
            "last_updated_at": "2026-01-01T00:00:00.000Z",
            "holders": [{"rank": 1, "address": "addr1", "label": null,
                "amount": "310021323.44", "percentage": "31.0041", "value": "6157733.22",
                "average_buy_price_usd": null, "total_buy_count": null, "total_sell_count": null,
                "unrealized_pnl_usd": null, "unrealized_pnl_percentage": null,
                "realized_pnl_usd": null, "realized_pnl_percentage": null,
                "explorer_url": "https://example.com/address/addr1"}]}}}"#;
        let r: TokenHoldersResponse = serde_json::from_str(json).unwrap();
        let attrs = r.data.attributes.unwrap();
        assert_eq!(attrs.holders[0].rank, Some(1));
        assert_eq!(attrs.holders[0].percentage.as_deref(), Some("31.0041"));
        assert_eq!(attrs.holders[0].value.as_deref(), Some("6157733.22"));
    }

    #[test]
    fn top_traders_data_is_an_object() {
        let json = r#"{"data": {"id": "eth_0xtoken", "type": "top_trader", "attributes": {
            "traders": [{"address": "addr1", "name": null, "label": null, "type": null,
                "realized_pnl_usd": "312.5", "unrealized_pnl_usd": null, "token_balance": null,
                "average_buy_price_usd": "0.1", "average_sell_price_usd": "0.2",
                "total_buy_count": 52, "total_sell_count": 110,
                "total_buy_token_amount": "270.6", "total_sell_token_amount": "325.1",
                "total_buy_usd": "29.1", "total_sell_usd": "60.4",
                "explorer_url": "https://example.com/address/addr1"}]}}}"#;
        let r: TokenTradersResponse = serde_json::from_str(json).unwrap();
        let trader = &r.data.attributes.unwrap().traders[0];
        assert_eq!(trader.total_sell_count, Some(110));
        assert_eq!(trader.realized_pnl_usd.as_deref(), Some("312.5"));
    }

    #[test]
    fn holders_chart_uses_token_holders_list_with_iso_timestamps() {
        let json = r#"{"data": {"id": "req-1", "type": "token_holders_snapshot", "attributes": {
            "token_holders_list": [["2026-04-28T00:00:00.000Z", 52790], ["2026-04-29T00:00:00.000Z", 52865]]}},
            "meta": {"token": {"name": "Example", "symbol": "ex", "coingecko_coin_id": null, "address": "addr1"}}}"#;
        let r: HoldersChartResponse = serde_json::from_str(json).unwrap();
        let list = r.data.attributes.unwrap().token_holders_list;
        assert_eq!(list[1], ("2026-04-29T00:00:00.000Z".to_string(), 52865));
        assert_eq!(r.meta.unwrap().token.unwrap().symbol.as_deref(), Some("ex"));
    }

    #[test]
    fn pool_info_data_is_a_list_of_token_info() {
        let json = r#"{"data": [
            {"id": "eth_0xbase", "type": "token", "attributes": {"address": "0xbase", "name": "Base", "symbol": "B", "decimals": 18, "gt_score": 55.5},
             "relationships": {"pool": {"data": {"id": "eth_0xpool", "type": "pool"}}}},
            {"id": "eth_0xquote", "type": "token", "attributes": {"address": "0xquote", "name": "Quote", "symbol": "Q", "decimals": 6}}
        ], "included": [{"id": "eth_0xpool", "type": "pool", "attributes": {"base_token_address": "0xbase"}}]}"#;
        let r: PoolInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.len(), 2);
        assert_eq!(
            r.data[1].attributes.as_ref().unwrap().symbol.as_deref(),
            Some("Q")
        );
        assert_eq!(r.included.unwrap().len(), 1);
    }

    #[test]
    fn megafilter_query_includes_sort() {
        assert_eq!(MegafilterOptions::new().to_query_string(), "");
        let opts = MegafilterOptions {
            networks: Some(vec!["eth".into()]),
            ..MegafilterOptions::new()
        }
        .with_sort("h24_volume_usd_desc");
        assert_eq!(
            opts.to_query_string(),
            "?networks=eth&sort=h24_volume_usd_desc"
        );
    }
}
