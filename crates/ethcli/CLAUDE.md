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
ethcli price      # Token prices from CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, CCXT
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
ethcli chainlink  # Chainlink price feeds (RPC-based, no API key needed)
ethcli ccxt       # Exchange data (Binance, Bitget, OKX, Hyperliquid)
ethcli kong       # Yearn Kong API (vaults, strategies, prices, TVL, reports)
ethcli uniswap    # Uniswap V2/V3/V4 (on-chain lens + subgraph)
ethcli goplus     # GoPlus Security API (token/address/NFT/approval security)
```

### Security & Token Analysis
```
ethcli blacklist  # Token blacklist management (spam/scam filtering)
```

## GoPlus Security Commands

Query the GoPlus Security API for token, address, NFT, and approval security analysis.

```bash
# Check token security (honeypot detection, taxes, ownership)
ethcli goplus token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain-id 1

# Check if address is malicious
ethcli goplus address 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --chain-id 1

# Check NFT collection security
ethcli goplus nft 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d --chain-id 1

# Check ERC20/721/1155 approval security
ethcli goplus approval 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain-id 1

# List supported chains
ethcli goplus chains

# JSON output
ethcli goplus token 0x... --chain-id 1 --json
```

### Notes

- **Free API**: No API key required for basic usage
- **Authenticated mode**: Set `GOPLUS_APP_KEY` and `GOPLUS_APP_SECRET` for batch queries and higher rate limits
- **Alias**: `ethcli gp` works as an alias for `ethcli goplus`
- **Chain IDs**: 1=Ethereum, 56=BSC, 137=Polygon, 42161=Arbitrum, 8453=Base, etc.

## Blacklist Commands

Manage a local token blacklist for filtering spam/scam tokens from portfolio views.

```bash
# Scan a token for security issues
ethcli blacklist scan 0x... --chain ethereum

# Scan entire portfolio for suspicious tokens
ethcli blacklist scan-portfolio 0xYourAddress --chain ethereum --auto-blacklist

# Only show suspicious tokens (skip safe ones)
ethcli blacklist scan-portfolio 0xYourAddress --suspicious-only --auto-blacklist

# List blacklisted tokens
ethcli blacklist list
ethcli blacklist list --links  # With Etherscan links

# Add token to blacklist
ethcli blacklist add 0x... --chain ethereum --reason "Honeypot token"

# Remove token from blacklist
ethcli blacklist remove 0x...

# Check if token is blacklisted
ethcli blacklist check 0x...

# Clear all blacklisted tokens
ethcli blacklist clear
```

### Notes

- **Storage**: Blacklist stored in `~/.config/ethcli/blacklist.toml`
- **Security checks**: Uses GoPlus API + Etherscan verification status
- **Known protocols**: Yearn, Curve, Aave, Compound, etc. are auto-whitelisted
- **Alias**: `ethcli bl` works as an alias for `ethcli blacklist`

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

## Chainlink Commands

Query Chainlink price feeds via RPC (no API key required). Supports Feed Registry (mainnet) and direct oracle queries (all chains).

```bash
# Get current price (uses Feed Registry on mainnet, direct oracles on L2s)
ethcli chainlink price ETH
ethcli chainlink price CVX
ethcli chainlink price BTC --chain arbitrum

# Historical price at a specific block (requires archive node)
ethcli chainlink price ETH --block 18000000
ethcli chainlink price CVX --block 19500000

# Query a specific oracle address directly
ethcli chainlink price ETH --oracle 0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419

# Get feed/oracle address for a token
ethcli chainlink feed CVX
ethcli chainlink feed ETH --quote usd

# List known oracle addresses
ethcli chainlink oracles
ethcli chainlink oracles --chain ethereum
ethcli chainlink oracles --chain arbitrum

# Data Streams (requires API credentials)
ethcli chainlink streams feeds
ethcli chainlink streams latest <feed_id>
ethcli chainlink streams report <feed_id> <timestamp>
ethcli chainlink streams bulk <feed_id1>,<feed_id2> <timestamp>
ethcli chainlink streams history <feed_id> <timestamp> --limit 10
```

### Notes

- **Ethereum mainnet**: Uses Feed Registry - supports any token Chainlink has a feed for. Pass token address or symbol.
- **L2 chains**: Uses hardcoded oracle mappings. Run `ethcli chainlink oracles --chain <chain>` to see available feeds.
- **Historical queries**: Require an archive node. Feed address is resolved at the target block for accuracy.
- **Stale detection**: Warns if `answeredInRound < roundId` (oracle hasn't updated recently).

## Kong (Yearn) Commands

Query Yearn Finance vault and strategy data via the Kong GraphQL API. No API key required.

```bash
# List vaults (optionally filtered)
ethcli kong vaults list                        # All vaults
ethcli kong vaults list --chain-id 1           # Ethereum mainnet vaults
ethcli kong vaults list --chain-id 1 --yearn   # Official Yearn vaults only
ethcli kong vaults list --v3                   # V3 vaults only
ethcli kong vaults list --erc4626              # ERC4626 compliant vaults

# Get specific vault details
ethcli kong vaults get --chain-id 1 0x7B5A0182E400b241b317e781a4e9dEdFc1429822

# Get user positions in vaults
ethcli kong vaults accounts --chain-id 1 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# List strategies
ethcli kong strategies list --chain-id 1
ethcli kong strategies list --vault 0x...      # Strategies for specific vault
ethcli kong strategies get --chain-id 1 0x...  # Get strategy details

# Token prices (contract addresses only)
ethcli kong prices current --chain-id 1 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli kong prices historical --chain-id 1 0x... 1700000000  # At timestamp

# TVL data
ethcli kong tvl current --chain-id 1 0x...
ethcli kong tvl history --chain-id 1 0x... --period day --limit 30

