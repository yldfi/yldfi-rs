//! Swap quote aggregation from multiple DEX aggregator sources
//!
//! This module fetches swap quotes from multiple DEX aggregators in parallel
//! and provides the best quote based on output amount and gas costs.

mod fetchers;
pub mod tokens;
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
    slippage_bps: u32,
) -> AggregatedResult<NormalizedQuote, QuoteAggregation> {
    let sources = vec![
        SwapSource::OpenOcean,
        SwapSource::KyberSwap,
        SwapSource::Zerox,
        SwapSource::OneInch,
        SwapSource::CowSwap,
        SwapSource::LiFi,
        SwapSource::Velora,
        SwapSource::VeloraDelta,
        SwapSource::Enso,
    ];

    fetch_quotes_parallel(
        chain,
        token_in,
        token_out,
        amount_in,
        sender,
        slippage_bps,
        &sources,
    )
    .await
}

/// Fetch quotes from specified sources in parallel
pub async fn fetch_quotes_parallel(
    chain: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    slippage_bps: u32,
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
                    slippage_bps,
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

/// Address sent to sources that cannot price without a sender, when the
/// caller gave none. Nobody holds its key, the sources accept it (they reject
/// the zero address), and quotes priced for it never carry transaction data.
pub const PLACEHOLDER_SENDER: &str = "0x000000000000000000000000000000000000dEaD";

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Whether `source` refuses to quote without a sender address
#[must_use]
pub fn requires_sender(source: SwapSource) -> bool {
    matches!(
        source,
        SwapSource::CowSwap | SwapSource::LiFi | SwapSource::Enso
    )
}

/// Whether `address` may not be used as a real sender: the zero address or
/// ethcli's own placeholder. Transaction data built for either would pay the
/// swap output to an address nobody controls.
#[must_use]
pub fn is_reserved_sender(address: &str) -> bool {
    address.eq_ignore_ascii_case(ZERO_ADDRESS) || address.eq_ignore_ascii_case(PLACEHOLDER_SENDER)
}

/// Turn a result fetched for the placeholder sender into a price-only quote.
///
/// This is the one place transaction data is removed, so a fetcher cannot
/// leak calldata that pays the placeholder address.
fn mark_placeholder(result: &mut SourceResult<NormalizedQuote>) {
    if let Some(quote) = result.data.as_mut() {
        quote.tx_data = None;
        quote.tx_value = None;
        quote.placeholder_sender = true;
    }
}

/// Fetch quote from a single source
///
/// `slippage_bps` is the slippage tolerance in basis points (50 = 0.5%). It is
/// converted to each provider's unit (bps for 0x/Enso, fraction for LI.FI)
/// for sources that return transaction data.
///
/// Without a `sender`, sources that need one (see [`requires_sender`]) are
/// priced for [`PLACEHOLDER_SENDER`] and return no transaction data. Other
/// sources are never sent the placeholder.
pub async fn fetch_quote_from_source(
    chain: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    slippage_bps: u32,
    source: SwapSource,
) -> SourceResult<NormalizedQuote> {
    let sender = sender.filter(|s| !s.is_empty());
    let placeholder = sender.is_none() && requires_sender(source);
    let effective = if placeholder {
        Some(PLACEHOLDER_SENDER)
    } else {
        sender
    };

    let mut result = fetch_for_sender(
        chain,
        token_in,
        token_out,
        amount_in,
        effective,
        slippage_bps,
        source,
    )
    .await;
    if placeholder {
        mark_placeholder(&mut result);
    }
    result
}

async fn fetch_for_sender(
    chain: u64,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: Option<&str>,
    slippage_bps: u32,
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
            fetchers::fetch_zerox_quote(
                chain,
                token_in,
                token_out,
                amount_in,
                sender,
                slippage_bps,
                measure,
            )
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
            fetchers::fetch_lifi_quote(
                chain,
                token_in,
                token_out,
                amount_in,
                sender,
                slippage_bps,
                measure,
            )
            .await
        }
        SwapSource::Velora => {
            fetchers::fetch_velora_quote(chain, token_in, token_out, amount_in, sender, measure)
                .await
        }
        SwapSource::VeloraDelta => {
            fetchers::fetch_velora_delta_quote(
                chain, token_in, token_out, amount_in, sender, measure,
            )
            .await
        }
        SwapSource::Enso => {
            fetchers::fetch_enso_quote(
                chain,
                token_in,
                token_out,
                amount_in,
                sender,
                slippage_bps,
                measure,
            )
            .await
        }
        // A single-source call cannot represent "all"; callers should use
        // fetch_quotes_all (ethcli quote from all does this).
        SwapSource::All => SourceResult::error("all", "Use fetch_quotes_all for all sources", 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_three_sources_require_a_sender() {
        for source in [SwapSource::CowSwap, SwapSource::LiFi, SwapSource::Enso] {
            assert!(requires_sender(source), "{source}");
        }
        for source in [
            SwapSource::OpenOcean,
            SwapSource::KyberSwap,
            SwapSource::Zerox,
            SwapSource::OneInch,
            SwapSource::Velora,
            SwapSource::VeloraDelta,
            SwapSource::All,
        ] {
            assert!(!requires_sender(source), "{source}");
        }
    }

    #[test]
    fn zero_and_placeholder_addresses_are_reserved_in_any_case() {
        assert!(is_reserved_sender(PLACEHOLDER_SENDER));
        assert!(is_reserved_sender(&PLACEHOLDER_SENDER.to_lowercase()));
        assert!(is_reserved_sender(
            "0x0000000000000000000000000000000000000000"
        ));
        assert!(!is_reserved_sender(
            "0x1111111111111111111111111111111111111111"
        ));
    }

    #[test]
    fn placeholder_quote_keeps_its_price_and_loses_transaction_data() {
        let quote = NormalizedQuote::new("enso", "0xin", "0xout", "1", "1987")
            .with_router("0xrouter")
            .with_tx_data("0xb94c3609dead")
            .with_tx_value("0");
        let mut result = SourceResult::success("enso", quote, 1);
        mark_placeholder(&mut result);

        let quote = result.data.unwrap();
        assert!(quote.placeholder_sender);
        assert_eq!(quote.amount_out, "1987");
        let json = serde_json::to_value(&quote).unwrap();
        assert!(json.get("tx_data").is_none());
        assert!(json.get("tx_value").is_none());
        assert_eq!(json["placeholder_sender"], true);
    }

    #[test]
    fn placeholder_marking_leaves_an_error_untouched() {
        let mut result: SourceResult<NormalizedQuote> = SourceResult::error("enso", "boom", 1);
        mark_placeholder(&mut result);
        assert!(result.data.is_none());
        assert_eq!(result.error.as_deref(), Some("boom"));
    }
}
