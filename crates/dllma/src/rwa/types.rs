//! Types for real-world asset (RWA) data (`/rwa/*`, Pro)
//!
//! Three measures recur: Onchain AUM (`on_chain_mcap`, the USD value of all
//! supply onchain), Active AUM (`active_aum`, the part held in user or
//! protocol positions rather than by the issuer) and `DeFi` Active TVL
//! (`defi_active_tvl`, the part of Active AUM deployed in `DeFi`
//! protocols). Structs are `#[non_exhaustive]` so new upstream fields can be
//! added without a breaking change.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::de::{null_default, opt_flex_u64};

/// Measure of an RWA chain breakdown chart (`key`); the default is
/// [`RwaMetric::OnChainMcap`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RwaMetric {
    /// Onchain AUM (`onChainMcap`)
    OnChainMcap,
    /// Active AUM (`activeAum`)
    ActiveAum,
    /// `DeFi` Active TVL (`defiActiveTvl`)
    DefiActiveTvl,
}

impl RwaMetric {
    /// The query value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OnChainMcap => "onChainMcap",
            Self::ActiveAum => "activeAum",
            Self::DefiActiveTvl => "defiActiveTvl",
        }
    }
}

/// Options of [`super::RwaApi::chain_breakdown`]
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct RwaBreakdownQuery {
    /// Measure to chart (default Onchain AUM)
    pub metric: Option<RwaMetric>,
    /// Include stablecoins
    pub include_stablecoin: Option<bool>,
    /// Include governance tokens
    pub include_governance: Option<bool>,
}

impl RwaBreakdownQuery {
    /// Defaults: Onchain AUM, without stablecoins or governance tokens
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Measure to chart
    #[must_use]
    pub fn metric(mut self, metric: RwaMetric) -> Self {
        self.metric = Some(metric);
        self
    }

    /// Include stablecoins
    #[must_use]
    pub fn include_stablecoin(mut self, include: bool) -> Self {
        self.include_stablecoin = Some(include);
        self
    }

    /// Include governance tokens
    #[must_use]
    pub fn include_governance(mut self, include: bool) -> Self {
        self.include_governance = Some(include);
        self
    }
}

/// RWA response wrapper: `{"data": ...}`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RwaData<T> {
    /// Response data
    pub data: T,
}

/// A real-world asset (`/rwa/current`, `/rwa/chain/{chain}`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RwaAsset {
    /// Asset id
    #[serde(default)]
    pub id: String,
    /// Canonical market id, e.g. `ondo/usdy`
    pub canonical_market_id: Option<String>,
    /// Ticker
    pub ticker: Option<String>,
    /// Asset name
    pub asset_name: Option<String>,
    /// Asset group
    pub asset_group: Option<String>,
    /// Primary chain
    pub primary_chain: Option<String>,
    /// Chains
    #[serde(default, deserialize_with = "null_default")]
    pub chain: Vec<String>,
    /// Categories
    #[serde(default, deserialize_with = "null_default")]
    pub category: Vec<String>,
    /// Asset classes
    #[serde(default, deserialize_with = "null_default")]
    pub asset_class: Vec<String>,
    /// Type
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// RWA classification
    pub rwa_classification: Option<String>,
    /// Access model
    pub access_model: Option<String>,
    /// Issuer
    pub issuer: Option<String>,
    /// Parent platform
    pub parent_platform: Option<String>,
    /// Whether it is a stablecoin
    pub stablecoin: Option<bool>,
    /// Whether it is a governance token
    pub governance: Option<bool>,
    /// Onchain AUM by chain
    #[serde(default, deserialize_with = "null_default")]
    pub on_chain_mcap: HashMap<String, f64>,
    /// Active AUM by chain
    #[serde(default, deserialize_with = "null_default")]
    pub active_aum: HashMap<String, f64>,
    /// `DeFi` Active TVL by chain, then by protocol
    #[serde(default, deserialize_with = "null_default")]
    pub defi_active_tvl: HashMap<String, HashMap<String, f64>>,
    /// Price
    pub price: Option<f64>,
    /// Whether Active AUM is tracked for this asset
    pub active_aum_data: Option<bool>,
    /// `CoinGecko` id
    pub coingecko_id: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
}

/// Ids and filter values for RWA discovery (`/rwa/list`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RwaList {
    /// Canonical market ids
    #[serde(default, deserialize_with = "null_default")]
    pub canonical_market_ids: Vec<String>,
    /// Platforms
    #[serde(default, deserialize_with = "null_default")]
    pub platforms: Vec<String>,
    /// Chains
    #[serde(default, deserialize_with = "null_default")]
    pub chains: Vec<String>,
    /// Categories
    #[serde(default, deserialize_with = "null_default")]
    pub categories: Vec<String>,
    /// Asset groups
    #[serde(default, deserialize_with = "null_default")]
    pub asset_groups: Vec<String>,
    /// Asset id by canonical market id
    #[serde(default, deserialize_with = "null_default")]
    pub id_map: HashMap<String, String>,
}

/// RWA aggregates (`/rwa/stats`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RwaStats {
    /// Total Onchain AUM
    pub total_on_chain_mcap: Option<f64>,
    /// Total Active AUM
    pub total_active_mcap: Option<f64>,
    /// Total `DeFi` Active TVL
    pub total_defi_active_tvl: Option<f64>,
    /// Number of assets
    pub asset_count: Option<u64>,
    /// Number of issuers
    pub asset_issuers: Option<u64>,
    /// By chain
    #[serde(default, deserialize_with = "null_default")]
    pub by_chain: HashMap<String, RwaBuckets>,
    /// By category
    #[serde(default, deserialize_with = "null_default")]
    pub by_category: HashMap<String, RwaBuckets>,
    /// By platform
    #[serde(default, deserialize_with = "null_default")]
    pub by_platform: HashMap<String, RwaBuckets>,
    /// By asset group
    #[serde(default, deserialize_with = "null_default")]
    pub by_asset_group: HashMap<String, RwaBuckets>,
}

/// Aggregates of one chain, category, platform or asset group, split by
/// whether stablecoins and governance tokens are counted
///
/// `base` leaves both out; add the other buckets to include them.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RwaBuckets {
    /// Without stablecoins or governance tokens
    #[serde(default, deserialize_with = "null_default")]
    pub base: RwaTotals,
    /// Stablecoins only
    #[serde(default, deserialize_with = "null_default")]
    pub stablecoins_only: RwaTotals,
    /// Governance tokens only
    #[serde(default, deserialize_with = "null_default")]
    pub governance_only: RwaTotals,
    /// Tokens that are both
    #[serde(default, deserialize_with = "null_default")]
    pub stablecoins_and_governance: RwaTotals,
}

/// Totals of one bucket
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RwaTotals {
    /// Onchain AUM
    pub on_chain_mcap: Option<f64>,
    /// Active AUM
    pub active_aum: Option<f64>,
    /// `DeFi` Active TVL
    pub defi_active_tvl: Option<f64>,
    /// Number of assets
    pub asset_count: Option<u64>,
    /// Issuers
    #[serde(default, deserialize_with = "null_default")]
    pub asset_issuers: Vec<String>,
}

/// A point of a chain's RWA chart (`/rwa/chart/chain/{chain}`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RwaChartPoint {
    /// Unix timestamp (seconds)
    #[serde(default, deserialize_with = "opt_flex_u64")]
    pub timestamp: Option<u64>,
    /// Onchain AUM
    pub on_chain_mcap: Option<f64>,
    /// Active AUM
    pub active_aum: Option<f64>,
    /// `DeFi` Active TVL
    pub defi_active_tvl: Option<f64>,
}
