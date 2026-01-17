# gecko

Rust client for the CoinGecko API.

## Overview

An unofficial Rust client for the [CoinGecko API](https://www.coingecko.com/api/documentation), providing access to cryptocurrency prices, market data, exchanges, NFTs, and more.

## Features

- **Simple** - Current prices for coins
- **Coins** - Detailed coin data, historical prices, markets
- **Categories** - Coin categories and market data
- **Exchanges** - Exchange info, tickers, volume
- **Derivatives** - Derivatives exchanges and data
- **NFTs** - NFT collections, floor prices, market data
- **On-chain** - On-chain DEX data (Pro)
- **Global** - Global crypto statistics
- **Treasury** - Company crypto holdings

## Quick Start

```rust
use gecko::Client;

#[tokio::main]
async fn main() -> Result<(), gecko::Error> {
    // Free tier
    let client = Client::new()?;

    // Get Bitcoin price
    let prices = client.simple().price(&["bitcoin"], &["usd"]).await?;
    println!("{:?}", prices);

    // Get trending coins
    let trending = client.global().trending().await?;
    for item in trending.coins.iter().take(5) {
        println!("{}: #{}", item.item.name, item.item.market_cap_rank.unwrap_or(0));
    }

    Ok(())
}
```

## Pro API

```rust
// Pro tier with API key
let client = gecko::Client::pro("your-api-key")?;
let markets = client.coins().markets("usd").await?;
```

## Installation

```toml
[dependencies]
gecko = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

- `COINGECKO_API_KEY` - Your CoinGecko Pro API key (optional, for higher rate limits)

## License

MIT
