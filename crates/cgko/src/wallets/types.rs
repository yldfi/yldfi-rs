//! Types for onchain wallet endpoints
//!
//! Amounts and USD values are decimal strings, as `CoinGecko` sends them,
//! so no precision is lost.

use crate::query::Query;
use serde::{Deserialize, Serialize};

/// Cursor pagination metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorMeta {
    /// Pass back as the `cursor` option for the next page; `None` on the
    /// last page
    pub next_cursor: Option<String>,
}

// ---------------------------------------------------------------------------
// Balances
// ---------------------------------------------------------------------------

/// Options for [`super::WalletsApi::balances`]
#[derive(Debug, Clone, Default)]
pub struct WalletBalancesOptions {
    /// `value_usd_desc` (API default) or `value_usd_asc`
    pub sort: Option<String>,
    /// `native` or `non_native`; both when unset
    pub token_type: Option<String>,
    /// Minimum holding value in USD
    pub value_usd_min: Option<f64>,
    /// Minimum total reserve in USD of the holding's token
    pub reserve_in_usd_min: Option<f64>,
    /// Page through results (API default: 1)
    pub page: Option<u32>,
    /// Results per page, 1 to 100 (API default: 50)
    pub per_page: Option<u32>,
}

impl WalletBalancesOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sort by `value_usd_desc` or `value_usd_asc`
    #[must_use]
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Only `native` or only `non_native` tokens
    #[must_use]
    pub fn token_type(mut self, token_type: impl Into<String>) -> Self {
        self.token_type = Some(token_type.into());
        self
    }

    /// Only holdings worth at least `usd`
    #[must_use]
    pub fn value_usd_min(mut self, usd: f64) -> Self {
        self.value_usd_min = Some(usd);
        self
    }

    /// Only tokens with at least `usd` total reserve
    #[must_use]
    pub fn reserve_in_usd_min(mut self, usd: f64) -> Self {
        self.reserve_in_usd_min = Some(usd);
        self
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of results per page (1 to 100)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    pub(crate) fn add_to(&self, query: Query) -> Query {
        query
            .text("sort", self.sort.as_deref())
            .text("token_type", self.token_type.as_deref())
            .value("value_usd_min", self.value_usd_min)
            .value("reserve_in_usd_min", self.reserve_in_usd_min)
            .value("page", self.page)
            .value("per_page", self.per_page)
    }
}

/// Token balances of a wallet (`/onchain/wallets/{address}/balances`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletBalancesResponse {
    pub data: WalletBalancesData,
}

/// Token balances data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletBalancesData {
    /// The wallet address
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<WalletBalances>,
}

/// A wallet's holdings
///
/// `total_value_usd` and `total_holdings` count only the requested
/// networks, after the `token_type`, `value_usd_min` and
/// `reserve_in_usd_min` filters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletBalances {
    pub wallet_address: Option<String>,
    pub total_value_usd: Option<String>,
    pub total_holdings: Option<u64>,
    /// Totals per network
    #[serde(default)]
    pub networks: Vec<WalletNetworkBalance>,
    #[serde(default)]
    pub balances: Vec<WalletTokenBalance>,
}

/// A wallet's holdings on one network
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletNetworkBalance {
    pub network: String,
    pub value_usd: Option<String>,
    pub holdings: Option<u64>,
    /// ISO 8601 time
    pub last_updated_at: Option<String>,
}

/// A wallet's balance of one token
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTokenBalance {
    pub network: Option<String>,
    /// Token address (`0xeeee...eeee` for the native token on EVM networks)
    pub address: Option<String>,
    /// `native`, `erc20` or `spl`
    pub token_type: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u32>,
    pub coingecko_coin_id: Option<String>,
    /// Balance in the smallest unit; always present
    pub balance_raw: Option<String>,
    /// Full-precision balance; `None` when the token cannot be resolved
    /// (use `balance_raw / 10^decimals`)
    pub balance: Option<String>,
    /// `None` when the token has no price
    pub price_usd: Option<String>,
    /// `None` when the token has no price
    pub value_usd: Option<String>,
    pub h24_price_change_percentage: Option<String>,
    pub total_reserve_in_usd: Option<String>,
}

// ---------------------------------------------------------------------------
// Transfers
// ---------------------------------------------------------------------------

