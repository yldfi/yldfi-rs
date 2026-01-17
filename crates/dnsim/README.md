<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">dune-sim</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://docs.sim.dune.com/">Dune SIM</a> API
</p>

<p align="center">
  <a href="https://crates.io/crates/dnsim"><img src="https://img.shields.io/crates/v/dune-sim.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/dune-sim/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

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

## Installation

```toml
[dependencies]
dnsim = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use dnsim::Client;

#[tokio::main]
async fn main() -> Result<(), dnsim::Error> {
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

## Environment Variables

- `DUNE_SIM_API_KEY` - Your Dune SIM API key (required)

## License

MIT
