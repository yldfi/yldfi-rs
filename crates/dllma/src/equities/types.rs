//! Types for public company (equities) data (`/equities/v1/*`, Pro)
//!
//! Money figures are in USD and `*Percentage*`, yield and margin figures in
//! percent. Structs are `#[non_exhaustive]` so new upstream fields can be
//! added without a breaking change.

use std::collections::HashMap;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::de::{null_default, value_to_f64, value_to_string, value_to_u64};

/// Lookback window of a price chart (`timeframe`); the default is `MAX`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Timeframe {
    /// The latest trading session, at 5-minute resolution
    Day1,
    /// The last 7 days, at 5-minute resolution
    Days7,
    /// The last month
    Month1,
    /// The last 3 months
    Months3,
    /// The last 6 months
    Months6,
    /// Since the start of the year
    YearToDate,
    /// The last year
    Year1,
    /// The last 5 years
    Years5,
    /// All history
    Max,
}

impl Timeframe {
    /// The query value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day1 => "1D",
            Self::Days7 => "7D",
            Self::Month1 => "1M",
            Self::Months3 => "3M",
            Self::Months6 => "6M",
            Self::YearToDate => "YTD",
            Self::Year1 => "1Y",
            Self::Years5 => "5Y",
            Self::Max => "MAX",
        }
    }
}

/// Status of an earnings date
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EarningsStatus {
    /// Reported in a filing
    Reported,
    /// Inferred from filings
    Inferred,
    /// Confirmed by the company
    Confirmed,
    /// Estimated from the company's cadence
    Estimated,
}

impl EarningsStatus {
    /// The query value
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reported => "reported",
            Self::Inferred => "inferred",
            Self::Confirmed => "confirmed",
            Self::Estimated => "estimated",
        }
    }
}

/// Filters of [`super::EquitiesApi::earnings_calendar`]
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct EarningsQuery {
    /// One company's full calendar
    pub ticker: Option<String>,
    /// Earliest report date, `YYYY-MM-DD` (inclusive)
    pub from: Option<String>,
    /// Latest report date, `YYYY-MM-DD` (inclusive)
    pub to: Option<String>,
    /// Only rows with this status
    pub status: Option<EarningsStatus>,
    /// Maximum rows (the API caps it at 20000)
    pub limit: Option<u32>,
}

impl EarningsQuery {
    /// No filters
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// One company's full calendar
    #[must_use]
    pub fn ticker(mut self, ticker: impl Into<String>) -> Self {
        self.ticker = Some(ticker.into());
        self
    }

    /// Earliest report date, `YYYY-MM-DD`
    #[must_use]
    pub fn from(mut self, date: impl Into<String>) -> Self {
        self.from = Some(date.into());
        self
    }

    /// Latest report date, `YYYY-MM-DD`
    #[must_use]
    pub fn to(mut self, date: impl Into<String>) -> Self {
        self.to = Some(date.into());
        self
    }

    /// Only rows with this status
    #[must_use]
    pub fn status(mut self, status: EarningsStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Maximum rows
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// A tracked public company (`/equities/v1/companies-list`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Company {
    /// Ticker
    #[serde(default)]
    pub ticker: String,
    /// Company name
    pub company_name: Option<String>,
    /// ISO 3166-1 alpha-2 country code, as the other endpoints take it
    pub country: Option<String>,
    /// Country name
    pub country_name: Option<String>,
    /// Sector slug
    pub sector_slug: Option<String>,
    /// Industry slug
    pub industry_slug: Option<String>,
}

/// A company's income statement, balance sheet and cash flow statement
/// (`/equities/v1/statements`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FinancialStatements {
    /// Income statement
    #[serde(default, deserialize_with = "null_default")]
    pub income_statement: Statement,
    /// Balance sheet
    #[serde(default, deserialize_with = "null_default")]
    pub balance_sheet: Statement,
    /// Cash flow statement
    #[serde(default, deserialize_with = "null_default")]
    pub cashflow: Statement,
}

