<p align="center">
  <img src="../../logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">cexm</h1>

<p align="center">
  Unofficial Rust client for public market data on centralised exchanges and perp venues
</p>

<p align="center">
  <a href="https://crates.io/crates/cexm"><img src="https://img.shields.io/crates/v/cexm.svg" alt="crates.io"></a>
  <a href="https://opensource.org/license/mit"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Overview

One client and one set of types for tickers, order books, candles, recent
trades and market lists across eight venues. Only public, unauthenticated
endpoints are used: no API keys and no trading.

| Exchange | Markets | API |
|----------|---------|-----|
| [Binance](https://www.binance.com/) | spot | `data-api.binance.vision` (market-data host, not region-locked) |
| [Bitget](https://www.bitget.com/) | spot | v2 |
| [OKX](https://www.okx.com/) | spot | v5 |
| [Bybit](https://www.bybit.com/) | spot | v5 |
| [Coinbase](https://www.coinbase.com/) | spot | Exchange API |
| [Kraken](https://www.kraken.com/) | spot | public REST |
| [Hyperliquid](https://hyperliquid.xyz/) | perpetuals | `/info` |
| [Lighter](https://lighter.xyz/) | perpetuals and spot | `/api/v1` |

## Features

- **Tickers** - Last, bid, ask, 24h high/low/volume and 24h change
- **Order books** - Aggregated price levels, best first
- **Candles** - OHLCV from 1m to 1M, UTC-aligned
- **Trades** - Recent public trades with taker side and cost
- **Markets** - Every listed market, spot or perpetual, with its status
- **Exact numbers** - Prices and amounts kept as the exchange's decimal strings

## Installation

```toml
[dependencies]
cexm = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use cexm::{Client, Exchange, Timeframe};

#[tokio::main]
async fn main() -> cexm::Result<()> {
    let client = Client::new()?;

    // 24h ticker
    let ticker = client.ticker(Exchange::Okx, "ETH/USDT").await?;
    println!("ETH on OKX: {:?} ({:?}% 24h)", ticker.last, ticker.change_pct);

    // Top 10 levels of the order book
    let book = client.order_book(Exchange::Kraken, "BTC/USD", 10).await?;
    println!("best bid {:?}, best ask {:?}", book.bids.first(), book.asks.first());

    // Last 24 hourly candles of a perpetual
    let candles = client
        .candles(Exchange::Hyperliquid, "BTC/USDC:USDC", Timeframe::H1, 24)
        .await?;
    println!("{} candles", candles.len());

    // Recent trades and the market list
    let trades = client.trades(Exchange::Bitget, "ETH/USDT", 50).await?;
    let markets = client.markets(Exchange::Coinbase).await?;
    println!("{} trades, {} markets", trades.len(), markets.len());

    Ok(())
}
```

## Conventions

- **Symbols** are unified: `BASE/QUOTE` for spot (`BTC/USDT`, `ETH/USD`) and
  `BASE/QUOTE:SETTLE` for perpetuals (`BTC/USDC:USDC`). Each exchange module
  maps them to its own ids (`BTCUSDT`, `BTC-USDT`, `XBTUSD`, ...).
- **Numbers** are the exchange's own decimal strings, so nothing is lost to
  floating point. `cexm::as_f64` converts when an approximation is fine; trade
  `cost` is computed exactly (`cexm::mul_decimal`) when the venue doesn't give it.
- **Candles** come back oldest first, trades newest first, both trimmed to the
  requested limit. Bars of 6h and longer are UTC-aligned on every venue.
- **24h change** is in percent, against the price 24 hours ago. Coinbase,
  Kraken and Lighter publish no 24h open of their own (Kraken's is since
  00:00 UTC; Coinbase's and Lighter's match neither), so for them it comes
  from the venue's own 5-minute candle covering 24 hours ago.
- **Unsupported timeframes** (e.g. 8h on OKX) fail with
  `Error::UnsupportedTimeframe` rather than silently using another one.

## Regions

Some venues refuse some regions (Bybit and Lighter, for example). Route
requests through a proxy with `Config::with_proxy`:

```rust
use cexm::{Client, Config};

fn main() -> cexm::Result<()> {
    let client = Client::with_config(
        Config::default().with_proxy("http://user:pass@proxy.example:8080"),
    )?;
    Ok(())
}
```

## Tests

```bash
# Unit tests
cargo test -p cexm

# Every method on every exchange, against the live APIs
cargo test -p cexm --test live_api -- --ignored --nocapture

# Only some exchanges
CEXM_LIVE=okx,kraken cargo test -p cexm --test live_api -- --ignored --nocapture
```

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply
with the API terms of each exchange you query.

## Disclaimer

This crate is not affiliated with or endorsed by Binance, Bitget, OKX, Bybit,
Coinbase, Kraken, Hyperliquid or Lighter.

## License

MIT
