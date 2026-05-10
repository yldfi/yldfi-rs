//! Types for the Token API

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

/// Deserialize a value that could be either a float, integer, or a string containing a number.
/// Moralis API sometimes returns numeric values as strings.
fn string_or_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| de::Error::custom("number cannot be represented as f64")),
        serde_json::Value::String(value) => {
            if value.is_empty() || value == "null" {
                Ok(None)
            } else {
                value.parse().map(Some).map_err(de::Error::custom)
            }
        }
        value => Err(de::Error::custom(format!(
            "expected a float, integer, or string containing a number, got {value}"
        ))),
    }
}

/// Deserialize a value that could be either a string or an integer.
/// Moralis API sometimes returns block numbers and similar fields as integers
/// rather than strings.
fn string_or_int<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> Visitor<'de> for StringOrIntVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string or an integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// Token address
    pub address: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals (API may return as string or integer)
    #[serde(default, deserialize_with = "string_or_int")]
    pub decimals: Option<String>,
    /// Token logo URL
    pub logo: Option<String>,
    /// Token thumbnail URL
    pub thumbnail: Option<String>,
    /// Block number when the token was created
    #[serde(default, deserialize_with = "string_or_int")]
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
    #[serde(default, deserialize_with = "string_or_int")]
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

/// Paginated token transfer response (Moralis API wraps transfers in an object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransferResponse {
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    #[serde(default)]
    pub result: Vec<TokenTransfer>,
}

/// Token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    /// Pair address
    #[serde(alias = "pair_address")]
    pub pair_address: Option<String>,
    /// Pair label
    #[serde(alias = "pair_label")]
    pub pair_label: Option<String>,
    /// Exchange name
    #[serde(alias = "exchange_name")]
    pub exchange_name: Option<String>,
    /// Exchange logo
    #[serde(alias = "exchange_logo")]
    pub exchange_logo: Option<String>,
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64", alias = "usd_price")]
    pub usd_price: Option<f64>,
    /// USD price 24hr change
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        rename = "usdPrice24hrPercentChange",
        alias = "usd_price_24hr_percent_change"
    )]
    pub usd_price_24hr_percent_change: Option<f64>,
    /// Liquidity in USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "liquidity_usd")]
    pub liquidity_usd: Option<f64>,
}

/// Token pairs response (Moralis API returns pairs in an object wrapper)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPairsResponse {
    /// The pairs array
    pub pairs: Vec<TokenPair>,
}

/// Top token holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    /// Holder address
    pub owner_address: String,
    /// Holder address label
    pub owner_address_label: Option<String>,
    /// Entity name
    pub entity: Option<String>,
    /// Entity logo URL
    pub entity_logo: Option<String>,
    /// Balance (raw)
    pub balance: String,
    /// Balance formatted
    pub balance_formatted: Option<String>,
    /// Is contract
    pub is_contract: Option<bool>,
    /// USD value (Moralis returns as string)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_value: Option<f64>,
    /// Percentage of total supply
    #[serde(default, deserialize_with = "string_or_f64")]
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
    #[serde(alias = "transaction_hash")]
    pub transaction_hash: Option<String>,
    /// Block timestamp
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: Option<String>,
    /// Block number (Moralis may return as integer or string)
    #[serde(default, deserialize_with = "string_or_int", alias = "block_number")]
    pub block_number: Option<String>,
    /// Pair address
    #[serde(alias = "pair_address")]
    pub pair_address: Option<String>,
    /// Pair label
    #[serde(alias = "pair_label")]
    pub pair_label: Option<String>,
    /// Exchange name
    #[serde(alias = "exchange_name")]
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
    #[serde(alias = "total_value_usd")]
    pub total_value_usd: Option<f64>,
    /// Wallet address
    #[serde(alias = "wallet_address")]
    pub wallet_address: Option<String>,
}

/// Token stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    /// Token address
    #[serde(alias = "token_address")]
    pub token_address: Option<String>,
    /// Total supply
    #[serde(alias = "total_supply")]
    pub total_supply: Option<String>,
    /// Total supply formatted
    #[serde(alias = "total_supply_formatted")]
    pub total_supply_formatted: Option<String>,
    /// Circulating supply
    #[serde(alias = "circulating_supply")]
    pub circulating_supply: Option<String>,
    /// Market cap USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "market_cap_usd")]
    pub market_cap_usd: Option<f64>,
    /// Fully diluted valuation
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "fully_diluted_valuation"
    )]
    pub fully_diluted_valuation: Option<f64>,
    /// Holders count
    #[serde(alias = "holders_count")]
    pub holders_count: Option<i64>,
    /// Transfer count
    #[serde(alias = "transfer_count")]
    pub transfer_count: Option<i64>,
}

