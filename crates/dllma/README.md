<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">defillama</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://defillama.com/">DefiLlama</a> API
</p>

<p align="center">
  <a href="https://crates.io/crates/dllma"><img src="https://img.shields.io/crates/v/defillama.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/defillama/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **TVL** - Protocol TVL data, chains, historical data
- **Coins** - Token prices, historical prices, charts
- **Yields** - DeFi yield data, pool APYs
- **Fees** - Protocol fees and revenue
- **Volumes** - DEX and protocol volumes
- **Stablecoins** - Stablecoin market data
- **Bridges** - Cross-chain bridge data
- **DAT** - Developer analytics tools (Pro)
- **Emissions** - Token emission schedules (Pro)

## Installation

```toml
[dependencies]
dllma = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use dllma::Client;

#[tokio::main]
async fn main() -> Result<(), dllma::Error> {
    // Create a free-tier client
    let client = Client::new()?;

    // Get all protocols
    let protocols = client.tvl().protocols().await?;
    println!("Found {} protocols", protocols.len());

    // Get current ETH price
    use dllma::coins::Token;
    let tokens = vec![Token::coingecko("ethereum")];
    let prices = client.coins().current(&tokens).await?;

    Ok(())
}
```

## Pro API Access

Some endpoints require a Pro API key. Get one at [defillama.com/subscription](https://defillama.com/subscription).

```rust
// Create client with Pro API key
let client = Client::with_api_key("your-api-key")?;

// Or from environment variable
let client = Client::from_env()?; // reads DEFILLAMA_API_KEY

// Access Pro endpoints
let yields = client.yields().pools().await?;
```

## Environment Variables

- `DEFILLAMA_API_KEY` - Your DefiLlama Pro API key (optional, for Pro endpoints)

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [DefiLlama's Terms of Service](https://defillama.com/terms).

## Disclaimer

This crate is not affiliated with or endorsed by DefiLlama.

## License

MIT
