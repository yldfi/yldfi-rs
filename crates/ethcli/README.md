<p align="center">
  <img src="logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">ethcli</h1>

<p align="center">
  Comprehensive Ethereum CLI for logs, transactions, accounts, and contracts
</p>

<p align="center">
  <a href="https://crates.io/crates/ethcli"><img src="https://img.shields.io/crates/v/ethcli.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/ethcli/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
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

## Installation

### Download Binary

```bash
# macOS (Apple Silicon)
curl -sL https://github.com/yldfi/ethcli/releases/latest/download/ethcli-macos-aarch64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/

# macOS (Intel)
curl -sL https://github.com/yldfi/ethcli/releases/latest/download/ethcli-macos-x86_64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/

# Linux (x86_64)
curl -sL https://github.com/yldfi/ethcli/releases/latest/download/ethcli-linux-x86_64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/

# Linux (ARM64)
curl -sL https://github.com/yldfi/ethcli/releases/latest/download/ethcli-linux-aarch64.tar.gz | tar xz
sudo mv ethcli /usr/local/bin/
```

### Install with Cargo

```bash
cargo install ethcli
```

Or from source:
```bash
cargo install --git https://github.com/yldfi/ethcli.git
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
```

## Commands

### Logs - Fetch Historical Events

```bash
# Fetch specific events
ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  -e "Transfer(address,address,uint256)" \
  -f 18000000 -t 18100000 -o transfers.json

# Fetch all events (auto-fetches ABI from Etherscan)
ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  -f 18000000 -t latest --format csv -o events.csv

# Output to SQLite
ethcli logs -c 0x... -f 18000000 -t 18100000 --format sqlite -o events.db

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

```bash
ETHERSCAN_API_KEY    # Etherscan API key (optional)
```

## License

MIT
