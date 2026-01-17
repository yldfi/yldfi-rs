# cowswap

Rust client for the CoW Protocol (CowSwap) API.

## Overview

CoW Protocol is a fully permissionless trading protocol that provides:
- **MEV Protection** - Batch auctions protect against frontrunning
- **Gasless Trading** - Fees are taken from output tokens
- **Coincidence of Wants** - Direct peer-to-peer matching for better prices

## Features

- **Quote API** - Get swap quotes (free, no authentication)
- **Order Management** - Create, query, and cancel orders
- **Trade History** - Query executed trades
- **Multi-chain** - Supports Ethereum, Gnosis Chain, and Arbitrum

## Quick Start

```rust
use cowswap::{Client, Chain, QuoteRequest};

#[tokio::main]
async fn main() -> Result<(), cowswap::Error> {
    let client = Client::new()?;

    // Get a sell quote (exact input)
    let request = QuoteRequest::sell(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000", // 1 WETH
        "0xYourAddress",
    );

    let quote = client.get_quote(None, &request).await?;
    println!("Output: {} USDC", quote.quote.buy_amount);
    println!("Fee: {} WETH", quote.quote.fee_amount);

    Ok(())
}
```

## Buy Orders (Exact Output)

```rust
use cowswap::{Client, QuoteRequest};

let request = QuoteRequest::buy(
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    "1000000000", // Want exactly 1000 USDC
    "0xYourAddress",
);

let quote = client.get_quote(None, &request).await?;
println!("You will pay: {} WETH", quote.quote.sell_amount);
```

## Multi-Chain Support

```rust
use cowswap::{Client, Chain, QuoteRequest};

let client = Client::new()?;

// Query on different chains
let mainnet_quote = client.get_quote(Some(Chain::Mainnet), &request).await?;
let gnosis_quote = client.get_quote(Some(Chain::Gnosis), &request).await?;
let arbitrum_quote = client.get_quote(Some(Chain::Arbitrum), &request).await?;
```

## Supported Chains

| Chain | API URL |
|-------|---------|
| Ethereum | `https://api.cow.fi/mainnet` |
| Gnosis | `https://api.cow.fi/xdai` |
| Arbitrum | `https://api.cow.fi/arbitrum_one` |
| Sepolia | `https://api.cow.fi/sepolia` |

## Order Submission

Note: Getting quotes is free and doesn't require signing. Order submission
requires signing the order data with your wallet:

```rust
use cowswap::{Client, QuoteRequest, OrderCreation, SigningScheme};

// 1. Get a quote
let quote = client.get_quote(None, &request).await?;

// 2. Sign the order (external - use ethers/alloy)
let signature = sign_order(&quote.quote)?; // You implement this

// 3. Submit the order
let order = OrderCreation {
    sell_token: quote.quote.sell_token,
    buy_token: quote.quote.buy_token,
    sell_amount: quote.quote.sell_amount,
    buy_amount: quote.quote.buy_amount,
    valid_to: quote.quote.valid_to,
    app_data: quote.quote.app_data,
    fee_amount: quote.quote.fee_amount,
    kind: quote.quote.kind,
    partially_fillable: false,
    receiver: quote.quote.receiver,
    signature,
    signing_scheme: SigningScheme::Eip712,
    from: "0xYourAddress".to_string(),
    quote_id: quote.id,
};

let order_uid = client.create_order(None, &order).await?;
println!("Order submitted: {}", order_uid);
```

## API Reference

- [CoW Protocol Docs](https://docs.cow.fi/)
- [Order Book API](https://api.cow.fi/docs/)

## License

MIT
