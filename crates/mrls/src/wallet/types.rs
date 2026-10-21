//! Types for the Wallet API

use crate::de::{string_or_f64, string_or_int};
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

/// Deserialize a value that could be either a number or a nested object
/// containing a "total" field. Moralis API returns wallet stats as nested objects
/// like `{"nfts": {"total": 5}}` rather than flat integers.
fn int_or_nested_total<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntOrNestedVisitor;

    impl<'de> Visitor<'de> for IntOrNestedVisitor {
        type Value = Option<i64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an integer, a string containing an integer, or a map with a 'total' field")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as i64))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse().map(Some).map_err(de::Error::custom)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut total: Option<i64> = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "total" {
                    total = Some(map.next_value::<i64>()?);
                } else {
                    let _: serde_json::Value = map.next_value()?;
                }
            }
            Ok(total)
        }
    }

    deserializer.deserialize_any(IntOrNestedVisitor)
}

/// Native balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeBalance {
    /// Balance in wei
    pub balance: String,
}

/// Token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Token address
    pub token_address: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token logo URL
    pub logo: Option<String>,
    /// Token thumbnail URL
    pub thumbnail: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Balance (raw)
    pub balance: String,
    /// USD price
    pub usd_price: Option<f64>,
    /// USD value
    pub usd_value: Option<f64>,
    /// Possible spam
    pub possible_spam: Option<bool>,
}

/// Wallet transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    /// Transaction hash
    pub hash: String,
    /// Nonce
    pub nonce: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// From address
    pub from_address: String,
    /// To address
    pub to_address: Option<String>,
    /// Value in wei
    pub value: String,
    /// Gas
    pub gas: Option<String>,
    /// Gas price
    pub gas_price: Option<String>,
    /// Input data
    pub input: Option<String>,
    /// Receipt status
    pub receipt_status: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Current page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Results
    pub result: Vec<T>,
}

/// Net worth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorth {
    /// Total net worth in USD
    pub total_networth_usd: String,
    /// Chains breakdown
    pub chains: Vec<ChainNetWorth>,
}

/// Chain-specific net worth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNetWorth {
    /// Chain identifier
    pub chain: String,
    /// Native balance in USD
    pub native_balance_usd: String,
    /// Token balance in USD
    pub token_balance_usd: String,
    /// Total net worth in USD
    pub networth_usd: String,
}

/// Active chains response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveChains {
    /// Address
    pub address: String,
    /// List of active chains
    pub active_chains: Vec<ActiveChain>,
}

/// Active chain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveChain {
    /// Chain identifier
    pub chain: String,
    /// Chain ID
    pub chain_id: String,
    /// First transaction details
    pub first_transaction: Option<TransactionInfo>,
    /// Last transaction details
    pub last_transaction: Option<TransactionInfo>,
}

/// Basic transaction info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
}

/// Token approval (`/wallets/{address}/approvals`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenApproval {
    /// Block number of the approval
    #[serde(default, deserialize_with = "string_or_int")]
    pub block_number: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Approved amount (raw)
    pub value: Option<String>,
    /// Approved amount (formatted)
    pub value_formatted: Option<String>,
    /// Approved token
    pub token: Option<ApprovalToken>,
    /// Approved spender
    pub spender: Option<ApprovalSpender>,
}

/// Token in a [`TokenApproval`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalToken {
    /// Token address
    pub address: Option<String>,
    /// Address label
    pub address_label: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token logo
    pub logo: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// Verified contract
    pub verified_contract: Option<bool>,
    /// Current wallet balance (raw)
    pub current_balance: Option<String>,
    /// Current wallet balance (formatted)
    pub current_balance_formatted: Option<String>,
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_price: Option<f64>,
    /// USD value at risk
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_at_risk: Option<f64>,
}

/// Spender in a [`TokenApproval`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSpender {
    /// Spender address
    pub address: Option<String>,
    /// Address label
    pub address_label: Option<String>,
    /// Entity name
    pub entity: Option<String>,
    /// Entity logo
    pub entity_logo: Option<String>,
}

/// Wallet history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletHistoryEntry {
    /// Transaction hash
    #[serde(alias = "hash")]
    pub hash: Option<String>,
    /// From address
    #[serde(alias = "from_address")]
    pub from_address: Option<String>,
    /// To address
    #[serde(alias = "to_address")]
    pub to_address: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Block number
    #[serde(alias = "block_number")]
    pub block_number: Option<String>,
    /// Block timestamp
    #[serde(alias = "block_timestamp")]
    pub block_timestamp: Option<String>,
    /// Category (send, receive, token send, etc)
    pub category: Option<String>,
    /// Summary
    pub summary: Option<String>,
    /// Possible spam
    #[serde(alias = "possible_spam")]
    pub possible_spam: Option<bool>,
    /// NFT transfers
    #[serde(default, alias = "nft_transfers")]
    pub nft_transfers: Option<Vec<serde_json::Value>>,
    /// ERC20 transfers
    #[serde(default, alias = "erc20_transfers")]
    pub erc20_transfers: Option<Vec<serde_json::Value>>,
    /// Native transfers
    #[serde(default, alias = "native_transfers")]
    pub native_transfers: Option<Vec<serde_json::Value>>,
}

