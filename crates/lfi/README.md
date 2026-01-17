<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">lifi</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://li.fi/">LI.FI</a> Cross-Chain Bridge and DEX Aggregator API
</p>

<p align="center">
  <a href="https://crates.io/crates/lfi"><img src="https://img.shields.io/crates/v/lifi.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/lifi/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

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
use lfi::{Client, QuoteRequest, chains};

#[tokio::main]
async fn main() -> Result<(), lfi::Error> {
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
lfi = "0.1"
tokio = { version = "1", features = ["full"] }
```

## License

MIT
