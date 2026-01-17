<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">mrls</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://docs.moralis.io/">Moralis Web3</a> API
</p>

<p align="center">
  <a href="https://crates.io/crates/mrls"><img src="https://img.shields.io/crates/v/mrls.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/mrls/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Wallet API** - Native balances, token balances, transactions, approvals, net worth, profitability
- **Token API** - Metadata, prices, transfers, swaps, pairs, holders, stats, trending
- **NFT API** - NFT metadata, transfers, owners, trades, floor prices, collections
- **DeFi API** - Pair prices, reserves, positions, protocol summaries
- **Block API** - Block data, timestamps, date-to-block lookups
- **Transaction API** - Transaction details, decoded calls, internal transactions
- **Resolve API** - ENS, Unstoppable Domains, domain resolution
- **Market Data API** - Top tokens, movers, NFT collections, global stats
- **Discovery API** - Token discovery, trending, analytics, scores
- **Entities API** - Wallet/protocol/exchange labels and categories

## Installation

```toml
[dependencies]
mrls = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use mrls::Client;

#[tokio::main]
async fn main() -> Result<(), mrls::Error> {
    // Create client from MORALIS_API_KEY env var
    let client = Client::from_env()?;

    // Get native balance
    let balance = client.wallet().get_native_balance(
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        Some("eth"),
    ).await?;
    println!("Balance: {} wei", balance.balance);

    // Get token price
    let price = client.token().get_price(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        Some("eth"),
    ).await?;
    println!("WETH Price: ${:?}", price.usd_price);

    Ok(())
}
```

## Environment Variables

- `MORALIS_API_KEY` - Your Moralis API key (required)

## License

MIT
