# gplus

Unofficial Rust client for the [GoPlus Security API](https://docs.gopluslabs.io/).

GoPlus provides real-time token security analysis including honeypot detection, tax analysis, ownership checks, and more.

## Features

- Token security analysis (honeypot detection, buy/sell tax, ownership)
- Batch queries for multiple tokens
- Type-safe response parsing with helper methods
- Free API, no key required

## Installation

```toml
[dependencies]
gplus = "0.1"
```

## Quick Start

```rust
use gplus::Client;

#[tokio::main]
async fn main() -> gplus::Result<()> {
    let client = Client::new()?;

    // Check token security (USDT on Ethereum)
    let security = client.token_security(1, "0xdac17f958d2ee523a2206206994597c13d831ec7").await?;

    println!("Token: {}", security.token_symbol.as_deref().unwrap_or("Unknown"));
    println!("Is honeypot: {}", security.is_honeypot());
    println!("Is verified: {}", security.is_verified());
    println!("Buy tax: {:?}%", security.buy_tax_percent());
    println!("Sell tax: {:?}%", security.sell_tax_percent());

    if security.has_major_risks() {
        println!("WARNING: Token has major risks!");
        for issue in security.get_issues() {
            println!("  - {}", issue);
        }
    }

    Ok(())
}
```

## Batch Queries

```rust
let addresses = &[
    "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
];

let results = client.token_security_batch(1, addresses).await?;
for (addr, security) in results {
    println!("{}: honeypot={}", addr, security.is_honeypot());
}
```

## Supported Chains

| Chain | ID |
|-------|-----|
| Ethereum | 1 |
| BSC | 56 |
| Polygon | 137 |
| Arbitrum | 42161 |
| Base | 8453 |
| Avalanche | 43114 |
| Optimism | 10 |
| Fantom | 250 |
| Cronos | 25 |
| Gnosis | 100 |
| Linea | 59144 |
| Scroll | 534352 |
| Mantle | 5000 |
| zkSync Era | 324 |
| Blast | 81457 |

## Security Checks

The `TokenSecurity` struct provides helper methods:

- `is_honeypot()` - Cannot sell
- `is_verified()` - Contract source is open
- `is_proxy()` - Is a proxy contract
- `is_mintable()` - Tokens can be minted
- `is_transfer_pausable()` - Transfers can be paused
- `can_blacklist()` - Owner can blacklist addresses
- `has_hidden_owner()` - Hidden owner detected
- `has_anti_whale()` - Anti-whale mechanism
- `buy_tax_percent()` / `sell_tax_percent()` - Tax percentages
- `has_high_sell_tax()` - Sell tax > 10%
- `is_owner_renounced()` - Owner is zero address
- `has_major_risks()` - Any major red flags
- `get_issues()` - List of detected issues

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [GoPlus Terms of Service](https://gopluslabs.io/terms).

## Disclaimer

This crate is not affiliated with or endorsed by GoPlus Security.

## License

MIT
