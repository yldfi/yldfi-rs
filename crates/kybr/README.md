<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">kyberswap</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://kyberswap.com/">KyberSwap</a> Aggregator API
</p>

<p align="center">
  <a href="https://crates.io/crates/kybr"><img src="https://img.shields.io/crates/v/kyberswap.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/kyberswap/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Route API** - Get optimal swap routes across DEXs
- **Build API** - Get transaction calldata for execution
- **Multi-chain** - Supports 15+ EVM chains

## Installation

```toml
[dependencies]
kybr = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use kybr::{Client, Chain, RouteRequest};

#[tokio::main]
async fn main() -> Result<(), kybr::Error> {
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
use kybr::{Client, Chain, RouteRequest, BuildRouteRequest};

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

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [KyberSwap Terms of Use](https://kyberswap.com/terms-of-use).

## Disclaimer

This crate is not affiliated with or endorsed by Kyber Network.

## License

MIT
