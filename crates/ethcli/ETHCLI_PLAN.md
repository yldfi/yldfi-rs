# ethcli Implementation Plan

> Expanding eth-log-fetcher into a comprehensive Ethereum CLI tool

## Overview

Rename and expand `eth-log-fetcher` to `ethcli` - a unified CLI for Ethereum data fetching, transaction analysis, and blockchain exploration. Integrates `foundry-block-explorers` for Etherscan API access while retaining custom RPC pool and log fetching capabilities.

## Current State

```
eth-log-fetcher v0.3.2
├── Log fetching (parallel RPC, checkpoints, streaming)
├── Transaction analysis (tx command)
├── RPC endpoint management
├── Signature cache + 4byte.directory (internal only)
├── Multi-chain support (7 chains)
└── Output: JSON, CSV, SQLite
```

## Target State

```
ethcli v0.4.0
├── logs      - Fetch historical logs (existing)
├── tx        - Analyze transactions (existing)
├── account   - Balance, transactions, token transfers (NEW)
├── contract  - ABI, source, creation, verify (NEW)
├── token     - Info, holders, balances (NEW)
├── gas       - Oracle, estimates (NEW)
├── sig       - Signature lookup (expose existing internal code)
├── endpoints - RPC management (existing)
└── config    - Configuration (existing)
```

---

## Phase 1: Project Rename & Restructure

### 1.1 Package Rename

**Cargo.toml changes:**
```toml
[package]
name = "ethcli"
version = "0.4.0"
description = "Comprehensive Ethereum CLI for logs, transactions, accounts, and contracts"
keywords = ["ethereum", "cli", "etherscan", "blockchain", "web3"]

[[bin]]
name = "ethcli"
path = "src/main.rs"

[lib]
name = "ethcli"
path = "src/lib.rs"
```

**New dependency:**
```toml
foundry-block-explorers = "0.9"
```

### 1.2 Directory Restructure

```
src/
├── main.rs                    # CLI entry point
├── lib.rs                     # Library exports
│
├── cli/                       # NEW: CLI command modules
│   ├── mod.rs
│   ├── logs.rs               # logs subcommand (extract from main.rs)
│   ├── tx.rs                 # tx subcommand (extract from main.rs)
│   ├── account.rs            # NEW
│   ├── contract.rs           # NEW
│   ├── token.rs              # NEW
│   ├── gas.rs                # NEW
│   ├── sig.rs                # NEW (exposes existing functionality)
│   ├── endpoints.rs          # extract from main.rs
│   └── config.rs             # extract from main.rs
│
├── etherscan/                 # NEW: Etherscan client wrapper
│   ├── mod.rs
│   ├── client.rs             # Extended client wrapping foundry-block-explorers
│   └── cache.rs              # Move from src/cache.rs
│
├── rpc/                       # Keep existing
├── abi/                       # Keep existing (simplified, delegate to etherscan/)
├── tx/                        # Keep existing
├── output/                    # Keep existing
├── config/                    # Keep existing
├── fetcher.rs                 # Keep existing
├── checkpoint.rs              # Keep existing
├── proxy.rs                   # Keep existing
└── error.rs                   # Extend with new error types
```

---

## Phase 2: Integrate foundry-block-explorers

### 2.1 Create Extended Client