/// One financial statement: row labels and their values per period
///
/// `quarterly.values[i]` (and `annual.values[i]`) holds row `labels[i]`'s
/// value for each period in `period_ending`. Rows with sub-rows have them
/// under `children`, keyed by the parent row's label.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Statement {
    /// Row labels
    #[serde(default, deserialize_with = "null_default")]
    pub labels: Vec<String>,
    /// Sub-row labels
    #[serde(default, deserialize_with = "null_default")]
    pub children: StatementChildren,
    /// Quarterly values
    #[serde(default, deserialize_with = "null_default")]
    pub quarterly: StatementPeriods,
    /// Annual values
    #[serde(default, deserialize_with = "null_default")]
    pub annual: StatementPeriods,
}

/// Sub-row labels keyed by parent row label
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StatementChildren {
    /// Quarterly
    #[serde(default, deserialize_with = "null_default")]
    pub quarterly: HashMap<String, StatementLabels>,
    /// Annual
    #[serde(default, deserialize_with = "null_default")]
    pub annual: HashMap<String, StatementLabels>,
}

/// Labels of a row's sub-rows
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StatementLabels {
    /// Sub-row labels
    #[serde(default, deserialize_with = "null_default")]
    pub labels: Vec<String>,
}

/// Values of a statement's rows per period
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatementPeriods {
    /// Period end dates
    #[serde(default, deserialize_with = "null_default")]
    pub period_ending: Vec<String>,
    /// One row of values per label, one value per period
    #[serde(default, deserialize_with = "null_default")]
    pub values: Vec<Vec<Option<f64>>>,
    /// Sub-row values keyed by parent row label
    #[serde(default, deserialize_with = "null_default")]
    pub children: HashMap<String, StatementValues>,
}

/// Values of a row's sub-rows
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StatementValues {
    /// One row of values per sub-row label, one value per period
    #[serde(default, deserialize_with = "null_default")]
    pub values: Vec<Vec<Option<f64>>>,
}

/// A value at a date, read from `[date, value]`
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct DatedValue {
    /// Date (`YYYY-MM-DD`) or date-time (RFC 3339)
    pub date: String,
    /// Value, `None` when not a number
    pub value: Option<f64>,
}

impl<'de> Deserialize<'de> for DatedValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        let pair = v
            .as_array()
            .filter(|a| a.len() >= 2)
            .ok_or_else(|| D::Error::custom(format!("expected [date, value], got {v}")))?;
        let date = value_to_string(&pair[0])
            .ok_or_else(|| D::Error::custom(format!("expected a date, got {}", pair[0])))?;
        Ok(Self {
            date,
            value: value_to_f64(&pair[1]),
        })
    }
}

/// Annual and quarterly history of one financial dimension (revenue,
/// earnings, shares outstanding, ...), newest first
/// (`/equities/v1/dimensions`, keyed by dimension name)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DimensionHistory {
    /// By fiscal year end
    #[serde(default, deserialize_with = "null_default")]
    pub annual: Vec<DatedValue>,
    /// By fiscal quarter end
    #[serde(default, deserialize_with = "null_default")]
    pub quarterly: Vec<DatedValue>,
}

/// A daily (or 5-minute) candle, read from
/// `[timestamp, open, high, low, close, volume]`
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Candle {
    /// Unix timestamp (seconds)
    pub timestamp: u64,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume, in shares
    pub volume: f64,
}

impl<'de> Deserialize<'de> for Candle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        let row = v.as_array().filter(|a| a.len() >= 6).ok_or_else(|| {
            D::Error::custom(format!(
                "expected [timestamp, open, high, low, close, volume], got {v}"
            ))
        })?;
        let num = |i: usize| {
            value_to_f64(&row[i])
                .ok_or_else(|| D::Error::custom(format!("expected a number, got {}", row[i])))
        };
        Ok(Self {
            timestamp: value_to_u64(&row[0])
                .ok_or_else(|| D::Error::custom(format!("expected a timestamp, got {}", row[0])))?,
            open: num(1)?,
            high: num(2)?,
            low: num(3)?,
            close: num(4)?,
            volume: num(5)?,
        })
    }
}