/// Options for [`super::WalletsApi::transfers`]
///
/// Set `from` and `to` together (at most 30 days apart); without them the
/// last 7 days are returned.
#[derive(Debug, Clone, Default)]
pub struct WalletTransfersOptions {
    /// Only transfers of this token contract address
    pub token: Option<String>,
    /// Window start: ISO date (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`) or Unix
    /// timestamp
    pub from: Option<String>,
    /// Window end: ISO date (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`) or Unix
    /// timestamp
    pub to: Option<String>,
    /// `in` (received) or `out` (sent); both when unset
    pub direction: Option<String>,
    /// `meta.next_cursor` from the previous page
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (API default: 100)
    pub per_page: Option<u32>,
}

impl WalletTransfersOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only transfers of this token
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Time window (both ends required together, at most 30 days apart)
    #[must_use]
    pub fn range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self.to = Some(to.into());
        self
    }

    /// Only `in` or only `out` transfers
    #[must_use]
    pub fn direction(mut self, direction: impl Into<String>) -> Self {
        self.direction = Some(direction.into());
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
        Query::new()
            .text("token", self.token.as_deref())
            .text("from", self.from.as_deref())
            .text("to", self.to.as_deref())
            .text("direction", self.direction.as_deref())
            .text("cursor", self.cursor.as_deref())
            .value("per_page", self.per_page)
            .finish()
    }
}

/// Token transfers of a wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTransfersResponse {
    pub data: Vec<WalletTransfer>,
    #[serde(default)]
    pub meta: Option<CursorMeta>,
}

/// A token transfer
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTransfer {
    pub id: String,
    #[serde(rename = "type")]
    pub transfer_type: Option<String>,
    pub attributes: Option<WalletTransferAttributes>,
}

/// Token transfer details
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTransferAttributes {
    pub block_number: Option<u64>,
    pub tx_hash: Option<String>,
    /// `in` (the wallet is `to_address`) or `out` (the wallet is
    /// `from_address`)
    pub direction: Option<String>,
    pub token_address: Option<String>,
    /// Amount in the smallest unit; always present
    pub amount_raw: Option<String>,
    /// Full-precision amount; `None` when the token cannot be resolved (use
    /// `amount_raw / 10^decimals`)
    pub amount: Option<String>,
    /// ISO 8601 time
    pub block_timestamp: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u32>,
}

// ---------------------------------------------------------------------------
// Trades
// ---------------------------------------------------------------------------

/// Options for [`super::WalletsApi::trades`]
///
/// Set `from` and `to` together (at most 30 days apart); without them the
/// last 7 days are returned.
#[derive(Debug, Clone, Default)]
pub struct WalletTradesOptions {
    /// Only trades involving this token contract address
    pub token: Option<String>,
    /// Window start: ISO date (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`) or Unix
    /// timestamp
    pub from: Option<String>,
    /// Window end: ISO date (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`) or Unix
    /// timestamp
    pub to: Option<String>,
    /// Include trades flagged as anomalous (API default: false)
    pub include_anomalous: Option<bool>,
    /// `meta.next_cursor` from the previous page (an invalid or expired
    /// cursor returns `400`)
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (API default: 100)
    pub per_page: Option<u32>,
}

impl WalletTradesOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only trades involving this token
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Time window (both ends required together, at most 30 days apart)
    #[must_use]
    pub fn range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self.to = Some(to.into());
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
        Query::new()
            .text("token", self.token.as_deref())
            .text("from", self.from.as_deref())
            .text("to", self.to.as_deref())
            .value("include_anomalous", self.include_anomalous)
            .text("cursor", self.cursor.as_deref())
            .value("per_page", self.per_page)
            .finish()
    }
}

/// DEX trades of a wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTradesResponse {
    pub data: Vec<WalletTrade>,
    #[serde(default)]
    pub meta: Option<CursorMeta>,
}

/// A DEX trade
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTrade {
    pub id: String,
    #[serde(rename = "type")]
    pub trade_type: Option<String>,
    pub attributes: Option<WalletTradeAttributes>,
}

/// DEX trade details
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTradeAttributes {
    pub pool_address: Option<String>,
    /// DEX ID, e.g. `uniswap_v2`
    pub pool_dex: Option<String>,
    pub block_number: Option<u64>,
    /// ISO 8601 time
    pub block_timestamp: Option<String>,
    pub tx_hash: Option<String>,
    pub tx_from_address: Option<String>,
    /// `buy` or `sell`
    pub kind: Option<String>,
    pub from_token_address: Option<String>,
    pub to_token_address: Option<String>,
    pub from_token_amount: Option<String>,
    pub to_token_amount: Option<String>,
    pub price_from_in_usd: Option<String>,
    pub price_to_in_usd: Option<String>,
    pub price_from_in_currency_token: Option<String>,
    pub price_to_in_currency_token: Option<String>,
    pub volume_in_usd: Option<String>,
    #[serde(default)]
    pub is_anomalous: Option<bool>,
}

