# cexm

Public market data from centralised exchanges and perp venues, through one
client and one set of types.

| Exchange | Markets | API |
|----------|---------|-----|
| Binance | spot | `data-api.binance.vision` (market-data host, not region-locked) |
| Bitget | spot | v2 |
| OKX | spot | v5 |
| Bybit | spot | v5 |
| Coinbase | spot | Exchange API |
| Kraken | spot | public REST |
| Hyperliquid | perpetuals | `/info` |
| Lighter | perpetuals and spot | `/api/v1` |

Only public, unauthenticated endpoints are used: tickers, order books,
candles, recent trades and market lists. No API keys and no trading.

## Usage

```rust,no_run
use cexm::{Client, Exchange, Timeframe};

# async fn example() -> cexm::Result<()> {
let client = Client::new()?;

let ticker = client.ticker(Exchange::Okx, "ETH/USDT").await?;
let book = client.order_book(Exchange::Kraken, "BTC/USD", 10).await?;
let candles = client
    .candles(Exchange::Hyperliquid, "BTC/USDC:USDC", Timeframe::H1, 24)
    .await?;
let trades = client.trades(Exchange::Bitget, "ETH/USDT", 50).await?;
let markets = client.markets(Exchange::Coinbase).await?;
# Ok(())
# }
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
- **24h change** is in percent. Kraken only publishes today's open (00:00
  UTC), so its change is since midnight UTC.
- **Unsupported timeframes** (e.g. 8h on OKX) fail with
  `Error::UnsupportedTimeframe` rather than silently using another one.

## Regions

Some venues refuse some regions (Bybit and Lighter, for example). Route
through a proxy with `Config::with_proxy`:

```rust,no_run
# fn example() -> cexm::Result<()> {
let client = cexm::Client::with_config(
    cexm::Config::default().with_proxy("http://user:pass@proxy.example:8080"),
)?;
# Ok(())
# }
```

## Tests

```bash
cargo test -p cexm                                   # unit + mock tests
cargo test -p cexm --test live_api -- --ignored      # every method on every exchange
CEXM_LIVE=okx,kraken cargo test -p cexm --test live_api -- --ignored
```