/// Live market data and trailing financials of a company
/// (`/equities/v1/summary`)
///
/// `TTM` is trailing twelve months.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EquitySummary {
    /// Share price
    pub current_price: Option<f64>,
    /// Volume, in shares
    pub volume: Option<f64>,
    /// Market cap
    pub market_cap: Option<f64>,
    /// Market cap of the circulating supply
    pub circulating_market_cap: Option<f64>,
    /// Enterprise value
    pub enterprise_value: Option<f64>,
    /// 52-week high
    pub fifty_two_week_high: Option<f64>,
    /// 52-week low
    pub fifty_two_week_low: Option<f64>,
    /// Dividend yield
    pub dividend_yield: Option<f64>,
    /// Trailing price to earnings
    #[serde(rename = "trailingPE")]
    pub trailing_pe: Option<f64>,
    /// Price to revenue
    pub price_to_revenue: Option<f64>,
    /// Price change over 1 day
    pub price_change_percentage_1d: Option<f64>,
    /// Price change over 7 days
    pub price_change_percentage_7d: Option<f64>,
    /// Price change over 1 month
    pub price_change_percentage_1m: Option<f64>,
    /// Price change over 1 day, in USD
    pub price_change_1d: Option<f64>,
    /// Market cap change over 1 day, in USD
    pub market_cap_change_1d: Option<f64>,
    /// Price to book
    pub price_to_book: Option<f64>,
    /// Enterprise value to EBITDA
    pub enterprise_value_to_ebitda: Option<f64>,
    /// Dividends plus buybacks, as a yield
    pub holders_yield: Option<f64>,
    /// When this was updated (RFC 3339)
    pub updated_at: Option<String>,
    /// Revenue, TTM
    #[serde(rename = "revenueTTM")]
    pub revenue_ttm: Option<f64>,
    /// Gross profit, TTM
    #[serde(rename = "grossProfitTTM")]
    pub gross_profit_ttm: Option<f64>,
    /// Earnings, TTM
    #[serde(rename = "earningsTTM")]
    pub earnings_ttm: Option<f64>,
    /// EBITDA, TTM
    #[serde(rename = "ebitdaTTM")]
    pub ebitda_ttm: Option<f64>,
    /// Operating margin, TTM
    #[serde(rename = "operatingProfitMarginTTM")]
    pub operating_profit_margin_ttm: Option<f64>,
    /// Revenue to holders, TTM
    #[serde(rename = "holdersRevenueTTM")]
    pub holders_revenue_ttm: Option<f64>,
    /// Earnings to holders, TTM
    #[serde(rename = "holderEarningsTTM")]
    pub holder_earnings_ttm: Option<f64>,
    /// Dividends paid, TTM
    #[serde(rename = "dividendsTTM")]
    pub dividends_ttm: Option<f64>,
    /// Buybacks, TTM
    #[serde(rename = "stockRepurchaseTTM")]
    pub stock_repurchase_ttm: Option<f64>,
    /// Shares issued, TTM
    #[serde(rename = "stockIssuanceTTM")]
    pub stock_issuance_ttm: Option<f64>,
    /// Stock-based compensation, TTM
    #[serde(rename = "stockBasedCompensationTTM")]
    pub stock_based_compensation_ttm: Option<f64>,
    /// Cash and cash equivalents
    pub cash_and_cash_equivalents: Option<f64>,
    /// Total assets
    pub total_assets: Option<f64>,
    /// Total liabilities
    pub total_liabilities: Option<f64>,
    /// Shareholders' equity
    pub total_shareholders_equity: Option<f64>,
    /// Total debt
    pub total_debt: Option<f64>,
    /// Circulating shares
    pub circulating_supply: Option<f64>,
    /// Total shares
    pub total_supply: Option<f64>,
    /// Employees
    pub employee_count: Option<u64>,
}

/// A regulatory filing (`/equities/v1/filings`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Filing {
    /// Filing date (`YYYY-MM-DD`)
    pub filing_date: Option<String>,
    /// Report period date (`YYYY-MM-DD`)
    pub report_date: Option<String>,
    /// Form, e.g. `10-K`
    pub form: Option<String>,
    /// Link to the filing
    pub primary_document_url: Option<String>,
    /// Description
    pub document_description: Option<String>,
}

