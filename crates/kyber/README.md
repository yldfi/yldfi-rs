# kyber

Rust client for the KyberSwap Aggregator API.

## Overview

KyberSwap is a multi-chain DEX aggregator that provides optimal swap routes across many DEXs.

## Quick Start

```rust
use kyber::{Client, Chain, RouteRequest};

#[tokio::main]
async fn main() -> Result<(), kyber::Error> {
    let client = Client::new()?;

    // Get swap routes
    let request = RouteRequest::new(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 WETH
    ).with_slippage_bps(50); // 0.5% slippage

    let route = client.get_routes(Chain::Ethereum, &request).await?;
    println!("Output: {} USDC", route.amount_out);

    Ok(())
}
```

## Getting Transaction Data

```rust
use kyber::{Client, Chain, RouteRequest, BuildRouteRequest};

// First get routes
let route_summary = client.get_routes(Chain::Ethereum, &request).await?;

// Then build for execution
let build_request = BuildRouteRequest {
    route_summary,
    sender: "0xYourAddress".to_string(),
    recipient: "0xYourAddress".to_string(),
    slippage_tolerance_bps: Some(50),
    deadline: None,
    enable_permit: None,
};

let tx_data = client.build_route(Chain::Ethereum, &build_request).await?;
println!("Router: {}", tx_data.router_address);
println!("Calldata: {}", tx_data.data);
```

## Supported Chains

Ethereum, BSC, Polygon, Arbitrum, Optimism, Avalanche, Base, Fantom, Linea, Scroll, zkSync, Blast, Mantle, Polygon zkEVM

## License

MIT
