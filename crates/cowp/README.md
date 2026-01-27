<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">cowswap</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://cow.fi/">CoW Protocol</a> (CowSwap) API
</p>

<p align="center">
  <a href="https://crates.io/crates/cowp"><img src="https://img.shields.io/crates/v/cowswap.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/cowswap/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **MEV Protection** - Batch auctions protect against frontrunning
- **Gasless Trading** - Fees are taken from output tokens
- **Coincidence of Wants** - Direct peer-to-peer matching for better prices
- **Quote API** - Get swap quotes (free, no authentication)
- **Order Management** - Create, query, and cancel orders
- **Trade History** - Query executed trades
- **Multi-chain** - Supports Ethereum, Gnosis Chain, and Arbitrum

## Installation

```toml
[dependencies]
cowp = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use cowp::{Client, Chain, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), cowp::Error> {
    let client = Client::new()?;

    // Get a sell quote (exact input)
    let request = QuoteRequest::sell(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 WETH
        "0xYourAddress",
    );

    let quote = client.get_quote(None, &request).await?;
    println!("Output: {} USDC", quote.quote.buy_amount);
    println!("Fee: {} WETH", quote.quote.fee_amount);

    Ok(())
}
```

## Buy Orders (Exact Output)

```rust
use cowp::{Client, QuoteRequest};

let request = QuoteRequest::buy(
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    "1000000000", // Want exactly 1000 USDC
    "0xYourAddress",
);

let quote = client.get_quote(None, &request).await?;
println!("You will pay: {} WETH", quote.quote.sell_amount);
```

## Multi-Chain Support

| Chain | API URL |
|-------|---------|
| Ethereum | `https://api.cow.fi/mainnet` |
| Gnosis | `https://api.cow.fi/xdai` |
| Arbitrum | `https://api.cow.fi/arbitrum_one` |
| Sepolia | `https://api.cow.fi/sepolia` |

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [CoW Protocol Terms and Conditions](https://cow.fi/legal/terms).

## Disclaimer

This crate is not affiliated with or endorsed by CoW Protocol.

## License

MIT
