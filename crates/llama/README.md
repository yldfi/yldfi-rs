# llama

Rust client for the DefiLlama API.

## Overview

An unofficial Rust client for the [DefiLlama API](https://defillama.com/), providing access to DeFi protocol data including TVL, prices, yields, volumes, fees, stablecoins, bridges, and more.

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

## Quick Start

```rust
use llama::Client;

#[tokio::main]
async fn main() -> Result<(), llama::Error> {
    // Create a free-tier client
    let client = Client::new()?;

    // Get all protocols
    let protocols = client.tvl().protocols().await?;
    println!("Found {} protocols", protocols.len());

    // Get current ETH price
    use llama::coins::Token;
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

## Installation

```toml
[dependencies]
llama = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

- `DEFILLAMA_API_KEY` - Your DefiLlama Pro API key (optional, for Pro endpoints)

## License

MIT
