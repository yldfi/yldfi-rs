# oneinch

Rust client for the 1inch DEX Aggregator Swap API v6.0.

## Overview

1inch is one of the most popular DEX aggregators, offering optimal swap routes across hundreds of liquidity sources on multiple chains.

## Features

- Multi-chain support (Ethereum, BSC, Polygon, Arbitrum, Optimism, Base, etc.)
- Advanced Pathfinder algorithm for optimal routing
- Type-safe request/response handling
- Automatic rate limiting awareness
- Token approval helpers

## Authentication

The 1inch API requires an API key. Get yours at [portal.1inch.dev](https://portal.1inch.dev).

## Rate Limits

- **Free tier**: 1 request per second, 100,000 calls per month
- Higher tiers available for production use

## Quick Start

```rust
use oneinch::{Client, Chain, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), oneinch::Error> {
    // Create client with your API key
    let client = Client::new("your-api-key")?;

    // Get a quote for swapping 1 ETH to USDC on Ethereum
    let request = QuoteRequest::new(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 ETH in wei
    );

    let quote = client.get_quote(Chain::Ethereum, &request).await?;
    println!("Expected output: {} USDC", quote.to_amount);

    Ok(())
}
```

## Installation

```toml
[dependencies]
oneinch = "0.1"
tokio = { version = "1", features = ["full"] }
```

## License

MIT
