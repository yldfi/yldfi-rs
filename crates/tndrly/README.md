<p align="center">
  <img src="logo-128.png" alt="yld_fi" width="128" height="128">
</p>
<h1 align="center">tndrly</h1>
<p align="center">
  Unofficial Rust client for the <a href="https://tenderly.co">Tenderly</a> API
</p>
<p align="center">
  <a href="https://crates.io/crates/tndrly"><img src="https://img.shields.io/crates/v/tndrly.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/tndrly/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Simulation API** - Simulate transactions without broadcasting
- **Virtual TestNets API** - Create isolated blockchain environments
- **Alerts API** - Monitor on-chain activity with notifications
- **Contract API** - Manage and verify smart contracts
- **Web3 Actions API** - Deploy serverless functions
- **Wallets API** - Track and monitor wallet addresses

## Installation

```toml
[dependencies]
tndrly = "0.3"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use tndrly::Client;
use tndrly::simulation::SimulationRequest;

#[tokio::main]
async fn main() -> Result<(), tndrly::Error> {
    // Create client from environment variables
    let client = Client::from_env()?;

    // Simulate a transaction
    let request = SimulationRequest::new(
        "0x0000000000000000000000000000000000000000",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045"
    )
    .gas(100000)
    .save(true);

    let result = client.simulation().simulate(&request).await?;
    println!("Gas used: {}", result.simulation.gas_used);
    println!("Status: {}", if result.simulation.status { "success" } else { "failed" });

    Ok(())
}
```

## Environment Variables

```bash
export TENDERLY_ACCESS_KEY="your-access-key"
export TENDERLY_ACCOUNT="your-account-slug"
export TENDERLY_PROJECT="your-project-slug"
```

## API Modules

### Simulation

```rust
use tndrly::simulation::{SimulationRequest, BundleSimulationRequest};

// Single simulation
let request = SimulationRequest::new(from, to, calldata)
    .network_id("1")
    .value_wei(1_000_000_000_000_000_000u128)
    .gas(100000)
    .save(true);
let result = client.simulation().simulate(&request).await?;

// Bundle simulation
let bundle = BundleSimulationRequest::new(vec![tx1, tx2, tx3]);
let results = client.simulation().simulate_bundle(&bundle).await?;

// List saved simulations
let sims = client.simulation().list(0, 10).await?;

// Share a simulation
let url = client.simulation().share("sim-id").await?;
```

### Virtual TestNets

```rust
use tndrly::vnets::{CreateVNetRequest, ListVNetsQuery};

// Create a VNet
let request = CreateVNetRequest::new("my-testnet", "My TestNet", 1)
    .block_number(18000000)
    .sync_state(true);
let vnet = client.vnets().create(&request).await?;

// List VNets
let vnets = client.vnets().list(None).await?;

// Delete VNets (CI cleanup)
client.vnets().delete_many(vec!["id1".into(), "id2".into()]).await?;
```

### Alerts

> **Note:** The Tenderly Alerts API uses an undocumented request format. Alert creation
> (`create()`) may not work as expected. Read operations (`list()`, `get()`, `history()`)
> work correctly. See [src/alerts/types.rs](src/alerts/types.rs) for details.

```rust
use tndrly::alerts::AlertHistoryQuery;

// List existing alerts
let alerts = client.alerts().list().await?;

// Get alert history
let history = client.alerts()
    .history(Some(AlertHistoryQuery::new().page(1).per_page(50)))
    .await?;

// Get a specific alert
let alert = client.alerts().get("alert-id").await?;
```

### Contracts

```rust
use tndrly::contracts::{AddContractRequest, VerifyContractRequest};

// Add a contract
let contract = client.contracts()
    .add(&AddContractRequest::new("1", "0xAddress")
        .display_name("My Contract")
        .tag("defi"))
    .await?;

// Verify source code
let result = client.contracts()
    .verify(&VerifyContractRequest::new(
        "1",
        "0xAddress",
        "MyContract",
        source_code,
        "v0.8.19+commit.7dd6d404",
    ).optimization(true, 200))
    .await?;
```

### Web3 Actions

```rust
use tndrly::actions::{CreateActionRequest, ActionTrigger, TriggerConfig};

// Create an action
let action = client.actions()
    .create(&CreateActionRequest::new(
        "Notify Slack",
        ActionTrigger::Alert,
        source_code,
    )
    .trigger_config(TriggerConfig::alert("alert-id"))
    .secret("SLACK_WEBHOOK", webhook_url))
    .await?;

// View execution logs
let logs = client.actions().logs(&action.id).await?;
```

### Wallets

```rust
use tndrly::wallets::{AddWalletRequest, UpdateWalletRequest};

// Add a wallet to monitor
let wallet = client.wallets()
    .add(&AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .display_name("vitalik.eth")
        .tag("whale"))
    .await?;

// List all wallets
let wallets = client.wallets().list().await?;

// Update wallet metadata
client.wallets()
    .update("0xd8dA...", &UpdateWalletRequest::new()
        .display_name("Vitalik Buterin"))
    .await?;

// Remove a wallet
client.wallets().remove("0xd8dA...").await?;
```

## License

MIT
