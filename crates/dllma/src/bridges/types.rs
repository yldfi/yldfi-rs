//! Types for bridge data (Pro)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for listing bridges
#[derive(Debug, Clone, Default)]
pub struct ListBridgesOptions {
    /// Include chain information in response
    pub include_chains: bool,
}

impl ListBridgesOptions {
    /// Create new options with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Include chain information
    #[must_use]
    pub fn include_chains(mut self) -> Self {
        self.include_chains = true;
        self
    }

    /// Build query string
    #[must_use]
    pub fn to_query_string(&self) -> String {
        if self.include_chains {
            "?includeChains=true".to_string()
        } else {
            String::new()
        }
    }
}

/// Options for querying bridge transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionsOptions {
    /// Maximum number of transactions to return
    pub limit: Option<u32>,
    /// Start timestamp filter
    pub start_timestamp: Option<u64>,
    /// End timestamp filter
    pub end_timestamp: Option<u64>,
    /// Filter by source chain
    pub source_chain: Option<String>,
    /// Filter by address
    pub address: Option<String>,
}

impl TransactionsOptions {
    /// Create new options with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set limit
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set time range
    #[must_use]
    pub fn time_range(mut self, start: u64, end: u64) -> Self {
        self.start_timestamp = Some(start);
        self.end_timestamp = Some(end);
        self
    }

    /// Filter by source chain
    pub fn source_chain(mut self, chain: impl Into<String>) -> Self {
        self.source_chain = Some(chain.into());
        self
    }

    /// Filter by address
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Build query string
    ///
    /// The API expects lowercase parameter names (`starttimestamp`,
    /// `sourcechain`, ...). Values are URL-encoded.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        if let Some(limit) = self.limit {
            q.append_pair("limit", &limit.to_string());
        }
        if let Some(start) = self.start_timestamp {
            q.append_pair("starttimestamp", &start.to_string());
        }
        if let Some(end) = self.end_timestamp {
            q.append_pair("endtimestamp", &end.to_string());
        }
        if let Some(ref chain) = self.source_chain {
            q.append_pair("sourcechain", chain);
        }
        if let Some(ref addr) = self.address {
            q.append_pair("address", addr);
        }
        let query = q.finish();
        if query.is_empty() {
            String::new()
        } else {
            format!("?{query}")
        }
    }
}

/// Bridge summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bridge {
    /// Bridge ID
    pub id: u64,
    /// Bridge name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Logo URL
    pub icon: Option<String>,
    /// URL slug
    pub slug: Option<String>,
    /// Bridge website
    pub url: Option<String>,
    /// Volume on the previous day
    pub volume_prev_day: Option<f64>,
    /// Volume two days ago
    pub volume_prev2_day: Option<f64>,
    /// Volume over the last hour
    pub last_hourly_volume: Option<f64>,
    /// Volume over the last 24h
    pub last24h_volume: Option<f64>,
    /// Volume on the last full day
    pub last_daily_volume: Option<f64>,
    /// Volume on the day before the last full day
    pub day_before_last_volume: Option<f64>,
    /// Volume over the last week
    pub weekly_volume: Option<f64>,
    /// Volume over the last month
    pub monthly_volume: Option<f64>,
    /// Chains supported
    #[serde(default)]
    pub chains: Vec<String>,
    /// Destination chain
    pub destination_chain: Option<String>,
}

/// Bridge list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgesResponse {
    /// List of bridges
    pub bridges: Vec<Bridge>,
}

/// Deposit and withdrawal transaction counts
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TxCounts {
    /// Number of deposit transactions
    pub deposits: Option<u64>,
    /// Number of withdrawal transactions
    pub withdrawals: Option<u64>,
}

/// Volume and transaction stats, used for the bridge as a whole and per chain
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStats {
    /// Volume over the last hour
    pub last_hourly_volume: Option<f64>,
    /// Volume so far today
    pub current_day_volume: Option<f64>,
    /// Volume on the last full day
    pub last_daily_volume: Option<f64>,
    /// Volume on the day before the last full day
    pub day_before_last_volume: Option<f64>,
    /// Volume over the last week
    pub weekly_volume: Option<f64>,
    /// Volume over the last month
    pub monthly_volume: Option<f64>,
    /// Volume over the last 24h (per-chain entries only)
    pub last24h_volume: Option<f64>,
    /// Transactions over the last hour
    pub last_hourly_txs: Option<TxCounts>,
    /// Transactions so far today
    pub current_day_txs: Option<TxCounts>,
    /// Transactions on the previous day
    pub prev_day_txs: Option<TxCounts>,
    /// Transactions on the day before the previous day
    pub day_before_last_txs: Option<TxCounts>,
    /// Transactions over the last week
    pub weekly_txs: Option<TxCounts>,
    /// Transactions over the last month
    pub monthly_txs: Option<TxCounts>,
}

