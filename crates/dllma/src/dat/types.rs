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
    #[serde(rename = "realized_mNAV")]
    pub realized_m_nav: Option<f64>,
    /// Realistic mNAV ratio
    #[serde(rename = "realistic_mNAV")]
    pub realistic_m_nav: Option<f64>,
    /// Maximum mNAV ratio
    #[serde(rename = "max_mNAV")]
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
///
/// Unlike [`InstitutionMetadata`], this endpoint uses snake_case for the
/// share count and market cap fields.
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
    #[serde(
        default,
        rename = "fd_realized",
        deserialize_with = "crate::de::opt_string_or_number"
    )]
    pub fd_realized: Option<String>,
    /// Fully diluted shares - realistic
    #[serde(
        default,
        rename = "fd_realistic",
        deserialize_with = "crate::de::opt_string_or_number"
    )]
    pub fd_realistic: Option<String>,
    /// Fully diluted shares - max
    #[serde(
        default,
        rename = "fd_max",
        deserialize_with = "crate::de::opt_string_or_number"
    )]
    pub fd_max: Option<String>,
    /// Market cap - realized
    #[serde(rename = "mcap_realized")]
    pub mcap_realized: Option<f64>,
    /// Market cap - realistic
    #[serde(rename = "mcap_realistic")]
    pub mcap_realistic: Option<f64>,
    /// Market cap - max
    #[serde(rename = "mcap_max")]
    pub mcap_max: Option<f64>,
    /// Realized mNAV
    #[serde(rename = "realized_mNAV")]
    pub realized_m_nav: Option<f64>,
    /// Realistic mNAV
    #[serde(rename = "realistic_mNAV")]
    pub realistic_m_nav: Option<f64>,
    /// Max mNAV
    #[serde(rename = "max_mNAV")]
    pub max_m_nav: Option<f64>,
    /// Total USD value
    pub total_usd_value: Option<f64>,
    /// Total cost
    pub total_cost: Option<f64>,
    /// Holdings keyed by asset
    #[serde(default)]
    pub assets: HashMap<String, Holding>,
    /// Asset names and tickers keyed by asset
    #[serde(default)]
    pub assets_meta: HashMap<String, AssetMetadata>,
    /// Share price history: [timestamp, open, high, low, close, volume]
    #[serde(default)]
    pub ohlcv: Vec<Vec<Option<f64>>>,
    /// Holdings value history: [timestamp, USD value]
    #[serde(default)]
    pub asset_value: Vec<Vec<Option<f64>>>,
    /// Daily stats: [timestamp, fd realized, fd realistic, fd max, mcap realized,
    /// mcap realistic, mcap max, mNAV realized, mNAV realistic, mNAV max]
    #[serde(default)]
    pub stats: Vec<Vec<Option<f64>>>,
    /// Transactions list
    #[serde(default)]
    pub transactions: Vec<DatTransaction>,
    /// Last updated timestamp
    pub last_updated: Option<String>,
}

