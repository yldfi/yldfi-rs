//! Live checks against the real exchange APIs. Run with:
//! ```bash
//! cargo test -p cexm --test live_api -- --ignored --nocapture
//! ```
//! Set `CEXM_LIVE=okx,bitget` to check only some exchanges.

use cexm::{as_f64, Client, Exchange, MarketType, Timeframe};

/// A liquid symbol on each venue
fn symbol(exchange: Exchange) -> &'static str {
    match exchange {
        Exchange::Hyperliquid | Exchange::Lighter => "ETH/USDC:USDC",
        Exchange::Coinbase | Exchange::Kraken => "ETH/USD",
        _ => "ETH/USDT",
    }
}

fn selected() -> Vec<Exchange> {
    match std::env::var("CEXM_LIVE") {
        Ok(list) => list
            .split(',')
            .map(|s| s.trim().parse().expect("CEXM_LIVE entry"))
            .collect(),
        Err(_) => Exchange::ALL.to_vec(),
    }
}

#[tokio::test]
#[ignore = "requires network access to the exchanges"]
async fn every_method_on_every_exchange() {
    let client = Client::new().unwrap();
    let mut failures = Vec::new();
    for ex in selected() {
        let sym = symbol(ex);
        let result: cexm::Result<()> =
            async {
                let t = client.ticker(ex, sym).await?;
                let last = t
                    .last
                    .as_deref()
                    .and_then(as_f64)
                    .expect("ticker has a price");
                assert!(last > 100.0 && last < 1_000_000.0, "{ex} ETH price {last}");

                let book = client.order_book(ex, sym, 5).await?;
                assert!(
                    !book.bids.is_empty() && !book.asks.is_empty(),
                    "{ex} empty book"
                );
                assert!(
                    book.bids.len() <= 5 && book.asks.len() <= 5,
                    "{ex} book over limit"
                );
                let (bid, ask) = (as_f64(&book.bids[0].price), as_f64(&book.asks[0].price));
                assert!(bid < ask, "{ex} crossed book {bid:?} >= {ask:?}");

                let candles = client.candles(ex, sym, Timeframe::H1, 10).await?;
                assert!(
                    !candles.is_empty() && candles.len() <= 10,
                    "{ex} {} candles",
                    candles.len()
                );
                assert!(
                    candles.windows(2).all(|w| w[0].timestamp < w[1].timestamp),
                    "{ex} order"
                );
                let gap = candles.last().map(|c| cexm_now() - c.timestamp).unwrap();
                assert!(gap < 2 * 3_600_000, "{ex} newest candle is {gap} ms old");

                let trades = client.trades(ex, sym, 5).await?;
                assert!(
                    !trades.is_empty() && trades.len() <= 5,
                    "{ex} {} trades",
                    trades.len()
                );
                assert!(
                    trades.iter().all(|t| t.cost.is_some()),
                    "{ex} trade without cost"
                );

                let markets = client.markets(ex).await?;
                assert!(
                    markets.iter().any(|m| m.symbol == sym),
                    "{ex} doesn't list {sym}"
                );
                let kind = if sym.contains(':') {
                    MarketType::Swap
                } else {
                    MarketType::Spot
                };
                assert!(markets.iter().any(|m| m.market_type == kind));

                println!(
                "{ex:<12} {sym:<14} last={last:<10} bid={} ask={} candles={} trades={} markets={}",
                book.bids[0].price, book.asks[0].price, candles.len(), trades.len(), markets.len()
            );
                Ok(())
            }
            .await;
        if let Err(e) = result {
            println!("{ex:<12} FAILED: {e}");
            failures.push(format!("{ex}: {e}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

fn cexm_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
