//! Types for swap quote aggregation

use serde::{Deserialize, Serialize};

use crate::aggregator::SourceResult;

/// Swap source enum for CLI selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapSource {
    All,
    OpenOcean,
    KyberSwap,
    Zerox,
    OneInch,
    CowSwap,
    LiFi,
    Velora,
    Enso,
}

impl std::str::FromStr for SwapSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(SwapSource::All),
            "openocean" | "ocean" | "oo" => Ok(SwapSource::OpenOcean),
            "kyberswap" | "kyber" | "kbr" => Ok(SwapSource::KyberSwap),
            "0x" | "zerox" | "zrx" => Ok(SwapSource::Zerox),
            "1inch" | "oneinch" | "inch" => Ok(SwapSource::OneInch),
            "cowswap" | "cow" => Ok(SwapSource::CowSwap),
            "lifi" | "li.fi" => Ok(SwapSource::LiFi),
            "velora" | "paraswap" | "para" => Ok(SwapSource::Velora),
            "enso" | "ensofi" => Ok(SwapSource::Enso),
            _ => Err(format!("Unknown swap source: {}", s)),
        }
    }
}

impl std::fmt::Display for SwapSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapSource::All => write!(f, "all"),
            SwapSource::OpenOcean => write!(f, "openocean"),
            SwapSource::KyberSwap => write!(f, "kyberswap"),
            SwapSource::Zerox => write!(f, "0x"),
            SwapSource::OneInch => write!(f, "1inch"),
            SwapSource::CowSwap => write!(f, "cowswap"),
            SwapSource::LiFi => write!(f, "lifi"),
            SwapSource::Velora => write!(f, "velora"),
            SwapSource::Enso => write!(f, "enso"),
        }
    }
}

/// Normalized swap quote with consistent formatting across all sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedQuote {
    /// Source aggregator name
    pub source: String,
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount (raw, with decimals)
    pub amount_in: String,
    /// Output amount (raw, with decimals)
    pub amount_out: String,
    /// Minimum output after slippage (if set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_out_min: Option<String>,
    /// Estimated gas cost
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_gas: Option<u64>,
    /// Gas price in wei
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Estimated gas cost in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_usd: Option<f64>,
    /// Price impact percentage (negative means loss)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_impact: Option<f64>,
    /// Router contract address to approve/call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub router_address: Option<String>,
    /// Encoded transaction data (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_data: Option<String>,
    /// Value to send with transaction (for native token swaps)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_value: Option<String>,
    /// Swap route path (token addresses or pool names)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Vec<String>>,
    /// Protocols/DEXs used in the route
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<String>>,
    /// Protocol fee amount (e.g., CowSwap fee in sell token units)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<String>,
    /// Token the fee is denominated in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_token: Option<String>,
}

impl NormalizedQuote {
    /// Create a new normalized quote with required fields
    pub fn new(
        source: impl Into<String>,
        token_in: impl Into<String>,
        token_out: impl Into<String>,
        amount_in: impl Into<String>,
        amount_out: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            token_in: token_in.into(),
            token_out: token_out.into(),
            amount_in: amount_in.into(),
            amount_out: amount_out.into(),
            amount_out_min: None,
            estimated_gas: None,
            gas_price: None,
            gas_usd: None,
            price_impact: None,
            router_address: None,
            tx_data: None,
            tx_value: None,
            route: None,
            protocols: None,
            fee_amount: None,
            fee_token: None,
        }
    }

    pub fn with_min_out(mut self, min: impl Into<String>) -> Self {
        self.amount_out_min = Some(min.into());
        self
    }

    pub fn with_gas(mut self, gas: u64) -> Self {
        self.estimated_gas = Some(gas);
        self
    }

    pub fn with_gas_price(mut self, price: impl Into<String>) -> Self {
        self.gas_price = Some(price.into());
        self
    }

    pub fn with_gas_usd(mut self, usd: f64) -> Self {
        self.gas_usd = Some(usd);
        self
    }

    pub fn with_price_impact(mut self, impact: f64) -> Self {
        self.price_impact = Some(impact);
        self
    }

    pub fn with_router(mut self, router: impl Into<String>) -> Self {
        self.router_address = Some(router.into());
        self
    }

    pub fn with_tx_data(mut self, data: impl Into<String>) -> Self {
        self.tx_data = Some(data.into());
        self
    }

    pub fn with_tx_value(mut self, value: impl Into<String>) -> Self {
        self.tx_value = Some(value.into());
        self
    }

    pub fn with_route(mut self, route: Vec<String>) -> Self {
        self.route = Some(route);
        self
    }

    pub fn with_protocols(mut self, protocols: Vec<String>) -> Self {
        self.protocols = Some(protocols);
        self
    }

    pub fn with_fee(mut self, amount: impl Into<String>, token: impl Into<String>) -> Self {
        self.fee_amount = Some(amount.into());
        self.fee_token = Some(token.into());
        self
    }

    /// Parse amount_out as u128 for comparison
    pub fn amount_out_u128(&self) -> Option<u128> {
        self.amount_out.parse().ok()
    }

    /// Calculate effective output (output minus gas cost in output token terms)
    /// This is a simplified calculation - real implementation would need price data
    pub fn effective_output(&self) -> Option<u128> {
        // For now, just use raw output. A more sophisticated version would
        // convert gas_usd to output token terms and subtract.
        self.amount_out_u128()
    }
}

