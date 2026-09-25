# pythc

Rust client for the [Pyth Network](https://pyth.network/) Hermes API.

Pyth Network provides real-time price feeds for crypto, equities, FX, and commodities. This crate interfaces with the Hermes REST API to fetch price data.

## API Key Required

Since the **Pyth Core upgrade (2026-08-26)**, Hermes requires an API key on every
request, sent as `Authorization: Bearer <key>`. Unauthenticated requests fail with
`401 Unauthorized`, which this crate surfaces as `DomainError::Unauthorized` with an
actionable message.

- Get a key from the Pyth Terminal: <https://pythdata.app>
- Pass it with `Client::with_api_key(key)` / `Config::with_api_key(key)`, or set
  `PYTH_API_KEY` and use `Client::from_env()`
- The key is stored as a `SecretApiKey` and redacted from `Debug` output

The default base URL is now `https://pyth.dourolabs.app/hermes` (a drop-in
replacement: routes and response shapes are unchanged). The legacy
`https://hermes.pyth.network` host is available as `base_urls::LEGACY` and also
requires a key.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pythc = "0.1"
```

## Quick Start

```rust
use pythc::{Client, feed_ids};

#[tokio::main]
async fn main() -> pythc::Result<()> {
    let client = Client::with_api_key(std::env::var("PYTH_API_KEY").unwrap_or_default())?;

    // Get ETH/USD price
    if let Some(feed) = client.get_latest_price(feed_ids::ETH_USD).await? {
        println!("ETH/USD: ${:.2}", feed.price_f64().unwrap_or(0.0));
        println!("Confidence: ${:.4}", feed.confidence_f64().unwrap_or(0.0));
        println!("Stale: {}", feed.is_stale(60));
    }

    Ok(())
}
```

## Features

- **API key auth** - Bearer token support (required since the Pyth Core upgrade), redacted from debug output
- **Multiple price feeds** - Fetch prices for multiple assets in a single request
- **Symbol mapping** - Use common symbols (ETH, BTC) instead of feed IDs
- **Stale detection** - Built-in check for outdated price data
- **Confidence intervals** - Access price confidence data

## API Coverage

### Implemented Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v2/updates/price/latest` | `get_latest_prices()` | Get latest prices for multiple feeds |
| `/v2/updates/price/latest` | `get_latest_price()` | Get latest price for a single feed |
| `/v2/price_feeds` | `get_price_feed_ids()` | List all available price feeds |
| `/v2/price_feeds?query=` | `search_feeds()` | Search feeds by symbol/name |
| `/v2/price_feeds?asset_type=` | `get_feeds_by_asset_type()` | Filter feeds by asset type |

### Supported Symbols

The following symbols are mapped to Pyth feed IDs:

| Symbol | Aliases | Feed |
|--------|---------|------|
| BTC | BITCOIN, WBTC | BTC/USD |
| ETH | ETHEREUM, WETH | ETH/USD |
| SOL | SOLANA | SOL/USD |
| USDC | - | USDC/USD |
| USDT | TETHER | USDT/USD |
| LINK | CHAINLINK | LINK/USD |
| ARB | ARBITRUM | ARB/USD |
| OP | OPTIMISM | OP/USD |
| AAVE | - | AAVE/USD |
| UNI | UNISWAP | UNI/USD |
| CRV | CURVE | CRV/USD |
| CVX | CONVEX | CVX/USD |
| MKR | MAKER | MKR/USD |
| SNX | SYNTHETIX | SNX/USD |
| LDO | LIDO | LDO/USD |
| DAI | - | DAI/USD |

You can also use feed IDs directly:
```rust
let price = client.get_latest_price(
    "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace"
).await?;
```

## Examples

### Fetch Multiple Prices

```rust
use pythc::{Client, feed_ids};

let client = Client::from_env()?; // reads PYTH_API_KEY

let feeds = client.get_latest_prices(&[
    feed_ids::BTC_USD,
    feed_ids::ETH_USD,
    feed_ids::SOL_USD,
]).await?;

for feed in feeds {
    println!("{}: ${:.2}", feed.id, feed.price_f64().unwrap_or(0.0));
}
```

### Search for Feeds

```rust
use pythc::Client;

let client = Client::from_env()?; // reads PYTH_API_KEY

// Search for BTC-related feeds
let feeds = client.search_feeds("BTC").await?;
for feed in feeds {
    println!("ID: {}", feed.id);
    println!("Symbol: {:?}", feed.attributes.symbol);
}
```

### Symbol Lookup

```rust
use pythc::{Client, symbol_to_feed_id};

let client = Client::from_env()?; // reads PYTH_API_KEY

// Convert symbol to feed ID
if let Some(feed_id) = symbol_to_feed_id("ETH") {
    let price = client.get_latest_price(feed_id).await?;
    println!("{:?}", price);
}
```

### Check Price Staleness

```rust
use pythc::{Client, feed_ids};

let client = Client::from_env()?; // reads PYTH_API_KEY

if let Some(feed) = client.get_latest_price(feed_ids::ETH_USD).await? {
    // Check if price is older than 60 seconds
    if feed.is_stale(60) {
        println!("Warning: Price data is stale!");
    }

    // Get confidence interval
    if let Some(conf) = feed.confidence_f64() {
        println!("Price: ${:.2} +/- ${:.4}",
            feed.price_f64().unwrap_or(0.0),
            conf
        );
    }
}
```

### Use the Legacy Host

```rust
use pythc::{base_urls, Client, Config};

let config = Config::mainnet()
    .with_base_url(base_urls::LEGACY)
    .with_api_key("your-pyth-api-key");
let client = Client::with_config(config)?;
```

`Client::testnet()` / `base_urls::TESTNET` (`hermes-beta.pyth.network`) are kept for
backwards compatibility only; Pyth no longer documents a beta endpoint, so treat it
as legacy/unverified.

### Custom Configuration

```rust
use pythc::{Client, Config};
use std::time::Duration;

let config = Config::mainnet()
    .with_api_key("your-pyth-api-key")
    .with_timeout(Duration::from_secs(60))
    .with_proxy("http://proxy:8080");

let client = Client::with_config(config)?;
```

## Configuration

### Base URLs

| Constant | URL | Notes |
|----------|-----|-------|
| `MAINNET` (default) | `https://pyth.dourolabs.app/hermes` | Pyth Core upgraded endpoint, API key required |
| `LEGACY` | `https://hermes.pyth.network` | Legacy host, API key required |
| `TESTNET` | `https://hermes-beta.pyth.network` | Legacy/unverified, not documented post-upgrade |

### Timeouts

Default timeout is 30 seconds. Configure with:

```rust
use pythc::Config;
use std::time::Duration;

let config = Config::mainnet()
    .with_timeout(Duration::from_secs(60));
```

## Error Handling

```rust
use pythc::{Client, DomainError, Error};

let client = Client::from_env()?; // reads PYTH_API_KEY

match client.get_latest_price(pythc::feed_ids::ETH_USD).await {
    Ok(Some(feed)) => println!("Price: {:?}", feed),
    Ok(None) => println!("No price data"),
    Err(Error::Domain(DomainError::Unauthorized { .. })) => {
        println!("Set PYTH_API_KEY (get a key at https://pythdata.app)");
    }
    Err(Error::Api { status, message }) => {
        println!("API error {}: {}", status, message);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Types

### ParsedPriceFeed

```rust
pub struct ParsedPriceFeed {
    pub id: String,           // Feed ID (hex)
    pub price: PriceData,     // Current price
    pub ema_price: Option<EmaPrice>,  // EMA price
    pub metadata: Option<PriceUpdateMetadata>,
}

impl ParsedPriceFeed {
    fn price_f64(&self) -> Option<f64>;      // Price as f64
    fn confidence_f64(&self) -> Option<f64>; // Confidence as f64
    fn is_stale(&self, max_age_secs: i64) -> bool;
}
```

### PriceData

```rust
pub struct PriceData {
    pub price: String,       // Price value (string for precision)
    pub conf: String,        // Confidence interval
    pub expo: i32,           // Exponent (price = price * 10^expo)
    pub publish_time: i64,   // Unix timestamp
}
```

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [Pyth Network Terms of Use](https://pyth.network/terms-of-use).

## Disclaimer

This crate is not affiliated with or endorsed by Pyth Network.

## License

MIT
