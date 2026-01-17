//! Types for the Token API

use serde::{Deserialize, Serialize};

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// Token address
    pub address: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Token logo URL
    pub logo: Option<String>,
    /// Token thumbnail URL
    pub thumbnail: Option<String>,
    /// Block number when the token was created
    pub block_number: Option<String>,
    /// Validated status
    pub validated: Option<i32>,
    /// Created at timestamp
    pub created_at: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
}

/// Token price
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    /// Token address
    pub token_address: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// USD price formatted
    pub usd_price_formatted: Option<String>,
    /// 24h price change percentage
    #[serde(rename = "24hrPercentChange")]
    pub percent_change_24h: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// Exchange address
    pub exchange_address: Option<String>,
    /// Native price
    pub native_price: Option<NativePrice>,
}

/// Native price info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePrice {
    /// Value
    pub value: Option<String>,
    /// Decimals
    pub decimals: Option<u8>,
    /// Name
    pub name: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Address
    pub address: Option<String>,
}

/// Token transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    /// Transaction hash
    pub transaction_hash: String,
    /// Token address
    pub address: String,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// From address
    pub from_address: String,
    /// To address
    pub to_address: String,
    /// Value
    pub value: String,
    /// Log index
    pub log_index: Option<i32>,
    /// Possible spam
    pub possible_spam: Option<bool>,
}

/// Token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    /// Pair address
    pub pair_address: Option<String>,
    /// Pair label
    pub pair_label: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// Exchange logo
    pub exchange_logo: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// USD price 24hr change
    #[serde(rename = "usdPrice24hrPercentChange")]
    pub usd_price_24hr_percent_change: Option<f64>,
    /// Liquidity in USD
    pub liquidity_usd: Option<f64>,
}

/// Top token holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    /// Holder address
    pub owner: String,
    /// Balance
    pub balance: String,
    /// Balance formatted
    pub balance_formatted: Option<String>,
    /// Is contract
    pub is_contract: Option<bool>,
    /// USD value
    pub usd_value: Option<f64>,
    /// Percentage of total supply
    pub percentage_relative_to_total_supply: Option<f64>,
}

/// Paginated token holders response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHoldersResponse {
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<TokenHolder>,
}

/// Token swap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSwap {
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Pair address
    pub pair_address: Option<String>,
    /// Pair label
    pub pair_label: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// Token 0 address
    pub token0_address: Option<String>,
    /// Token 1 address
    pub token1_address: Option<String>,
    /// Amount 0 in
    pub amount0_in: Option<String>,
    /// Amount 1 in
    pub amount1_in: Option<String>,
    /// Amount 0 out
    pub amount0_out: Option<String>,
    /// Amount 1 out
    pub amount1_out: Option<String>,
    /// USD value
    pub total_value_usd: Option<f64>,
    /// Wallet address
    pub wallet_address: Option<String>,
}

/// Token stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    /// Token address
    pub token_address: Option<String>,
    /// Total supply
    pub total_supply: Option<String>,
    /// Total supply formatted
    pub total_supply_formatted: Option<String>,
    /// Circulating supply
    pub circulating_supply: Option<String>,
    /// Market cap USD
    pub market_cap_usd: Option<f64>,
    /// Fully diluted valuation
    pub fully_diluted_valuation: Option<f64>,
    /// Holders count
    pub holders_count: Option<i64>,
    /// Transfer count
    pub transfer_count: Option<i64>,
}

/// Token search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSearchResult {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Token decimals
    pub token_decimals: Option<u8>,
    /// Chain
    pub chain: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// Market cap USD
    pub market_cap_usd: Option<f64>,
    /// Liquidity USD
    pub liquidity_usd: Option<f64>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// Verified
    pub verified: Option<bool>,
    /// Security score
    pub security_score: Option<i32>,
}

/// Trending token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingToken {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// Price change 24h
    pub price_change_24h: Option<f64>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Rank
    pub rank: Option<i32>,
}

/// Pair OHLCV data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairOhlcv {
    /// Timestamp
    pub timestamp: Option<String>,
    /// Open price
    pub open: Option<f64>,
    /// High price
    pub high: Option<f64>,
    /// Low price
    pub low: Option<f64>,
    /// Close price
    pub close: Option<f64>,
    /// Volume
    pub volume: Option<f64>,
}

