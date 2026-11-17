//! Types for the v2 chart and metrics API (`/api/v2/chart/*`,
//! `/api/v2/metrics/*`, Pro)
//!
//! Structs are `#[non_exhaustive]` so new upstream fields can be added
//! without a breaking change. Time series come in three shapes, read by two
//! types: totals as `[timestamp, value]` pairs ([`ChartPoint`]), and
//! breakdowns either as `[timestamp, {name: value}]` pairs or as
//! `{"timestamp": ..., name: value}` objects ([`BreakdownPoint`]).

use std::collections::HashMap;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::de::{
    null_default, opt_flex_f64, opt_flex_u64, value_to_f64, value_to_string, value_to_u64,
};

// ---------------------------------------------------------------------------
// Request parameters
// ---------------------------------------------------------------------------

/// A dimension metric: the `{metric}` of `/api/v2/chart/{metric}` and
/// `/api/v2/metrics/{metric}`
///
/// Not every metric is served at every scope: `ActiveUsers` only has the
/// overall chart, its chain breakdown, the overall overview and the
/// category list, and `Bridges` has no label breakdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Dimension {
    /// Fees and revenue (`fees`)
    Fees,
    /// Liquidations (`liquidations`)
    Liquidations,
    /// DEX volume (`dexs`)
    Dexs,
    /// Perps volume (`derivatives`)
    Derivatives,
    /// Options volume (`options`)
    Options,
    /// DEX aggregator volume (`aggregators`)
    Aggregators,
    /// Bridge aggregator volume (`bridge-aggregators`)
    BridgeAggregators,
    /// Bridge volume and transfers (`bridges`)
    Bridges,
    /// Perps open interest (`open-interest`)
    OpenInterest,
    /// Normalized volume (`normalized-volume`)
    NormalizedVolume,
    /// Active users, transactions and gas (`active-users`)
    ActiveUsers,
}

impl Dimension {
    /// The path segment for this metric
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fees => "fees",
            Self::Liquidations => "liquidations",
            Self::Dexs => "dexs",
            Self::Derivatives => "derivatives",
            Self::Options => "options",
            Self::Aggregators => "aggregators",
            Self::BridgeAggregators => "bridge-aggregators",
            Self::Bridges => "bridges",
            Self::OpenInterest => "open-interest",
            Self::NormalizedVolume => "normalized-volume",
            Self::ActiveUsers => "active-users",
        }
    }
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a chart or overview covers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scope {
    /// Everything (no scope segment)
    All,
    /// One chain (`/chain/{chain}`), by display name, e.g. `Ethereum`
    Chain(String),
    /// One protocol (`/protocol/{protocol}`), by slug, e.g. `aave-v3`
    Protocol(String),
    /// One category (`/category/{category}`), by slug, e.g. `dexs`
    Category(String),
    /// One category on one chain (`/category/{category}/chain/{chain}`)
    CategoryChain {
        /// Category slug
        category: String,
        /// Chain name
        chain: String,
    },
}

impl Scope {
    /// One chain
    pub fn chain(chain: impl Into<String>) -> Self {
        Self::Chain(chain.into())
    }

    /// One protocol
    pub fn protocol(protocol: impl Into<String>) -> Self {
        Self::Protocol(protocol.into())
    }

    /// One category
    pub fn category(category: impl Into<String>) -> Self {
        Self::Category(category.into())
    }

    /// One category on one chain
    pub fn category_chain(category: impl Into<String>, chain: impl Into<String>) -> Self {
        Self::CategoryChain {
            category: category.into(),
            chain: chain.into(),
        }
    }

    /// The path segments for this scope, starting with `/` (empty for
    /// [`Scope::All`])
    pub(crate) fn path(&self) -> String {
        use crate::client::path_segment as seg;
        match self {
            Self::All => String::new(),
            Self::Chain(c) => format!("/chain/{}", seg(c)),
            Self::Protocol(p) => format!("/protocol/{}", seg(p)),
            Self::Category(c) => format!("/category/{}", seg(c)),
            Self::CategoryChain { category, chain } => {
                format!("/category/{}/chain/{}", seg(category), seg(chain))
            }
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Chain(_) => "chain",
            Self::Protocol(_) => "protocol",
            Self::Category(_) => "category",
            Self::CategoryChain { .. } => "category+chain",
        }
    }
}

