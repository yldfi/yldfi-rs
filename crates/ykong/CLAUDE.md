# ykong

Rust client for Yearn's Kong GraphQL API. Provides typed access to vault, strategy, price, TVL, and report data.

## Build Commands

```bash
# Development build
cargo build -p ykong

# Release build
cargo build -p ykong --release

# Run tests
cargo test -p ykong

# Check without building
cargo check -p ykong

# Format code
cargo fmt -p ykong

# Lint
cargo clippy -p ykong
```

## API Overview

The client provides access to five main API endpoints:

- **Vaults** - List, get, and query Yearn vault data
- **Strategies** - List and get strategy details
- **Prices** - Current and historical token prices
- **TVLs** - Current and historical Total Value Locked
- **Reports** - Vault and strategy harvest reports

## Quick Start

```rust
use ykong::Client;

#[tokio::main]
async fn main() -> ykong::Result<()> {
    let client = Client::new()?;

    // Get all Ethereum mainnet vaults
    let vaults = client.vaults().by_chain(1).await?;

    // Get v3 vaults only
    let v3_vaults = client.vaults().v3_vaults().await?;

    // Get token price
    let price = client.prices().current(1, "0x...").await?;

    // Get TVL history
    let tvls = client.tvls().daily(1, "0x...", 30).await?;

    Ok(())
}
```

## Configuration

The client supports custom configuration via the builder pattern:

```rust
use ykong::{Client, Config};
use std::time::Duration;

let client = Client::with_config(
    Config::new()
        .with_timeout(Duration::from_secs(60))
        .with_proxy("http://proxy.example.com:8080")
        .with_rate_limit(10, Duration::from_secs(1))  // Optional: 10 req/sec
)?;
```

### Rate Limiting

The client supports optional client-side rate limiting to prevent API abuse:

```rust
use ykong::{Client, Config};
use std::time::Duration;

// 10 requests per second
let client = Client::with_config(
    Config::new().with_rate_limit(10, Duration::from_secs(1))
)?;

// 100 requests per minute
let client = Client::with_config(
    Config::new().with_rate_limit(100, Duration::from_secs(60))
)?;
```

Rate limiting is disabled by default. When enabled, requests exceeding the limit will automatically wait until a slot becomes available.

## Supported Chains

| Chain ID | Network |
|----------|---------|
| 1 | Ethereum Mainnet |
| 137 | Polygon |
| 42161 | Arbitrum |
| 10 | Optimism |
| 8453 | Base |
| 250 | Fantom |
| 100 | Gnosis |

## Project Structure

```
src/
├── lib.rs        # Public exports and Client API methods
├── client.rs     # HTTP client and GraphQL query execution
├── error.rs      # Error types (ApiError<DomainError>)
├── types.rs      # Data types (Vault, Strategy, Price, etc.)
├── vaults.rs     # VaultsApi - vault queries
├── strategies.rs # StrategiesApi - strategy queries
├── prices.rs     # PricesApi - price queries
├── tvls.rs       # TvlsApi - TVL queries
└── reports.rs    # ReportsApi - report queries
```

## Key Dependencies

- **reqwest** - HTTP client with rustls-tls
- **serde/serde_json** - JSON serialization
- **thiserror** - Error types
- **yldfi-common** - Shared API utilities (ApiError, ApiConfig, RateLimiter, Chain)

## Error Handling

The crate uses `yldfi_common::api::ApiError<DomainError>` for errors:

```rust
use ykong::{Error, Result};

async fn example(client: &ykong::Client) -> Result<()> {
    match client.vaults().get(1, "0x...").await {
        Ok(vault) => println!("Found: {}", vault.name),
        Err(Error::Api { status, message }) => eprintln!("API error {}: {}", status, message),
        Err(Error::Domain(ykong::error::DomainError::VaultNotFound(addr))) => {
            eprintln!("Vault not found: {}", addr)
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
    Ok(())
}
```

## Environment Variables

None required. The Kong API is free and public.

## Testing

```bash
# Run all tests
cargo test -p ykong

# Run with output
cargo test -p ykong -- --nocapture

# Run specific test
cargo test -p ykong test_vaults
```

## Notes

- Kong API base URL: `https://kong.yearn.farm/api/gql`
- No API key required
- Client-side rate limiting available via `Config::with_rate_limit()`
- GraphQL errors are extracted and returned as `DomainError::GraphQL`
