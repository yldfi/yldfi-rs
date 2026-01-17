<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">velora</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://www.paraswap.io/">Velora (ParaSwap)</a> DEX Aggregator API
</p>

<p align="center">
  <a href="https://crates.io/crates/vlra"><img src="https://img.shields.io/crates/v/velora.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/velora/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Overview

Velora (formerly ParaSwap) is a leading DEX aggregator that provides:
- **MEV Protection** - Private transactions through Flashbots
- **Multi-path Routing** - Split orders across multiple DEXs for best execution
- **Gas Optimization** - Efficient transaction routing to minimize gas costs
- **Delta Algorithm** - Advanced pricing algorithm for optimal rates

## Features

- **Price API** - Get optimal swap prices and routing
- **Transaction Builder** - Build executable swap transactions
- **Token Lists** - Query supported tokens per chain
- **Multi-chain** - Supports Ethereum, Polygon, BSC, Arbitrum, Optimism, Base, and more

## Quick Start

```rust
use vlra::{Client, Chain, PriceRequest};

#[tokio::main]
async fn main() -> Result<(), velora::Error> {
    let client = Client::new()?;

    // Get price for swapping 1 ETH to USDC
    let request = PriceRequest::sell(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 ETH in wei
    );

    let response = client.get_price(Chain::Ethereum, &request).await?;
    println!("Output: {} USDC (minimal units)", response.price_route.dest_amount);

    Ok(())
}
```

## Building Transactions

```rust
use vlra::{Client, Chain, PriceRequest, TransactionRequest};

let client = Client::new()?;

// 1. Get a price quote
let price_request = PriceRequest::sell(
    "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "1000000000000000000",
);
let price = client.get_price(Chain::Ethereum, &price_request).await?;

// 2. Build the transaction
let tx_request = TransactionRequest::new(
    &price.price_route,
    "0xYourWalletAddress",
    100, // 1% slippage in basis points
);

let tx = client.build_transaction(Chain::Ethereum, &tx_request).await?;
println!("Send to: {}", tx.to);
println!("Data: {}", tx.data);
println!("Value: {}", tx.value);
```

## Token Lists

```rust
use vlra::{Client, Chain};

let client = Client::new()?;

let tokens = client.get_tokens(Chain::Ethereum).await?;
for token in &tokens.tokens[..5] {
    println!("{}: {}", token.symbol, token.address);
}
```

## Supported Chains

| Chain | Chain ID |
|-------|----------|
| Ethereum | 1 |
| Polygon | 137 |
| BSC | 56 |
| Arbitrum | 42161 |
| Optimism | 10 |
| Base | 8453 |
| Avalanche | 43114 |
| Gnosis | 100 |

## Configuration

```rust
use vlra::{Client, Config};
use std::time::Duration;

// With API key (recommended for production)
let client = Client::with_api_key("your-api-key")?;

// With custom configuration
let config = Config::new()
    .api_key("your-api-key")
    .timeout(Duration::from_secs(30));
let client = Client::with_config(config)?;
```

## API Reference

- [ParaSwap Docs](https://developers.paraswap.network/)
- [API Reference](https://developers.paraswap.network/api/get-rate-for-a-token-pair)

## License

MIT
