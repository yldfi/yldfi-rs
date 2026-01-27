//! Types for bridge data (Pro)

use serde::{Deserialize, Serialize};

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
    #[must_use] 
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(start) = self.start_timestamp {
            params.push(format!("startTimestamp={start}"));
        }
        if let Some(end) = self.end_timestamp {
            params.push(format!("endTimestamp={end}"));
        }
        if let Some(ref chain) = self.source_chain {
            params.push(format!("sourceChain={chain}"));
        }
        if let Some(ref addr) = self.address {
            params.push(format!("address={addr}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
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
    /// Volume last 24h
    pub volume_prev_day: Option<f64>,
    /// Volume last month
    pub volume_prev_month: Option<f64>,
    /// Chains supported
    #[serde(default)]
    pub chains: Vec<String>,
    /// Destination chains
    #[serde(default)]
    pub destination_chains: Vec<String>,
}

/// Bridge list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgesResponse {
    /// List of bridges
    pub bridges: Vec<Bridge>,
}

/// Detailed bridge data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeDetail {
    /// Bridge ID
    pub id: u64,
    /// Bridge name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Logo URL
    pub icon: Option<String>,
    /// Last hourly volume
    pub last_hourly_volume: Option<f64>,
    /// Last daily volume
    pub last_daily_volume: Option<f64>,
    /// Current day volume
    pub current_day_volume: Option<f64>,
    /// Previous day volume
    pub prev_day_volume: Option<f64>,
    /// Day before prev volume
    pub day_before_prev_volume: Option<f64>,
    /// Weekly volume
    pub weekly_volume: Option<f64>,
    /// Monthly volume
    pub monthly_volume: Option<f64>,
    /// Historical volume chart
    #[serde(default)]
    pub volume_chart: Vec<BridgeVolumePoint>,
}

/// Bridge volume data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeVolumePoint {
    /// Unix timestamp
    pub date: u64,
    /// Deposit volume
    pub deposit_usd: Option<f64>,
    /// Withdrawal volume
    pub withdrawal_usd: Option<f64>,
}

/// Bridge volume for a chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainBridgeVolume(pub Vec<BridgeVolumePoint>);

/// Daily bridge stats
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyBridgeStats {
    /// Date
    pub date: u64,
    /// Total deposit volume
    pub total_deposit_usd: Option<f64>,
    /// Total withdrawal volume
    pub total_withdrawal_usd: Option<f64>,
    /// Volume by token
    #[serde(default)]
    pub total_deposit_txs: u64,
    /// Withdrawal transactions
    #[serde(default)]
    pub total_withdrawal_txs: u64,
}

/// Bridge transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeTransaction {
    /// Transaction hash
    pub tx_hash: String,
    /// Timestamp
    pub timestamp: u64,
    /// Source chain
    pub source_chain: Option<String>,
    /// Destination chain
    pub destination_chain: Option<String>,
    /// Amount in USD
    pub usd_value: Option<f64>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token address
    pub token: Option<String>,
    /// Deposit address (from)
    pub deposit_address: Option<String>,
    /// Withdrawal address (to)
    pub withdrawal_address: Option<String>,
}

/// Bridge transactions response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeTransactionsResponse(pub Vec<BridgeTransaction>);
