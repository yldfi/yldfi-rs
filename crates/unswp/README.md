# unswp

Unofficial Rust client for Uniswap V2, V3, and V4.

## Features

- **Multi-version support** - V2, V3, and V4 protocols
- **On-chain queries** via ephemeral lens contracts (no API key required)
- **Historical data** via The Graph subgraph (optional, requires API key)
- **Unified client** combining both data sources

## Installation

```toml
[dependencies]
unswp = "0.1"
```

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

    // Historical query
    let eth_price = client.get_eth_price().await?;
    println!("ETH price: ${:.2}", eth_price);

    let top_pools = client.get_top_pools(10).await?;
    for pool in top_pools {
        println!("{}/{}: ${}", pool.token0.symbol, pool.token1.symbol, pool.total_value_locked_usd);
    }

    Ok(())
}
```

## Supported Networks

| Network | On-chain | Subgraph |
|---------|----------|----------|
| Ethereum | Yes | Yes |
| Arbitrum | Yes | Yes |
| Optimism | Yes | Yes |
| Polygon | Yes | Yes |
| Base | Yes | Yes |

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `THEGRAPH_API_KEY` | For subgraph | The Graph API key |

## Re-exported SDK Crates

```rust
use unswp::sdk_core;  // uniswap-sdk-core
use unswp::v2_sdk;    // uniswap-v2-sdk
use unswp::v3_sdk;    // uniswap-v3-sdk
use unswp::v4_sdk;    // uniswap-v4-sdk
```

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [Uniswap Labs Terms of Service](https://uniswap.org/terms-of-service) and [The Graph Terms of Service](https://thegraph.com/terms-of-service/) (for subgraph queries).

## Disclaimer

This crate is not affiliated with or endorsed by Uniswap Labs or The Graph.

## License

MIT
