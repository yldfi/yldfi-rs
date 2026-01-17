# mrls

Rust client for the Moralis Web3 API.

## Overview

A comprehensive Rust client for the [Moralis Web3 API](https://docs.moralis.io/), providing access to wallet, token, NFT, DeFi, and market data across multiple chains.

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

## Installation

```toml
[dependencies]
mrls = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

- `MORALIS_API_KEY` - Your Moralis API key (required)

## License

MIT