/// Detailed bridge data
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeDetail {
    /// Bridge ID
    pub id: u64,
    /// Bridge name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Destination chain
    pub destination_chain: Option<String>,
    /// Bridge-wide volume and transaction stats (top-level fields)
    #[serde(flatten)]
    pub stats: BridgeStats,
    /// Stats keyed by chain name
    pub chain_breakdown: HashMap<String, BridgeStats>,
}

// Hand-written rather than `#[serde(flatten)]`: flattened numbers fail to
// deserialize with serde_json's `arbitrary_precision` (see `crate::de`).
impl<'de> Deserialize<'de> for BridgeDetail {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Head {
            id: u64,
            name: String,
            display_name: Option<String>,
            destination_chain: Option<String>,
            #[serde(default)]
            chain_breakdown: HashMap<String, BridgeStats>,
        }

        let v = serde_json::Value::deserialize(deserializer)?;
        let head: Head = serde_json::from_value(v.clone()).map_err(D::Error::custom)?;
        let stats: BridgeStats = serde_json::from_value(v).map_err(D::Error::custom)?;
        Ok(Self {
            id: head.id,
            name: head.name,
            display_name: head.display_name,
            destination_chain: head.destination_chain,
            stats,
            chain_breakdown: head.chain_breakdown,
        })
    }
}

/// Bridge volume data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeVolumePoint {
    /// Unix timestamp (sent as a string)
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub date: Option<u64>,
    /// Deposit volume in USD
    #[serde(rename = "depositUSD")]
    pub deposit_usd: Option<f64>,
    /// Withdrawal volume in USD
    #[serde(rename = "withdrawUSD")]
    pub withdraw_usd: Option<f64>,
    /// Number of deposit transactions
    #[serde(rename = "depositTxs")]
    pub deposit_txs: Option<u64>,
    /// Number of withdrawal transactions
    #[serde(rename = "withdrawTxs")]
    pub withdraw_txs: Option<u64>,
}

/// Bridge volume for a chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainBridgeVolume(pub Vec<BridgeVolumePoint>);

/// Token totals in daily bridge stats
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgedTokenTotal {
    /// USD value
    pub usd_value: Option<f64>,
    /// Token amount (decimal string)
    pub amount: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u32>,
}

/// Address totals in daily bridge stats
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgedAddressTotal {
    /// USD value
    pub usd_value: Option<f64>,
    /// Number of transactions
    pub txs: Option<u64>,
}

/// Daily bridge stats
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyBridgeStats {
    /// Day (Unix timestamp)
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub date: Option<u64>,
    /// Tokens deposited, keyed by token identifier
    #[serde(default)]
    pub total_tokens_deposited: HashMap<String, BridgedTokenTotal>,
    /// Tokens withdrawn, keyed by token identifier
    #[serde(default)]
    pub total_tokens_withdrawn: HashMap<String, BridgedTokenTotal>,
    /// Deposits by address, keyed by `chain:address`
    #[serde(default)]
    pub total_address_deposited: HashMap<String, BridgedAddressTotal>,
    /// Withdrawals by address, keyed by `chain:address`
    #[serde(default)]
    pub total_address_withdrawn: HashMap<String, BridgedAddressTotal>,
}

/// Bridge transaction
///
/// Field names are snake_case on the wire, except `sourceChain`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeTransaction {
    /// Transaction hash
    pub tx_hash: String,
    /// Timestamp (as returned, e.g. an ISO date string)
    pub ts: Option<String>,
    /// Block number
    pub tx_block: Option<u64>,
    /// Sender address
    pub tx_from: Option<String>,
    /// Recipient address
    pub tx_to: Option<String>,
    /// Token address
    pub token: Option<String>,
    /// Raw token amount (decimal string)
    pub amount: Option<String>,
    /// Chain the transaction is on
    pub chain: Option<String>,
    /// Bridge name
    pub bridge_name: Option<String>,
    /// Amount in USD
    pub usd_value: Option<f64>,
    /// Source chain
    #[serde(rename = "sourceChain")]
    pub source_chain: Option<String>,
}