/// How a chart is split into series
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Breakdown {
    /// One series per chain (`chain-breakdown`)
    Chain,
    /// One series per protocol (`protocol-breakdown`)
    Protocol,
    /// One series per token (`token-breakdown`, TVL and treasury only)
    Token,
    /// One series per revenue label, e.g. `Borrow Interest`
    /// (`label-breakdown`, dimension metrics of one protocol only)
    Label,
    /// One series per protocol version (`version-breakdown`, dimension
    /// metrics of one protocol only)
    Version,
}

impl Breakdown {
    /// The path segment for this breakdown
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chain => "chain-breakdown",
            Self::Protocol => "protocol-breakdown",
            Self::Token => "token-breakdown",
            Self::Label => "label-breakdown",
            Self::Version => "version-breakdown",
        }
    }
}

/// Which TVL a protocol TVL chart sums (`key`); the default (no key) is
/// plain TVL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TvlKey {
    /// TVL plus borrowed, staking, pool2 and vesting (`all`)
    All,
    /// Staking only (`staking`)
    Staking,
    /// Borrowed only (`borrowed`)
    Borrowed,
    /// Vesting only (`vesting`)
    Vesting,
    /// Pool2 only (`pool2`)
    Pool2,
}

impl TvlKey {
    /// The query value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Staking => "staking",
            Self::Borrowed => "borrowed",
            Self::Vesting => "vesting",
            Self::Pool2 => "pool2",
        }
    }
}

/// Which tokens a treasury chart sums (`key`); the default (no key)
/// leaves out the protocol's own tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TreasuryKey {
    /// Only the protocol's own tokens (`OwnTokens`)
    OwnTokens,
    /// All tokens, own tokens included (`all`)
    All,
}

impl TreasuryKey {
    /// The query value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OwnTokens => "OwnTokens",
            Self::All => "all",
        }
    }
}

/// Unit of a token breakdown (`currency`); the default is USD
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Currency {
    /// USD value (`usd`)
    Usd,
    /// Token amounts (`token`)
    Token,
    /// Raw token amounts, without decimals applied (`raw`)
    Raw,
}

impl Currency {
    /// The query value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Usd => "usd",
            Self::Token => "token",
            Self::Raw => "raw",
        }
    }
}

// ---------------------------------------------------------------------------
// Time series
// ---------------------------------------------------------------------------

/// One point of a total chart, read from `[timestamp, value]`
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ChartPoint {
    /// Unix timestamp (seconds)
    pub timestamp: u64,
    /// Value at that time (USD, or a count for transaction and user metrics)
    pub value: f64,
}

impl ChartPoint {
    /// A point at `timestamp`
    #[must_use]
    pub fn new(timestamp: u64, value: f64) -> Self {
        Self { timestamp, value }
    }
}

impl<'de> Deserialize<'de> for ChartPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        let pair = v
            .as_array()
            .filter(|a| a.len() >= 2)
            .ok_or_else(|| D::Error::custom(format!("expected [timestamp, value], got {v}")))?;
        let timestamp = value_to_u64(&pair[0])
            .ok_or_else(|| D::Error::custom(format!("expected a timestamp, got {}", pair[0])))?;
        let value = value_to_f64(&pair[1])
            .ok_or_else(|| D::Error::custom(format!("expected a number, got {}", pair[1])))?;
        Ok(Self { timestamp, value })
    }
}

/// One point of a breakdown chart: a value per series (chain, protocol,
/// token, label or version)
///
/// Read from `[timestamp, {name: value}]` (dimension, TVL and treasury
/// charts) or `{"timestamp": ..., name: value}` (oracle, fork and RWA
/// charts). `null` values are left out.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct BreakdownPoint {
    /// Unix timestamp (seconds)
    pub timestamp: u64,
    /// Value per series name
    pub values: HashMap<String, f64>,
}

fn read_values<E: serde::de::Error>(
    map: serde_json::Map<String, Value>,
) -> Result<HashMap<String, f64>, E> {
    map.into_iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| match value_to_f64(&v) {
            Some(n) => Ok((k, n)),
            None => Err(E::custom(format!("expected a number for {k}, got {v}"))),
        })
        .collect()
}

