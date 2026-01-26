# unswp

Unofficial Rust client for Uniswap V2, V3, and V4, combining on-chain lens queries and subgraph historical data.

## Build Commands

```bash
# Development build
cargo build -p unswp

# Release build
cargo build -p unswp --release

# Run tests
cargo test -p unswp

# Check without building
cargo check -p unswp

# Format code
cargo fmt -p unswp

# Lint
cargo clippy -p unswp
```

## Features

- **Multi-version support** - V2, V3, and V4 protocols
- **On-chain queries** via ephemeral lens contracts (no API key required)
- **Historical data** via The Graph subgraph (optional, requires API key)
- **Unified client** that combines both data sources

## Quick Start

### On-chain only (no API key needed)

```rust
use unswp::Client;
use unswp::lens::pools;

#[tokio::main]
async fn main() -> unswp::Result<()> {
    let client = Client::mainnet("https://eth.llamarpc.com")?;

    // Get current pool state
    let state = client.get_pool_state(pools::MAINNET_WETH_USDC_005).await?;
    println!("Current tick: {}", state.tick);

    // Get pool liquidity
    let liquidity = client.get_liquidity(pools::MAINNET_WETH_USDC_005).await?;
    println!("Liquidity: {}", liquidity);

    Ok(())
}
```

### With historical data (requires The Graph API key)

```rust
use unswp::Client;

#[tokio::main]
async fn main() -> unswp::Result<()> {
    let client = Client::mainnet_with_subgraph(
        "https://eth.llamarpc.com",
        "your-graph-api-key"
    )?;

    // On-chain query
    let block = client.get_block_number().await?;
    println!("Current block: {}", block);

    // Historical query (requires API key)
    let eth_price = client.get_eth_price().await?;
    println!("ETH price: ${:.2}", eth_price);

    let top_pools = client.get_top_pools(10).await?;
    for pool in top_pools {
        println!("{}/{}: ${}", pool.token0.symbol, pool.token1.symbol, pool.total_value_locked_usd);
    }

    Ok(())
}
```

## Project Structure

```
src/
├── lib.rs        # Public exports and re-exports
├── client.rs     # Unified Client combining lens + subgraph
├── error.rs      # Error types (ApiError<DomainError>)
├── types.rs      # Data types (Pool, Swap, Token, etc.)
├── lens/         # On-chain queries via ephemeral contracts
│   ├── mod.rs    # LensClient
│   ├── pools.rs  # Well-known pool addresses
│   ├── tokens.rs # Well-known token addresses
│   └── factories.rs # Factory addresses
└── subgraph/     # Historical data via The Graph
    ├── mod.rs    # SubgraphClient
    ├── config.rs # SubgraphConfig with network presets
    └── queries.rs # GraphQL queries
```

## Supported Networks

| Network | On-chain | Subgraph V2 | Subgraph V3 | Subgraph V4 |
|---------|----------|-------------|-------------|-------------|
| Ethereum | Yes | Yes | Yes | Yes |
| Arbitrum | Yes | Yes | Yes | Yes |
| Optimism | Yes | Yes | Yes | - |
| Polygon | Yes | Yes | Yes | - |
| Base | Yes | - | Yes | Yes |

## Key Dependencies

- **alloy** - Ethereum provider and types
- **uniswap-sdk-core** - Core Uniswap types
- **uniswap-v2-sdk** - V2 protocol support
- **uniswap-v3-sdk** - V3 protocol support
- **uniswap-v4-sdk** - V4 protocol support
- **uniswap-lens** - Ephemeral contract queries
- **reqwest** - HTTP client for subgraph
- **yldfi-common** - Shared API utilities

## Error Handling

The crate uses `yldfi_common::api::ApiError<DomainError>` for errors:

```rust
use unswp::{Error, Result};

async fn example(client: &unswp::Client) -> Result<()> {
    match client.get_pool_state("0x...").await {
        Ok(state) => println!("Tick: {}", state.tick),
        Err(Error::Domain(unswp::error::DomainError::PoolNotFound(addr))) => {
            eprintln!("Pool not found: {}", addr)
        }
        Err(Error::Domain(unswp::error::DomainError::SubgraphKeyRequired)) => {
            eprintln!("Set THEGRAPH_API_KEY for historical queries")
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `THEGRAPH_API_KEY` | For subgraph queries | The Graph API key |

## Testing

```bash
# Run all tests
cargo test -p unswp

# Run with output
cargo test -p unswp -- --nocapture

# Run specific test
cargo test -p unswp test_pool_not_found
```

## Re-exported SDK Crates

The crate re-exports the underlying Uniswap SDK crates for direct access:

```rust
use unswp::sdk_core;  // uniswap-sdk-core
use unswp::v2_sdk;    // uniswap-v2-sdk
use unswp::v3_sdk;    // uniswap-v3-sdk
use unswp::v4_sdk;    // uniswap-v4-sdk
```

## Notes

- On-chain queries use ephemeral lens contracts (no deployment needed)
- Subgraph queries require `THEGRAPH_API_KEY`
- Pool addresses can be found in `unswp::lens::pools`
- Token addresses can be found in `unswp::lens::tokens`
- Factory addresses can be found in `unswp::lens::factories`