/// Onchain markets trading a company (`/equities/v1/onchain`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OnchainMarkets {
    /// Perpetual futures
    #[serde(default, deserialize_with = "null_default")]
    pub perps: Vec<EquityPerp>,
    /// Tokenized shares
    #[serde(default, deserialize_with = "null_default")]
    pub tokens: Vec<EquityToken>,
}

/// A perpetual futures market on a company's stock
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EquityPerp {
    /// Pair, e.g. `AAPL-USDC`
    pub pair: Option<String>,
    /// Last price
    pub price: Option<f64>,
    /// 24h volume
    pub volume_24h: Option<f64>,
    /// Open interest
    pub open_interest: Option<f64>,
    /// Annualized funding rate, in percent
    pub annualized_funding_rate: Option<f64>,
    /// Trade link
    pub trade_url: Option<String>,
    /// `DefiLlama` contract id
    pub contract_slug: Option<String>,
    /// Exchange protocol slug
    pub exchange_protocol_slug: Option<String>,
    /// RWA platform slug
    pub rwa_platform_slug: Option<String>,
    /// `CEX` or `DEX`
    pub exchange_type: Option<String>,
    /// When this was updated (RFC 3339)
    pub updated_at: Option<String>,
}

/// A token tracking a company's stock
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EquityToken {
    /// Issuer
    pub issuer: Option<String>,
    /// Issuer's RWA platform slug
    pub issuer_rwa_platform_slug: Option<String>,
    /// Token price
    pub price: Option<f64>,
    /// Token symbol, e.g. `AAPLx`
    pub asset_slug: Option<String>,
    /// Active market cap
    pub active_marketcap: Option<f64>,
}

/// One of a sector's or industry's largest companies
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TopHolding {
    /// Ticker
    pub ticker: Option<String>,
    /// Company name
    pub name: Option<String>,
    /// Market cap
    pub market_cap: Option<f64>,
    /// Share of the sector's market cap, in percent
    pub sector_weight: Option<f64>,
    /// Price change over 1 day
    pub price_change_percentage_1d: Option<f64>,
}

/// Defines a struct with the aggregate market fields that sectors,
/// industries and the market overview share, plus its own fields.
/// (`#[serde(flatten)]` would read numbers wrongly with
/// `serde_json/arbitrary_precision`, so the fields are repeated.)
macro_rules! market_stats {
    (
        $(#[$meta:meta])*
        pub struct $name:ident { $($body:tt)* }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, Deserialize, Serialize)]
        #[serde(rename_all = "camelCase")]
        #[non_exhaustive]
        pub struct $name {
            $($body)*
            /// Market cap
            pub market_cap: Option<f64>,
            /// Market cap change over 1 day, in USD
            pub market_cap_change_1d: Option<f64>,
            /// Dollar volume over 24 hours
            pub dollar_volume_24h: Option<f64>,
            /// Revenue, trailing twelve months
            #[serde(rename = "revenueTTM")]
            pub revenue_ttm: Option<f64>,
            /// Earnings, trailing twelve months
            #[serde(rename = "earningsTTM")]
            pub earnings_ttm: Option<f64>,
            /// EBITDA, trailing twelve months
            #[serde(rename = "ebitdaTTM")]
            pub ebitda_ttm: Option<f64>,
            /// Enterprise value
            pub enterprise_value: Option<f64>,
            /// Employees
            pub employee_count: Option<f64>,
            /// Trailing price to earnings
            #[serde(rename = "trailingPE")]
            pub trailing_pe: Option<f64>,
            /// Earnings yield
            pub earnings_yield: Option<f64>,
            /// Price to revenue
            pub price_to_revenue: Option<f64>,
            /// Price to book
            pub price_to_book: Option<f64>,
            /// Enterprise value to EBITDA
            pub enterprise_value_to_ebitda: Option<f64>,
            /// Dividend yield
            pub dividend_yield: Option<f64>,
            /// Dividends plus buybacks, as a yield
            pub holders_yield: Option<f64>,
            /// Operating margin, trailing twelve months
            #[serde(rename = "operatingProfitMarginTTM")]
            pub operating_profit_margin_ttm: Option<f64>,
            /// Price change over 1 day
            pub price_change_percentage_1d: Option<f64>,
            /// Price change over 7 days
            pub price_change_percentage_7d: Option<f64>,
            /// Price change over 1 month
            pub price_change_percentage_1m: Option<f64>,
            /// Price change over 3 months
            pub price_change_percentage_3m: Option<f64>,
            /// Price change over 6 months
            pub price_change_percentage_6m: Option<f64>,
            /// Price change since the start of the year
            pub price_change_percentage_ytd: Option<f64>,
            /// Price change over 1 year
            pub price_change_percentage_1y: Option<f64>,
            /// Companies included
            pub company_count: Option<u64>,
            /// Share of companies with data, per field
            #[serde(default, deserialize_with = "null_default")]
            pub coverage: HashMap<String, f64>,
            /// When this was updated (RFC 3339)
            pub updated_at: Option<String>,
            /// Time of the volume data (RFC 3339)
            pub volume_as_of: Option<String>,
        }
    };
}

