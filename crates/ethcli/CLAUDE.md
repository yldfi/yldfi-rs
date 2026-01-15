# ethcli

Comprehensive Ethereum CLI for logs, transactions, accounts, and contracts.

## Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Binary Location

- Debug: `./target/debug/ethcli`
- Release: `./target/release/ethcli`
- Legacy alias: `./target/release/eth-log-fetch`

## Available Commands

```
ethcli logs       # Fetch historical logs from contracts
ethcli tx         # Analyze transaction(s)
ethcli account    # Account operations (balance, transactions, transfers)
ethcli address    # Address book (save and lookup addresses by label)
ethcli contract   # Contract operations (ABI, source, creation)
ethcli token      # Token operations (info, holders, balance)
ethcli gas        # Gas price oracle and estimates
ethcli sig        # Signature lookup (function selectors, event topics)
ethcli simulate   # Transaction simulation and tracing
ethcli tenderly   # Tenderly API (vnets, wallets, contracts, alerts, actions)
ethcli cast       # Type conversions, hashing, encoding
ethcli rpc        # Direct RPC calls
ethcli ens        # ENS name resolution
ethcli endpoints  # Manage RPC endpoints
ethcli config     # Manage configuration
ethcli update     # Check for updates and self-update
ethcli doctor     # Diagnose configuration and connectivity
```

## Simulation Commands

```bash
# Simulate a contract call (uses cast by default)
ethcli simulate call <contract> --sig "balanceOf(address)" <address> --rpc-url https://eth.llamarpc.com

# Simulate with trace (requires debug-capable node)
ethcli simulate call <contract> --sig "transfer(address,uint256)" <to> <amount> --trace

# Trace an existing transaction
ethcli simulate tx <tx_hash> --rpc-url https://eth.llamarpc.com

# Use different backends
ethcli simulate call ... --via cast      # Default: uses cast call
ethcli simulate call ... --via anvil     # Forks mainnet with Anvil
ethcli simulate call ... --via tenderly  # Uses Tenderly API (rich output)
ethcli simulate call ... --via debug     # Uses debug_traceCall RPC
ethcli simulate call ... --via trace     # Uses trace_call RPC (Erigon/OpenEthereum)
```

## Tenderly Commands

Requires `TENDERLY_ACCESS_KEY` environment variable. Most commands also need `--project` and `--account` flags.

```bash
# Virtual TestNets (VNets)
ethcli tenderly vnets list --project <slug> --account <slug>
ethcli tenderly vnets create --slug <slug> --name "My VNet" --network-id 1 --project <slug> --account <slug>
ethcli tenderly vnets get <vnet-id> --project <slug> --account <slug>
ethcli tenderly vnets delete <vnet-id> --project <slug> --account <slug>
ethcli tenderly vnets delete <id1> <id2> <id3> --project <slug> --account <slug>
ethcli tenderly vnets delete --all --project <slug> --account <slug>
ethcli tenderly vnets rpc <vnet-id> --project <slug> --account <slug>

# VNet Admin RPC - Balance Management
ethcli tenderly vnets admin --vnet <id> set-balance <address> 10eth --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> add-balance <address> 1eth --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> set-erc20-balance --token <token> --wallet <wallet> <amount> --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> set-max-erc20-balance --token <token> --wallet <wallet> --project <slug> --account <slug>

# VNet Admin RPC - Time Manipulation
ethcli tenderly vnets admin --vnet <id> increase-time 3600 --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> set-timestamp <epoch> --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> increase-blocks 10 --project <slug> --account <slug>

# VNet Admin RPC - State Management
ethcli tenderly vnets admin --vnet <id> snapshot --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> revert <snapshot-id> --project <slug> --account <slug>

# VNet Admin RPC - Storage/Code (slot/value accept decimal or hex, auto-padded to 32 bytes)
ethcli tenderly vnets admin --vnet <id> set-storage --address <addr> --slot 0 --value 1 --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> set-code --address <addr> --code <bytecode> --project <slug> --account <slug>

# VNet Admin RPC - Transactions
ethcli tenderly vnets admin --vnet <id> send-tx --from <addr> --to <addr> --value 0x1 --project <slug> --account <slug>
ethcli tenderly vnets admin --vnet <id> get-latest --project <slug> --account <slug>

# Virtual Wallets
ethcli tenderly wallets list --project <slug> --account <slug>
ethcli tenderly wallets add <address> --project <slug> --account <slug>
ethcli tenderly wallets get <address> --network 1 --project <slug> --account <slug>

# Contracts
ethcli tenderly contracts list --project <slug> --account <slug>
ethcli tenderly contracts get <address> --network 1 --project <slug> --account <slug>
ethcli tenderly contracts add <address> --network 1 --project <slug> --account <slug>
ethcli tenderly contracts verify <address> --network 1 --name <name> --source <file> --compiler <ver> --project <slug> --account <slug>

# Alerts
ethcli tenderly alerts list --project <slug> --account <slug>
ethcli tenderly alerts create --name "Alert" --alert-type successful_transaction --network 1 --project <slug> --account <slug>
ethcli tenderly alerts delete <alert-id> --project <slug> --account <slug>
ethcli tenderly alerts webhooks list --project <slug> --account <slug>
ethcli tenderly alerts webhooks create --name "Hook" --url https://... --project <slug> --account <slug>

# Web3 Actions
ethcli tenderly actions list --project <slug> --account <slug>
ethcli tenderly actions get <action-id> --project <slug> --account <slug>
ethcli tenderly actions invoke <action-id> --project <slug> --account <slug>
ethcli tenderly actions logs <action-id> --project <slug> --account <slug>

# Networks
ethcli tenderly networks list
ethcli tenderly networks get <network-id>

# Delivery Channels (Slack, Discord, Email, etc.)
ethcli tenderly channels list --project <slug> --account <slug>
ethcli tenderly channels account --project <slug> --account <slug>
ethcli tenderly channels project --project <slug> --account <slug>

# Simulation (alias to ethcli simulate)
ethcli tenderly simulate call <contract> --sig "balanceOf(address)" <args>
```

