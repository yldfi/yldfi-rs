<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">yldfi-common</h1>

<p align="center">
  Shared utilities for <a href="https://github.com/yldfi/yldfi-rs">yldfi-rs</a> API clients
</p>

<p align="center">
  <a href="https://crates.io/crates/yldfi-common"><img src="https://img.shields.io/crates/v/yldfi-common.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/yldfi-common/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Retry utilities** - Retry with exponential backoff for transient errors
- **Ethereum utilities** - Address and transaction hash validation
- **Chain mappings** - EVM chain ID and name mappings
- **Unit conversions** - Wei/Gwei/Ether conversion utilities
- **HTTP client** - Pre-configured reqwest client builder

## Retry Utilities

```rust
use yldfi_common::{with_retry, RetryConfig, RetryableError};

// Implement RetryableError for your error type
impl RetryableError for MyError {
    fn is_retryable(&self) -> bool { true }
}

async fn example() {
    let config = RetryConfig::default();
    let result = with_retry(&config, || async {
        Ok::<_, MyError>("success")
    }).await;
}
```

## Ethereum Utilities

```rust
use yldfi_common::eth::{is_valid_address, normalize_address};

assert!(is_valid_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));
```

## Chain Mappings

```rust
use yldfi_common::chains::Chain;

let chain = Chain::from_id(1);
assert_eq!(chain, Chain::Ethereum);
assert_eq!(chain.name(), "ethereum");
```

## Unit Conversions

```rust
use yldfi_common::units::{to_wei, from_wei};

// Convert 1.5 ETH to wei
let wei = to_wei("1.5", 18).unwrap();
```

## Installation

```toml
[dependencies]
yldfi-common = "0.1"
```

## License

MIT
