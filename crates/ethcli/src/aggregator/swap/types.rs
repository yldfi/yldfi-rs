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
    VeloraDelta,
    Enso,
    PropAmm,
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
            "velora-delta" | "veloradelta" | "delta" => Ok(SwapSource::VeloraDelta),
            "enso" | "ensofi" => Ok(SwapSource::Enso),
            "propamm" | "prop-amm" | "pamm" => Ok(SwapSource::PropAmm),
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
            SwapSource::VeloraDelta => write!(f, "velora-delta"),
            SwapSource::Enso => write!(f, "enso"),
            SwapSource::PropAmm => write!(f, "propamm"),
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
    /// No sender was given and this source needs one, so it was priced for
    /// ethcli's placeholder address. The figure is a price only: transaction
    /// data is never returned for it.
    #[serde(default)]
    pub placeholder_sender: bool,
    /// The figure only holds under a condition the caller does not control
    /// (for a prop AMM: the trade landing in a block from a builder that
    /// carries the maker's price update). Conditional quotes are shown but
    /// are never picked as the best quote or counted in the summary figures.
    #[serde(default)]
    pub conditional: bool,
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
            placeholder_sender: false,
            conditional: false,
        }
    }

    /// Mark the quote as conditional (see [`NormalizedQuote::conditional`])
    #[must_use]
    pub fn conditional(mut self) -> Self {
        self.conditional = true;
        self
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

/// Minimum number of parsed quotes before any of them can be called an outlier.
/// With fewer there is no majority to compare against.
const OUTLIER_MIN_QUOTES: usize = 3;

/// A quote is a low outlier when its output is below `median / OUTLIER_MEDIAN_DIVISOR`
/// and a high outlier when it is above `median * OUTLIER_MEDIAN_DIVISOR`.
/// Aggregators that only know a thin pool for a token return a real quote that
/// is a small fraction of what the others offer; it must not shape the summary.
/// A quote far above the rest is kept, because it can be the one good route,
/// but it is reported so the caller can check it.
const OUTLIER_MEDIAN_DIVISOR: u128 = 2;

/// Aggregated quote result with best quote selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteAggregation {
    /// Best quote by output amount
    pub best_quote: Option<BestQuote>,
    /// Number of sources that returned valid quotes (outliers included)
    pub valid_quotes: usize,
    /// Total sources queried
    pub total_sources: usize,
    /// Average output across valid quotes, excluding outliers
    pub avg_output: Option<String>,
    /// Output spread percentage ((max-min)/max * 100), excluding outliers
    pub output_spread_pct: Option<f64>,
    /// Sources whose output is less than half the median. They are left out of
    /// `avg_output`, `output_spread_pct` and `improvement_pct`.
    #[serde(default)]
    pub outliers: Vec<String>,
    /// Sources whose output is more than twice the median. They still count,
    /// and one of them is the best quote, but a figure this far above the
    /// others may be wrong (or the only good route): check it before use.
    #[serde(default)]
    pub high_outliers: Vec<String>,
    /// Sources whose quote is conditional. They are left out of `best_quote`
    /// and of every summary figure, however high their output.
    #[serde(default)]
    pub conditional: Vec<String>,
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
    /// Improvement over the worst quote that is not an outlier (percentage)
    pub improvement_pct: Option<f64>,
    /// The best quote was priced for the placeholder sender (see
    /// `NormalizedQuote::placeholder_sender`)
    #[serde(default)]
    pub placeholder_sender: bool,
}

/// Median of `values`; the mean of the two middle values for an even count.
fn median(values: &[u128]) -> Option<u128> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    Some(if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        // Halve before adding so two near-max values cannot overflow.
        sorted[mid - 1] / 2 + sorted[mid] / 2 + (sorted[mid - 1] % 2 + sorted[mid] % 2) / 2
    })
}

impl QuoteAggregation {
    /// Whether `source` was flagged as an outlier
    #[must_use]
    pub fn is_outlier(&self, source: &str) -> bool {
        self.outliers.iter().any(|s| s == source)
    }

    /// Whether `source` returned a conditional quote
    #[must_use]
    pub fn is_conditional(&self, source: &str) -> bool {
        self.conditional.iter().any(|s| s == source)
    }

