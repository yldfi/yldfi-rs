# openocean

Rust client for the OpenOcean DEX Aggregator API.

## Overview

OpenOcean is a multi-chain DEX aggregator that provides optimal swap routes across 40+ chains and hundreds of DEXs.

## Features

- **Quote API** - Get swap quotes without transaction data
- **Swap API** - Get quotes with ready-to-execute transaction data
- **Token List** - Query supported tokens on each chain
- **DEX List** - Query available DEXs on each chain
- **Reverse Quote** - Calculate input amount for desired output

## Quick Start

```rust
use openocean::{Client, Chain, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), openocean::Error> {
    let client = Client::new()?;

    // Get a quote for swapping 1 ETH to USDC
    let request = QuoteRequest::new(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 ETH in wei
    ).with_slippage(1.0);

    let quote = client.get_quote(Chain::Eth, &request).await?;
    println!("Output: {} USDC", quote.out_amount);

    Ok(())
}
```

## Getting Transaction Data

```rust
use openocean::{Client, Chain, SwapRequest};

#[tokio::main]
async fn main() -> Result<(), openocean::Error> {
    let client = Client::new()?;

    let request = SwapRequest::new(
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "1000000000000000000",
        "0xYourWalletAddress",
    ).with_slippage(1.0);

    let swap = client.get_swap_quote(Chain::Eth, &request).await?;

    // Ready to sign and send
    println!("To: {}", swap.to);
    println!("Data: {}", swap.data);
    println!("Value: {}", swap.value);

    Ok(())
}
```

## Supported Chains

| Chain | ID |
|-------|-----|
| Ethereum | `eth` |
| BSC | `bsc` |
| Polygon | `polygon` |
| Arbitrum | `arbitrum` |
| Optimism | `optimism` |
| Base | `base` |
| Avalanche | `avax` |
| Fantom | `fantom` |
| Gnosis | `gnosis` |
| zkSync Era | `zksync` |
| Linea | `linea` |
| Scroll | `scroll` |
| Mantle | `mantle` |
| Blast | `blast` |
| Solana | `solana` |
| Sui | `sui` |

## API Reference

- [OpenOcean API V4 Docs](https://apis.openocean.finance/developer/apis/swap-api/api-v4)

## License

MIT
