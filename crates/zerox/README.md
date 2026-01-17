# zerox

Rust client for the 0x (ZeroEx) DEX Aggregator API v2.

## Overview

0x is a professional-grade DEX aggregator that finds optimal swap routes across 100+ DEXs on Ethereum and EVM-compatible chains.

## Features

- Multi-chain support (Ethereum, Polygon, Arbitrum, Optimism, Base, BSC, and more)
- Professional-grade liquidity aggregation across 100+ DEXs
- Permit2 integration for efficient token approvals
- Gasless trading support
- MEV protection options
- Type-safe request/response handling

## API Key

An API key is required for production use. Get one at [0x.org](https://0x.org/docs/introduction/getting-started).

## Quick Start

```rust
use zerox::{Client, Chain, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), zerox::Error> {
    // Create a client with your API key
    let client = Client::with_api_key("your-api-key")?;

    // Get an indicative price for swapping 1 ETH to USDC
    let request = QuoteRequest::sell(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 ETH in wei
    );

    let price = client.get_price(Chain::Ethereum, &request).await?;
    println!("You would receive: {} USDC", price.buy_amount);

    Ok(())
}
```

## Installation

```toml
[dependencies]
zerox = "0.1"
tokio = { version = "1", features = ["full"] }
```

## License

MIT
