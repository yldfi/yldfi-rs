//! Types for the Volume API

use serde::{Deserialize, Serialize};

/// Chain volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainVolume {
    /// Chain identifier
    pub chain: Option<String>,
    /// Chain ID
    pub chain_id: Option<String>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Volume change 24h percent
    pub volume_change_24h: Option<f64>,
    /// Transaction count 24h
    pub transactions_24h: Option<i64>,
    /// Active addresses 24h
    pub active_addresses_24h: Option<i64>,
}

/// Category volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryVolume {
    /// Category ID
    pub category_id: Option<String>,
    /// Category name
    pub category_name: Option<String>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Volume change 24h percent
    pub volume_change_24h: Option<f64>,
    /// Token count
    pub token_count: Option<i64>,
}

/// Volume timeseries data point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeDataPoint {
    /// Timestamp
    pub timestamp: Option<String>,
    /// Volume USD
    pub volume_usd: Option<f64>,
    /// Transaction count
    pub transactions: Option<i64>,
}

/// Volume timeseries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeTimeseries {
    /// Chain or token identifier
    pub identifier: Option<String>,
    /// Data points
    pub data: Vec<VolumeDataPoint>,
}