impl<'de> Deserialize<'de> for BreakdownPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::Array(mut pair) if pair.len() >= 2 => {
                let timestamp = value_to_u64(&pair[0]).ok_or_else(|| {
                    D::Error::custom(format!("expected a timestamp, got {}", pair[0]))
                })?;
                let values = match pair.swap_remove(1) {
                    Value::Object(map) => read_values(map)?,
                    Value::Null => HashMap::new(),
                    v => return Err(D::Error::custom(format!("expected an object, got {v}"))),
                };
                Ok(Self { timestamp, values })
            }
            Value::Object(mut map) => {
                let timestamp = map
                    .remove("timestamp")
                    .as_ref()
                    .and_then(value_to_u64)
                    .ok_or_else(|| D::Error::custom("missing timestamp"))?;
                Ok(Self {
                    timestamp,
                    values: read_values(map)?,
                })
            }
            v => Err(D::Error::custom(format!(
                "expected [timestamp, {{...}}] or {{\"timestamp\": ...}}, got {v}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Dimension metrics
// ---------------------------------------------------------------------------

/// Totals of a dimension metric overall, on a chain, in a category or in a
/// category on a chain (`/api/v2/metrics/{metric}[/...]`)
///
/// `change_*` fields are percentages.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DimensionOverview {
    /// Chains with data for this metric
    #[serde(default, rename = "allChains", deserialize_with = "null_default")]
    pub all_chains: Vec<String>,
    /// Categories with data (category scopes)
    #[serde(default, rename = "allCategories", deserialize_with = "null_default")]
    pub all_categories: Vec<String>,
    /// Chain (chain scopes)
    pub chain: Option<String>,
    /// Category (category scopes)
    pub category: Option<String>,
    /// Last 24 hours
    pub total24h: Option<f64>,
    /// The 24 hours before that
    pub total48hto24h: Option<f64>,
    /// Last 7 days
    pub total7d: Option<f64>,
    /// The 7 days before that
    pub total14dto7d: Option<f64>,
    /// Last 30 days
    pub total30d: Option<f64>,
    /// The 30 days before that
    pub total60dto30d: Option<f64>,
    /// Last year
    pub total1y: Option<f64>,
    /// Change over 1 day
    pub change_1d: Option<f64>,
    /// Change over 7 days
    pub change_7d: Option<f64>,
    /// Change over 1 month
    pub change_1m: Option<f64>,
    /// Last 7 days against the 7 days before
    pub change_7dover7d: Option<f64>,
    /// Last 30 days against the 30 days before
    pub change_30dover30d: Option<f64>,
}

/// Totals of a dimension metric for one protocol
/// (`/api/v2/metrics/{metric}/protocol/{protocol}`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DimensionProtocolOverview {
    /// Protocol id
    #[serde(default)]
    pub id: String,
    /// Protocol name
    #[serde(default)]
    pub name: String,
    /// Website
    pub url: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
    /// Chains
    #[serde(default, deserialize_with = "null_default")]
    pub chains: Vec<String>,
    /// `CoinGecko` id
    pub gecko_id: Option<String>,
    /// `CoinMarketCap` id
    #[serde(rename = "cmcId")]
    pub cmc_id: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Twitter handle
    pub twitter: Option<String>,
    /// GitHub organizations
    #[serde(default, deserialize_with = "null_default")]
    pub github: Vec<String>,
    /// Treasury adapter
    pub treasury: Option<String>,
    /// Governance ids
    #[serde(default, rename = "governanceID", deserialize_with = "null_default")]
    pub governance_id: Vec<String>,
    /// Whether the token's liquidity is flagged as unreliable
    #[serde(rename = "wrongLiquidity")]
    pub wrong_liquidity: Option<bool>,
    /// Stablecoins issued by the protocol
    #[serde(default, deserialize_with = "null_default")]
    pub stablecoins: Vec<String>,
    /// Last 24 hours
    pub total24h: Option<f64>,
    /// The 24 hours before that
    pub total48hto24h: Option<f64>,
    /// Last 7 days
    pub total7d: Option<f64>,
    /// Last 30 days
    pub total30d: Option<f64>,
    /// Last year
    pub total1y: Option<f64>,
    /// Change over 1 day, in percent
    pub change_1d: Option<f64>,
    /// Change over 7 days, in percent
    pub change_7d: Option<f64>,
    /// Change over 1 month, in percent
    pub change_1m: Option<f64>,
}

/// A category with data for a dimension metric
/// (`/api/v2/metrics/{metric}/categories`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DimensionCategory {
    /// Category name
    #[serde(default)]
    pub category: String,
    /// Last 24 hours
    pub total24h: Option<f64>,
    /// Last 7 days
    pub total7d: Option<f64>,
    /// Last 30 days
    pub total30d: Option<f64>,
    /// Chains with data in this category
    #[serde(default, deserialize_with = "null_default")]
    pub chains: Vec<String>,
}

