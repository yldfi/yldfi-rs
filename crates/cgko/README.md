<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">gecko</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://www.coingecko.com/api/documentation">CoinGecko</a> API
</p>

<p align="center">
  <a href="https://crates.io/crates/cgko"><img src="https://img.shields.io/crates/v/gecko.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/gecko/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

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

## Installation

```toml
[dependencies]
cgko = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use cgko::Client;

#[tokio::main]
async fn main() -> Result<(), cgko::Error> {
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
let client = cgko::Client::pro("your-api-key")?;
let markets = client.coins().markets("usd").await?;
```

## Environment Variables

- `COINGECKO_API_KEY` - Your CoinGecko Pro API key (optional, for higher rate limits)

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [CoinGecko's Terms of Service](https://www.coingecko.com/en/terms).

## Disclaimer

This crate is not affiliated with or endorsed by CoinGecko.

## License

MIT