/// Token search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSearchResult {
    /// Token address
    #[serde(alias = "token_address")]
    pub token_address: Option<String>,
    /// Token name
    #[serde(alias = "token_name")]
    pub token_name: Option<String>,
    /// Token symbol
    #[serde(alias = "token_symbol")]
    pub token_symbol: Option<String>,
    /// Token logo
    #[serde(alias = "token_logo")]
    pub token_logo: Option<String>,
    /// Token decimals
    #[serde(alias = "token_decimals")]
    pub token_decimals: Option<u8>,
    /// Chain
    pub chain: Option<String>,
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64", alias = "usd_price")]
    pub usd_price: Option<f64>,
    /// Market cap USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "market_cap_usd")]
    pub market_cap_usd: Option<f64>,
    /// Liquidity USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "liquidity_usd")]
    pub liquidity_usd: Option<f64>,
    /// Possible spam
    #[serde(alias = "possible_spam")]
    pub possible_spam: Option<bool>,
    /// Verified
    pub verified: Option<bool>,
    /// Security score
    #[serde(alias = "security_score")]
    pub security_score: Option<i32>,
}

/// Trending token
///
/// Moralis API may return field names with or without the "token" prefix
/// (e.g., "name" vs "tokenName", "symbol" vs "tokenSymbol").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingToken {
    /// Token address
    #[serde(alias = "token_address", alias = "address")]
    pub token_address: Option<String>,
    /// Token name
    #[serde(alias = "token_name", alias = "name")]
    pub token_name: Option<String>,
    /// Token symbol
    #[serde(alias = "token_symbol", alias = "symbol")]
    pub token_symbol: Option<String>,
    /// Token logo
    #[serde(alias = "token_logo", alias = "logo")]
    pub token_logo: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64", alias = "usd_price")]
    pub usd_price: Option<f64>,
    /// Price change 24h
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "price_change_24h"
    )]
    pub price_change_24h: Option<f64>,
    /// Volume 24h USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "volume_24h_usd")]
    pub volume_24h_usd: Option<f64>,
    /// Rank
    pub rank: Option<i32>,
}

/// Pair OHLCV data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairOhlcv {
    /// Timestamp
    #[serde(alias = "timestamp")]
    pub timestamp: Option<String>,
    /// Open price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub open: Option<f64>,
    /// High price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub high: Option<f64>,
    /// Low price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub low: Option<f64>,
    /// Close price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub close: Option<f64>,
    /// Volume
    #[serde(default, deserialize_with = "string_or_f64")]
    pub volume: Option<f64>,
}

/// Pair stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairStats {
    /// Pair address
    #[serde(alias = "pair_address")]
    pub pair_address: Option<String>,
    /// Pair label
    #[serde(alias = "pair_label")]
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
    #[serde(default, deserialize_with = "string_or_f64", alias = "liquidity_usd")]
    pub liquidity_usd: Option<f64>,
    /// Volume 24h USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "volume_24h_usd")]
    pub volume_24h_usd: Option<f64>,
    /// Price change 24h
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "price_change_24h"
    )]
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
    #[serde(alias = "total_pairs")]
    pub total_pairs: Option<i32>,
    /// Total liquidity USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "total_liquidity_usd"
    )]
    pub total_liquidity_usd: Option<f64>,
    /// Total volume 24h USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "total_volume_24h_usd"
    )]
    pub total_volume_24h_usd: Option<f64>,
    /// Top pairs
    #[serde(default, alias = "top_pairs")]
    pub top_pairs: Option<Vec<PairStats>>,
}

