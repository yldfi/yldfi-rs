# enso

Rust client for the Enso Finance DeFi aggregator API.

## Overview

Enso Finance is a DeFi infrastructure platform that provides:
- **Multi-action Bundling** - Combine swap, deposit, stake in one transaction
- **Cross-protocol Routing** - Access to 100+ DeFi protocols
- **Position Management** - Enter/exit complex DeFi strategies
- **Gas Efficiency** - Batched transactions for lower costs

## Features

- **Route API** - Get optimal routes for token swaps
- **Bundle API** - Combine multiple DeFi actions into one transaction
- **Price API** - Query token prices
- **Balance API** - Get token balances for addresses
- **Multi-chain** - Supports Ethereum, Polygon, Arbitrum, Optimism, and more

## Quick Start

```rust
use enso::{Client, Chain, RouteRequest};

#[tokio::main]
async fn main() -> Result<(), enso::Error> {
    let client = Client::with_api_key("your-api-key")?;

    // Get route for swapping tokens
    let request = RouteRequest::new(
        Chain::Ethereum.chain_id(),
        "0xYourAddress",
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 ETH
        100, // 1% slippage in basis points
    );

    let response = client.get_route(&request).await?;
    println!("Output: {}", response.amount_out);

    // Execute the swap
    if let Some(tx) = &response.tx {
        println!("Send to: {}", tx.to);
        println!("Data: {}", tx.data);
    }

    Ok(())
}
```

## Action Bundling

Enso's killer feature is bundling multiple DeFi actions:

```rust
use enso::{Client, BundleRequest, BundleAction};

let client = Client::with_api_key("your-key")?;

// Bundle: Swap ETH -> USDC, then deposit USDC to Aave
let actions = vec![
    BundleAction::swap(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "1000000000000000000",
    ),
    BundleAction::deposit("aave-v3", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
];

let request = BundleRequest::new(1, "0xYourAddress", actions);
let bundle = client.bundle(&request).await?;

println!("Execute bundled tx to: {}", bundle.tx.to);
```

## Token Prices

```rust
use enso::Client;

let client = Client::with_api_key("your-key")?;

let price = client.get_token_price(
    1, // Ethereum
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
).await?;

println!("USDC price: ${}", price.price);
```

## Token Balances

```rust
use enso::Client;

let client = Client::with_api_key("your-key")?;

let balances = client.get_balances(1, "0xYourAddress").await?;
for balance in balances {
    println!("{}: {}", balance.symbol, balance.balance);
}
```

## Supported Chains

| Chain | Chain ID |
|-------|----------|
| Ethereum | 1 |
| Polygon | 137 |
| Arbitrum | 42161 |
| Optimism | 10 |
| Base | 8453 |
| BSC | 56 |

## Configuration

```rust
use enso::{Client, Config};
use std::time::Duration;

// API key is required
let client = Client::with_api_key("your-api-key")?;

// With custom configuration
let config = Config::new()
    .api_key("your-api-key")
    .timeout(Duration::from_secs(30));
let client = Client::with_config(config)?;
```

## API Reference

- [Enso Finance Docs](https://docs.enso.finance/)
- [API Reference](https://api.enso.finance/docs)

## License

MIT
