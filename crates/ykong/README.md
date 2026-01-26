# ykong

Rust client for Yearn's Kong GraphQL API.

## Features

- Typed access to Yearn vault, strategy, price, TVL, and report data
- Async/await support with tokio
- Builder pattern for configuration
- Comprehensive error handling

## Installation

```toml
[dependencies]
ykong = "0.1"
```

## Quick Start

```rust
use ykong::Client;

#[tokio::main]
async fn main() -> ykong::Result<()> {
    let client = Client::new()?;

    // Get all Ethereum mainnet vaults
    let vaults = client.vaults().by_chain(1).await?;
    println!("Found {} vaults", vaults.len());

    // Get v3 vaults only
    let v3_vaults = client.vaults().v3_vaults().await?;

    // Get token price
    let price = client.prices().current(1, "0x...").await?;

    Ok(())
}
```

## Supported Chains

| Chain ID | Network |
|----------|---------|
| 1 | Ethereum |
| 137 | Polygon |
| 42161 | Arbitrum |
| 10 | Optimism |
| 8453 | Base |

## API Reference

- `client.vaults()` - Vault queries
- `client.strategies()` - Strategy queries
- `client.prices()` - Price queries
- `client.tvls()` - TVL queries
- `client.reports()` - Report queries

## License

MIT
