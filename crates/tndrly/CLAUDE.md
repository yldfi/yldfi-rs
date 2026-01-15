# tndrly

Unofficial Rust client library for the Tenderly API.

## Build Commands

```bash
# Build library
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Project Structure

```
src/
├── lib.rs            # Library entry point and Client impl
├── client.rs         # Core HTTP client (reqwest)
├── error.rs          # Error types (thiserror)
├── utils.rs          # Address validation utilities
├── simulation/
│   ├── mod.rs        # Simulation module exports
│   ├── api.rs        # Simulation API client
│   └── types.rs      # SimulationRequest, SimulationResponse
├── vnets/
│   ├── mod.rs        # Virtual TestNets module exports
│   ├── api.rs        # Virtual TestNets API client
│   ├── admin_rpc.rs  # Admin RPC client (time, balance, storage, snapshots)
│   └── types.rs      # VNet, CreateVNetRequest, etc.
├── alerts/
│   ├── mod.rs        # Alerts module exports
│   ├── api.rs        # Alerts API client
│   └── types.rs      # AlertType, CreateAlertRequest, etc.
├── contracts/
│   ├── mod.rs        # Contracts module exports
│   ├── api.rs        # Contracts API client
│   └── types.rs      # Contract, AddContractRequest, etc.
├── actions/
│   ├── mod.rs        # Web3 Actions module exports
│   ├── api.rs        # Web3 Actions API client
│   └── types.rs      # ActionTrigger, CreateActionRequest, etc.
└── wallets/
    ├── mod.rs        # Wallets module exports
    ├── api.rs        # Wallets API client
    └── types.rs      # Wallet, AddWalletRequest, etc.

tests/
└── admin_rpc_integration.rs  # Admin RPC integration tests (requires credentials)

examples/
├── test_admin_rpc.rs        # Comprehensive Admin RPC test
├── test_all_endpoints.rs    # Test all API endpoints (read operations)
├── test_write_operations.rs # Test write operations (creates/updates/deletes)
├── test_vnet_transactions.rs # VNet transaction listing tests
├── integration_test.rs      # Full API integration test
└── ...
```

## Key Dependencies

- **reqwest**: HTTP client with rustls-tls
- **tokio**: Async runtime (rt-multi-thread, macros)
- **serde/serde_json**: Serialization
- **secrecy**: Secret protection for API keys
- **thiserror**: Error handling

## Environment Variables

- `TENDERLY_ACCESS_KEY` - API access key
- `TENDERLY_ACCOUNT` - Account slug
- `TENDERLY_PROJECT` - Project slug

## Git Hooks

Install pre-commit hooks to run fmt, clippy, and tests before each commit:

```bash
./.githooks/install.sh
```

## Code Style

- Use `#[must_use]` on all builder methods
- Use `#[non_exhaustive]` on all public enums
- Implement `FromStr`/`Display` for enums
- Use `SecretString` for sensitive values
- Validate addresses with `utils::is_valid_address()`

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests (requires TENDERLY_* env vars)
cargo test --test admin_rpc_integration -- --ignored

# Run specific integration test
cargo test --test admin_rpc_integration test_get_latest_returns_block_object -- --ignored

# Run examples (requires credentials)
cargo run --example test_connection
cargo run --example test_admin_rpc
```

## Usage

```rust
use tndrly::Client;
use tndrly::simulation::SimulationRequest;

// From environment variables
let client = Client::from_env()?;

// Or construct directly
let client = Client::new(Config::new("access_key", "account", "project"))?;

// Use APIs
let result = client.simulation().simulate(&request).await?;
```

## Admin RPC (Virtual TestNets)

The Admin RPC client provides JSON-RPC methods for manipulating VNet state:

```rust
use tndrly::vnets::{SendTransactionParams, LatestBlock};

// Get admin RPC client for a VNet
let admin = client.vnets().admin_rpc("vnet-id").await?;

// Time manipulation (all return tx hash)
admin.increase_time(3600).await?;                      // Advance 1 hour
let hash = admin.set_next_block_timestamp(1234567890).await?;  // Returns tx hash
let hash = admin.set_next_block_timestamp_no_mine(ts).await?;  // Returns tx hash
admin.increase_blocks(100).await?;

// Balance management (accepts decimal or hex strings)
admin.set_balance("0x...", "1000000000000000000").await?;  // 1 ETH (decimal)
admin.set_balance("0x...", "0xde0b6b3a7640000").await?;    // 1 ETH (hex)
admin.add_balance("0x...", "1000000000000000000").await?;
admin.set_erc20_balance("0xtoken", "0xwallet", "1000000").await?;

// Storage manipulation (slot/value auto-padded to 32 bytes)
admin.set_storage_at("0x...", "0", "1").await?;       // Unpadded OK
admin.set_storage_at("0x...", "0x5", "0x64").await?;  // Hex OK
admin.set_code("0x...", "0x6080...").await?;

// Snapshots
let snapshot_id = admin.snapshot().await?;
// ... do stuff ...
admin.revert(&snapshot_id).await?;

// Transaction info
let latest: LatestBlock = admin.get_latest().await?;  // Returns block object
println!("Block: {:?}", latest.block_number);

// Send transactions (value auto-converts decimal to hex)
let tx = SendTransactionParams::new("0xfrom")
    .to("0xto")
    .value("1000000000000000000")  // Decimal auto-converted
    .gas("0x5208");
let hash = admin.send_transaction(&tx).await?;
```

### Admin RPC Types

- `LatestBlock` - Block info from `get_latest()` with `block_number`, `block_hash`, `transaction_hash`
- `SendTransactionParams` - Builder for transaction parameters
- `AccessListResult` - Result from `create_access_list()`