# Vault/strategy reports (harvests)
ethcli kong reports vault --chain-id 1 0x...
ethcli kong reports strategy --chain-id 1 0x...
```

### Notes

- **Chain IDs**: 1=Ethereum, 137=Polygon, 42161=Arbitrum, 10=Optimism, 8453=Base
- **Alias**: `ethcli yearn` works as an alias for `ethcli kong`
- **No API key**: Kong API is free and public

## Uniswap Commands

Query Uniswap V2, V3, and V4 pools via on-chain lens queries and The Graph subgraph.

```bash
# On-chain queries (no API key needed)
ethcli uniswap pool 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640        # Get V3 pool state
ethcli uniswap liquidity 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640  # Get pool liquidity
ethcli uniswap balance <token> <account>                              # Get token balance

# Subgraph queries (requires THEGRAPH_API_KEY)
ethcli uniswap eth-price                            # Current ETH price
ethcli uniswap eth-price --version v2               # From V2 subgraph
ethcli uniswap top-pools 10                         # Top 10 pools by TVL
ethcli uniswap top-pools 20 --version v4            # Top V4 pools
ethcli uniswap swaps 0x... --limit 20               # Recent swaps for a pool
ethcli uniswap day-data 0x... --days 7              # Daily data for a pool

# LP positions (subgraph, queries all versions by default)
ethcli uniswap positions 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
ethcli uniswap positions <address> --version v3 --chain arbitrum
ethcli uniswap positions <address> --json           # JSON output

# List well-known addresses
ethcli uniswap addresses                            # All factories, pools, tokens
ethcli uniswap addresses --factories                # Only factories
ethcli uniswap addresses --pools --version v3       # Only V3 pools
```

### Notes

- **Alias**: `ethcli uni` works as an alias for `ethcli uniswap`
- **On-chain queries**: Use `pool`, `liquidity`, `balance` - no API key needed
- **Subgraph queries**: Use `eth-price`, `top-pools`, `swaps`, `day-data`, `positions` - requires `THEGRAPH_API_KEY`
- **Multi-chain**: Supports Ethereum, Arbitrum, Optimism, Polygon, Base
- **Multi-version**: Supports V2, V3, and V4 protocols

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
├── chainlink/        # Chainlink price feed queries
│   ├── mod.rs        # Public exports, fetch_price(), fetch_price_at_block()
│   ├── types.rs      # PriceData, ChainlinkError
│   ├── constants.rs  # Feed Registry, denominations, oracle addresses
│   ├── registry.rs   # Feed Registry queries (mainnet only)
│   └── aggregator.rs # Direct AggregatorV3 queries (all chains)
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
    ├── chainlink.rs  # Chainlink price feeds (RPC + Data Streams)
    ├── ccxt.rs       # Exchange data via CCXT
    ├── kong.rs       # Direct Yearn Kong API
    ├── uniswap.rs    # Uniswap V2/V3/V4 queries
    ├── goplus.rs     # GoPlus Security API (token/address/NFT/approval)
    └── blacklist.rs  # Token blacklist management
```

## Key Dependencies

- **alloy 1.0**: Ethereum provider, types, ABI decoding
- **foundry-block-explorers**: Etherscan API client
- **chainlink-data-streams-sdk**: Chainlink Data Streams API (optional, requires API key)
- **pyth**: Pyth Network Hermes API client (no API key needed)
- **tndrly**: Tenderly API client
- **alcmy**: Alchemy API client
- **gecko**: CoinGecko API client
- **llama**: DefiLlama API client
- **mrls**: Moralis API client
- **dsim**: Dune SIM API client
- **dune**: Dune Analytics API client
- **crv**: Curve Finance API client
- **ykong**: Yearn Kong GraphQL API client
- **unswp**: Uniswap V2/V3/V4 client (on-chain lens + subgraph)
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

# Chainlink price feeds (RPC-based, no API key)
ethcli chainlink price ETH
ethcli chainlink price CVX --chain ethereum
ethcli chainlink oracles --chain arbitrum
```

## Environment Variables

**Note:** Some services have similar-named keys that serve different purposes:
- `DUNE_API_KEY` (Dune Analytics queries) vs `DUNE_SIM_API_KEY` (Dune SIM wallet simulation)
- `CHAINLINK_API_KEY` + `CHAINLINK_USER_SECRET` are only for Data Streams (premium), not needed for RPC-based price feeds

| Variable | Required For | Description |
|----------|-------------|-------------|
| `ETHERSCAN_API_KEY` | Optional | Increases Etherscan rate limit |
| `TENDERLY_ACCESS_KEY` | `ethcli tenderly` | Tenderly API access |
| `ALCHEMY_API_KEY` | `ethcli alchemy`, `--via alcmy` | Alchemy API access |
| `COINGECKO_API_KEY` | Optional | CoinGecko Pro API (increases rate limit) |
| `DEFILLAMA_API_KEY` | Optional | DefiLlama Pro endpoints |
| `MORALIS_API_KEY` | `ethcli moralis` | Moralis API access |
| `DUNE_SIM_API_KEY` | `ethcli dsim` | Dune SIM wallet simulation |
| `DUNE_API_KEY` | `ethcli dune` | Dune Analytics queries |
| `CHAINLINK_API_KEY` | `chainlink streams` only | Data Streams API key |
| `CHAINLINK_USER_SECRET` | `chainlink streams` only | Data Streams secret |
| `THEGRAPH_API_KEY` | Uniswap subgraph | The Graph API access |
| `GOPLUS_APP_KEY` | Optional | GoPlus batch queries (>1 token) |
| `GOPLUS_APP_SECRET` | Optional | GoPlus batch queries (>1 token) |

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
