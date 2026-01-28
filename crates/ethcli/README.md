<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">ethcli</h1>

<p align="center">
  Comprehensive Ethereum CLI for logs, transactions, accounts, and contracts
</p>

<p align="center">
  <a href="https://crates.io/crates/ethcli"><img src="https://img.shields.io/crates/v/ethcli.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/ethcli/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Log Fetching**: Parallel RPC requests across multiple endpoints
- **Transaction Analysis**: Decode transactions with signature lookup
- **Account Operations**: Balance, transactions, token transfers
- **Contract Tools**: ABI fetching, source code, verification status
- **Type Conversions**: Wei/Gwei/Eth, hex/dec, checksums, hashing
- **RPC Commands**: Direct blockchain calls (call, block, storage, etc.)
- **ENS Resolution**: Resolve names to addresses and reverse lookup
- **Gas Oracle**: Real-time gas prices from Etherscan
- **Simulation**: Transaction simulation via cast, Tenderly, or debug RPC
- **Tenderly Integration**: Virtual testnets, contracts, alerts, and actions
- **Address Book**: Save and lookup addresses by label
- **Self-Updating**: Check for updates and auto-install
- **Multi-chain**: Ethereum, Polygon, Arbitrum, Optimism, Base, BSC, Avalanche

### Aggregation Commands
- **Price Aggregation**: Multi-source prices from CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, CCXT
- **Portfolio Aggregation**: Balance data from Alchemy, Dune SIM, Moralis
- **NFT Aggregation**: Holdings from Alchemy, CoinGecko, Moralis, Dune SIM
- **Yield Aggregation**: DeFi yields from DefiLlama and Curve
- **Quote Aggregation**: Swap quotes from OpenOcean, KyberSwap, 0x, 1inch, CowSwap, LI.FI, Velora, Enso

### Direct API Access
- **Alchemy**: NFTs, prices, portfolio, transfers, debug traces
- **CoinGecko**: Coins, prices, NFTs, exchanges
- **DefiLlama**: TVL, prices, yields, stablecoins
- **Moralis**: Wallet, token, NFT, DeFi, transactions
- **Dune SIM**: Balances, activity, collectibles, DeFi positions
- **Dune Analytics**: Queries, executions, tables
- **Curve Finance**: Pools, volumes, lending, tokens, router
- **Chainlink**: Price feeds (RPC-based, no API key needed)
- **CCXT**: Exchange data (Binance, Bitget, OKX, Hyperliquid)
- **Uniswap**: V2/V3/V4 pool queries (on-chain + subgraph)
- **Yearn Kong**: Vaults, strategies, prices, TVL, reports

### Security & Analysis
- **GoPlus Security**: Token, address, NFT, and approval security analysis
- **Solodit**: Smart contract vulnerability database search
- **Blacklist**: Token blacklist management for spam/scam filtering

## Installation

### Download Binary

```bash
# macOS (Apple Silicon)
curl -sL https://github.com/yldfi/yldfi-rs/releases/latest/download/ethcli-macos-aarch64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/

# macOS (Intel)
curl -sL https://github.com/yldfi/yldfi-rs/releases/latest/download/ethcli-macos-x86_64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/

# Linux (x86_64)
curl -sL https://github.com/yldfi/yldfi-rs/releases/latest/download/ethcli-linux-x86_64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/
```

### Install with Cargo

```bash
cargo install ethcli
```

Or from source:
```bash
cargo install --git https://github.com/yldfi/yldfi-rs.git ethcli
```

## Quick Start

```bash
# Fetch Transfer events from USDC
ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  -e "Transfer(address,address,uint256)" \
  -f 21000000 -t 21000100

# Analyze a transaction
ethcli tx 0x123...

# Get account balance
ethcli account balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Resolve ENS name
ethcli ens resolve vitalik.eth

# Get current gas prices
ethcli gas oracle

# Get swap quotes from all DEX aggregators
ethcli quote compare ETH USDC 1000000000000000000
```

## Commands

### Logs - Fetch Historical Events

