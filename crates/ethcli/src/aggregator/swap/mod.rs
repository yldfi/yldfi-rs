//! Swap quote aggregation from multiple DEX aggregator sources
//!
//! This module fetches swap quotes from multiple DEX aggregators in parallel
//! and provides the best quote based on output amount and gas costs.

mod fetchers;
mod types;

pub use fetchers::*;
pub use types::*;

use super::{AggregatedResult, LatencyMeasure, SourceResult};
use futures::future::join_all;

/// Fetch quotes from all available DEX aggregators in parallel
pub async fn fetch_quotes_all(
    chain: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
) -> AggregatedResult<NormalizedQuote, QuoteAggregation> {
    let sources = vec![
        SwapSource::OpenOcean,
        SwapSource::KyberSwap,
        SwapSource::Zerox,
        SwapSource::OneInch,
        SwapSource::CowSwap,
        SwapSource::LiFi,
        SwapSource::Velora,
        SwapSource::Enso,
    ];

    fetch_quotes_parallel(chain, token_in, token_out, amount_in, sender, &sources).await
}

/// Fetch quotes from specified sources in parallel
pub async fn fetch_quotes_parallel(
    chain: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    sources: &[SwapSource],
) -> AggregatedResult<NormalizedQuote, QuoteAggregation> {
    let start = LatencyMeasure::start();

    // Build futures for each source
    let futures: Vec<_> = sources
        .iter()
        .filter(|s| **s != SwapSource::All)
        .map(|source| {
            let token_in = token_in.to_string();
            let token_out = token_out.to_string();
            let amount_in = amount_in.to_string();
            let sender = sender.map(|s| s.to_string());
            let source = *source;
            async move {
                fetch_quote_from_source(
                    chain,
                    &token_in,
                    &token_out,
                    &amount_in,
                    sender.as_deref(),
                    source,
                )
                .await
            }
        })
        .collect();

    // Execute ALL in parallel
    let results: Vec<SourceResult<NormalizedQuote>> = join_all(futures).await;

    // Calculate aggregation (find best quote)
    let aggregation = QuoteAggregation::from_quotes(&results);

    AggregatedResult::new(aggregation, results, start.elapsed_ms())
}

/// Fetch quote from a single source
pub async fn fetch_quote_from_source(
    chain: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    source: SwapSource,
) -> SourceResult<NormalizedQuote> {
    let measure = LatencyMeasure::start();

    match source {
        SwapSource::OpenOcean => {
            fetchers::fetch_openocean_quote(chain, token_in, token_out, amount_in, sender, measure)
                .await
        }
        SwapSource::KyberSwap => {
            fetchers::fetch_kyber_quote(chain, token_in, token_out, amount_in, measure).await
        }
        SwapSource::Zerox => {
            fetchers::fetch_zerox_quote(chain, token_in, token_out, amount_in, sender, measure)
                .await
        }
        SwapSource::OneInch => {
            fetchers::fetch_oneinch_quote(chain, token_in, token_out, amount_in, sender, measure)
                .await
        }
        SwapSource::CowSwap => {
            fetchers::fetch_cowswap_quote(chain, token_in, token_out, amount_in, sender, measure)
                .await
        }
        SwapSource::LiFi => {
            fetchers::fetch_lifi_quote(chain, token_in, token_out, amount_in, sender, measure).await
        }
        SwapSource::Velora => {
            fetchers::fetch_velora_quote(chain, token_in, token_out, amount_in, sender, measure)
                .await
        }
        SwapSource::Enso => {
            fetchers::fetch_enso_quote(chain, token_in, token_out, amount_in, sender, measure).await
        }
        SwapSource::All => SourceResult::error("all", "Use fetch_quotes_all instead", 0),
    }
}