/// Aggregated quote result with best quote selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteAggregation {
    /// Best quote by output amount
    pub best_quote: Option<BestQuote>,
    /// Number of sources that returned valid quotes
    pub valid_quotes: usize,
    /// Total sources queried
    pub total_sources: usize,
    /// Average output across all valid quotes
    pub avg_output: Option<String>,
    /// Output spread percentage ((max-min)/max * 100)
    pub output_spread_pct: Option<f64>,
}

/// Best quote summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestQuote {
    /// Source that provided the best quote
    pub source: String,
    /// Output amount
    pub amount_out: String,
    /// Gas cost in USD (if available)
    pub gas_usd: Option<f64>,
    /// Price impact (if available)
    pub price_impact: Option<f64>,
    /// Improvement over worst quote (percentage)
    pub improvement_pct: Option<f64>,
}

impl QuoteAggregation {
    /// Create aggregation from quote results
    pub fn from_quotes(results: &[SourceResult<NormalizedQuote>]) -> Self {
        let valid_quotes: Vec<&NormalizedQuote> =
            results.iter().filter_map(|r| r.data.as_ref()).collect();

        let total_sources = results.len();
        let valid_count = valid_quotes.len();

        if valid_quotes.is_empty() {
            return Self {
                best_quote: None,
                valid_quotes: 0,
                total_sources,
                avg_output: None,
                output_spread_pct: None,
            };
        }

        // Parse outputs for comparison
        let outputs: Vec<(usize, u128)> = valid_quotes
            .iter()
            .enumerate()
            .filter_map(|(i, q)| q.amount_out_u128().map(|out| (i, out)))
            .collect();

        if outputs.is_empty() {
            return Self {
                best_quote: None,
                valid_quotes: valid_count,
                total_sources,
                avg_output: None,
                output_spread_pct: None,
            };
        }

        // Find best (highest output)
        let (best_idx, best_output) = outputs.iter().max_by_key(|(_, out)| out).unwrap();
        let best = &valid_quotes[*best_idx];

        // Find worst for improvement calculation
        let (_, worst_output) = outputs.iter().min_by_key(|(_, out)| out).unwrap();

        let improvement_pct = if *worst_output > 0 {
            Some(((*best_output - *worst_output) as f64 / *worst_output as f64) * 100.0)
        } else {
            None
        };

        // Calculate average
        let sum: u128 = outputs.iter().map(|(_, out)| out).sum();
        let avg = sum / outputs.len() as u128;

        // Calculate spread
        let spread_pct = if *best_output > 0 {
            Some(((*best_output - *worst_output) as f64 / *best_output as f64) * 100.0)
        } else {
            None
        };

        Self {
            best_quote: Some(BestQuote {
                source: best.source.clone(),
                amount_out: best.amount_out.clone(),
                gas_usd: best.gas_usd,
                price_impact: best.price_impact,
                improvement_pct,
            }),
            valid_quotes: valid_count,
            total_sources,
            avg_output: Some(avg.to_string()),
            output_spread_pct: spread_pct,
        }
    }
}