**src/etherscan/client.rs:**
```rust
use foundry_block_explorers::Client as EtherscanClient;
use crate::etherscan::cache::SignatureCache;
use std::sync::Arc;

/// Extended Etherscan client with signature caching and 4byte lookups
pub struct Client {
    inner: EtherscanClient,
    sig_cache: Arc<SignatureCache>,
    http: reqwest::Client,
    chain: Chain,
}

impl Client {
    pub fn new(chain: Chain, api_key: Option<&str>) -> Result<Self> {
        let inner = EtherscanClient::builder()
            .chain(chain.into())
            .with_api_key(api_key.unwrap_or_default())
            .build()?;

        Ok(Self {
            inner,
            sig_cache: Arc::new(SignatureCache::new()),
            http: reqwest::Client::new(),
            chain,
        })
    }

    // ========== Delegated methods (from foundry-block-explorers) ==========

    // Contract
    pub async fn contract_abi(&self, addr: Address) -> Result<JsonAbi>;
    pub async fn contract_source_code(&self, addr: Address) -> Result<ContractMetadata>;
    pub async fn contract_creation_data(&self, addr: Address) -> Result<ContractCreationData>;

    // Account
    pub async fn get_ether_balance_single(&self, addr: Address) -> Result<U256>;
    pub async fn get_ether_balance_multi(&self, addrs: &[Address]) -> Result<Vec<AccountBalance>>;
    pub async fn get_transactions(&self, addr: Address, params: TxListParams) -> Result<Vec<NormalTransaction>>;
    pub async fn get_internal_transactions(&self, params: InternalTxParams) -> Result<Vec<InternalTransaction>>;
    pub async fn get_erc20_token_transfer_events(&self, params: TokenQueryOption) -> Result<Vec<Erc20TokenTransfer>>;
    pub async fn get_erc721_token_transfer_events(&self, params: TokenQueryOption) -> Result<Vec<Erc721TokenTransfer>>;
    pub async fn get_erc1155_token_transfer_events(&self, params: TokenQueryOption) -> Result<Vec<Erc1155TokenTransfer>>;
    pub async fn get_mined_blocks(&self, addr: Address, page: u64, offset: u64) -> Result<Vec<MinedBlock>>;

    // Gas
    pub async fn gas_oracle(&self) -> Result<GasOracle>;
    pub async fn gas_estimate(&self, gas_price: u64) -> Result<u64>;

    // ========== Our extensions (unique value-add) ==========

    /// Lookup function selector - checks cache, then 4byte.directory
    pub async fn lookup_selector(&self, selector: &str) -> Option<String> {
        let normalized = normalize_selector(selector);

        if let Some(sig) = self.sig_cache.get_function(&normalized) {
            return Some(sig);
        }

        let sig = self.fetch_4byte_function(&normalized).await?;
        self.sig_cache.set_function(&normalized, &sig);
        Some(sig)
    }

    /// Lookup event topic - checks cache, then 4byte.directory
    pub async fn lookup_event(&self, topic0: &str) -> Option<String> {
        let normalized = normalize_topic(topic0);

        if let Some(sig) = self.sig_cache.get_event(&normalized) {
            return Some(sig);
        }

        let sig = self.fetch_4byte_event(&normalized).await?;
        self.sig_cache.set_event(&normalized, &sig);
        Some(sig)
    }

    /// Get token metadata via eth_call (name, symbol, decimals)
    pub async fn get_token_metadata(&self, addr: &str) -> Result<TokenMetadata>;

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.sig_cache.stats()
    }
}

impl std::ops::Deref for Client {
    type Target = EtherscanClient;
    fn deref(&self) -> &Self::Target { &self.inner }
}
```

### 2.2 Chain Compatibility

**src/config/chain.rs (add conversion):**
```rust
impl From<Chain> for alloy_chains::Chain {
    fn from(chain: Chain) -> Self {
        alloy_chains::Chain::from_id(chain.chain_id())
    }
}
```

---

## Phase 3: CLI Commands

### 3.1 Command Structure

**src/main.rs:**
```rust
#[derive(Parser)]
#[command(name = "ethcli")]
#[command(version, about = "Comprehensive Ethereum CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = "ethereum", global = true)]
    chain: String,

    #[arg(long, env = "ETHERSCAN_API_KEY", global = true)]
    etherscan_key: Option<String>,

    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch historical logs from contracts
    Logs { /* existing args */ },

    /// Analyze transaction(s)
    Tx { /* existing args */ },

    /// Account operations
    Account {
        #[command(subcommand)]
        action: AccountCommands,
    },

    /// Contract operations
    Contract {
        #[command(subcommand)]
        action: ContractCommands,
    },

    /// Token operations
    Token {
        #[command(subcommand)]
        action: TokenCommands,
    },

    /// Gas oracle and estimates
    Gas {
        #[command(subcommand)]
        action: GasCommands,
    },

    /// Signature lookup
    Sig {
        #[command(subcommand)]
        action: SigCommands,
    },

    /// RPC endpoint management
    Endpoints { /* existing */ },

    /// Configuration
    Config { /* existing */ },
}
```

### 3.2 Account Commands

```bash
ethcli account balance 0x123...                    # Single address
ethcli account balance 0x123... 0x456... 0x789...  # Multiple addresses
ethcli account txs 0x123... --page 1 --limit 50   # Transaction history
ethcli account internal-txs 0x123...               # Internal transactions
ethcli account erc20 0x123... --token 0xUSDC       # ERC20 transfers
ethcli account erc721 0x123...                     # NFT transfers
ethcli account erc1155 0x123...                    # ERC1155 transfers
ethcli account mined-blocks 0x123...               # Blocks validated
ethcli account funded-by 0x123...                  # First funder
```

### 3.3 Contract Commands

```bash
ethcli contract abi 0x123...                # Get ABI (JSON)
ethcli contract abi 0x123... -o abi.json    # Save to file
ethcli contract source 0x123...             # Get verified source
ethcli contract source 0x123... -o src/     # Save to directory
ethcli contract creation 0x123...           # Creator + tx hash
```

### 3.4 Token Commands

```bash
ethcli token info 0xUSDC           # Name, symbol, decimals, supply
ethcli token holders 0xUSDC        # Top 100 holders
ethcli token holders 0xUSDC --limit 500
ethcli token balance 0xUSDC --holder 0x123...
ethcli token supply 0xUSDC         # Current supply
ethcli token supply 0xUSDC --block 18000000  # Historical
```

### 3.5 Gas Commands

