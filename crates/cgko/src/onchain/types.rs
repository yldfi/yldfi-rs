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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolInfoResponse {
    pub data: PoolInfo,
}

/// Pool info data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub pool_type: Option<String>,
    pub attributes: Option<PoolInfoAttributes>,
}

/// Pool info attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolInfoAttributes {
    pub base_token_id: Option<String>,
    pub base_token_name: Option<String>,
    pub base_token_symbol: Option<String>,
    pub quote_token_id: Option<String>,
    pub quote_token_name: Option<String>,
    pub quote_token_symbol: Option<String>,
}

/// Token holders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHoldersResponse {
    pub data: Vec<TokenHolder>,
}

/// Token holder
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHolder {
    pub id: String,
    #[serde(rename = "type")]
    pub holder_type: Option<String>,
    pub attributes: Option<TokenHolderAttributes>,
}

/// Token holder attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHolderAttributes {
    pub wallet_address: Option<String>,
    pub balance: Option<String>,
    pub percentage: Option<f64>,
    pub value_usd: Option<String>,
}

/// Token traders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTradersResponse {
    pub data: Vec<TokenTrader>,
}

/// Token trader
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTrader {
    pub id: String,
    #[serde(rename = "type")]
    pub trader_type: Option<String>,
    pub attributes: Option<TokenTraderAttributes>,
}

/// Token trader attributes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenTraderAttributes {
    pub wallet_address: Option<String>,
    pub bought_volume_usd: Option<String>,
    pub sold_volume_usd: Option<String>,
    pub bought_count: Option<u64>,
    pub sold_count: Option<u64>,
}

/// Holders chart response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldersChartResponse {
    pub data: HoldersChartData,
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
    /// Vec of [timestamp, holder_count]
    pub holders_chart: Option<Vec<(u64, u64)>>,
}

/// Token OHLCV response (for token, not pool)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenOhlcvResponse {
    pub data: TokenOhlcvData,
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
}

impl MegafilterOptions {
    pub fn new() -> Self {
        Self::default()
    }

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
            params.push(format!("page={}", page));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