/// Bridge transactions response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeTransactionsResponse(pub Vec<BridgeTransaction>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transactions_query_uses_lowercase_names_and_encodes_values() {
        let q = TransactionsOptions::new()
            .limit(200)
            .time_range(1_700_000_000, 1_700_086_400)
            .source_chain("Example Chain")
            .address("ethereum:0x0000000000000000000000000000000000000001")
            .to_query_string();
        assert_eq!(
            q,
            "?limit=200&starttimestamp=1700000000&endtimestamp=1700086400\
             &sourcechain=Example+Chain\
             &address=ethereum%3A0x0000000000000000000000000000000000000001"
        );
        assert_eq!(TransactionsOptions::new().to_query_string(), "");
    }

    #[test]
    fn bridge_transaction_snake_case_fields() {
        let json = r#"[{"tx_hash":"0xabc","ts":"2024-01-01T00:00:00.000Z","tx_block":123,
            "tx_from":"0x01","tx_to":"0x02","token":"0x03","amount":"1000","chain":"ethereum",
            "bridge_name":"foo","usd_value":null,"sourceChain":"Ethereum"}]"#;
        let r: BridgeTransactionsResponse = serde_json::from_str(json).unwrap();
        let tx = &r.0[0];
        assert_eq!(tx.tx_hash, "0xabc");
        assert_eq!(tx.ts.as_deref(), Some("2024-01-01T00:00:00.000Z"));
        assert_eq!(tx.tx_block, Some(123));
        assert_eq!(tx.amount.as_deref(), Some("1000"));
        assert_eq!(tx.source_chain.as_deref(), Some("Ethereum"));
        assert!(tx.usd_value.is_none());
    }

    #[test]
    fn chain_volume_reads_usd_fields_and_string_date() {
        let json = r#"[{"date":"1700000000","depositUSD":11.5,"withdrawUSD":13.25,
            "depositTxs":218,"withdrawTxs":56}]"#;
        let v: ChainBridgeVolume = serde_json::from_str(json).unwrap();
        let p = &v.0[0];
        assert_eq!(p.date, Some(1_700_000_000));
        assert_eq!(p.deposit_usd, Some(11.5));
        assert_eq!(p.withdraw_usd, Some(13.25));
        assert_eq!(p.deposit_txs, Some(218));
        assert_eq!(p.withdraw_txs, Some(56));
    }

    #[test]
    fn bridge_detail_and_day_stats_parse() {
        let json = r#"{"id":1,"name":"foo","displayName":"Foo Bridge","lastHourlyVolume":1.5,
            "lastDailyVolume":20.0,"dayBeforeLastVolume":10.0,
            "prevDayTxs":{"deposits":3,"withdrawals":4},
            "chainBreakdown":{"ExampleChain":{"last24hVolume":7.0,"weeklyTxs":{"deposits":1,"withdrawals":2}}},
            "destinationChain":"ExampleChain"}"#;
        let d: BridgeDetail = serde_json::from_str(json).unwrap();
        assert_eq!(d.id, 1);
        assert_eq!(d.stats.last_hourly_volume, Some(1.5));
        assert_eq!(d.stats.day_before_last_volume, Some(10.0));
        assert_eq!(d.stats.prev_day_txs.as_ref().unwrap().withdrawals, Some(4));
        let c = &d.chain_breakdown["ExampleChain"];
        assert_eq!(c.last24h_volume, Some(7.0));
        assert_eq!(d.destination_chain.as_deref(), Some("ExampleChain"));

        let json = r#"{"date":1700000000,
            "totalTokensDeposited":{"ethereum:0x01":{"usdValue":5.0,"amount":"5","symbol":"FOO","decimals":18}},
            "totalTokensWithdrawn":{},
            "totalAddressDeposited":{"ethereum:0x02":{"usdValue":5.0,"txs":1}},
            "totalAddressWithdrawn":{}}"#;
        let s: DailyBridgeStats = serde_json::from_str(json).unwrap();
        assert_eq!(s.date, Some(1_700_000_000));
        assert_eq!(s.total_tokens_deposited["ethereum:0x01"].decimals, Some(18));
        assert_eq!(s.total_address_deposited["ethereum:0x02"].txs, Some(1));
    }

    #[test]
    fn bridges_list_parses() {
        let json = r#"{"bridges":[{"id":80,"name":"foo","displayName":"Foo","icon":"icons:foo",
            "volumePrevDay":1.0,"volumePrev2Day":2.0,"last24hVolume":3.0,"chains":["A","B"],
            "destinationChain":"B","url":"https://example.com","slug":"foo-bridge"}]}"#;
        let r: BridgesResponse = serde_json::from_str(json).unwrap();
        let b = &r.bridges[0];
        assert_eq!(b.volume_prev2_day, Some(2.0));
        assert_eq!(b.last24h_volume, Some(3.0));
        assert_eq!(b.destination_chain.as_deref(), Some("B"));
    }
}
