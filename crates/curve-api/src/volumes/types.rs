//! Types for the Volumes and APYs API

use serde::{Deserialize, Serialize};

/// Response for gauge data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugesResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Gauge data
    pub data: serde_json::Value,
}

/// Response for volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumesResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Volume data
    pub data: VolumesData,
}

/// Volume data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumesData {
    /// Pool volumes
    pub pools: Option<Vec<PoolVolume>>,
    /// Total volume
    pub total_volume: Option<f64>,
}

/// Volume data for a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolVolume {
    /// Pool address
    pub address: String,
    /// 24h volume in USD
    pub volume_usd: Option<f64>,
    /// Base APY
    pub apy: Option<f64>,
}

/// Response for base APYs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseApysResponse {
    /// Whether the request was successful
    pub success: bool,
    /// APY data
    pub data: serde_json::Value,
}
