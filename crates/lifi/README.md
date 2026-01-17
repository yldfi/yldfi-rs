# lifi

Rust client for the LI.FI cross-chain bridge and DEX aggregator API.

## Overview

LI.FI is a multi-chain liquidity aggregation protocol that integrates multiple bridges and DEXs to provide optimal cross-chain swap routes across 20+ chains.

## Features

- **Cross-chain swaps** - Swap tokens across different blockchains in a single transaction
- **Bridge aggregation** - Access multiple bridges (Stargate, Hop, Connext, Across, etc.)
- **DEX aggregation** - Optimal routing through DEXs on each chain
- **Route optimization** - Find the best route by price, speed, or security
- **Transaction tracking** - Monitor cross-chain transaction status

## Quick Start

```rust
use lifi::{Client, QuoteRequest, chains};

#[tokio::main]
async fn main() -> Result<(), lifi::Error> {
    // Create a client with an integrator identifier
    let client = Client::with_integrator("my-app")?;

    // Get a quote for swapping 1 ETH on Ethereum to USDC on Arbitrum
    let request = QuoteRequest::new(
        chains::ETHEREUM,
        chains::ARBITRUM,
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
        "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
        "1000000000000000000",
        "0xYourWalletAddress",
    ).with_slippage(0.5);

    let quote = client.get_quote(&request).await?;
    println!("Estimated output: {}", quote.estimate.to_amount);

    Ok(())
}
```

## Installation

```toml
[dependencies]
lifi = "0.1"
tokio = { version = "1", features = ["full"] }
```

## License

MIT