    /// Create aggregation from quote results
    pub fn from_quotes(results: &[SourceResult<NormalizedQuote>]) -> Self {
        let valid_quotes: Vec<&NormalizedQuote> =
            results.iter().filter_map(|r| r.data.as_ref()).collect();

        let total_sources = results.len();
        let valid_count = valid_quotes.len();

        // Conditional quotes are reported, not compared
        let conditional: Vec<String> = valid_quotes
            .iter()
            .filter(|q| q.conditional)
            .map(|q| q.source.clone())
            .collect();

        // Parse outputs for comparison
        let outputs: Vec<(usize, u128)> = valid_quotes
            .iter()
            .enumerate()
            .filter(|(_, q)| !q.conditional)
            .filter_map(|(i, q)| q.amount_out_u128().map(|out| (i, out)))
            .collect();

        // Find best (highest output). On an exact tie a quote priced for the
        // caller beats one priced for the placeholder sender, and otherwise
        // the first source listed wins.
        let Some((best_idx, best_output)) = outputs
            .iter()
            .rev()
            .max_by_key(|(i, out)| (*out, !valid_quotes[*i].placeholder_sender))
        else {
            return Self {
                best_quote: None,
                valid_quotes: valid_count,
                total_sources,
                avg_output: None,
                output_spread_pct: None,
                outliers: Vec::new(),
                high_outliers: Vec::new(),
                conditional,
            };
        };
        let best = &valid_quotes[*best_idx];

        // Split off quotes far below the median. The best quote is the highest
        // output, so it is never one of them.
        let amounts: Vec<u128> = outputs.iter().map(|(_, out)| *out).collect();
        let (floor, ceiling) = match median(&amounts) {
            Some(m) if outputs.len() >= OUTLIER_MIN_QUOTES => (
                m / OUTLIER_MEDIAN_DIVISOR,
                m.saturating_mul(OUTLIER_MEDIAN_DIVISOR),
            ),
            _ => (0, u128::MAX),
        };
        let sources_where = |keep: &dyn Fn(u128) -> bool| -> Vec<String> {
            outputs
                .iter()
                .filter(|(_, out)| keep(*out))
                .map(|(i, _)| valid_quotes[*i].source.clone())
                .collect()
        };
        let outliers = sources_where(&|out| out < floor);
        let high_outliers = sources_where(&|out| out > ceiling);
        let inliers: Vec<u128> = amounts
            .iter()
            .copied()
            .filter(|out| *out >= floor)
            .collect();

        // Worst remaining quote, for the improvement and spread figures
        let worst_output = inliers.iter().copied().min().unwrap_or(*best_output);

        let improvement_pct = if worst_output > 0 {
            Some(((*best_output - worst_output) as f64 / worst_output as f64) * 100.0)
        } else {
            None
        };

        // Calculate average
        let sum: u128 = inliers.iter().sum();
        let avg = sum / inliers.len().max(1) as u128;

        // Calculate spread
        let spread_pct = if *best_output > 0 {
            Some(((*best_output - worst_output) as f64 / *best_output as f64) * 100.0)
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
                placeholder_sender: best.placeholder_sender,
            }),
            valid_quotes: valid_count,
            total_sources,
            avg_output: Some(avg.to_string()),
            output_spread_pct: spread_pct,
            outliers,
            high_outliers,
            conditional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(source: &'static str, amount_out: &str) -> SourceResult<NormalizedQuote> {
        SourceResult::success(
            source,
            NormalizedQuote::new(source, "0xin", "0xout", "1", amount_out),
            1,
        )
    }