market_stats! {
    /// A sector's aggregate market data (`/equities/v1/sectors`)
    pub struct Sector {
        /// Sector name
        pub sector: Option<String>,
        /// Sector slug
        pub sector_slug: Option<String>,
        /// Rank by market cap
        pub rank: Option<u32>,
        /// Share of the total market cap, in percent
        pub weight: Option<f64>,
        /// Share of the total 24h dollar volume, in percent
        pub dollar_volume_share_24h: Option<f64>,
        /// Number of industries
        pub industry_count: Option<u32>,
        /// Largest companies
        #[serde(default, deserialize_with = "null_default")]
        pub top_holdings: Vec<TopHolding>,
    }
}

market_stats! {
    /// One sector in detail (`/equities/v1/sector`)
    pub struct SectorDetail {
        /// Sector name
        pub sector: Option<String>,
        /// Sector slug
        pub sector_slug: Option<String>,
        /// Rank by market cap
        pub rank: Option<u32>,
        /// Share of the total market cap, in percent
        pub weight: Option<f64>,
        /// Share of the total 24h dollar volume, in percent
        pub dollar_volume_share_24h: Option<f64>,
        /// Number of industries
        pub industry_count: Option<u32>,
        /// Largest companies
        #[serde(default, deserialize_with = "null_default")]
        pub top_holdings: Vec<TopHolding>,
        /// Description
        pub description: Option<String>,
        /// Valuation ratios, aggregate and median
        #[serde(default, deserialize_with = "null_default")]
        pub valuation: SectorValuation,
        /// Market cap concentration
        #[serde(default, deserialize_with = "null_default")]
        pub concentration: SectorConcentration,
    }
}

market_stats! {
    /// An industry's aggregate market data (`/equities/v1/industries`)
    pub struct Industry {
        /// Industry name
        pub industry: Option<String>,
        /// Industry slug
        pub industry_slug: Option<String>,
        /// Sector name
        pub sector: Option<String>,
        /// Sector slug
        pub sector_slug: Option<String>,
        /// Rank by market cap
        pub rank: Option<u32>,
        /// Share of the total market cap, in percent
        pub weight: Option<f64>,
        /// Share of its sector's market cap, in percent
        pub sector_weight: Option<f64>,
        /// Share of the total 24h dollar volume, in percent
        pub dollar_volume_share_24h: Option<f64>,
        /// Largest companies
        #[serde(default, deserialize_with = "null_default")]
        pub top_holdings: Vec<TopHolding>,
    }
}

market_stats! {
    /// Market-wide aggregates across sectors
    /// (`/equities/v1/sectors-overview`)
    pub struct SectorsOverview {
        /// Median trailing price to earnings
        #[serde(rename = "medianTrailingPE")]
        pub median_trailing_pe: Option<f64>,
        /// Median price to revenue
        pub median_price_to_revenue: Option<f64>,
        /// Median price to book
        pub median_price_to_book: Option<f64>,
        /// Median enterprise value to EBITDA
        pub median_enterprise_value_to_ebitda: Option<f64>,
        /// Median operating margin
        #[serde(rename = "medianOperatingProfitMarginTTM")]
        pub median_operating_profit_margin_ttm: Option<f64>,
        /// Median dividend yield
        pub median_dividend_yield: Option<f64>,
        /// Median market cap
        pub median_market_cap: Option<f64>,
        /// Number of sectors
        pub sector_count: Option<u32>,
        /// Number of industries
        pub industry_count: Option<u32>,
        /// Best sector over 1 day
        pub best_sector_1d: Option<SectorMove>,
        /// Worst sector over 1 day
        pub worst_sector_1d: Option<SectorMove>,
        /// Largest sector
        pub largest_sector: Option<SectorMove>,
    }
}

