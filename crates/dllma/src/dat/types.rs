//! Types for Digital Asset Treasury (DAT) data (Pro)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DAT institutions response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatInstitutionsResponse {
    /// Institution metadata keyed by ticker
    #[serde(default)]
    pub institution_metadata: HashMap<String, InstitutionMetadata>,
    /// Asset metadata keyed by asset ticker
    #[serde(default)]
    pub asset_metadata: HashMap<String, AssetMetadata>,
    /// Simple institution list with holdings
    #[serde(default)]
    pub institutions: Vec<InstitutionSummary>,
    /// Asset holdings by asset ticker
    #[serde(default)]
    pub assets: HashMap<String, Vec<AssetHolding>>,
    /// Total number of companies tracked
    pub total_companies: Option<u64>,
    /// Flow data by asset
    #[serde(default)]
    pub flows: HashMap<String, Vec<FlowDataPoint>>,
    /// mNAV data by institution
    #[serde(default, rename = "mNAV")]
    pub mnav: HashMap<String, HashMap<String, Vec<MnavDataPoint>>>,
    /// Last updated timestamp
    pub last_updated: Option<String>,
}

/// Institution metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionMetadata {
    /// Unique institution ID
    pub institution_id: Option<u64>,
    /// Ticker symbol
    pub ticker: Option<String>,
    /// Institution name
    pub name: Option<String>,
    /// Institution type (Stock, ETF, etc.)
    #[serde(rename = "type")]
    pub institution_type: Option<String>,
    /// Current share price
    pub price: Option<f64>,
    /// 24h price change percentage
    pub price_change_24h: Option<f64>,
    /// 24h trading volume
    pub volume_24h: Option<f64>,
    /// Market cap using current shares (realized)
    pub mcap_realized: Option<f64>,
    /// Market cap with unavoidable dilution (realistic)
    pub mcap_realistic: Option<f64>,
    /// Market cap under max dilution (max)
    pub mcap_max: Option<f64>,
    /// Realized mNAV ratio
    pub realized_m_nav: Option<f64>,
    /// Realistic mNAV ratio
    pub realistic_m_nav: Option<f64>,
    /// Maximum mNAV ratio
    pub max_m_nav: Option<f64>,
    /// Total USD value of crypto holdings
    pub total_usd_value: Option<f64>,
    /// Total cost basis
    pub total_cost: Option<f64>,
    /// Holdings by asset
    #[serde(default)]
    pub holdings: HashMap<String, Holding>,
}

/// Individual holding
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Holding {
    /// Amount held
    pub amount: Option<f64>,
    /// Average purchase price
    pub avg_price: Option<f64>,
    /// Current USD value
    pub usd_value: Option<f64>,
    /// Total cost basis
    pub cost: Option<f64>,
    /// Number of transactions
    pub transaction_count: Option<u64>,
    /// First announcement date
    pub first_announcement_date: Option<String>,
    /// Last announcement date
    pub last_announcement_date: Option<String>,
    /// Percentage of supply held
    pub supply_percentage: Option<f64>,
}

/// Asset metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMetadata {
    /// Asset name
    pub name: Option<String>,
    /// Asset ticker
    pub ticker: Option<String>,
    /// `CoinGecko` ID
    pub gecko_id: Option<String>,
    /// Number of companies holding
    pub companies: Option<u64>,
    /// Total amount held
    pub total_amount: Option<f64>,
    /// Total USD value
    pub total_usd_value: Option<f64>,
    /// Percentage of circulating supply
    pub circ_supply_perc: Option<f64>,
}

/// Simple institution summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionSummary {
    /// Institution ID
    pub institution_id: Option<u64>,
    /// Total USD value
    pub total_usd_value: Option<f64>,
    /// Total cost basis
    pub total_cost: Option<f64>,
}

/// Asset holding by an institution
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetHolding {
    /// Institution ID
    pub institution_id: Option<u64>,
    /// USD value of holding
    pub usd_value: Option<f64>,
    /// Amount held
    pub amount: Option<f64>,
}

/// Flow data point [timestamp, `net_flow`, inflow, outflow, `usd_value`, `usd_net_flow`]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FlowDataPoint(pub Vec<f64>);

/// mNAV data point [timestamp, realized, realistic, max]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct MnavDataPoint(pub Vec<f64>);

/// Individual institution DAT detail
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionDetail {
    /// Institution ID
    pub institution_id: Option<u64>,
    /// Ticker symbol
    pub ticker: Option<String>,
    /// Institution name
    pub name: Option<String>,
    /// Institution type
    #[serde(rename = "type")]
    pub institution_type: Option<String>,
    /// Rank
    pub rank: Option<u64>,
    /// Current price
    pub price: Option<f64>,
    /// 24h price change
    pub price_change_24h: Option<f64>,
    /// 24h volume
    pub volume_24h: Option<f64>,
    /// Fully diluted shares - realized
    pub fd_realized: Option<String>,
    /// Fully diluted shares - realistic
    pub fd_realistic: Option<String>,
    /// Fully diluted shares - max
    pub fd_max: Option<String>,
    /// Market cap - realized
    pub mcap_realized: Option<f64>,
    /// Market cap - realistic
    pub mcap_realistic: Option<f64>,
    /// Market cap - max
    pub mcap_max: Option<f64>,
    /// Realized mNAV
    pub realized_m_nav: Option<f64>,
    /// Realistic mNAV
    pub realistic_m_nav: Option<f64>,
    /// Max mNAV
    pub max_m_nav: Option<f64>,
    /// Total USD value
    pub total_usd_value: Option<f64>,
    /// Total cost
    pub total_cost: Option<f64>,
    /// Holdings by asset
    #[serde(default)]
    pub holdings: HashMap<String, Holding>,
    /// Transactions list
    #[serde(default)]
    pub transactions: Vec<DatTransaction>,
    /// Historical mNAV
    #[serde(default, rename = "mNAV")]
    pub mnav: HashMap<String, Vec<MnavDataPoint>>,
}

/// DAT transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatTransaction {
    /// Transaction date
    pub date: Option<String>,
    /// Asset ticker
    pub asset: Option<String>,
    /// Transaction type (buy/sell)
    pub tx_type: Option<String>,
    /// Amount
    pub amount: Option<f64>,
    /// Price per unit
    pub price: Option<f64>,
    /// Total value
    pub value: Option<f64>,
    /// Source of information
    pub source: Option<String>,
}