// ---------------------------------------------------------------------------
// Oracles and forks
// ---------------------------------------------------------------------------

/// Oracles and the protocols using them (`/api/v2/metrics/oracle`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OracleOverview {
    /// Protocol names keyed by oracle name
    #[serde(default, deserialize_with = "null_default")]
    pub oracles: HashMap<String, Vec<String>>,
}

// ---------------------------------------------------------------------------
// Protocol TVL / treasury metrics
// ---------------------------------------------------------------------------

/// A notable event in a protocol's history, read from
/// `[timestamp, description]` or `[[start, end], description]`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Hallmark {
    /// Unix timestamp (the start, for a range)
    pub timestamp: Option<u64>,
    /// End of the range, for events given as `[start, end]`
    pub end_timestamp: Option<u64>,
    /// What happened
    pub description: Option<String>,
}

impl<'de> Deserialize<'de> for Hallmark {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        let Value::Array(items) = v else {
            return Err(D::Error::custom(format!(
                "expected [timestamp, description], got {v}"
            )));
        };
        let (timestamp, end_timestamp) = match items.first() {
            Some(Value::Array(range)) => (
                range.first().and_then(value_to_u64),
                range.get(1).and_then(value_to_u64),
            ),
            Some(t) => (value_to_u64(t), None),
            None => (None, None),
        };
        Ok(Self {
            timestamp,
            end_timestamp,
            description: items.get(1).and_then(value_to_string),
        })
    }
}

/// A funding round of a protocol
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProtocolRaise {
    /// Protocol name
    pub name: Option<String>,
    /// Round, e.g. `Seed`
    pub round: Option<String>,
    /// Amount raised, in millions of USD
    #[serde(default, deserialize_with = "opt_flex_f64")]
    pub amount: Option<f64>,
    /// Valuation, in millions of USD
    #[serde(default, deserialize_with = "opt_flex_f64")]
    pub valuation: Option<f64>,
    /// Source link
    pub source: Option<String>,
    /// Unix timestamp
    #[serde(default, deserialize_with = "opt_flex_u64")]
    pub date: Option<u64>,
    /// `DefiLlama` id
    pub defillama_id: Option<String>,
    /// Lead investors
    #[serde(default, deserialize_with = "null_default")]
    pub lead_investors: Vec<String>,
    /// Other investors
    #[serde(default, deserialize_with = "null_default")]
    pub other_investors: Vec<String>,
    /// All investors
    #[serde(default, deserialize_with = "null_default")]
    pub investors: Vec<String>,
}

/// A warning shown on a protocol's page
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WarningBanner {
    /// Message
    pub message: Option<String>,
    /// Shown until this date
    pub until: Option<String>,
    /// Severity: `low`, `alert` or `rug`
    pub level: Option<String>,
}

