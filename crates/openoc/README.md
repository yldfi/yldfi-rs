<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">openoc</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://openocean.finance/">OpenOcean</a> DEX Aggregator API
</p>

<p align="center">
  <a href="https://crates.io/crates/openoc"><img src="https://img.shields.io/crates/v/openoc.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/openoc/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Quote API** - Get swap quotes without transaction data
- **Swap API** - Get quotes with ready-to-execute transaction data
- **Token List** - Query supported tokens on each chain
- **DEX List** - Query available DEXs on each chain
- **Reverse Quote** - Calculate input amount for desired output

## Installation

```toml
[dependencies]
openoc = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use openoc::{Client, Chain, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), openoc::Error> {
    let client = Client::new()?;

    // Get a quote for swapping 1 ETH to USDC
    let request = QuoteRequest::new(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1", // 1 ETH (human readable)
    ).with_gas_price("30000000000");

    let quote = client.get_quote(Chain::Eth, &request).await?;
    println!("Output: {} USDC", quote.out_amount);

    Ok(())
}
```

## Getting Transaction Data

```rust
use openoc::{Client, Chain, SwapRequest};

#[tokio::main]
async fn main() -> Result<(), openoc::Error> {
    let client = Client::new()?;

    let request = SwapRequest::new(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "1", // Human readable amount
        "0xYourWalletAddress",
    ).with_slippage(1.0);

    let swap = client.get_swap_quote(Chain::Eth, &request).await?;

    // Ready to sign and send
    println!("To: {}", swap.to);
    println!("Data: {}", swap.data);
    println!("Value: {}", swap.value);

    Ok(())
}
```

## Supported Chains

Ethereum, BSC, Polygon, Arbitrum, Optimism, Base, Avalanche, Fantom, Gnosis, zkSync Era, Linea, Scroll, Mantle, Blast, Solana, Sui

## License

MIT
