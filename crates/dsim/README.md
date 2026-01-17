# dsim

Rust client for the Dune SIM API.

## Overview

An unofficial Rust client for the [Dune SIM API](https://docs.sim.dune.com/), providing access to wallet balances, activity, collectibles, DeFi positions, and more.

## Features

- **Chains** - List supported chains
- **Balances** - Get wallet token balances across chains
- **Activity** - Get wallet transaction history
- **Collectibles** - Get NFT holdings
- **DeFi** - Get DeFi protocol positions
- **Tokens** - Get token metadata
- **Holders** - Get token holders
- **Transactions** - Get transaction details
- **Webhooks** - Set up activity webhooks

## Quick Start

```rust
use dsim::Client;

#[tokio::main]
async fn main() -> Result<(), dsim::Error> {
    let client = Client::new("your-api-key")?;

    // Get supported chains
    let chains = client.chains().list().await?;
    for chain in chains.chains {
        println!("{}: {}", chain.chain_id, chain.name);
    }

    // Get wallet balances
    let balances = client.balances().get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
    for balance in balances.balances {
        println!("{}: {} {}", balance.chain, balance.amount, balance.symbol);
    }

    Ok(())
}
```

## Installation

```toml
[dependencies]
dsim = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

- `DUNE_SIM_API_KEY` - Your Dune SIM API key (required)

## License

MIT