    #[test]
    fn median_handles_odd_even_and_large_values() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[7]), Some(7));
        assert_eq!(median(&[3, 1, 2]), Some(2));
        assert_eq!(median(&[4, 1, 3, 2]), Some(2));
        assert_eq!(median(&[3, 5]), Some(4));
        assert_eq!(median(&[u128::MAX, u128::MAX]), Some(u128::MAX));
    }

    #[test]
    fn thin_pool_quote_is_flagged_and_left_out_of_the_summary() {
        // Figures from a live stablecoin comparison: one source routed
        // through a nearly empty pool.
        let results = vec![
            ok("openocean", "4869286"),
            ok("kyberswap", "1987004936"),
            ok("1inch", "1981044220"),
            SourceResult::error("0x", "no key", 0),
        ];
        let agg = QuoteAggregation::from_quotes(&results);

        assert_eq!(agg.outliers, vec!["openocean".to_string()]);
        assert!(agg.high_outliers.is_empty());
        assert!(agg.is_outlier("openocean"));
        assert!(!agg.is_outlier("kyberswap"));
        assert_eq!(agg.valid_quotes, 3);
        assert_eq!(agg.total_sources, 4);
        assert_eq!(agg.avg_output.as_deref(), Some("1984024578"));
        let best = agg.best_quote.unwrap();
        assert_eq!(best.source, "kyberswap");
        assert!(best.improvement_pct.unwrap() < 1.0);
        assert!(agg.output_spread_pct.unwrap() < 1.0);
    }

    #[test]
    fn half_of_the_quotes_being_low_still_flags_them() {
        let results = vec![ok("a", "5"), ok("b", "6"), ok("c", "1987"), ok("d", "1990")];
        let agg = QuoteAggregation::from_quotes(&results);
        assert_eq!(agg.outliers, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(agg.best_quote.unwrap().source, "d");
    }

    #[test]
    fn quote_far_above_the_others_is_best_but_reported() {
        let results = vec![ok("a", "1980"), ok("b", "1987"), ok("c", "1987000")];
        let agg = QuoteAggregation::from_quotes(&results);
        assert_eq!(agg.high_outliers, vec!["c".to_string()]);
        assert!(agg.outliers.is_empty());
        assert_eq!(agg.best_quote.unwrap().source, "c");
    }

    #[test]
    fn exact_tie_prefers_a_quote_not_priced_for_the_placeholder() {
        let mut placeholder = ok("enso", "1988");
        placeholder.data.as_mut().unwrap().placeholder_sender = true;
        // Whichever side the placeholder quote is on, the firm quote wins
        for results in [
            vec![
                ok("kyberswap", "1988"),
                placeholder.clone(),
                ok("1inch", "1980"),
            ],
            vec![
                placeholder.clone(),
                ok("kyberswap", "1988"),
                ok("1inch", "1980"),
            ],
        ] {
            let best = QuoteAggregation::from_quotes(&results).best_quote.unwrap();
            assert_eq!(best.source, "kyberswap");
            assert!(!best.placeholder_sender);
        }
    }

    #[test]
    fn tie_between_firm_quotes_goes_to_the_first_source() {
        let results = vec![ok("kyberswap", "1988"), ok("velora", "1988")];
        let best = QuoteAggregation::from_quotes(&results).best_quote.unwrap();
        assert_eq!(best.source, "kyberswap");
    }

    #[test]
    fn higher_placeholder_quote_still_wins() {
        let mut placeholder = ok("enso", "1989");
        placeholder.data.as_mut().unwrap().placeholder_sender = true;
        let results = vec![ok("kyberswap", "1988"), placeholder];
        let best = QuoteAggregation::from_quotes(&results).best_quote.unwrap();
        assert_eq!(best.source, "enso");
        assert!(best.placeholder_sender);
    }

    #[test]
    fn conditional_quote_is_listed_but_never_best_or_counted() {
        let results = vec![
            ok("kyberswap", "1000"),
            SourceResult::success(
                "propamm",
                NormalizedQuote::new("propamm", "0xin", "0xout", "1", "5000").conditional(),
                1,
            ),
            ok("velora", "990"),
        ];
        let agg = QuoteAggregation::from_quotes(&results);
        assert_eq!(agg.conditional, vec!["propamm".to_string()]);
        assert!(agg.is_conditional("propamm"));
        assert_eq!(agg.valid_quotes, 3);
        // Not the best, not an outlier, not in the average or spread
        assert_eq!(agg.best_quote.as_ref().unwrap().source, "kyberswap");
        assert!(agg.high_outliers.is_empty());
        assert_eq!(agg.avg_output.as_deref(), Some("995"));
        assert!(agg.output_spread_pct.unwrap() < 2.0);
    }

    #[test]
    fn only_conditional_quotes_gives_no_best_quote() {
        let results = vec![SourceResult::success(
            "propamm",
            NormalizedQuote::new("propamm", "0xin", "0xout", "1", "5000").conditional(),
            1,
        )];
        let agg = QuoteAggregation::from_quotes(&results);
        assert!(agg.best_quote.is_none());
        assert_eq!(agg.conditional, vec!["propamm".to_string()]);
        assert_eq!(agg.valid_quotes, 1);
    }

    #[test]
    fn two_quotes_are_never_outliers() {
        let results = vec![ok("a", "5"), ok("b", "1987")];
        let agg = QuoteAggregation::from_quotes(&results);
        assert!(agg.outliers.is_empty());
        assert!(agg.high_outliers.is_empty());
        assert_eq!(agg.avg_output.as_deref(), Some("996"));
        assert!(agg.output_spread_pct.unwrap() > 99.0);
    }

    #[test]
    fn close_quotes_are_all_kept() {
        let results = vec![ok("a", "1000"), ok("b", "700"), ok("c", "990")];
        let agg = QuoteAggregation::from_quotes(&results);
        assert!(agg.outliers.is_empty());
        assert_eq!(agg.avg_output.as_deref(), Some("896"));
    }

    #[test]
    fn no_parsed_quotes_gives_no_best_quote() {
        let results = vec![SourceResult::error("a", "boom", 0), ok("b", "not-a-number")];
        let agg = QuoteAggregation::from_quotes(&results);
        assert!(agg.best_quote.is_none());
        assert_eq!(agg.valid_quotes, 1);
        assert_eq!(agg.total_sources, 2);
        assert!(agg.outliers.is_empty());
    }
}