/// DAT transaction (snake_case on the wire, amounts as decimal strings)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatTransaction {
    /// Transaction ID
    pub id: Option<u64>,
    /// Asset ID (e.g. "bitcoin")
    pub asset: Option<String>,
    /// Asset name
    #[serde(rename = "assetName")]
    pub asset_name: Option<String>,
    /// Asset ticker
    #[serde(rename = "assetTicker")]
    pub asset_ticker: Option<String>,
    /// Institution ticker
    pub ticker: Option<String>,
    /// Transaction type (e.g. purchase, sale)
    #[serde(rename = "type")]
    pub tx_type: Option<String>,
    /// Amount
    #[serde(default, deserialize_with = "crate::de::opt_string_or_number")]
    pub amount: Option<String>,
    /// Average price per unit
    #[serde(default, deserialize_with = "crate::de::opt_string_or_number")]
    pub avg_price: Option<String>,
    /// Total USD value
    #[serde(default, deserialize_with = "crate::de::opt_string_or_number")]
    pub usd_value: Option<String>,
    /// Period start date
    pub start_date: Option<String>,
    /// Period end date
    pub end_date: Option<String>,
    /// Report date
    pub report_date: Option<String>,
    /// Source type (e.g. filing)
    pub source_type: Option<String>,
    /// Source URL
    pub source_url: Option<String>,
    /// Source note
    pub source_note: Option<String>,
    /// Whether the transaction is approved
    pub is_approved: Option<bool>,
    /// Rejection reason
    pub reject_reason: Option<String>,
    /// Last updated timestamp
    pub last_updated: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_metadata_mnav_fields() {
        let json = r#"{"institutionMetadata":{"FOO":{"institutionId":1,"ticker":"FOO",
            "name":"Foo Inc.","type":"Stock","mcapRealized":100.0,"mcapRealistic":110.0,
            "mcapMax":120.0,"realized_mNAV":1.5,"realistic_mNAV":1.6,"max_mNAV":1.7,
            "holdings":{"BTC":{"amount":2.0,"avgPrice":3.0}}}},
            "mNAV":{"FOO":{"BTC":[[1700000000,1.5,1.6,1.7]]}},"lastUpdated":"2024-01-01"}"#;
        let r: DatInstitutionsResponse = serde_json::from_str(json).unwrap();
        let m = &r.institution_metadata["FOO"];
        assert_eq!(m.mcap_realized, Some(100.0));
        assert_eq!(m.realized_m_nav, Some(1.5));
        assert_eq!(m.realistic_m_nav, Some(1.6));
        assert_eq!(m.max_m_nav, Some(1.7));
        assert_eq!(m.holdings["BTC"].avg_price, Some(3.0));
        assert_eq!(r.mnav["FOO"]["BTC"][0].0[1], 1.5);
    }

    #[test]
    fn institution_detail_snake_case_fields() {
        let json = r#"{"institutionId":1,"ticker":"FOO","name":"Foo Inc.","type":"Stock",
            "rank":1,"fd_realized":"1000","fd_realistic":"1100","fd_max":1200,
            "mcap_realized":10.0,"mcap_realistic":11.0,"mcap_max":12.0,
            "realized_mNAV":1.1,"realistic_mNAV":1.2,"max_mNAV":1.3,
            "totalCost":5.0,"totalUsdValue":9.0,
            "assets":{"bitcoin":{"amount":2.0,"avgPrice":2.5,"usdValue":9.0,"cost":5.0}},
            "assetsMeta":{"bitcoin":{"name":"Bitcoin","ticker":"BTC"}},
            "ohlcv":[[1700000000,1,2,0.5,1.5,100]],"assetValue":[[1700000000,9.0]],
            "stats":[[1700000000,1000,1100,1200,10,11,12,1.1,1.2,null]],
            "transactions":[{"id":7,"asset":"bitcoin","amount":"2","avg_price":"2.5",
            "usd_value":"5","start_date":"2024-01-01","end_date":"2024-01-02",
            "report_date":"2024-01-03","type":"purchase","source_type":"filing",
            "source_url":"https://example.com/filing","source_note":null,"is_approved":true,
            "reject_reason":null,"last_updated":"2024-01-04","ticker":"FOO",
            "assetName":"Bitcoin","assetTicker":"BTC"}],"lastUpdated":"2024-01-05"}"#;
        let d: InstitutionDetail = serde_json::from_str(json).unwrap();
        assert_eq!(d.fd_realized.as_deref(), Some("1000"));
        assert_eq!(d.fd_max.as_deref(), Some("1200"));
        assert_eq!(d.mcap_realized, Some(10.0));
        assert_eq!(d.max_m_nav, Some(1.3));
        assert_eq!(d.assets["bitcoin"].avg_price, Some(2.5));
        assert_eq!(d.assets_meta["bitcoin"].ticker.as_deref(), Some("BTC"));
        assert_eq!(d.stats[0][9], None);
        let tx = &d.transactions[0];
        assert_eq!(tx.amount.as_deref(), Some("2"));
        assert_eq!(tx.tx_type.as_deref(), Some("purchase"));
        assert_eq!(tx.asset_ticker.as_deref(), Some("BTC"));
        assert_eq!(tx.is_approved, Some(true));
    }
}
