<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">crv</h1>

<p align="center">
  Unofficial Rust client for <a href="https://curve.fi">Curve Finance</a> - REST API bindings and local swap router
</p>

<p align="center">
  <a href="https://crates.io/crates/crv"><img src="https://img.shields.io/crates/v/crv.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/crv/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

### REST API Clients
Bindings to the Curve Finance REST APIs at `api.curve.finance` and `prices.curve.finance`:
- **Pools API** - Query all Curve pools across chains
- **Volumes API** - 24h volumes and base APYs
- **Gauges API** - Gauge data and CRV rewards
- **Lending API** - Lending vault information
- **Tokens API** - Token metadata from pools
- **crvUSD API** - crvUSD and scrvUSD supply data
- **Prices API** - Token pricing via prices.curve.finance

### Local Router Implementation
A Rust port of the routing algorithm from [curve-js](https://github.com/curvefi/curve-js):
- **Route Finding** - Build a graph from pool data and find optimal swap paths
- **Calldata Generation** - Encode transactions for the on-chain [curve-router-ng](https://github.com/curvefi/curve-router-ng) contract

> **Note:** The Curve REST API does not provide routing endpoints. Like curve-js, our router fetches pool data from the API, builds a local graph, and computes routes client-side.

## Installation

```toml
[dependencies]
crv = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use crv::Client;

#[tokio::main]
async fn main() -> Result<(), crv::Error> {
    let client = Client::new()?;

    // Get all pools on Ethereum
    let pools = client.pools().get_all_on_chain("ethereum").await?;
    println!("Found {} pools", pools.data.pool_data.len());

    Ok(())
}
```

## Router

The router builds a graph of tokens and pools, then uses depth-first search to find optimal swap routes. This mirrors the approach used by the official [curve-js](https://github.com/curvefi/curve-js) SDK.

### How It Works

1. **Fetch pool data** from the Curve REST API
2. **Build a graph** where nodes are tokens and edges are pool swaps
3. **Find routes** using DFS with TVL-based ranking
4. **Generate calldata** for the on-chain router contract

```rust
use crv::Client;

#[tokio::main]
async fn main() -> Result<(), crv::Error> {
    let client = Client::new()?;

    // Build router from pool data (fetches from API)
    let router = client.build_router("ethereum").await?;
    println!("Graph: {} tokens, {} edges",
        router.stats().token_count,
        router.stats().edge_count);

    // Find routes between tokens
    let dai = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    let routes = router.find_routes(dai, usdc);
    println!("Found {} routes", routes.len());

    if let Some(best) = routes.first() {
        println!("Best route: {} hops, min TVL: ${:.0}",
            best.steps.len(), best.min_tvl);

        // Generate calldata for the on-chain router contract
        let calldata = router.encode_swap(best, "1000000000000000000", "990000")?;
        println!("Calldata: {} bytes", calldata.len());

        // Use calldata with ethers/alloy to send transaction to router contract
        let router_addr = router.router_address().unwrap();
        println!("Send to router: {}", router_addr);
    }

    Ok(())
}
```

### Swap Types

The router supports all 9 swap types from curve-router-ng:

| Type | Description |
|------|-------------|
| 1 | Standard pool exchange |
| 2 | Exchange underlying tokens |
| 3 | Underlying exchange via zap (metapools) |
| 4 | Coin → LP token (add_liquidity) |
| 5 | Underlying coin → LP token (lending pools) |
| 6 | LP token → coin (remove_liquidity_one_coin) |
| 7 | LP token → underlying coin (lending pools) |
| 8 | Wrapper operations (ETH↔WETH, stETH↔wstETH, frxETH↔sfrxETH) |
| 9 | ERC4626 asset ↔ share |

### Router Contract Addresses

| Chain | Address |
|-------|---------|
| Ethereum | `0x16C6521Dff6baB339122a0FE25a9116693265353` |
| Polygon | `0x0DCDED3545D565bA3B19E683431381007245d983` |
| Arbitrum | `0x2191718CD32d02B8E60BAdFFeA33E4B5DD9A0A0D` |
| Optimism | `0x0DCDED3545D565bA3B19E683431381007245d983` |
| Base | `0x4f37A9d177470499A2dD084621020b023fcffc1F` |
| Avalanche | `0x0DCDED3545D565bA3B19E683431381007245d983` |
| Fantom | `0x0DCDED3545D565bA3B19E683431381007245d983` |
| BSC | `0xA72C85C258A81761433B4e8da60505Fe3Dd551CC` |
| Gnosis | `0x0DCDED3545D565bA3B19E683431381007245d983` |
| Fraxtal | `0x9f2Fa7709B30c75047980a0d70A106728f0Ef2db` |
| Mantle | `0x4f37A9d177470499A2dD084621020b023fcffc1F` |
| zkSync | `0x7C915390e109CA66934f1eB285854375D1B127FA` |

## Prices API

Query token prices via the separate prices.curve.finance API:

```rust
use crv::PricesClient;

#[tokio::main]
async fn main() -> Result<(), crv::Error> {
    let client = PricesClient::new()?;

    // Get USD price for WETH
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let price = client.get_usd_price("ethereum", weth).await?;

    Ok(())
}
```

## License

MIT