```bash
ethcli gas oracle
# Output:
# Gas Prices (Ethereum)
# ─────────────────────
# Safe:      15 gwei  (~5 min)
# Standard:  18 gwei  (~3 min)
# Fast:      25 gwei  (~30 sec)
# Base Fee:  14.2 gwei
# Priority:  1-3 gwei

ethcli gas estimate 20
# Estimated confirmation: ~45 seconds
```

### 3.6 Signature Commands

```bash
ethcli sig fn 0xa9059cbb
# transfer(address,uint256)

ethcli sig event 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
# Transfer(address,address,uint256)

ethcli sig cache-stats
# Signature Cache
# ───────────────
# Functions: 1,234 cached (1,180 valid)
# Events:    567 cached (542 valid)
# Path:      ~/.cache/ethcli/signatures.json

ethcli sig cache-clear
# Cache cleared
```

---

## Phase 4: Output Formatting

All commands support consistent output options:

```bash
--output pretty   # Human-readable (default)
--output json     # JSON object
--output ndjson   # Newline-delimited JSON (streaming)
--output csv      # CSV (where applicable)
```

Example:
```bash
ethcli account balance 0x123... 0x456... -o json
```
```json
[
  {"address": "0x123...", "balance": "1234567890000000000", "balance_eth": "1.23456789"},
  {"address": "0x456...", "balance": "9876543210000000000", "balance_eth": "9.87654321"}
]
```

---

## Phase 5: Implementation Order

### Sprint 1: Foundation
- [ ] Rename package to `ethcli` in Cargo.toml
- [ ] Add `foundry-block-explorers` dependency
- [ ] Create `src/etherscan/mod.rs` and `client.rs`
- [ ] Move `cache.rs` to `src/etherscan/cache.rs`
- [ ] Create `src/cli/` directory structure
- [ ] Extract existing commands to cli modules
- [ ] Verify all existing functionality works

### Sprint 2: Account & Contract Commands
- [ ] Implement `ethcli account balance`
- [ ] Implement `ethcli account txs`
- [ ] Implement `ethcli account erc20/721/1155`
- [ ] Implement `ethcli contract abi`
- [ ] Implement `ethcli contract source`
- [ ] Implement `ethcli contract creation`

### Sprint 3: Token & Gas Commands
- [ ] Implement `ethcli token info`
- [ ] Implement `ethcli token holders`
- [ ] Implement `ethcli token balance/supply`
- [ ] Implement `ethcli gas oracle`
- [ ] Implement `ethcli gas estimate`

### Sprint 4: Sig & Polish
- [ ] Implement `ethcli sig fn`
- [ ] Implement `ethcli sig event`
- [ ] Implement `ethcli sig cache-stats/cache-clear`
- [ ] Add pretty output formatting
- [ ] Update README and docs
- [ ] Add integration tests

---

## Migration & Compatibility

### Backward Compatibility

Option 1: Shell alias
```bash
alias eth-log-fetch='ethcli logs'
```

Option 2: Wrapper binary (src/bin/eth-log-fetch.rs)
```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut new_args = vec!["ethcli".to_string(), "logs".to_string()];
    new_args.extend(args.into_iter().skip(1));
    // exec ethcli with new args
}
```

### Config Migration

On first run, auto-migrate:
- `~/.config/eth-log-fetcher/` → `~/.config/ethcli/`
- `~/.cache/eth-log-fetch/` → `~/.cache/ethcli/`

---

## Why foundry-block-explorers?

| Aspect | Roll Our Own | Use foundry-block-explorers |
|--------|--------------|----------------------------|
| Maintenance | We maintain all Etherscan API bindings | Foundry team maintains |
| API changes | We fix | They fix |
| Type safety | Build from scratch | Already done |
| Testing | Need comprehensive tests | Already tested |
| Ecosystem | Standalone | Part of alloy/foundry ecosystem |
| Our value-add | Duplicated effort | Focus on sig cache + 4byte |

**Conclusion:** Use foundry-block-explorers for Etherscan API, keep our unique additions (signature cache, 4byte.directory, RPC pool, log fetching).

---

## File Sizes (Estimated)

| File | Lines | Notes |
|------|-------|-------|
| src/cli/account.rs | ~250 | 8 subcommands |
| src/cli/contract.rs | ~150 | 3 subcommands |
| src/cli/token.rs | ~200 | 4 subcommands |
| src/cli/gas.rs | ~80 | 2 subcommands |
| src/cli/sig.rs | ~100 | 4 subcommands |
| src/etherscan/client.rs | ~300 | Wrapper + extensions |
| **Total new code** | ~1,100 | Plus refactored existing |

---

## Success Criteria

- [ ] All existing `eth-log-fetch` functionality preserved
- [ ] `ethcli logs` works identically to old `eth-log-fetch`
- [ ] All new commands have `--help` documentation
- [ ] JSON output works for all commands
- [ ] Tests pass on Linux, macOS, Windows
- [ ] Binary size increase < 3MB
- [ ] No performance regression in log fetching