/// Wallet stats
///
/// Moralis API returns stats as nested objects like `{"nfts": {"total": 5}}`
/// or as flat integers depending on the endpoint version. This struct handles both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStats {
    /// Address
    pub address: Option<String>,
    /// NFTs owned
    #[serde(default, deserialize_with = "int_or_nested_total", alias = "nfts")]
    pub nfts_owned: Option<i64>,
    /// Collections owned
    #[serde(
        default,
        deserialize_with = "int_or_nested_total",
        alias = "collections"
    )]
    pub collections_owned: Option<i64>,
    /// NFT transfers
    #[serde(
        default,
        deserialize_with = "int_or_nested_total",
        alias = "nftTransfers"
    )]
    pub nft_transfers: Option<i64>,
    /// Token transfers
    #[serde(
        default,
        deserialize_with = "int_or_nested_total",
        alias = "tokenTransfers"
    )]
    pub token_transfers: Option<i64>,
    /// Transactions count
    #[serde(
        default,
        deserialize_with = "int_or_nested_total",
        alias = "transactions"
    )]
    pub transactions_count: Option<i64>,
}

/// Wallet profitability summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletProfitability {
    /// Total profit USD
    #[serde(alias = "total_realized_profit_usd")]
    pub total_realized_profit_usd: Option<f64>,
    /// Total loss USD
    #[serde(alias = "total_realized_loss_usd")]
    pub total_realized_loss_usd: Option<f64>,
    /// Total count profitable
    #[serde(alias = "total_count_of_profitable_trades")]
    pub total_count_of_profitable_trades: Option<i64>,
    /// Total count losing
    #[serde(alias = "total_count_of_losing_trades")]
    pub total_count_of_losing_trades: Option<i64>,
    /// Total count of trades
    #[serde(alias = "total_count_of_trades")]
    pub total_count_of_trades: Option<i64>,
}

/// Token profitability detail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenProfitability {
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
    /// Realized profit USD
    #[serde(alias = "realized_profit_usd")]
    pub realized_profit_usd: Option<f64>,
    /// Average buy price USD
    #[serde(alias = "avg_buy_price_usd")]
    pub avg_buy_price_usd: Option<f64>,
    /// Average sell price USD
    #[serde(alias = "avg_sell_price_usd")]
    pub avg_sell_price_usd: Option<f64>,
    /// Total tokens bought
    #[serde(alias = "total_tokens_bought")]
    pub total_tokens_bought: Option<String>,
    /// Total tokens sold
    #[serde(alias = "total_tokens_sold")]
    pub total_tokens_sold: Option<String>,
    /// Count of trades
    #[serde(alias = "count_of_trades")]
    pub count_of_trades: Option<i64>,
}

/// Request for multi-wallet balances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletBalancesRequest {
    /// Wallet addresses to fetch balances for
    pub wallet_addresses: Vec<String>,
}

/// Native balances of several wallets on one chain (`GET /wallets/balances`
/// returns one entry per chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalances {
    /// Chain name
    pub chain: Option<String>,
    /// Chain ID
    #[serde(default, deserialize_with = "string_or_int")]
    pub chain_id: Option<String>,
    /// Block number the balances are at
    #[serde(default, deserialize_with = "string_or_int")]
    pub block_number: Option<String>,
    /// Block timestamp
    #[serde(default, deserialize_with = "string_or_int")]
    pub block_timestamp: Option<String>,
    /// Sum of all wallets' balances (raw)
    pub total_balance: Option<String>,
    /// Sum of all wallets' balances (formatted)
    pub total_balance_formatted: Option<String>,
    /// Per-wallet native balances
    #[serde(default)]
    pub wallet_balances: Vec<WalletNativeBalance>,
}

