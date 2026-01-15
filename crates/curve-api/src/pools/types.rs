//! Types for the Pools API

use serde::{Deserialize, Serialize};

/// Response wrapper for pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolsResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Pool data
    pub data: PoolsData,
}

/// Pool data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolsData {
    /// List of pools
    pub pool_data: Vec<Pool>,
    /// Total TVL in USD
    pub tvl: Option<f64>,
    /// Total TVL across all pools
    pub tvl_all: Option<f64>,
}

/// A Curve pool
///
/// The Curve API has inconsistent types (some fields are sometimes strings,
/// sometimes integers), so we use serde_json::Value internally and provide
/// typed accessor methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Pool(pub serde_json::Value);

impl Pool {
    /// Get pool ID
    pub fn id(&self) -> Option<&str> {
        self.0.get("id").and_then(|v| v.as_str())
    }

    /// Get pool name
    pub fn name(&self) -> Option<&str> {
        self.0.get("name").and_then(|v| v.as_str())
    }

    /// Get pool address
    pub fn address(&self) -> Option<&str> {
        self.0.get("address").and_then(|v| v.as_str())
    }

    /// Get LP token address
    pub fn lp_token_address(&self) -> Option<&str> {
        self.0.get("lpTokenAddress").and_then(|v| v.as_str())
    }

    /// Get gauge address
    pub fn gauge_address(&self) -> Option<&str> {
        self.0.get("gaugeAddress").and_then(|v| v.as_str())
    }

    /// Get asset type name (e.g., "usd", "eth", "crypto")
    pub fn asset_type_name(&self) -> Option<&str> {
        self.0.get("assetTypeName").and_then(|v| v.as_str())
    }

    /// Get USD total
    pub fn usd_total(&self) -> Option<f64> {
        self.0.get("usdTotal").and_then(|v| v.as_f64())
    }

    /// Get total supply
    pub fn total_supply(&self) -> Option<&str> {
        self.0.get("totalSupply").and_then(|v| v.as_str())
    }

    /// Get virtual price
    pub fn virtual_price(&self) -> Option<&str> {
        self.0.get("virtualPrice").and_then(|v| v.as_str())
    }

    /// Get amplification coefficient
    pub fn amplification_coefficient(&self) -> Option<&str> {
        self.0.get("amplificationCoefficient").and_then(|v| v.as_str())
    }

    /// Get symbol
    pub fn symbol(&self) -> Option<&str> {
        self.0.get("symbol").and_then(|v| v.as_str())
    }

    /// Get coins as raw JSON array
    pub fn coins(&self) -> Option<&Vec<serde_json::Value>> {
        self.0.get("coins").and_then(|v| v.as_array())
    }

    /// Check if pool is a metapool
    pub fn is_meta_pool(&self) -> Option<bool> {
        self.0.get("isMetaPool").and_then(|v| v.as_bool())
    }

    /// Get the underlying raw JSON value
    pub fn raw(&self) -> &serde_json::Value {
        &self.0
    }
}

/// Response for pool list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolListResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Pool addresses
    pub data: PoolListData,
}

/// Pool list data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolListData {
    /// Pool addresses by registry
    pub pool_list: Vec<String>,
}

/// Hidden pools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenPoolsResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Hidden pools by chain
    pub data: serde_json::Value,
}