```bash
# Fetch specific events
ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  -e "Transfer(address,address,uint256)" \
  -f 18000000 -t 18100000 -O transfers.json

# Fetch all events (auto-fetches ABI from Etherscan), output as CSV
ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  -f 18000000 -t latest -o csv -O events.csv

# Output to SQLite
ethcli logs -c 0x... -f 18000000 -t 18100000 -o sqlite -O events.db

# High concurrency with resume
ethcli logs -c 0x... -f 0 -t latest -n 20 --resume
```

### Transaction - Analyze Transactions

```bash
# Analyze a transaction
ethcli tx 0x1234567890abcdef...

# Show decoded input data
ethcli tx 0x... --decode

# Output as JSON
ethcli tx 0x... --json
```

### Account - Balance and History

```bash
# Get ETH balance
ethcli account balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Get token balance
ethcli account balance 0x... --token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# List recent transactions
ethcli account txlist 0x...

# List token transfers
ethcli account tokentx 0x...
```

### Contract - ABI and Source Code

```bash
# Get contract ABI
ethcli contract abi 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Get source code
ethcli contract source 0x...

# Get creation transaction
ethcli contract creation 0x...
```

### Cast - Type Conversions and Hashing

```bash
# Unit conversions
ethcli cast to-wei 1.5 eth      # 1500000000000000000
ethcli cast from-wei 1000000000 gwei  # 1.0

# Hex/decimal
ethcli cast to-hex 255          # 0xff
ethcli cast to-dec 0xff         # 255

# Hashing
ethcli cast keccak "hello"      # 0x1c8aff950...
ethcli cast sig "transfer(address,uint256)"  # 0xa9059cbb
ethcli cast topic "Transfer(address,address,uint256)"

# Address tools
ethcli cast checksum 0xd8da6bf26964af9d7eed9e03e53415d37aa96045
ethcli cast compute-address 0x... 5  # CREATE address

# ABI encode/decode
ethcli cast abi-encode "transfer(address,uint256)" 0x123... 1000
ethcli cast abi-decode "(address,uint256)" 0x...
```

### RPC - Direct Blockchain Calls

```bash
# Call a contract (read-only)
ethcli rpc call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 0x18160ddd --decode uint256

# Get block info
ethcli rpc block latest
ethcli rpc block 21000000 --json

# Read storage slot
ethcli rpc storage 0x... 0

# Get contract code
ethcli rpc code 0x...

# Get nonce
ethcli rpc nonce 0x...

# Get transaction receipt
ethcli rpc receipt 0x...

# Chain info
ethcli rpc chain-id
ethcli rpc block-number
ethcli rpc gas-price
```

### ENS - Name Resolution

```bash
# Resolve name to address
ethcli ens resolve vitalik.eth

# Reverse lookup (address to name)
ethcli ens lookup 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Get resolver
ethcli ens resolver vitalik.eth

# Compute namehash
ethcli ens namehash vitalik.eth
```

### Gas - Gas Oracle

```bash
# Get current gas prices
ethcli gas oracle

# Estimate confirmation time
ethcli gas estimate 30
```

### Token - Token Operations

```bash
# Get token info
ethcli token info 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Get top holders
ethcli token holders 0x...
```

### Signature - Lookup Function/Event Signatures

```bash
# Lookup function by selector
ethcli sig function 0xa9059cbb

# Lookup event by topic
ethcli sig event 0xddf252ad...
```

### Address Book

```bash
# Save an address with a label
ethcli address add vitalik 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Lookup by label
ethcli address get vitalik

# List all saved addresses
ethcli address list

# Remove an address
ethcli address remove vitalik
```

### Simulate - Transaction Simulation

```bash
# Simulate a contract call
ethcli simulate call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --sig "balanceOf(address)" 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Simulate with trace output
ethcli simulate call 0x... --sig "transfer(address,uint256)" 0x... 1000 --trace

# Trace an existing transaction
ethcli simulate tx 0x1234...

# Use different backends
ethcli simulate call ... --via cast      # Default
ethcli simulate call ... --via tenderly  # Tenderly API
ethcli simulate call ... --via debug     # debug_traceCall RPC
```

