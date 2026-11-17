//! Types for pre-IPO company data (`/pre-ipo/v1/*`, Pro)
//!
//! Money figures are in USD. Structs are `#[non_exhaustive]` so new
//! upstream fields can be added without a breaking change.

use serde::{Deserialize, Serialize};

use crate::de::null_default;

/// Pre-IPO response wrapper: `{"updatedAt": ..., "data": ...}`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PreIpoResponse<T> {
    /// When this snapshot was generated (RFC 3339)
    pub updated_at: Option<String>,
    /// Response data
    pub data: T,
}

/// A tracked pre-IPO company (`/pre-ipo/v1/companies-list`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PreIpoCompany {
    /// Company id, as the other pre-IPO endpoints take it
    #[serde(default)]
    pub id: String,
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Website
    pub website: Option<String>,
    /// Sector
    pub sector: Option<String>,
    /// Latest monthly secondary-market valuation estimate, or the latest
    /// round's post-money valuation when there is none
    pub latest_estimated_valuation: Option<f64>,
    /// Amount raised in the latest round
    pub latest_raise: Option<f64>,
    /// Post-money valuation of the latest round
    pub latest_funding_valuation: Option<f64>,
    /// Date of the latest round (`YYYY-MM-DD`)
    pub latest_funding_date: Option<String>,
    /// Date of the latest secondary-market mark (`YYYY-MM-DD`)
    pub last_valuation_date: Option<String>,
    /// Sum of all disclosed rounds
    pub total_raised: Option<f64>,
    /// Change of the secondary estimate over a year, in percent
    #[serde(rename = "valuationChange1YPercentage")]
    pub valuation_change_1y_percentage: Option<f64>,
    /// Premium of the secondary estimate over the latest round's
    /// valuation, in percent
    pub premium_vs_latest_funding_percentage: Option<f64>,
}

/// A monthly valuation estimate (`/pre-ipo/v1/valuations`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Valuation {
    /// First day of the month (`YYYY-MM-DD`)
    #[serde(default)]
    pub date: String,
    /// Valuation estimate for the month
    pub estimated_valuation: Option<f64>,
}

/// A funding round (`/pre-ipo/v1/raises`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FundingRound {
    /// Announcement date (`YYYY-MM-DD`)
    #[serde(default)]
    pub date: String,
    /// Amount raised
    pub amount_raised: Option<f64>,
    /// Post-money valuation, when disclosed
    pub valuation: Option<f64>,
    /// Round label, e.g. `Series C`
    pub series: Option<String>,
}

/// A pre-IPO company's profile and latest valuation
/// (`/pre-ipo/v1/summary`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PreIpoSummary {
    /// Company id
    #[serde(default)]
    pub id: String,
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Short summary
    pub summary: Option<String>,
    /// Website
    pub website: Option<String>,
    /// Sector
    pub sector: Option<String>,
    /// Latest valuation estimate (see [`PreIpoCompany`])
    pub latest_estimated_valuation: Option<f64>,
    /// Date of the latest secondary-market mark (`YYYY-MM-DD`)
    pub last_valuation_date: Option<String>,
    /// Latest funding round
    pub latest_raise: Option<FundingRound>,
}

/// Where a pre-IPO company can be traded (`/pre-ipo/v1/integrations`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PreIpoIntegrations {
    /// Perpetual futures markets
    #[serde(default, deserialize_with = "null_default")]
    pub perps: Vec<PreIpoPerp>,
    /// Tokenized exposure to the company's shares
    #[serde(default, deserialize_with = "null_default")]
    pub token_issuers: Vec<PreIpoToken>,
}

/// A perpetual futures market on a pre-IPO company
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PreIpoPerp {
    /// Venue name
    pub exchange_name: Option<String>,
    /// `DefiLlama` venue slug
    pub platform_slug: Option<String>,
    /// `DefiLlama` market id, `{venue}:{pair}`
    pub defillama_contract: Option<String>,
    /// `CEX` or `DEX`
    pub exchange_type: Option<String>,
    /// Open interest
    pub open_interest: Option<f64>,
    /// 24h volume
    pub volume: Option<f64>,
    /// Last price
    pub price: Option<f64>,
    /// Annualized funding rate, in percent
    pub annualized_funding_rate: Option<f64>,
    /// Trade link
    pub trade_url: Option<String>,
    /// Company valuation implied by the price
    pub estimated_valuation: Option<f64>,
}

/// A token giving exposure to a pre-IPO company's shares
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PreIpoToken {
    /// Token issuer
    pub issuer_name: Option<String>,
    /// `DefiLlama` RWA platform slug of the issuer
    pub slug: Option<String>,
    /// What the token is a claim on: `Direct shares`, `Trust-held SPV`,
    /// `SPV interest`, `Unsecured note` or `Synthetic`
    pub backing: Option<String>,
    /// Product link
    pub product_url: Option<String>,
    /// Token price
    pub price: Option<f64>,
    /// Token market cap, where the issuer publishes one
    pub market_cap: Option<f64>,
    /// Company valuation implied by the price
    pub estimated_valuation: Option<f64>,
}