/// One wallet's native balance in [`WalletBalances`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletNativeBalance {
    /// Wallet address
    pub address: Option<String>,
    /// Balance (raw)
    pub balance: Option<String>,
    /// Balance (formatted)
    pub balance_formatted: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_stats_nested_objects() {
        // Moralis returns stats as nested objects like {"nfts": {"total": 5}}
        let json = r#"{
            "address": "0xvitalik",
            "nfts_owned": {"total": 42},
            "collections_owned": {"total": 10},
            "nft_transfers": {"total": 100},
            "token_transfers": {"total": 500},
            "transactions_count": {"total": 1000}
        }"#;
        let stats: WalletStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.address, Some("0xvitalik".to_string()));
        assert_eq!(stats.nfts_owned, Some(42));
        assert_eq!(stats.collections_owned, Some(10));
        assert_eq!(stats.nft_transfers, Some(100));
        assert_eq!(stats.token_transfers, Some(500));
        assert_eq!(stats.transactions_count, Some(1000));
    }

    #[test]
    fn test_wallet_stats_flat_integers() {
        // Should also handle flat integer values
        let json = r#"{
            "address": "0xvitalik",
            "nfts_owned": 42,
            "collections_owned": 10,
            "nft_transfers": 100,
            "token_transfers": 500,
            "transactions_count": 1000
        }"#;
        let stats: WalletStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.nfts_owned, Some(42));
        assert_eq!(stats.transactions_count, Some(1000));
    }

    #[test]
    fn test_wallet_stats_with_aliases() {
        // API may use "nfts", "collections", "transactions" as field names
        let json = r#"{
            "address": "0xvitalik",
            "nfts": {"total": 42},
            "collections": {"total": 10},
            "nftTransfers": {"total": 100},
            "tokenTransfers": {"total": 500},
            "transactions": {"total": 1000}
        }"#;
        let stats: WalletStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.nfts_owned, Some(42));
        assert_eq!(stats.collections_owned, Some(10));
        assert_eq!(stats.transactions_count, Some(1000));
    }

    #[test]
    fn test_multiple_balances_per_chain_entry() {
        let json = r#"[{
            "chain": "eth",
            "chain_id": "0x1",
            "block_number": "123",
            "block_timestamp": "2026-01-01T00:00:00Z",
            "total_balance": "3000",
            "total_balance_formatted": "0.000000000000003",
            "wallet_balances": [
                {"address": "0x0000000000000000000000000000000000000001", "balance": "1000", "balance_formatted": "0.000000000000001"},
                {"address": "0x0000000000000000000000000000000000000002", "balance": "2000", "balance_formatted": "0.000000000000002"}
            ]
        }]"#;
        let resp: Vec<WalletBalances> = serde_json::from_str(json).unwrap();
        assert_eq!(resp[0].chain_id.as_deref(), Some("0x1"));
        assert_eq!(resp[0].wallet_balances.len(), 2);
        assert_eq!(resp[0].wallet_balances[1].balance.as_deref(), Some("2000"));
    }

    #[test]
    fn test_token_approval_nested_token_and_spender() {
        let json = r#"{
            "block_number": 12345,
            "block_timestamp": "2026-01-01T00:00:00.000Z",
            "transaction_hash": "0x01",
            "value": "1000000",
            "value_formatted": "1.0",
            "token": {
                "address": "0x0000000000000000000000000000000000000001",
                "name": "Test Token",
                "symbol": "TST",
                "possible_spam": false,
                "current_balance": "5000000",
                "usd_price": "1.0",
                "usd_at_risk": 1.0
            },
            "spender": {
                "address": "0x0000000000000000000000000000000000000002",
                "address_label": "Test Router",
                "entity": "Test DEX"
            }
        }"#;
        let approval: TokenApproval = serde_json::from_str(json).unwrap();
        assert_eq!(approval.block_number.as_deref(), Some("12345"));
        let token = approval.token.unwrap();
        assert_eq!(token.symbol.as_deref(), Some("TST"));
        assert_eq!(token.usd_at_risk, Some(1.0));
        let spender = approval.spender.unwrap();
        assert_eq!(spender.entity.as_deref(), Some("Test DEX"));
    }

    #[test]
    fn test_wallet_profitability_camel_case() {
        let json = r#"{
            "totalRealizedProfitUsd": 1000.0,
            "totalRealizedLossUsd": 500.0,
            "totalCountOfProfitableTrades": 10,
            "totalCountOfLosingTrades": 5,
            "totalCountOfTrades": 15
        }"#;
        let profit: WalletProfitability = serde_json::from_str(json).unwrap();
        assert_eq!(profit.total_realized_profit_usd, Some(1000.0));
        assert_eq!(profit.total_count_of_trades, Some(15));
    }

    #[test]
    fn test_token_profitability_camel_case() {
        let json = r#"{
            "tokenAddress": "0xtoken",
            "tokenName": "Test",
            "tokenSymbol": "TST",
            "realizedProfitUsd": 500.0,
            "avgBuyPriceUsd": 1.0,
            "avgSellPriceUsd": 1.5,
            "countOfTrades": 10
        }"#;
        let profit: TokenProfitability = serde_json::from_str(json).unwrap();
        assert_eq!(profit.token_address, Some("0xtoken".to_string()));
        assert_eq!(profit.realized_profit_usd, Some(500.0));
        assert_eq!(profit.count_of_trades, Some(10));
    }

    #[test]
    fn test_wallet_history_camel_case() {
        let json = r#"{
            "hash": "0xabc",
            "fromAddress": "0xfrom",
            "toAddress": "0xto",
            "value": "1000",
            "blockNumber": "18000000",
            "blockTimestamp": "2024-01-01",
            "category": "send",
            "possibleSpam": false,
            "nftTransfers": [],
            "erc20Transfers": [],
            "nativeTransfers": []
        }"#;
        let entry: WalletHistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.from_address, Some("0xfrom".to_string()));
        assert_eq!(entry.to_address, Some("0xto".to_string()));
        assert_eq!(entry.category, Some("send".to_string()));
    }
}