### Tenderly - Virtual TestNets & API

Requires `TENDERLY_ACCESS_KEY` environment variable.

```bash
# List virtual testnets
ethcli tenderly vnets list --project <slug> --account <slug>

# Create a vnet
ethcli tenderly vnets create --slug my-vnet --name "My VNet" --network-id 1 \
  --project <slug> --account <slug>

# Get vnet RPC URL
ethcli tenderly vnets rpc <vnet-id> --project <slug> --account <slug>

# Set wallet balance on vnet
ethcli tenderly vnets admin --vnet <id> set-balance 0x... 10eth \
  --project <slug> --account <slug>

# List contracts
ethcli tenderly contracts list --project <slug> --account <slug>

# List alerts
ethcli tenderly alerts list --project <slug> --account <slug>
```

### Endpoints - Manage RPC Endpoints

```bash
# List configured endpoints
ethcli endpoints list

# Add an endpoint
ethcli endpoints add https://eth.llamarpc.com

# Test an endpoint
ethcli endpoints test https://eth.llamarpc.com

# Remove an endpoint
ethcli endpoints remove https://eth.llamarpc.com
```

### Config - Configuration Management

```bash
# Initialize config with template
ethcli config init

# Show config file path
ethcli config path

# Show current config
ethcli config show

# Set Etherscan API key
ethcli config set-etherscan-key YOUR_KEY

# Set Tenderly credentials
ethcli config set-tenderly --key KEY --account ACCOUNT --project PROJECT
```

### Update & Doctor

```bash
# Check for updates
ethcli update

# Auto-install latest version
ethcli update --install

# Check configuration and endpoint health
ethcli doctor
```

---

## Aggregation Commands

### Price - Multi-Source Price Aggregation

Query token prices from multiple sources in parallel and get aggregated results.

```bash
# Get aggregated price for a token
ethcli price ETH
ethcli price BTC

# Query specific chain
ethcli price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain ethereum

# Query specific source
ethcli price ETH --source gecko
ethcli price ETH --source chainlink
ethcli price ETH --source pyth

# LP token prices (Curve priority)
ethcli price 0x... --lp

# Output formats
ethcli price ETH -o json
ethcli price ETH -o table
```

**Sources**: CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, CCXT

### Portfolio - Multi-Source Balance Aggregation

```bash
# Get aggregated portfolio balances
ethcli portfolio 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# With options
ethcli portfolio 0x... --chain polygon
ethcli portfolio 0x... -o json
```

**Sources**: Alchemy, Dune SIM, Moralis

### NFTs - Multi-Source NFT Aggregation

```bash
# Get aggregated NFT holdings
ethcli nfts 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# With options
ethcli nfts 0x... --chain polygon
ethcli nfts 0x... -o json
```

**Sources**: Alchemy, CoinGecko, Moralis, Dune SIM

### Yields - DeFi Yield Aggregation

```bash
# Get DeFi yields
ethcli yields --protocol aave
ethcli yields --protocol curve
ethcli yields --chain ethereum
```

**Sources**: DefiLlama, Curve Finance

### Quote - DEX Aggregator Quotes

Get swap quotes from multiple DEX aggregators in parallel.

```bash
# Get the best quote from all aggregators
ethcli quote best ETH USDC 1000000000000000000 --chain ethereum

# Get quote from a specific aggregator
ethcli quote from openocean ETH USDC 1000000000000000000
ethcli quote from 1inch WETH DAI 1000000000000000000 --chain polygon

# Compare quotes from all aggregators side-by-side
ethcli quote compare ETH USDC 1000000000000000000 --chain ethereum

# Use human-readable amounts with --decimals
ethcli quote best ETH USDC 1.5 --decimals 18 --chain ethereum

# Include transaction data in output
ethcli quote best ETH USDC 1000000000000000000 --show-tx

# Set slippage tolerance (basis points, default 50 = 0.5%)
ethcli quote best ETH USDC 1000000000000000000 --slippage 100

# Provide sender address for more accurate quotes
ethcli quote best ETH USDC 1000000000000000000 --sender 0xYourAddress

# JSON output
ethcli quote compare ETH USDC 1000000000000000000 --format json
```

