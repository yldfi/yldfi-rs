<p align="center">
  <img src="../../logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">pamm</h1>

<p align="center">
  Unofficial Rust client for prop AMM quotes, venues and price levels on Ethereum
</p>

<p align="center">
  <a href="https://crates.io/crates/pamm"><img src="https://img.shields.io/crates/v/pamm.svg" alt="crates.io"></a>
  <a href="https://opensource.org/license/mit"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Overview

A prop AMM (proprietary AMM) is an on-chain pool run by one market maker, who re-prices it every block. The block builder applies the maker's latest price just before any trade against the pool. The pools are the makers' own contracts; a builder adds the ordering guarantee and publishes taker-side data, which this crate reads.

The client talks to [Titan Builder](https://docs.titanbuilder.xyz/propamms)'s public data endpoint by default. No API key is needed for the public data. Pass another base URL in the config for a builder that serves the same JSON-RPC methods.

## Features

- **Best quote** - `quote` scans every prop AMM in the builder's latest snapshot
- **Single-venue quote** - `quote_venue` asks one prop AMM by address
- **Venues** - `venues` lists makers, addresses, pairs and the state override for simulating swaps locally
- **Price levels** - `books` returns bid and ask levels per maker and market
- **Regions** - EU (default), US and Asia-Pacific endpoints

## A quote is conditional

A quote is the maker's price in the builder's snapshot for the block being built. A trade only gets it when it is included by a builder that carries that maker's price update; in any other block the pool still holds its last landed price. Treat the figure as an indication, not a firm quote. Executing against a prop AMM means sending a transaction or bundle through the router the quote names.

Pairs are quoted in wrapped tokens: pass WETH, not a native-ETH placeholder. Amounts are in the token's smallest units.

## Installation

```toml
[dependencies]
pamm = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust,no_run
#[tokio::main]
async fn main() -> Result<(), pamm::Error> {
    let client = pamm::Client::new()?;

    // 1 WETH -> USDC
    let quote = client
        .quote(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
        )
        .await?;

    let venues = client.venues().await?;
    let maker = venues.maker_of(&quote.pamm).unwrap_or("unknown maker");
    println!("{} USDC units from {maker} (block {})", quote.amount_out, quote.block_number);

    Ok(())
}
```

A pair no prop AMM quotes returns `DomainError::UnknownPair`.

## Other regions and builders

```rust,no_run
use pamm::{ApiConfig, Client, Region};

# fn main() -> Result<(), pamm::Error> {
let us = Client::with_region(Region::Us)?;
let other = Client::with_config(ApiConfig::new("https://data.builder.example"))?;
# Ok(())
# }
```

## Not covered

- Maker endpoints (sending price updates) and bundle submission
- The WebSocket streams; this client polls over JSON-RPC
- Permissioned maker data that needs an access key

## Terms of Service

This is an unofficial client. Use of the data endpoints is subject to the terms of the builder that serves them.

## Disclaimer

This software is provided as is. Quotes are indicative and may not be achievable; verify before trading.

## License

MIT