/// Protocol metadata and current TVL by chain, from
/// `/api/v2/metrics/tvl/protocol/{protocol}` and
/// `/api/v2/metrics/treasury/protocol/{protocol}`
///
/// `current_chain_tvls` keys are chains (`Ethereum`), TVL kinds
/// (`staking`, `borrowed`, `OwnTokens`) and both (`Ethereum-staking`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProtocolMetrics {
    /// Protocol id (`parent#slug` for parent protocols)
    #[serde(default)]
    pub id: String,
    /// Protocol name
    #[serde(default)]
    pub name: String,
    /// Token address
    pub address: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Website
    pub url: Option<String>,
    /// Referral URL
    pub referral_url: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Main chain
    pub chain: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
    /// Number of audits
    pub audits: Option<String>,
    /// Audit note
    #[serde(rename = "audit_note")]
    pub audit_note: Option<String>,
    /// `CoinGecko` id
    #[serde(rename = "gecko_id")]
    pub gecko_id: Option<String>,
    /// `CoinMarketCap` id
    pub cmc_id: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Tags
    #[serde(default, deserialize_with = "null_default")]
    pub tags: Vec<String>,
    /// Chains
    #[serde(default, deserialize_with = "null_default")]
    pub chains: Vec<String>,
    /// Adapter module path
    pub module: Option<String>,
    /// Treasury adapter
    pub treasury: Option<String>,
    /// Twitter handle
    pub twitter: Option<String>,
    /// Audit links
    #[serde(default, rename = "audit_links", deserialize_with = "null_default")]
    pub audit_links: Vec<String>,
    /// Whether the code is open source
    pub open_source: Option<bool>,
    /// Protocols this one is forked from
    #[serde(default, deserialize_with = "null_default")]
    pub forked_from: Vec<String>,
    /// Oracles used, by chain
    #[serde(default, deserialize_with = "null_default")]
    pub oracles_by_chain: HashMap<String, Vec<String>>,
    /// Parent protocol id
    pub parent_protocol: Option<String>,
    /// Governance ids
    #[serde(default, rename = "governanceID", deserialize_with = "null_default")]
    pub governance_id: Vec<String>,
    /// GitHub organizations
    #[serde(default, deserialize_with = "null_default")]
    pub github: Vec<String>,
    /// Current TVL (or treasury value) in USD by chain and kind
    #[serde(default, deserialize_with = "null_default")]
    pub current_chain_tvls: HashMap<String, f64>,
    /// Whether this is a parent protocol
    pub is_parent_protocol: Option<bool>,
    /// Token market cap in USD
    pub mcap: Option<f64>,
    /// TVL methodology
    pub methodology: Option<String>,
    /// Funding rounds
    #[serde(default, deserialize_with = "null_default")]
    pub raises: Vec<ProtocolRaise>,
    /// Names of the protocol's other versions
    #[serde(default, deserialize_with = "null_default")]
    pub other_protocols: Vec<String>,
    /// Notable events
    #[serde(default, deserialize_with = "null_default")]
    pub hallmarks: Vec<Hallmark>,
    /// Stablecoins issued by the protocol
    #[serde(default, deserialize_with = "null_default")]
    pub stablecoins: Vec<String>,
    /// Whether some tokens are misrepresented
    pub misrepresented_tokens: Option<bool>,
    /// Whether the protocol is deprecated
    pub deprecated: Option<bool>,
    /// Whether the protocol rugged
    pub rugged: Option<bool>,
    /// Whether the website is down
    pub dead_url: Option<bool>,
    /// Warnings shown on the protocol's page
    #[serde(default, deserialize_with = "null_default")]
    pub warning_banners: Vec<WarningBanner>,
    /// Token governance and value accrual rights. The spec declares an
    /// object without properties, so it is kept as JSON.
    pub token_rights: Option<Value>,
    /// Whether the token's liquidity is flagged as unreliable
    pub wrong_liquidity: Option<bool>,
    /// Link to the TVL adapter source
    pub tvl_code_path: Option<String>,
}

// ---------------------------------------------------------------------------
// Financial statement
// ---------------------------------------------------------------------------