**Available Aggregators**:

| Alias | Aggregator | Notes |
|-------|------------|-------|
| `openocean`, `oo` | OpenOcean | Multi-chain DEX aggregator |
| `kyberswap`, `kyber` | KyberSwap | Dynamic routing |
| `0x`, `zerox` | 0x Protocol | Professional-grade liquidity |
| `1inch`, `oneinch` | 1inch | Pathfinder algorithm |
| `cowswap`, `cow` | CowSwap | MEV-protected trades |
| `li.fi`, `lifi` | LI.FI | Cross-chain aggregator |
| `velora`, `paraswap` | Velora/ParaSwap | Multi-protocol routing |
| `enso`, `ensofi` | Enso Finance | DeFi shortcuts |

---

## Direct API Commands

### Alchemy - Alchemy API

Requires `ALCHEMY_API_KEY` environment variable.

```bash
# NFT queries
ethcli alchemy nfts 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
ethcli alchemy nft-metadata 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d 1

# Token data
ethcli alchemy balances 0x...
ethcli alchemy token-metadata 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Transfers
ethcli alchemy transfers 0x... --category erc20

# Debug traces
ethcli alchemy trace-tx 0x...
```

### Gecko - CoinGecko API

Optional `COINGECKO_API_KEY` for Pro API (higher rate limits).

```bash
# Coin data
ethcli gecko coin bitcoin
ethcli gecko coin ethereum --format json

# Price queries
ethcli gecko price bitcoin,ethereum --vs usd,eur
ethcli gecko price-history bitcoin --days 30

# Market data
ethcli gecko markets --vs usd --per-page 100
ethcli gecko trending

# NFT data
ethcli gecko nft-collection boredapeyachtclub

# Exchanges
ethcli gecko exchanges
ethcli gecko exchange binance
```

### Llama - DefiLlama API

Optional `DEFILLAMA_API_KEY` for Pro endpoints.

```bash
# TVL data
ethcli llama tvl aave
ethcli llama protocols
ethcli llama chains

# Prices
ethcli llama price ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli llama prices ethereum:0x...,ethereum:0x...
ethcli llama price-history ethereum:0x... --period 1d

# Yields
ethcli llama yields
ethcli llama yields --chain ethereum --project aave

# Stablecoins
ethcli llama stablecoins
ethcli llama stablecoin-history tether
```

### Moralis - Moralis API

Requires `MORALIS_API_KEY` environment variable.

```bash
# Wallet data
ethcli moralis balance 0x...
ethcli moralis tokens 0x...
ethcli moralis transactions 0x...
ethcli moralis transfers 0x...

# NFT data
ethcli moralis nfts 0x...
ethcli moralis nft-metadata 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d 1

# Token data
ethcli moralis token-price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli moralis token-metadata 0x...

# DeFi positions
ethcli moralis defi-positions 0x...
```

### Dsim - Dune SIM API

Requires `DUNE_SIM_API_KEY` environment variable.

```bash
# Wallet simulation
ethcli dsim balances 0x...
ethcli dsim activity 0x...
ethcli dsim collectibles 0x...
ethcli dsim defi 0x...
```

### Dune - Dune Analytics API

Requires `DUNE_API_KEY` environment variable.

```bash
# Run queries
ethcli dune query 1234567
ethcli dune query 1234567 --params '{"address": "0x..."}'

# Get results
ethcli dune results 1234567
ethcli dune results 1234567 --format csv

# Executions
ethcli dune execute 1234567
ethcli dune status <execution-id>

# Tables
ethcli dune tables --namespace dune
```

### Curve - Curve Finance API

```bash
# Pool data
ethcli curve pools
ethcli curve pools --chain ethereum
ethcli curve pool 0x...

# Volume and TVL
ethcli curve volumes
ethcli curve tvl

# Lending
ethcli curve lending-pools
ethcli curve lending-pool 0x...

# Token data
ethcli curve tokens
ethcli curve token 0x...

# Router - find optimal swap routes
ethcli curve router route <from_token> <to_token> --chain ethereum --limit 5
ethcli curve router encode <from> <to> <amount> <min_out> --chain ethereum
ethcli curve router stats --chain ethereum
ethcli curve router address ethereum
```