## Project Structure

```
src/
├── main.rs           # CLI entry point (clap)
├── lib.rs            # Library exports
├── error.rs          # Error types (thiserror)
├── fetcher.rs        # Main LogFetcher coordinator
├── checkpoint.rs     # Resume/checkpoint system
├── proxy.rs          # Proxy rotation support
├── tx/
│   ├── mod.rs        # Transaction analysis module
│   ├── addresses.rs  # Address extraction from traces
│   ├── analyzer.rs   # Transaction analyzer
│   ├── flow.rs       # Token flow analysis
│   └── types.rs      # Transaction types
├── config/
│   ├── mod.rs        # Config structs, builder pattern
│   ├── addressbook.rs # Address book storage
│   ├── chain.rs      # Chain enum (Ethereum, Polygon, etc.)
│   ├── endpoint.rs   # EndpointConfig
│   └── file.rs       # TOML config file handling
├── rpc/
│   ├── mod.rs
│   ├── endpoint.rs   # Single RPC endpoint wrapper (alloy)
│   ├── pool.rs       # RPC pool with parallel requests
│   ├── health.rs     # Endpoint health tracking
│   ├── multicall.rs  # Multicall batching
│   ├── optimizer.rs  # Request optimization
│   ├── retry.rs      # Retry logic
│   └── selector.rs   # Endpoint selection
├── abi/
│   ├── mod.rs
│   ├── parser.rs     # Event signature parser
│   ├── fetcher.rs    # Etherscan ABI fetcher (v2 API)
│   └── decoder.rs    # Log decoder (alloy dyn-abi)
├── output/
│   ├── mod.rs        # OutputWriter trait
│   ├── json.rs       # JSON/NDJSON output
│   ├── csv.rs        # CSV output
│   └── sqlite.rs     # SQLite output
├── etherscan/
│   ├── mod.rs        # Etherscan client wrapper
│   ├── client.rs     # Extended Etherscan API client
│   └── cache.rs      # Signature caching
├── utils/
│   ├── mod.rs        # Shared utility functions
│   ├── format.rs     # Number/token formatting
│   └── address.rs    # Address resolution utilities
└── cli/
    ├── mod.rs        # CLI command structure
    ├── logs.rs       # Log fetching arguments
    ├── tx.rs         # Transaction analysis args
    ├── account.rs    # Account commands
    ├── address.rs    # Address book commands
    ├── cast.rs       # Type conversions, hashing, encoding
    ├── config.rs     # Config management
    ├── contract.rs   # Contract commands
    ├── doctor.rs     # Diagnostics and health checks
    ├── endpoints.rs  # Endpoint management
    ├── ens.rs        # ENS name resolution
    ├── gas.rs        # Gas oracle commands
    ├── rpc.rs        # Direct RPC calls
    ├── sig.rs        # Signature lookup commands
    ├── simulate.rs   # Transaction simulation
    ├── tenderly.rs   # Tenderly API commands
    ├── token.rs      # Token commands
    └── update.rs     # Self-update from GitHub
```

## Key Dependencies

- **alloy 1.0**: Ethereum provider, types, ABI decoding
- **foundry-block-explorers**: Etherscan API client
- **tndrly**: Tenderly API client
- **tokio**: Async runtime
- **clap**: CLI parsing
- **serde/serde_json**: Serialization
- **rusqlite**: SQLite output
- **indicatif**: Progress bars

## Testing Locally

```bash
# Fetch USDC Transfer events (small range)
ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  -e "Transfer(address,address,uint256)" \
  -f 21500000 -t 21500010

# Analyze a transaction
ethcli tx 0x123abc...

# Get account balance
ethcli account balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Get recent transactions for an address
ethcli account txs 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Get contract ABI
ethcli contract abi 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Get verified source code
ethcli contract source 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Lookup function selector
ethcli sig fn 0xa9059cbb

# Get gas prices
ethcli gas oracle

# Get token info
ethcli token info 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# List available RPC endpoints
ethcli endpoints list

# Test a specific RPC endpoint
ethcli endpoints test https://eth.llamarpc.com
```

## Environment Variables

- `ETHERSCAN_API_KEY`: Etherscan API key (optional, increases rate limit)
- `TENDERLY_ACCESS_KEY`: Tenderly API access key (required for `ethcli tenderly` commands)

## Release Process

**Automated with release-please:**
- Use [Conventional Commits](https://www.conventionalcommits.org/) in commit messages
- `feat:` - triggers minor version bump
- `fix:` - triggers patch version bump
- `feat!:` or `BREAKING CHANGE:` - triggers major version bump
- Release-please creates a PR with version bump and CHANGELOG
- Merging the PR triggers binary builds for all platforms

**Manual release (alternative):**
```bash
git tag v0.x.x
git push origin v0.x.x
```

## Pre-commit Hooks

Installed hooks run automatically on commit:
- `cargo fmt --check` - formatting check
- `cargo clippy -- -D warnings` - lint check

## Architecture Notes

- Uses user-configured RPC endpoints (add with `ethcli endpoints add <url>`)
- Parallel requests with automatic failover on errors
- Health tracking disables failing endpoints temporarily
- Checkpoint system allows resuming interrupted fetches
- Etherscan API v2 for ABI fetching (works without API key, rate limited)
- foundry-block-explorers for account, contract, token, and gas commands