/// A protocol's income statement: methodology and aggregated revenue line
/// items (`/api/v2/metrics/financial-statement/protocol/{protocol}`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FinancialStatement {
    /// Protocol id (`parent#slug` for parent protocols)
    #[serde(default)]
    pub id: String,
    /// Protocol name
    #[serde(default)]
    pub name: String,
    /// Token address
    pub address: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Website
    pub url: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Main chain (child protocols)
    pub chain: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
    /// Number of audits
    pub audits: Option<String>,
    /// Audit links
    #[serde(default, rename = "audit_links", deserialize_with = "null_default")]
    pub audit_links: Vec<String>,
    /// Category (child protocols)
    pub category: Option<String>,
    /// Chains
    #[serde(default, deserialize_with = "null_default")]
    pub chains: Vec<String>,
    /// Oracles used
    #[serde(default, deserialize_with = "null_default")]
    pub oracles_breakdown: Vec<OracleUsage>,
    /// Adapter module path
    pub module: Option<String>,
    /// Twitter handle
    pub twitter: Option<String>,
    /// GitHub organizations
    #[serde(default, deserialize_with = "null_default")]
    pub github: Vec<String>,
    /// Unix timestamp the protocol was listed at
    #[serde(default, deserialize_with = "opt_flex_u64")]
    pub listed_at: Option<u64>,
    /// Parent protocol id (child protocols)
    pub parent_protocol: Option<String>,
    /// Dimension adapters, keyed by metric (`fees`, `dexs`, ...)
    #[serde(default, deserialize_with = "null_default")]
    pub dimensions: HashMap<String, DimensionAdapter>,
    /// Line item descriptions (child protocols; parents carry them per
    /// child in `child_protocols`)
    #[serde(default, deserialize_with = "null_default")]
    pub methodology: HashMap<String, String>,
    /// Label descriptions per line item
    #[serde(default, deserialize_with = "null_default")]
    pub breakdown_methodology: HashMap<String, HashMap<String, String>>,
    /// Link to the methodology
    #[serde(rename = "methodologyURL")]
    pub methodology_url: Option<String>,
    /// Link to the TVL adapter source
    pub tvl_code_path: Option<String>,
    /// Notable events
    #[serde(default, deserialize_with = "null_default")]
    pub hallmarks: Vec<Hallmark>,
    /// Linked protocols
    #[serde(default, deserialize_with = "null_default")]
    pub linked_protocols: Vec<String>,
    /// Child protocols with their methodology (parent protocols)
    #[serde(default, deserialize_with = "null_default")]
    pub child_protocols: Vec<ChildProtocolMethodology>,
    /// Whether line items have label breakdowns
    pub has_label_breakdown: Option<bool>,
    /// `DefiLlama` id
    pub defillama_id: Option<String>,
    /// Display name
    pub display_name: Option<String>,
    /// Slug
    pub slug: Option<String>,
    /// Protocol type
    pub protocol_type: Option<String>,
    /// Line items aggregated by period
    #[serde(default, deserialize_with = "null_default")]
    pub aggregates: FinancialAggregates,
}

/// An oracle a protocol uses
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OracleUsage {
    /// Oracle name
    pub name: Option<String>,
    /// How it is used
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Links proving the usage
    #[serde(default, deserialize_with = "null_default")]
    pub proof: Vec<String>,
}

/// A dimension adapter of a protocol, sent as its name or as
/// `{"adapter": ..., "genuineSpikes": [...]}`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DimensionAdapter {
    /// Adapter name
    pub adapter: Option<String>,
    /// Days whose spikes are genuine, as `[date, note]` rows
    #[serde(rename = "genuineSpikes")]
    pub genuine_spikes: Vec<Vec<String>>,
}

