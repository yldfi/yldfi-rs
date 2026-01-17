<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>
<h1 align="center">alcmy</h1>
<p align="center">
  Unofficial Rust client for the <a href="https://www.alchemy.com">Alchemy</a> API
</p>
<p align="center">
  <a href="https://crates.io/crates/alcmy"><img src="https://img.shields.io/crates/v/alcmy.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/alcmy/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **NFT API** - Ownership, metadata, sales, spam detection
- **Prices API** - Token prices by symbol/address, historical data
- **Portfolio API** - Multi-chain token balances and NFT holdings
- **Token API** - ERC-20 balances, metadata, allowances
- **Transfers API** - Historical transaction data
- **Debug API** - Transaction and block tracing
- **Trace API** - Parity-style tracing
- **Simulation API** - Simulate transactions and asset changes
- **Bundler API** - ERC-4337 Account Abstraction
- **Gas Manager API** - Gas sponsorship and policy management
- **Wallet API** - Smart wallet operations
- **Accounts API** - Authentication (email, passkey, JWT)
- **Notify API** - Webhook management
- **Beacon API** - Ethereum consensus layer
- **Solana DAS API** - Digital Asset Standard queries

## Installation

```toml
[dependencies]
alcmy = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use alcmy::{Client, Network};

#[tokio::main]
async fn main() -> Result<(), alcmy::Error> {
    let client = Client::new("your-api-key", Network::EthMainnet)?;

    // Get NFTs for an address
    let nfts = client.nft().get_nfts_for_owner("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
    println!("Found {} NFTs", nfts.total_count);

    // Get token balances
    let balances = client.token().get_token_balances("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
    println!("Found {} tokens", balances.token_balances.len());

    Ok(())
}
```

## Supported Networks

Ethereum, Polygon, Arbitrum, Optimism, Base, zkSync, Solana, and 20+ more networks.

```rust
use alcmy::Network;

let client = Client::new("api-key", Network::EthMainnet)?;
let client = Client::new("api-key", Network::Polygon)?;
let client = Client::new("api-key", Network::Arbitrum)?;
let client = Client::new("api-key", Network::Base)?;
let client = Client::new("api-key", Network::SolanaMainnet)?;
```

## API Examples

### NFT API

```rust
// Get NFTs for owner
let nfts = client.nft().get_nfts_for_owner("0x...").await?;

// Get NFT metadata
let nft = client.nft().get_nft_metadata("0xcontract", "1").await?;

// Check if address owns NFT from collection
let is_holder = client.nft().is_holder_of_contract("0xwallet", "0xcontract").await?;

// Get floor price
let floor = client.nft().get_floor_price("0xcontract").await?;
```

### Token API

```rust
// Get ERC-20 balances
let balances = client.token().get_token_balances("0x...").await?;

// Get token metadata
let metadata = client.token().get_token_metadata("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").await?;
println!("{} ({})", metadata.name.unwrap(), metadata.symbol.unwrap());

// Get allowance
let allowance = client.token().get_token_allowance("0xtoken", "0xowner", "0xspender").await?;
```

### Transfers API

```rust
use alcmy::transfers::AssetTransfersOptions;

// Get transfers from an address
let transfers = client.transfers().get_transfers_from("0x...").await?;

// Get transfers with options
let options = AssetTransfersOptions::from_address("0x...")
    .category(vec!["erc20", "erc721"])
    .with_metadata()
    .exclude_zero_value();
let transfers = client.transfers().get_asset_transfers(&options).await?;
```

### Debug API

```rust
// Trace a transaction
let trace = client.debug().trace_transaction("0xtxhash").await?;

// Trace a call
let call = TraceCallObject::new("0xfrom", "0xto").data("0xcalldata");
let trace = client.debug().trace_call(&call, "latest").await?;
```

### Simulation API

```rust
use alcmy::simulation::SimulationTransaction;

// Simulate asset changes
let tx = SimulationTransaction::new("0xfrom", "0xto")
    .data("0xcalldata")
    .value("1000000000000000000");
let result = client.simulation().simulate_asset_changes(&tx).await?;

for change in result.changes {
    println!("{}: {} {}", change.asset_type, change.amount, change.symbol.unwrap_or_default());
}
```

### Bundler API (ERC-4337)

```rust
// Get supported entry points
let entry_points = client.bundler().supported_entry_points().await?;

// Estimate gas for UserOperation
let gas = client.bundler().estimate_user_operation_gas(&user_op, "0xEntryPoint").await?;

// Send UserOperation
let hash = client.bundler().send_user_operation(&user_op, "0xEntryPoint").await?;

// Get UserOperation receipt
let receipt = client.bundler().get_user_operation_receipt(&hash).await?;
```

### Beacon API

```rust
// Get genesis info
let genesis = client.beacon().get_genesis().await?;

// Get validators
let validators = client.beacon().get_validators("head").await?;

// Get block
let block = client.beacon().get_block("head").await?;
```

### Solana DAS API

```rust
// Get asset by ID
let asset = client.solana().get_asset("AssetId123...").await?;

// Get assets by owner
let assets = client.solana().get_assets_by_owner("WalletAddress...").await?;

// Search assets
let request = SearchAssetsRequest::new().owner("...").collection("...");
let results = client.solana().search_assets(&request).await?;
```

## License

MIT