### CCXT - Exchange Data

Query cryptocurrency exchanges via CCXT.

```bash
# Ticker data
ethcli ccxt ticker binance BTC/USDT
ethcli ccxt ticker okx ETH/USDT

# Order book
ethcli ccxt orderbook binance BTC/USDT --limit 10

# OHLCV candles
ethcli ccxt ohlcv binance BTC/USDT --timeframe 1h --limit 100

# Exchange info
ethcli ccxt exchanges
ethcli ccxt markets binance

# Multiple exchanges
ethcli ccxt ticker bitget,hyperliquid BTC/USDT
```

**Supported Exchanges**: Binance, Bitget, OKX, Hyperliquid, and many more

### Chainlink - Price Feeds

RPC-based price feeds (no API key needed for on-chain queries).

```bash
# Get current price
ethcli chainlink price ETH
ethcli chainlink price BTC --chain arbitrum

# Historical price (requires archive node)
ethcli chainlink price ETH --block 18000000

# Query specific oracle
ethcli chainlink price ETH --oracle 0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419

# Get feed address
ethcli chainlink feed CVX
ethcli chainlink feed ETH --quote usd

# List known oracles
ethcli chainlink oracles
ethcli chainlink oracles --chain arbitrum

# Data Streams (requires CHAINLINK_API_KEY and CHAINLINK_USER_SECRET)
ethcli chainlink streams feeds
ethcli chainlink streams latest <feed_id>
ethcli chainlink streams report <feed_id> <timestamp>
```

### Uniswap - V2/V3/V4 Pool Queries

```bash
# On-chain queries (no API key needed)
ethcli uniswap pool 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640
ethcli uniswap liquidity 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640
ethcli uniswap balance <token> <account>

# Subgraph queries (requires THEGRAPH_API_KEY)
ethcli uniswap eth-price
ethcli uniswap eth-price --version v2
ethcli uniswap top-pools 10
ethcli uniswap top-pools 20 --version v4
ethcli uniswap swaps 0x... --limit 20
ethcli uniswap day-data 0x... --days 7

# LP positions
ethcli uniswap positions 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
ethcli uniswap positions <address> --version v3 --chain arbitrum
ethcli uniswap positions <address> --format json

# Well-known addresses
ethcli uniswap addresses
ethcli uniswap addresses --factories
ethcli uniswap addresses --pools --version v3
```

**Alias**: `ethcli uni`

### Kong - Yearn Finance Data

Query Yearn Finance vault and strategy data via the Kong GraphQL API. No API key required.

```bash
# List vaults
ethcli kong vaults list
ethcli kong vaults list --chain-id 1 --yearn
ethcli kong vaults list --v3 --erc4626

# Get vault details
ethcli kong vaults get --chain-id 1 0x...

# Strategies
ethcli kong strategies list --chain-id 1
ethcli kong strategies get --chain-id 1 0x...

# Prices
ethcli kong prices current --chain-id 1 0x...
ethcli kong prices historical --chain-id 1 0x... 1700000000

# TVL
ethcli kong tvl current --chain-id 1 0x...
ethcli kong tvl history --chain-id 1 0x... --period day --limit 30

# Reports (harvests)
ethcli kong reports vault --chain-id 1 0x...
ethcli kong reports strategy --chain-id 1 0x...
```

**Alias**: `ethcli yearn`

---

## Security & Analysis Commands

### GoPlus - Security API

Query GoPlus Security API for token, address, NFT, and approval security analysis.

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
ethcli goplus token 0x... --chain-id 1 --format json
```

**Notes**:
- Free API - no API key required for basic usage
- Set `GOPLUS_APP_KEY` and `GOPLUS_APP_SECRET` for batch queries and higher rate limits
- **Alias**: `ethcli gp`

### Solodit - Vulnerability Database

Search smart contract security vulnerability findings from Solodit (by Cyfrin).

Requires `SOLODIT_API_KEY` environment variable.

```bash
# Search for vulnerability findings
ethcli solodit search "reentrancy"
ethcli solodit search "oracle manipulation" --impact HIGH,MEDIUM
ethcli solodit search "flash loan" --firm "Trail of Bits" --tag Reentrancy