/// Pair stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairStats {
    /// Pair address
    pub pair_address: Option<String>,
    /// Pair label
    pub pair_label: Option<String>,
    /// Token 0 address
    pub token0_address: Option<String>,
    /// Token 1 address
    pub token1_address: Option<String>,
    /// Reserve 0
    pub reserve0: Option<String>,
    /// Reserve 1
    pub reserve1: Option<String>,
    /// Liquidity USD
    pub liquidity_usd: Option<f64>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Price change 24h
    pub price_change_24h: Option<f64>,
    /// Buys 24h
    pub buys_24h: Option<i64>,
    /// Sells 24h
    pub sells_24h: Option<i64>,
    /// Buyers 24h
    pub buyers_24h: Option<i64>,
    /// Sellers 24h
    pub sellers_24h: Option<i64>,
}

/// Token category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCategory {
    /// Category ID
    pub id: Option<String>,
    /// Category name
    pub name: Option<String>,
    /// Category description
    pub description: Option<String>,
}

/// New token on exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewToken {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Created at
    pub created_at: Option<String>,
    /// Pair address
    pub pair_address: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// Liquidity USD
    pub liquidity_usd: Option<f64>,
}

/// Paginated token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse<T> {
    /// Cursor
    pub cursor: Option<String>,
    /// Page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<T>,
}

/// Request for batch token prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMultiplePricesRequest {
    /// Token addresses to fetch prices for
    pub tokens: Vec<TokenAddressInput>,
}

/// Token address input for batch requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAddressInput {
    /// Token address
    pub token_address: String,
    /// Exchange (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
}

/// Request for tokens by symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTokensBySymbolsRequest {
    /// Token symbols to fetch
    pub symbols: Vec<String>,
}

/// Token holders summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenHoldersSummary {
    /// Total holders
    pub total_holders: Option<i64>,
    /// Holders change 24h
    pub holders_change_24h: Option<i64>,
    /// Holders change percentage 24h
    pub holders_change_percent_24h: Option<f64>,
}

/// Historical holders data point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalHolders {
    /// Timestamp
    pub timestamp: Option<String>,
    /// Total holders
    pub total_holders: Option<i64>,
}

/// Aggregated token pair stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedPairStats {
    /// Total pairs
    pub total_pairs: Option<i32>,
    /// Total liquidity USD
    pub total_liquidity_usd: Option<f64>,
    /// Total volume 24h USD
    pub total_volume_24h_usd: Option<f64>,
    /// Top pairs
    pub top_pairs: Option<Vec<PairStats>>,
}

/// Top trader for a token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopTrader {
    /// Wallet address
    pub wallet_address: Option<String>,
    /// Realized profit USD
    pub realized_profit_usd: Option<f64>,
    /// Unrealized profit USD
    pub unrealized_profit_usd: Option<f64>,
    /// Total profit USD
    pub total_profit_usd: Option<f64>,
    /// Total tokens bought
    pub total_tokens_bought: Option<String>,
    /// Total tokens sold
    pub total_tokens_sold: Option<String>,
    /// Average buy price USD
    pub avg_buy_price_usd: Option<f64>,
    /// Average sell price USD
    pub avg_sell_price_usd: Option<f64>,
    /// Trade count
    pub trade_count: Option<i64>,
}

/// Pair sniper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairSniper {
    /// Wallet address
    pub wallet_address: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Amount bought
    pub amount_bought: Option<String>,
    /// USD value
    pub usd_value: Option<f64>,
    /// Profit USD
    pub profit_usd: Option<f64>,
}

/// Token bonding status (for pump.fun, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBondingStatus {
    /// Token address
    pub token_address: Option<String>,
    /// Is bonding
    pub is_bonding: Option<bool>,
    /// Has graduated
    pub graduated: Option<bool>,
    /// Bonding progress percentage
    pub bonding_progress: Option<f64>,
    /// Bonding curve address
    pub bonding_curve_address: Option<String>,
    /// Market cap USD
    pub market_cap_usd: Option<f64>,
}