/// A sector's 1-day move
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorMove {
    /// Sector name
    pub sector: Option<String>,
    /// Sector slug
    pub sector_slug: Option<String>,
    /// Price change over 1 day
    pub price_change_percentage_1d: Option<f64>,
    /// Share of the total market cap, in percent
    pub weight: Option<f64>,
}

/// A sector's valuation ratios
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorValuation {
    /// Trailing price to earnings
    #[serde(rename = "trailingPE")]
    pub trailing_pe: Option<f64>,
    /// Earnings yield
    pub earnings_yield: Option<f64>,
    /// Price to revenue
    pub price_to_revenue: Option<f64>,
    /// Price to book
    pub price_to_book: Option<f64>,
    /// Enterprise value to EBITDA
    pub enterprise_value_to_ebitda: Option<f64>,
    /// Operating margin, trailing twelve months
    #[serde(rename = "operatingProfitMarginTTM")]
    pub operating_profit_margin_ttm: Option<f64>,
    /// Dividend yield
    pub dividend_yield: Option<f64>,
    /// Median trailing price to earnings
    #[serde(rename = "medianTrailingPE")]
    pub median_trailing_pe: Option<f64>,
    /// Median price to revenue
    pub median_price_to_revenue: Option<f64>,
    /// Median price to book
    pub median_price_to_book: Option<f64>,
    /// Median enterprise value to EBITDA
    pub median_enterprise_value_to_ebitda: Option<f64>,
    /// Median operating margin
    #[serde(rename = "medianOperatingProfitMarginTTM")]
    pub median_operating_profit_margin_ttm: Option<f64>,
    /// Median dividend yield
    pub median_dividend_yield: Option<f64>,
}

/// How concentrated a sector's market cap is
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorConcentration {
    /// Share of the 10 largest companies, in percent
    pub top10_market_cap_share: Option<f64>,
    /// Herfindahl-Hirschman index
    pub hhi: Option<f64>,
    /// Companies above the mega-cap threshold
    pub mega_cap_count: Option<u32>,
    /// Mega-cap threshold
    pub mega_cap_threshold: Option<f64>,
    /// Median market cap
    pub median_market_cap: Option<f64>,
}

/// An earnings date (`/equities/v1/earnings-calendar`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct EarningsEvent {
    /// Ticker
    #[serde(default)]
    pub ticker: String,
    /// Country code
    pub iso2_country_code: Option<String>,
    /// SEC CIK
    pub cik: Option<String>,
    /// Event key, e.g. `2026-Q3`
    pub event_key: Option<String>,
    /// `reported`, `inferred`, `confirmed` or `estimated`
    pub date_status: Option<String>,
    /// Report date (`YYYY-MM-DD`)
    pub report_date: Option<String>,
    /// `bmo` (before market open), `dmt` (during market), `amc` (after
    /// market close) or `unknown`
    pub report_time: Option<String>,
    /// When the filing was accepted (empty when not reported)
    pub accepted_at: Option<String>,
    /// Fiscal year
    pub fiscal_year: Option<i32>,
    /// Fiscal quarter, `Q1` to `Q4`
    pub fiscal_period: Option<String>,
    /// Period end date
    pub period_end: Option<String>,
    /// Where the date comes from: `8k_item_202`, `periodic_filing` or
    /// `cadence_model`
    pub source: Option<String>,
    /// Source form, e.g. `8-K`
    pub source_form: Option<String>,
    /// SEC accession number
    pub accession_number: Option<String>,
    /// How far an estimated date may move, in days
    pub max_drift_days: Option<i64>,
    /// When the row was first published
    pub first_published_at: Option<String>,
    /// Times the date has moved
    pub revision_count: Option<u32>,
    /// The report date before the latest move (empty if none)
    pub previous_report_date: Option<String>,
    /// When the row was last written
    pub inserted_at: Option<String>,
}