# Filter by various criteria
ethcli solodit search "access control" --protocol "Aave" --language Solidity
ethcli solodit search "price" --min-quality 3 --sort quality

# Get a specific finding
ethcli solodit get <finding-slug>

# Check API rate limit
ethcli solodit rate-limit

# List tags and firms
ethcli solodit tags
ethcli solodit firms
```

**Alias**: `ethcli sld`

### Blacklist - Token Blacklist Management

Manage a local token blacklist for filtering spam/scam tokens.

```bash
# Scan a token for security issues
ethcli blacklist scan 0x... --chain ethereum

# Scan entire portfolio for suspicious tokens
ethcli blacklist scan-portfolio 0xYourAddress --chain ethereum --auto-blacklist

# Only show suspicious tokens
ethcli blacklist scan-portfolio 0x... --suspicious-only --auto-blacklist

# List blacklisted tokens
ethcli blacklist list
ethcli blacklist list --links  # With Etherscan links

# Add/remove tokens
ethcli blacklist add 0x... --chain ethereum --reason "Honeypot token"
ethcli blacklist remove 0x...

# Check if token is blacklisted
ethcli blacklist check 0x...

# Clear all
ethcli blacklist clear
```

**Notes**:
- Blacklist stored in `~/.config/ethcli/blacklist.toml`
- Uses GoPlus API + Etherscan verification for security checks
- Known protocols (Yearn, Curve, Aave, etc.) are auto-whitelisted
- **Alias**: `ethcli bl`

---

## Multi-Chain Support

```bash
# Use --chain flag for other networks
ethcli --chain polygon account balance 0x...
ethcli --chain arbitrum logs -c 0x... -f 0 -t latest
ethcli --chain base gas oracle

# Supported chains:
# ethereum, polygon, arbitrum, optimism, base, bsc, avalanche
```

## Configuration

Config file: `~/.config/ethcli/config.toml`

```toml
# Set Etherscan API key (optional, increases rate limit)
etherscan_api_key = "YOUR_KEY"

# Add custom endpoints
[[endpoints]]
url = "https://my-private-node.example.com"
max_block_range = 10000000
max_logs = 1000000
priority = 100
```

Or via CLI:
```bash
ethcli config set-etherscan-key YOUR_KEY
```

## Environment Variables

| Variable | Required For | Description |
|----------|-------------|-------------|
| `ETHERSCAN_API_KEY` | Optional | Increases Etherscan rate limit |
| `TENDERLY_ACCESS_KEY` | `ethcli tenderly` | Tenderly API access |
| `ALCHEMY_API_KEY` | `ethcli alchemy`, aggregation | Alchemy API access |
| `COINGECKO_API_KEY` | Optional | CoinGecko Pro API (higher rate limits) |
| `DEFILLAMA_API_KEY` | Optional | DefiLlama Pro endpoints |
| `MORALIS_API_KEY` | `ethcli moralis` | Moralis API access |
| `DUNE_SIM_API_KEY` | `ethcli dsim` | Dune SIM wallet simulation |
| `DUNE_API_KEY` | `ethcli dune` | Dune Analytics queries |
| `THEGRAPH_API_KEY` | Uniswap subgraph | The Graph API access |
| `CHAINLINK_API_KEY` | `chainlink streams` | Chainlink Data Streams (premium) |
| `CHAINLINK_USER_SECRET` | `chainlink streams` | Chainlink Data Streams (premium) |
| `GOPLUS_APP_KEY` | Optional | GoPlus batch queries (higher rate limits) |
| `GOPLUS_APP_SECRET` | Optional | GoPlus batch queries (higher rate limits) |
| `SOLODIT_API_KEY` | `ethcli solodit` | Solodit vulnerability database |

## License

MIT