impl<'de> Deserialize<'de> for DimensionAdapter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(s) => Ok(Self {
                adapter: Some(s),
                genuine_spikes: Vec::new(),
            }),
            Value::Object(map) => {
                let adapter = map.get("adapter").and_then(value_to_string);
                let genuine_spikes = map
                    .get("genuineSpikes")
                    .and_then(Value::as_array)
                    .map(|rows| {
                        rows.iter()
                            .map(|row| match row {
                                Value::Array(cells) => {
                                    cells.iter().filter_map(value_to_string).collect()
                                }
                                other => value_to_string(other).into_iter().collect(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(Self {
                    adapter,
                    genuine_spikes,
                })
            }
            Value::Null => Ok(Self::default()),
            v => Err(D::Error::custom(format!(
                "expected an adapter name or object, got {v}"
            ))),
        }
    }
}

/// Methodology of one child protocol
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ChildProtocolMethodology {
    /// Name
    pub name: Option<String>,
    /// `DefiLlama` id
    pub defillama_id: Option<String>,
    /// Display name
    pub display_name: Option<String>,
    /// Link to the methodology
    #[serde(rename = "methodologyURL")]
    pub methodology_url: Option<String>,
    /// Line item descriptions
    #[serde(default, deserialize_with = "null_default")]
    pub methodology: HashMap<String, String>,
    /// Label descriptions per line item
    #[serde(default, deserialize_with = "null_default")]
    pub breakdown_methodology: HashMap<String, HashMap<String, String>>,
}

/// Line items per period, keyed by period (`2024`, `2024-Q1`, `2024-01`)
/// and then by line item name (e.g. `Gross Protocol Revenue`)
pub type PeriodLineItems = HashMap<String, HashMap<String, LineItem>>;

/// Financial line items aggregated by year, quarter and month
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FinancialAggregates {
    /// By year (`2024`)
    #[serde(default, deserialize_with = "null_default")]
    pub yearly: PeriodLineItems,
    /// By quarter (`2024-Q1`)
    #[serde(default, deserialize_with = "null_default")]
    pub quarterly: PeriodLineItems,
    /// By month (`2024-01`)
    #[serde(default, deserialize_with = "null_default")]
    pub monthly: PeriodLineItems,
}

/// One line item in one period
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct LineItem {
    /// Total in USD
    #[serde(default, deserialize_with = "opt_flex_f64")]
    pub value: Option<f64>,
    /// USD per label (e.g. `Borrow Interest`), where the line item has
    /// labels
    #[serde(default, rename = "by-label", deserialize_with = "null_default")]
    pub by_label: HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_point_reads_pairs() {
        let points: Vec<ChartPoint> =
            serde_json::from_str(r#"[[1700000000, 1.5], ["1700086400", "2"]]"#).unwrap();
        assert_eq!(points[0], ChartPoint::new(1_700_000_000, 1.5));
        assert_eq!(points[1], ChartPoint::new(1_700_086_400, 2.0));
        assert!(serde_json::from_str::<ChartPoint>(r#"[1, {"a": 1}]"#).is_err());
        assert!(serde_json::from_str::<ChartPoint>("[1]").is_err());
    }

    #[test]
    fn breakdown_point_reads_both_shapes() {
        let pairs: Vec<BreakdownPoint> =
            serde_json::from_str(r#"[[1700000000, {"Ethereum": 3, "Solana": 1.5, "X": null}]]"#)
                .unwrap();
        assert_eq!(pairs[0].timestamp, 1_700_000_000);
        assert_eq!(pairs[0].values.len(), 2);
        assert_eq!(pairs[0].values["Solana"], 1.5);

        let rows: Vec<BreakdownPoint> =
            serde_json::from_str(r#"[{"timestamp": 1700000000, "Chainlink": 12.5}]"#).unwrap();
        assert_eq!(rows[0].timestamp, 1_700_000_000);
        assert_eq!(rows[0].values["Chainlink"], 12.5);
        assert!(!rows[0].values.contains_key("timestamp"));

        assert!(serde_json::from_str::<BreakdownPoint>(r#"{"Chainlink": 1}"#).is_err());
        assert!(serde_json::from_str::<BreakdownPoint>(r#"[1, {"a": "x"}]"#).is_err());
    }

    #[test]
    fn hallmarks_read_points_and_ranges() {
        let h: Vec<Hallmark> =
            serde_json::from_str(r#"[[1650412800, "Start"], [[1, 2], "Range"], []]"#).unwrap();
        assert_eq!(h[0].timestamp, Some(1_650_412_800));
        assert_eq!(h[0].description.as_deref(), Some("Start"));
        assert_eq!(h[1].timestamp, Some(1));
        assert_eq!(h[1].end_timestamp, Some(2));
        assert_eq!(h[2], Hallmark::default());
    }

    #[test]
    fn dimension_adapter_reads_name_or_object() {
        let d: HashMap<String, DimensionAdapter> = serde_json::from_str(
            r#"{"fees": "aave", "dexs": {"adapter": "x", "genuineSpikes": [["2024-08-05", "-"]]}}"#,
        )
        .unwrap();
        assert_eq!(d["fees"].adapter.as_deref(), Some("aave"));
        assert_eq!(d["dexs"].genuine_spikes, vec![vec!["2024-08-05", "-"]]);
    }
}