/// Top trader for a token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopTrader {
    /// Wallet address
    #[serde(alias = "wallet_address")]
    pub wallet_address: Option<String>,
    /// Realized profit USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "realized_profit_usd"
    )]
    pub realized_profit_usd: Option<f64>,
    /// Unrealized profit USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "unrealized_profit_usd"
    )]
    pub unrealized_profit_usd: Option<f64>,
    /// Total profit USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "total_profit_usd"
    )]
    pub total_profit_usd: Option<f64>,
    /// Total tokens bought
    #[serde(alias = "total_tokens_bought")]
    pub total_tokens_bought: Option<String>,
    /// Total tokens sold
    #[serde(alias = "total_tokens_sold")]
    pub total_tokens_sold: Option<String>,
    /// Average buy price USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "avg_buy_price_usd"
    )]
    pub avg_buy_price_usd: Option<f64>,
    /// Average sell price USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "avg_sell_price_usd"
    )]
    pub avg_sell_price_usd: Option<f64>,
    /// Trade count
    #[serde(alias = "trade_count")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_swap_block_number_as_integer() {
        // Moralis returns block_number as integer in swap responses
        let json = r#"{
            "transactionHash": "0xabc",
            "blockTimestamp": "2024-01-01",
            "blockNumber": 24448562,
            "pairAddress": "0xdef",
            "exchangeName": "uniswap"
        }"#;
        let swap: TokenSwap = serde_json::from_str(json).unwrap();
        assert_eq!(swap.block_number, Some("24448562".to_string()));
        assert_eq!(swap.transaction_hash, Some("0xabc".to_string()));
    }

    #[test]
    fn test_token_swap_block_number_as_string() {
        // Block number as string should also work
        let json = r#"{
            "transactionHash": "0xabc",
            "blockNumber": "24448562"
        }"#;
        let swap: TokenSwap = serde_json::from_str(json).unwrap();
        assert_eq!(swap.block_number, Some("24448562".to_string()));
    }

    #[test]
    fn test_token_transfer_response_wrapper() {
        // API returns transfers wrapped in paginated response
        let json = r#"{
            "page": 0,
            "page_size": 100,
            "cursor": null,
            "result": [
                {
                    "transaction_hash": "0xabc",
                    "address": "0xtoken",
                    "from_address": "0xfrom",
                    "to_address": "0xto",
                    "value": "1000000"
                }
            ]
        }"#;
        let response: TokenTransferResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.result.len(), 1);
        assert_eq!(response.result[0].transaction_hash, "0xabc");
    }

    #[test]
    fn test_token_pair_camel_case() {
        let json = r#"{
            "pairAddress": "0xpair",
            "pairLabel": "WETH/USDC",
            "exchangeName": "uniswap",
            "usdPrice": 3500.0,
            "liquidityUsd": 1000000.0
        }"#;
        let pair: TokenPair = serde_json::from_str(json).unwrap();
        assert_eq!(pair.pair_address, Some("0xpair".to_string()));
        assert_eq!(pair.pair_label, Some("WETH/USDC".to_string()));
        assert_eq!(pair.usd_price, Some(3500.0));
    }

    #[test]
    fn test_token_pair_string_numbers() {
        // Some numeric fields may come as strings
        let json = r#"{
            "pairAddress": "0xpair",
            "usdPrice": "3500.50",
            "liquidityUsd": "1000000"
        }"#;
        let pair: TokenPair = serde_json::from_str(json).unwrap();
        assert_eq!(pair.usd_price, Some(3500.50));
        assert_eq!(pair.liquidity_usd, Some(1000000.0));
    }

    #[test]
    fn test_token_stats_camel_case() {
        let json = r#"{
            "tokenAddress": "0xtoken",
            "totalSupply": "1000000",
            "marketCapUsd": 5000000.0,
            "holdersCount": 1500
        }"#;
        let stats: TokenStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.token_address, Some("0xtoken".to_string()));
        assert_eq!(stats.market_cap_usd, Some(5000000.0));
        assert_eq!(stats.holders_count, Some(1500));
    }

    #[test]
    fn test_trending_token_with_name_alias() {
        // API might return "name" instead of "tokenName"
        let json = r#"{
            "tokenAddress": "0xtoken",
            "name": "Test Token",
            "symbol": "TEST",
            "logo": "https://logo.png",
            "usdPrice": 1.5,
            "rank": 1
        }"#;
        let token: TrendingToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.token_name, Some("Test Token".to_string()));
        assert_eq!(token.token_symbol, Some("TEST".to_string()));
        assert_eq!(token.token_logo, Some("https://logo.png".to_string()));
    }

    #[test]
    fn test_trending_token_with_token_prefix() {
        // API might also return "tokenName", "tokenSymbol"
        let json = r#"{
            "tokenAddress": "0xtoken",
            "tokenName": "Test Token",
            "tokenSymbol": "TEST",
            "tokenLogo": "https://logo.png",
            "usdPrice": 1.5
        }"#;
        let token: TrendingToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.token_name, Some("Test Token".to_string()));
        assert_eq!(token.token_symbol, Some("TEST".to_string()));
    }

    #[test]
    fn test_pair_stats_camel_case() {
        let json = r#"{
            "pairAddress": "0xpair",
            "pairLabel": "WETH/USDC",
            "liquidityUsd": 1000000.0,
            "volume24hUsd": 500000.0,
            "buys24h": 100,
            "sells24h": 50
        }"#;
        let stats: PairStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.pair_address, Some("0xpair".to_string()));
        assert_eq!(stats.liquidity_usd, Some(1000000.0));
        assert_eq!(stats.volume_24h_usd, Some(500000.0));
    }

    #[test]
    fn test_pair_ohlcv_string_numbers() {
        let json = r#"{
            "timestamp": "2024-01-01",
            "open": "3500.5",
            "high": "3600.0",
            "low": "3400.0",
            "close": "3550.0",
            "volume": "1000000"
        }"#;
        let ohlcv: PairOhlcv = serde_json::from_str(json).unwrap();
        assert_eq!(ohlcv.open, Some(3500.5));
        assert_eq!(ohlcv.volume, Some(1000000.0));
    }

    #[test]
    fn test_token_metadata_block_number_as_int() {
        let json = r#"{
            "address": "0xtoken",
            "name": "Test",
            "symbol": "TST",
            "block_number": 18000000
        }"#;
        let meta: TokenMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.block_number, Some("18000000".to_string()));
    }

    #[test]
    fn test_string_or_f64_with_empty_string() {
        let json = r#"{"pairAddress": "0x", "usdPrice": ""}"#;
        let pair: TokenPair = serde_json::from_str(json).unwrap();
        assert_eq!(pair.usd_price, None);
    }
}