// ---------------------------------------------------------------------------
// PnL
// ---------------------------------------------------------------------------

/// Options for [`super::WalletsApi::pnl`]
#[derive(Debug, Clone, Default)]
pub struct WalletPnlOptions {
    /// `realized_pnl_usd_desc` (API default), `unrealized_pnl_usd_desc`,
    /// `total_buy_usd_desc` or `total_sell_usd_desc`
    pub sort: Option<String>,
    /// Page through results (API default: 1)
    pub page: Option<u32>,
    /// Results per page, 1 to 300 (API default: 100)
    pub per_page: Option<u32>,
}

impl WalletPnlOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sort order
    #[must_use]
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of results per page (1 to 300)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    pub(crate) fn add_to(&self, query: Query) -> Query {
        query
            .text("sort", self.sort.as_deref())
            .value("page", self.page)
            .value("per_page", self.per_page)
    }
}

/// Profit and loss of a wallet (`/onchain/wallets/{address}/pnl`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletPnlResponse {
    pub data: WalletPnlData,
}

/// Wallet PnL data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletPnlData {
    /// The wallet address
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: Option<String>,
    pub attributes: Option<WalletPnl>,
}

/// A wallet's realized and unrealized PnL in USD
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletPnl {
    pub wallet_address: Option<String>,
    pub total_realized_pnl_usd: Option<String>,
    pub total_unrealized_pnl_usd: Option<String>,
    pub total_tokens: Option<u64>,
    /// PnL per network
    #[serde(default)]
    pub networks: Vec<WalletNetworkPnl>,
    /// PnL per token
    #[serde(default)]
    pub token_stats: Vec<WalletTokenPnl>,
}

/// A wallet's PnL on one network
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletNetworkPnl {
    pub network: String,
    pub realized_pnl_usd: Option<String>,
    pub unrealized_pnl_usd: Option<String>,
    /// Number of tokens traded
    pub tokens: Option<u64>,
}

/// A wallet's PnL and trading stats for one token
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletTokenPnl {
    pub network: Option<String>,
    pub address: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u32>,
    pub realized_pnl_usd: Option<String>,
    /// `None` when the wallet no longer holds the token or it has no price
    pub unrealized_pnl_usd: Option<String>,
    pub average_buy_price_usd: Option<String>,
    pub average_sell_price_usd: Option<String>,
    pub total_buy_count: Option<u64>,
    pub total_sell_count: Option<u64>,
    pub total_buy_token_amount: Option<String>,
    pub total_sell_token_amount: Option<String>,
    pub total_buy_usd: Option<String>,
    pub total_sell_usd: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfers_options_query() {
        assert_eq!(WalletTransfersOptions::new().to_query_string(), "");
        let q = WalletTransfersOptions::new()
            .token("0xabc")
            .range("2026-09-01", "2026-09-08T12:00")
            .direction("in")
            .cursor("a+b=")
            .per_page(300)
            .to_query_string();
        assert_eq!(
            q,
            "?token=0xabc&from=2026-09-01&to=2026-09-08T12%3A00&direction=in&cursor=a%2Bb%3D&per_page=300"
        );
    }

    #[test]
    fn trades_options_query() {
        assert_eq!(WalletTradesOptions::new().to_query_string(), "");
        let q = WalletTradesOptions::new()
            .token("0xabc")
            .range("1788328247", "1788933047")
            .include_anomalous(true)
            .cursor("c")
            .per_page(10)
            .to_query_string();
        assert_eq!(
            q,
            "?token=0xabc&from=1788328247&to=1788933047&include_anomalous=true&cursor=c&per_page=10"
        );
    }

    #[test]
    fn nullable_values_read_as_none() {
        let b: WalletTokenBalance = serde_json::from_str(
            r#"{"network":"eth","address":"0x1","token_type":"erc20","decimals":18,
                "coingecko_coin_id":null,"balance_raw":"5","balance":null,
                "price_usd":null,"value_usd":null,"h24_price_change_percentage":null,
                "total_reserve_in_usd":null}"#,
        )
        .unwrap();
        assert_eq!(b.balance_raw.as_deref(), Some("5"));
        assert!(b.balance.is_none() && b.value_usd.is_none());
        let t: WalletTokenPnl =
            serde_json::from_str(r#"{"realized_pnl_usd":"1.5","unrealized_pnl_usd":null}"#)
                .unwrap();
        assert!(t.unrealized_pnl_usd.is_none());
    }
}
