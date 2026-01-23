# pyth

Rust client for the [Pyth Network](https://pyth.network/) Hermes API.

Pyth Network provides real-time price feeds for crypto, equities, FX, and commodities. This crate interfaces with the Hermes REST API to fetch price data.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pyth = { version = "0.1", path = "../pyth" }
```

## Quick Start

```rust
use pyth::{Client, feed_ids};

#[tokio::main]
async fn main() -> pyth::Result<()> {
    let client = Client::new()?;

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

- **No API key required** - Hermes API is free to use
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
use pyth::{Client, feed_ids};

let client = Client::new()?;

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
use pyth::Client;

let client = Client::new()?;

// Search for BTC-related feeds
let feeds = client.search_feeds("BTC").await?;
for feed in feeds {
    println!("ID: {}", feed.id);
    println!("Symbol: {:?}", feed.attributes.symbol);
}
```

### Symbol Lookup

```rust
use pyth::{Client, symbol_to_feed_id};

let client = Client::new()?;

// Convert symbol to feed ID
if let Some(feed_id) = symbol_to_feed_id("ETH") {
    let price = client.get_latest_price(feed_id).await?;
    println!("{:?}", price);
}
```

### Check Price Staleness

```rust
use pyth::{Client, feed_ids};

let client = Client::new()?;

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

### Use Testnet

```rust
use pyth::Client;

let client = Client::testnet()?;
let price = client.get_latest_price(pyth::feed_ids::ETH_USD).await?;
```

### Custom Configuration

```rust
use pyth::{Client, Config};
use std::time::Duration;

let config = Config::mainnet()
    .with_timeout(Duration::from_secs(60))
    .with_proxy("http://proxy:8080");

let client = Client::with_config(config)?;
```

## Configuration

### Base URLs

| Network | URL |
|---------|-----|
| Mainnet | `https://hermes.pyth.network` |
| Testnet | `https://hermes-beta.pyth.network` |

### Timeouts

Default timeout is 30 seconds. Configure with:

```rust
use pyth::Config;
use std::time::Duration;

let config = Config::mainnet()
    .with_timeout(Duration::from_secs(60));
```

## Error Handling

```rust
use pyth::{Client, Error};

let client = Client::new()?;

match client.get_latest_price("invalid-feed-id").await {
    Ok(Some(feed)) => println!("Price: {:?}", feed),
    Ok(None) => println!("No price data"),
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

## License

MIT
