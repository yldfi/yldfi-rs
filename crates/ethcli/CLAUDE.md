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

### Core Ethereum Commands
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
ethcli cast       # Type conversions, hashing, encoding
ethcli rpc        # Direct RPC calls
ethcli ens        # ENS name resolution
ethcli endpoints  # Manage RPC endpoints
ethcli config     # Manage configuration
ethcli update     # Check for updates and self-update
ethcli doctor     # Diagnose configuration and connectivity
```

### Aggregation Commands (parallel queries to multiple APIs)
```
ethcli price      # Token prices from CoinGecko, DefiLlama, Alchemy, Moralis, Curve
ethcli portfolio  # Portfolio balances from Alchemy, Dune SIM, Moralis
ethcli nfts       # NFT holdings from Alchemy, CoinGecko, Moralis, Dune SIM
ethcli yields     # DeFi yields from DefiLlama and Curve
```

### Direct API Access Commands
```
ethcli tenderly   # Tenderly API (vnets, wallets, contracts, alerts, actions)
ethcli alchemy    # Alchemy API (NFTs, prices, portfolio, transfers, debug)
ethcli gecko      # CoinGecko API (coins, prices, NFTs, exchanges)
ethcli llama      # DefiLlama API (TVL, prices, yields, stablecoins)
ethcli moralis    # Moralis API (wallet, token, NFT, DeFi, transactions)
ethcli dsim       # Dune SIM API (balances, activity, collectibles, DeFi)
ethcli dune       # Dune Analytics API (queries, executions, tables)
ethcli curve      # Curve Finance API (pools, volumes, lending, tokens, router)
ethcli chainlink  # Chainlink Data Streams (real-time market data)
ethcli ccxt       # Exchange data (Binance, Bitget, OKX, Hyperliquid)
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

## Curve Router Commands

The Curve router finds optimal swap routes across Curve pools (local implementation, not REST API).

```bash
# Find swap routes between two tokens
ethcli curve router route <from_token> <to_token> --chain ethereum --limit 5

# Example: DAI to USDC
ethcli curve router route 0x6B175474E89094C44Da98b954EedeAC495271d0F 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Get calldata for a swap (to send to router contract)
ethcli curve router encode <from> <to> <amount> <min_out> --chain ethereum

# Show router graph statistics
ethcli curve router stats --chain ethereum

# Get router contract address
ethcli curve router address ethereum
ethcli curve router address polygon
ethcli curve router address arbitrum
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
├── aggregator/       # Multi-source data aggregation
│   ├── mod.rs        # Core types (SourceResult, AggregatedResult)
│   ├── normalize.rs  # Data normalization across APIs
│   ├── chain_map.rs  # Chain name normalization
│   ├── price.rs      # Price fetching and aggregation
│   ├── portfolio.rs  # Portfolio aggregation
│   ├── nft.rs        # NFT aggregation
│   └── yields.rs     # DeFi yield aggregation
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
    ├── simulate/     # Transaction simulation (multiple backends)
    ├── tenderly.rs   # Tenderly API commands
    ├── token.rs      # Token commands
    ├── update.rs     # Self-update from GitHub
    ├── price.rs      # Aggregated price command
    ├── portfolio.rs  # Aggregated portfolio command
    ├── nfts.rs       # Aggregated NFT command
    ├── yields.rs     # Aggregated yields command
    ├── alchemy.rs    # Direct Alchemy API
    ├── gecko.rs      # Direct CoinGecko API
    ├── llama.rs      # Direct DefiLlama API
    ├── moralis.rs    # Direct Moralis API
    ├── dsim.rs       # Direct Dune SIM API
    ├── dune_cli.rs   # Direct Dune Analytics API
    ├── curve.rs      # Direct Curve Finance API
    ├── chainlink.rs  # Chainlink Data Streams
    └── ccxt.rs       # Exchange data via CCXT
```

## Key Dependencies

- **alloy 1.0**: Ethereum provider, types, ABI decoding
- **foundry-block-explorers**: Etherscan API client
- **tndrly**: Tenderly API client
- **alcmy**: Alchemy API client
- **gecko**: CoinGecko API client
- **llama**: DefiLlama API client
- **mrls**: Moralis API client
- **dsim**: Dune SIM API client
- **dune**: Dune Analytics API client
- **crv**: Curve Finance API client
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

# Aggregated price from all sources
ethcli price ETH
ethcli price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain ethereum

# Aggregated portfolio
ethcli portfolio 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Aggregated NFTs
ethcli nfts 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# DeFi yields
ethcli yields --protocol aave
```

## Environment Variables

- `ETHERSCAN_API_KEY`: Etherscan API key (optional, increases rate limit)
- `TENDERLY_ACCESS_KEY`: Tenderly API access key (required for `ethcli tenderly` commands)
- `ALCHEMY_API_KEY`: Alchemy API key (required for `ethcli alchemy` and `--via alcmy` simulation)
- `COINGECKO_API_KEY`: CoinGecko API key (optional, increases rate limit)
- `DEFILLAMA_API_KEY`: DefiLlama Pro API key (optional, for Pro endpoints)
- `MORALIS_API_KEY`: Moralis API key (required for `ethcli moralis` commands)
- `DUNE_SIM_API_KEY`: Dune SIM API key (required for `ethcli dsim` commands)
- `DUNE_API_KEY`: Dune Analytics API key (required for `ethcli dune` commands)
- `CHAINLINK_CLIENT_ID`: Chainlink Data Streams client ID
- `CHAINLINK_CLIENT_SECRET`: Chainlink Data Streams client secret

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
