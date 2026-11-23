<p align="center">
  <img src="../../logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">ethcli</h1>

<p align="center">
  <strong>The Swiss Army Knife for Blockchain</strong>
</p>

<p align="center">
  One CLI for everything: transactions, contracts, DeFi, DEX aggregators, oracles, security analysis, and 50+ data sources across all major chains.
</p>

<p align="center">
  <a href="https://crates.io/crates/ethcli"><img src="https://img.shields.io/crates/v/ethcli.svg" alt="crates.io"></a>
  <a href="https://opensource.org/license/mit"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Log Fetching**: Parallel RPC requests across multiple endpoints
- **Transaction Analysis**: Decode transactions with signature lookup
- **Account Operations**: Balance, transactions, token transfers
- **Contract Tools**: ABI fetching, source code, bytecode analysis, selector dispatchers, guard checks
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
- **Price Aggregation**: Multi-source prices from CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, exchanges (Binance, Bitget, OKX)
- **Portfolio Aggregation**: Balance data from Alchemy, Moralis
- **NFT Aggregation**: Holdings from Alchemy, CoinGecko, Moralis
- **Yield Aggregation**: DeFi yields from DefiLlama and Curve
- **Quote Aggregation**: Swap quotes from OpenOcean, KyberSwap, 0x, 1inch, CowSwap, LI.FI, Velora (Market and Delta), Enso, prop AMMs

### Direct API Access
- **Alchemy**: NFTs, prices, portfolio, transfers, debug traces
- **CoinGecko**: Coins, prices, NFTs, exchanges
- **DefiLlama**: TVL, prices, yields, stablecoins
- **Moralis**: Wallet, token, NFT, DeFi, transactions
- **Dune Analytics**: Queries, executions, tables
- **Curve Finance**: Pools, volumes, lending, tokens, router
- **Chainlink**: Price feeds (RPC-based, no API key needed)
- **Exchanges** (`cex`): Binance, Bitget, OKX, Bybit, Coinbase, Kraken, Hyperliquid, Lighter
- **Uniswap**: V2/V3/V4 pool queries (on-chain + subgraph)
- **Yearn Kong**: Vaults, strategies, prices, TVL, reports

### Direct DEX Aggregator Access
- **1inch**: Quote, swap, tokens, liquidity sources, approvals
- **De¹ (formerly OpenOcean)**: Quote, swap, reverse quote, tokens, DEX sources
- **KyberSwap**: Routes, route data, transaction building
- **0x Protocol**: Quote, price, liquidity sources
- **CowSwap**: MEV-protected quotes, orders, trades, auctions
- **LI.FI**: Cross-chain quotes, routes, bridges, chains, tokens
- **Velora/ParaSwap**: Price, transaction building, tokens
- **Enso Finance**: DeFi routes, prices, balances
- **Pyth Network**: Real-time price feeds, feed search

### Security & Analysis
- **Bytecode Analysis**: Function selector extraction (evmole), opcode disassembly, dispatcher mapping, guard/check heuristics, security pattern detection
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
cargo install --locked ethcli
```

Or from source:
```bash
cargo install --locked --git https://github.com/yldfi/yldfi-rs.git ethcli
```

## MCP Server (AI Integration)

For AI assistants and LLMs, use **ethcli-mcp** - a companion MCP (Model Context Protocol) server that exposes ethcli functionality as tools.

```bash
# Install
cargo install --locked ethcli-mcp

# Run (communicates via STDIO)
ethcli-mcp
```

**Features:**
- 594 typed tools covering read-oriented ethcli workflows
- JSON Schema validation for all inputs
- Read-only default for local/remote state changes
- Rate limiting, timeout protection, and process cleanup
- Works with Claude, GPT, and other MCP-compatible AI assistants

**Configuration (Claude Desktop):**
```json
{
  "mcpServers": {
    "ethcli": {
      "command": "ethcli-mcp"
    }
  }
}
```

Use `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1` only for trusted MCP clients that need mutating tools. If `ethcli` is not installed next to `ethcli-mcp`, set `ETHCLI_PATH` to an absolute binary path.

See [ethcli-mcp](https://crates.io/crates/ethcli-mcp) for full documentation.

### Agent skill

[`skills/ethcli/SKILL.md`](https://github.com/yldfi/yldfi-rs/blob/main/skills/ethcli/SKILL.md)
tells a coding agent (Claude Code, Codex and others that read skills) which
ethcli command answers which question, and which output fields to check
before trusting a result: partial balance changes, guessed error names,
incomplete histories. Copy the `skills/ethcli` folder into your agent's skills
directory, for example `~/.claude/skills/ethcli/` for Claude Code.

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

# Accept gaps: exit 0 with a warning if only some chunks fail (the default is an error)
ethcli logs -c 0x... -f 0 -t latest --allow-partial
```

### Transaction - Analyze Transactions

```bash
# Analyze a transaction
ethcli tx 0x1234567890abcdef...

# Enrich with Etherscan data (contract names, token symbols, function decoding)
ethcli tx 0x... --enrich

# The fee, the position in the block and the net balance change per address are
# always included. Native-token changes are marked "partial" unless traced.
# --full adds a call trace (exact native-token changes, including internal transfers),
# the block time and, for a failed transaction, why it reverted
ethcli tx 0x... --full
ethcli tx 0x... --full -o json

# Who paid whom, as a Mermaid flowchart (renders on GitHub and in most chat UIs)
ethcli tx 0x... --full --graph

# Analyze against a single endpoint (for example a local fork)
ethcli tx 0x... --rpc-url http://localhost:8545

# Output as JSON
ethcli tx 0x... -o json
```

### Account - Balance and History

```bash
# Get ETH balance
ethcli account balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Get token balance
ethcli token balance 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --holder 0x...

# List recent transactions
ethcli account txs 0x...

# Limit history to an inclusive block range; pagination applies within the range
ethcli account txs 0x... --start-block 25893052 --end-block 26108081 --page 1 --limit 1000 --sort asc -o json

# List token transfers
ethcli account erc20 0x...
ethcli account erc20 0x... -o ndjson     # one JSON object per line (all account subcommands)

# Whole history in one command: 1,000 rows per request, walking past the
# explorer's 10,000-row window (also on internal-txs and erc20)
ethcli account txs 0x... --all --max-pages 50 -o json

# CSV for spreadsheets (account txs, internal-txs, erc20, rpc multicall)
ethcli account txs 0x... --all --csv > history.csv
ethcli rpc multicall "0xB4e1... getReserves()(uint112,uint112,uint32)" --block-range 21000000:21070000:7200 --csv

# Who first funded an address. Uses Etherscan's funded-by endpoint on a paid
# plan; otherwise the earliest incoming native-token transfer
ethcli account funded-by 0x...

# Names and labels: Etherscan name tags (Pro Plus plan), the address book and
# ethcli's built-in labels
ethcli account labels 0x... 0x...

# Open token approvals and the amount each puts at risk (min of allowance and
# balance, with USD where priced), Permit2 allowances with expiry, NFT operator
# approvals and any EIP-7702 delegation. Pairs come from the owner's approval
# events (explorer logs); current values are read on chain in one multicall
ethcli account approvals 0x...
ethcli account approvals 0x... --json --max-pages 50 --no-price
```

### Contract - ABI, Source Code, and Bytecode Analysis

```bash
# Get contract ABI
ethcli contract abi 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Get source code
ethcli contract source 0x...
ethcli contract source 0x... --list                      # files and line counts
ethcli contract source 0x... --function withdraw         # one function, with NatSpec and file:line
ethcli contract source 0x... --grep "onlyOwner"          # matching lines as path:line: text
ethcli contract source 0x... --file Vault.sol            # one file

# Get creation transaction
ethcli contract creation 0x...

# Everything about an address in one call: EOA / contract / delegated EOA, proxy and
# implementation, verified name, creator, token details, Safe owners, and its functions
ethcli contract inspect 0x...
ethcli contract inspect 0x... --json

# Read a storage variable by name, decoded, from the verified storage layout
# (Sourcify). Keys, indexes and struct members follow the name or go in the path.
# For a proxy, the implementation's layout is used and the proxy's storage read.
ethcli contract storage 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --list
ethcli contract storage 0xA0b8...eB48 balanceAndBlacklistStates 0xHolder
ethcli contract storage 0xA0b8...eB48 allowed 0xOwner 0xSpender --block 2025-01-01
ethcli contract storage 0x88e6...5640 slot0.tick
ethcli contract storage 0x88e6...5640 "ticks[-887270]" -o json

# Extract function selectors from bytecode (no ABI needed)
ethcli contract selectors 0x...
ethcli contract sel 0x... --lookup  # Lookup signatures from 4byte.directory
ethcli contract sel 0x... --follow-proxy --lookup  # Inspect proxy implementation

# Disassemble bytecode into opcodes
ethcli contract disassemble 0x... --limit 50
ethcli contract dis 0x...

# Get opcode frequency statistics
ethcli contract opcodes 0x...

# Security analysis (detects dangerous patterns)
ethcli contract analyze 0x...
ethcli contract az 0x... --include-disassembly

# Unverified/proxy contract review: follow implementation, resolve selectors,
# map dispatcher entries, and scan side-effecting handlers for missing auth/hash checks
ethcli contract analyze 0x... --follow-proxy --lookup --dispatcher --checks

# Compiler fingerprint: solc/vyper version and metadata hash from the CBOR
# metadata, minimum EVM version (PUSH0, MCOPY, TLOAD), legacy/via-IR guess
ethcli contract fingerprint 0x...

# Decompile with the optional external evmdec (or heimdall, or all
# installed backends), cross-checked against ethcli's own facts first
ethcli contract decompile 0x... --timeout 60
ethcli contract decompile 0x... --backend all

# Compare a compiled artifact with the deployed code
ethcli contract match 0x... --artifact out/Router.sol/Router.json
ethcli contract match 0x... --artifact build-output.json --contract Router --creation-tx 0x...
```

#### Decompiling (`contract decompile`)

The backend is [evmdec](https://github.com/rink3y/Evmdec), which is optional
and not bundled. Build it from source at a pinned commit (the one this
release was checked against is `5d8b52c1bcdfe40abd5c7c63e75b22e51e5916f9`) and
put `evmdec` on `PATH` or set `EVMDEC_PATH`. ethcli fetches the runtime code
itself (following proxies like `contract inspect`, and saying which
implementation it decompiled), writes it to a temporary file and runs only
`evmdec decompile <file> --json`: no address, no RPC, no online signature
lookup and never its `audit` mode, with an empty environment and a hard
timeout. The version and the binary's sha256 are recorded (evmdec cannot
report its commit).

The output starts with a cross-check: code-block coverage, sensitive opcodes
(ORIGIN, CALLER, DELEGATECALL, SELFDESTRUCT, CREATE/CREATE2, SSTORE) whose
use the decompiled text does not show, argument types where evmole and evmdec
disagree, and any "0 writes" claim, reported as "not found by evmdec". The
pseudo-Solidity follows and is heuristic. Known evmdec defects seen in
testing: a `tx.origin` guard dropped from a hand-written MEV bot, "0 writes"
reported for Uniswap V3's `tickBitmap`, `calldata[block.chainid]` artefacts,
and `address[]` shown as `uint256[]`. heimdall is not supported. Without
evmdec the command prints install steps, ethcli's facts and a
[Dedaub](https://app.dedaub.com) link (Dedaub needs sign-in).

heimdall ([heimdall-rs](https://github.com/Jon-Becker/heimdall-rs)) is a
second optional backend: `--backend heimdall`, or `--backend all` to run every
installed backend and cross-check each. ethcli never installs, downloads or
builds it; it is found through `HEIMDALL_PATH` or `PATH`. ethcli runs only
`heimdall decompile <runtime bytecode> --include-sol --skip-resolving
--default --output <temp dir>`: never an address, RPC URL, LLM option or
explorer key. Its config file is not read (HOME is an empty temporary
directory) and all HTTP goes to a dead local proxy, which stops its
release check; the same empty environment and hard timeout apply.
heimdall tends to show guard conditions (`tx.origin`, callback
`msg.sender` checks) and revert strings well; its parameter types and
counts are weak, storage names are guessed, some conditions come out
garbled, and it can hang or run out of memory on some contracts (hence the
timeout).

#### Matching an artifact (`contract match`)

Takes a forge artifact or solc standard-JSON output; no compiler is
embedded. Verdicts: `exact` (identical, metadata included), `partial`
(identical once metadata is ignored, as Sourcify calls it; this is not a
verification) or `mismatch`, with the first differing offset, both byte
sequences, the function it falls in and a per-function equal/different
table. Build with `forge build --ast` so functions are mapped from the source
map and the inline-assembly share is reported. The rebuild procedure for
agents is in `skills/bytecode-to-source/SKILL.md`.

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
ethcli cast index address 0x123... 0    # storage slot of mapping[key] at slot 0
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

# Get block info (supports decimal or hex block numbers)
ethcli rpc block latest
ethcli rpc block 21000000 -o json
ethcli rpc block 0x1406f40 -o json   # Hex format also works

# Read storage slot
ethcli rpc storage 0x... 0

# Get contract code
ethcli rpc code 0x...

# Get nonce
ethcli rpc nonce 0x...

# Get transaction receipt
ethcli rpc receipt 0x...

# Get a transaction by hash, or a native balance at a block
ethcli rpc tx 0x... -o json
ethcli rpc balance 0x... --block 21000000 -o json

# A date, a Unix timestamp (@1706700000) or an age (7d) works wherever --block is accepted
ethcli rpc balance 0x... --block 2024-01-31
ethcli contract call 0x... totalSupply --block 7d
ethcli rpc block-at 2024-01-31T12:00:00Z    # the latest block at or before that time
# A plain date is 00:00 UTC at the start of that day: 2024-01-31 is the close of 30 January.
# For an end-of-day figure use the next day's date or a full time (2024-01-31T23:59:59Z).

# The block where a value changed, by binary search (about log2(range) reads,
# archive state needed). Target: a call, --balance, --storage <addr> <slot>,
# --code (deployment) or --nonce. Prints the block, its time, old and new
# values, top-level transactions to or from the address, and the RPC call count.
ethcli rpc find-change --code 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --from 6000000 --to 7000000
ethcli rpc find-change "0xA0b8... balanceOf(address)(uint256) 0xd8dA..." --from 2024-01-01 --json
# It assumes the value changed once; with several changes it finds one of them.
# --all searches again after each change (up to 100), but misses a change that is
# undone within a segment (a balance that goes up and back down).
# JSON carries single_change_assumed, and a warning when --all finds several
# changes: when the first change matters, use --all.

# Size and code hash only (is it a contract? did the code change?)
ethcli rpc code 0x... --summary

# Many calls at one block in a single request (Multicall3)
ethcli rpc multicall \
  "0xA0b8... totalSupply()(uint256)" \
  "0xA0b8... balanceOf(address)(uint256) 0xd8dA..."

# The same calls sampled across blocks (FROM:TO:STEP), as JSON
ethcli rpc multicall -f calls.txt --block-range 21000000:21070000:7200 -o json

# JSON from any command: --json selects the command's JSON output
ethcli rpc call 0x... -s "totalSupply()" -d uint256 --json

# Only some fields (implies --json; dot paths; applied to each array element)
ethcli account txs 0x... --fields hash,value
ethcli contract inspect 0x... --fields address,proxy,balance.wei

# With JSON output, or when stdout is not a terminal, errors are one JSON line
# on stderr (non-zero exit); ETHCLI_ERROR_FORMAT=json|text overrides:
# {"error":{"code":"not_verified","message":"...","hint":"..."}}
# codes: invalid_argument, not_verified, missing_api_key, unauthorized,
# rate_limited, unsupported_chain, not_found, rpc_error, internal

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

# Full cost of a transaction: L2 execution + L1 data fee, in ETH and USD
ethcli gas cost 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 \
  -s "transfer(address,uint256)" -a "0x000000000000000000000000000000000000dEaD 1" \
  --from 0x... --chain base
ethcli gas cost 0x... 0x<calldata> --chain arbitrum --json
```

`gas cost` takes L2 gas from `eth_estimateGas` and the price from
`eth_gasPrice`. The L1 data fee comes from the GasPriceOracle predeploy
(`getL1Fee`, given the unsigned RLP transaction) on OP-stack chains, plus
the operator fee (`getOperatorFee`, e.g. BOB's per-transaction constant);
from Scroll's L1GasPriceOracle (`0x5300…0002`, given the transaction with a
placeholder signature); and from NodeInterface (`gasEstimateL1Component`) on
Arbitrum, where it is part of the estimated gas. Chains outside the known
list are probed (ArbSys, the OP GasPriceOracle and the Scroll oracle) and the
answer is cached per chain for the process; a chain none of them matches is
reported as `unknown`, not as an L1. Rollups that fold the L1 cost into the
gas price (Linea, zkSync Era, Polygon zkEVM, Taiko) show `included`. Mantle
is excluded from estimates (it prices the L1 fee in MNT through a token
ratio). On L1 chains the L1 fee is not applicable. `--rpc-url` uses one
endpoint and its own `eth_chainId` (an explicit `--chain` that disagrees is
an error). USD prices the chain's actual gas token: ETH from the price
aggregation, others (FRAX on Fraxtal, MNT, POL, ...) by CoinGecko id on
DefiLlama. USD uses the median of
CoinGecko, DefiLlama, Chainlink, Pyth and an exchange, and names the sources
(`--no-usd` skips it).

For a mined transaction, `tx` and `rpc receipt` read the L1 fee fields of the
receipt (`l1Fee`, `l1GasUsed`, `l1GasPrice`, `l1BaseFeeScalar`,
`l1BlobBaseFee`, ... on OP-stack; `gasUsedForL1` on Arbitrum) and report
`fee_breakdown` / `feeBreakdown`: execution, L1 data and total. The receipt's
own fields decide, on any chain: `l1Fee` present means an OP-style L1 fee
(including Mantle and other chains outside the known list), `gasUsedForL1`
means Arbitrum. An OP-stack operator fee (`operatorFeeScalar`,
`operatorFeeConstant`) is added as `operator_fee_wei`; deposit transactions
pay nothing. Mantle's Tectonic-era receipts (a `tokenRatio`, no operator-fee
fields) report an `l1Fee` already inside gasUsed × price; it is shown with
`l1_fee_included: true` and not added. `tx --full` also checks the sender's
balance change and moves an `l1Fee` inside the execution fee when that is
what was paid. `tx`'s `fee_wei` is the total, so balance changes include
the L1 fee.
`tx --full` adds the fee in USD at the block's time (DefiLlama).

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
ethcli sig fn 0xa9059cbb
ethcli sig lookup 0xa9059cbb         # selector or topic, picked by length
ethcli sig error 0x08c379a0...       # decode revert data (Error, Panic or a custom error)

# Lookup event by topic
ethcli sig event 0xddf252ad...
```

### Safe - Multisig Info, Queue and Transactions (read-only)

```bash
ethcli safe info <safe>                 # owners, threshold, nonce, version, modules, guard, fallback handler (on chain)
ethcli safe queue <safe>                # pending txs (nonce >= on-chain nonce): who signed, who has not
ethcli safe tx <safeTxHash>             # one tx, calldata decoded, MultiSend batches unpacked
ethcli safe history <safe> --limit 20   # executed txs, newest first (--offset to page)
ethcli safe queue <safe> --chain arbitrum --json
```

`info` reads the Safe on chain (one Multicall3 request plus storage reads for
the guard, fallback handler and singleton), so it works on any chain. `queue`,
`tx` and `history` use the public Safe Transaction Service
(`https://api.safe.global/tx-service/<chain>`), which works without a key;
set `SAFE_API_KEY` for a higher rate limit, or `SAFE_TX_SERVICE_URL` to point
at another deployment. Calls are decoded from the target's verified ABI, or
from the signature database (marked with `?` / `"source": "signature_database"`,
a guess). Nothing is proposed, signed or executed.

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

# Decode richer Foundry traces and label addresses
ethcli simulate call 0x... --sig "swap()" --trace --decode-internal \
  --label 0x...:router --with-local-artifacts

# Trace an existing transaction
ethcli simulate tx 0x1234... --decode-internal --trace-depth 6

# Trace via debug_traceTransaction — decoded call tree by default (contract
# names, function args, events; Etherscan ABIs + 4byte fallback, no Foundry)
ethcli simulate tx 0x1234... --via debug
ethcli simulate tx 0x1234... --via debug --raw   # raw callTracer JSON

# Use different backends
ethcli simulate call ... --via cast      # Default
ethcli simulate call ... --via anvil --fork-block-number=-10 --fork-url https://...
ethcli simulate call ... --via tenderly  # Tenderly API
ethcli simulate call ... --via debug     # debug_traceCall RPC
```

`--sig` calldata is ABI-encoded natively, so `--via tenderly|debug|trace|alchemy`
work without Foundry. Foundry is still needed for `--via cast` (the default) and
`--via anvil`, and for signatures ethcli can't encode itself (parameter names,
`returns` clauses such as `balanceOf(address)(uint256)`); those fall back to
`cast calldata`. Pass raw calldata with `--data` to skip encoding entirely.

### Tenderly - Virtual TestNets & API

Requires `TENDERLY_ACCESS_KEY`, `TENDERLY_ACCOUNT` and `TENDERLY_PROJECT` (environment variables, matching `--tenderly-*` flags, or `ethcli config set-tenderly`).

```bash
# List virtual testnets
ethcli tenderly vnets --tenderly-project <slug> --tenderly-account <slug> list

# Create a vnet
ethcli tenderly vnets --tenderly-project <slug> --tenderly-account <slug> \
  create --slug my-vnet --name "My VNet" --network-id 1

# Get vnet RPC URL
ethcli tenderly vnets --tenderly-project <slug> --tenderly-account <slug> rpc <vnet-id>

# Set wallet balance on vnet
ethcli tenderly vnets --tenderly-project <slug> --tenderly-account <slug> \
  admin --vnet <id> set-balance 0x... 10eth

# Simulate transaction via RPC (tenderly_simulateTransaction)
ethcli tenderly vnets --tenderly-project <slug> --tenderly-account <slug> \
  admin --vnet <id> simulate-tx --from 0x... --to 0x... --data 0x...

# Simulate bundle via RPC (tenderly_simulateBundle)
ethcli tenderly vnets --tenderly-project <slug> --tenderly-account <slug> \
  admin --vnet <id> simulate-bundle '[{"from":"0x...","to":"0x...","data":"0x..."}]'

# List contracts
ethcli tenderly contracts --tenderly-project <slug> --tenderly-account <slug> list

# List alerts
ethcli tenderly alerts --tenderly-project <slug> --tenderly-account <slug> list
```

Tenderly has renamed Virtual TestNets to **Virtual Environments** (documented
under `/api/public/v1/account/{account}/project/{project}/environments`); the
legacy `/vnets` routes used by `ethcli tenderly vnets` are still live.

The following commands were removed because Tenderly's public API has no
matching endpoint: `tenderly contracts verify|update`,
`tenderly alerts add-destination|remove-destination`,
`tenderly alerts webhooks test`,
`tenderly actions enable|disable|invoke|logs|source|update-source` (use
`actions stop|resume|get-call`), and `simulate tx --via tenderly` (no
trace-by-hash API; use `--via debug|trace|alchemy|cast`).
`tenderly contracts encode-state` now calls `POST /contracts/encode-states`
and takes named-variable overrides, e.g.
`'{"0xToken": {"value": {"balances[0xHolder]": "1000"}}}'`.

### Endpoints - Manage RPC Endpoints

```bash
# List configured endpoints
ethcli endpoints list
ethcli endpoints list --archive     # Show only archive nodes
ethcli endpoints list --detailed    # Show full config details

# Add an endpoint (auto-detects capabilities). If it can't be reached, nothing
# is added and the command exits non-zero; --no-optimize adds it untested
ethcli endpoints add https://eth.llamarpc.com
ethcli endpoints add https://eth.llamarpc.com --no-optimize

# Add with explicit node type (archive nodes preferred for historical queries)
ethcli endpoints add https://my-archive.com --node-type archive
ethcli endpoints add https://my-node.com --node-type full --has-debug --priority 10

# Test an endpoint
ethcli endpoints test https://eth.llamarpc.com

# Check endpoint health; use more probes for a deeper check
ethcli endpoints health --chain ethereum --probes 1

# Remove an endpoint
ethcli endpoints remove https://eth.llamarpc.com
```

Single-endpoint reads prefer configured, tested full/archive nodes over unknown nodes. Relay-style RPCs that reject general reads, such as Flashbots, are kept as fallback-only when other endpoints are available.

**Node Types:**
- `archive` - Full historical state, required for queries at old blocks
- `full` - Only recent state (~128 blocks), cheaper to run
- `unknown` - Default, capabilities not yet tested

When querying historical blocks, ethcli automatically prefers archive nodes if configured.

### Config - Configuration Management

```bash
# Initialize config with template
ethcli config init

# Show config file path
ethcli config path

# Show current config (API keys, tokens and keyed RPC URLs are masked)
ethcli config show
ethcli config show --show-secrets   # print the file verbatim, secrets included

# Set Etherscan API key
ethcli config set-etherscan-key YOUR_KEY

# Set Tenderly credentials
ethcli config set-tenderly --key KEY --account ACCOUNT --project PROJECT

# Set Pyth API key (required for Pyth prices)
echo $PYTH_API_KEY | ethcli config set-pyth --stdin

# Set DefiLlama API key (needed for its yields from 2026-11-13)
echo $DEFILLAMA_API_KEY | ethcli config set-defillama --stdin
```

### Update & Doctor

```bash
# Check for updates
ethcli update

# Download and install the latest release
ethcli upgrade

# Check configuration and endpoint health
ethcli doctor
```

`ethcli upgrade` downloads the latest GitHub release for your platform (keeping
the musl build if the running binary is musl), verifies its SHA-256 checksum and
replaces the running binary and its `ethcli-mcp`:

- **Symlinks** are followed: the real binary behind a link is replaced and the
  link is kept.
- **ethcli-mcp** is the first of `ethcli-mcp<suffix>` (matching the running
  binary, e.g. `ethcli.real` → `ethcli-mcp.real`), `ethcli-mcp.real` and
  `ethcli-mcp` in the binary's directory that resolves to a native executable
  owned by the same user. Wrapper scripts, `ethcli` itself and other users'
  binaries are never overwritten; a warning says what was skipped.
- **Rollback:** both binaries are backed up, the new `ethcli --version` must run
  and report the release, otherwise the backups are restored.

Run it as the user that owns the binaries. The install logic is the *running*
binary's, so these guarantees apply to upgrades made from 0.23.5 or later.

---

## Aggregation Commands

### Price - Multi-Source Price Aggregation

Query token prices from multiple sources in parallel and get aggregated results.

```bash
# Get aggregated price for a token
ethcli price ETH
ethcli price ETH --at 2025-01-01             # historical: date, time, @unix, age (30d) or block number
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

**Sources**: CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth (requires `PYTH_API_KEY`), exchanges (`cex`: Binance, Bitget, OKX)

### Portfolio - Multi-Source Balance Aggregation

```bash
# Get aggregated portfolio balances
ethcli portfolio 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# With options
ethcli portfolio 0x... --chain polygon
ethcli portfolio 0x... -o json
```

**Sources**: Alchemy, Moralis

### NFTs - Multi-Source NFT Aggregation

```bash
# Get aggregated NFT holdings
ethcli nfts 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# With options
ethcli nfts 0x... --chain polygon
ethcli nfts 0x... -o json
```

**Sources**: Alchemy, CoinGecko, Moralis

### Yields - DeFi Yield Aggregation

```bash
# Get DeFi yields
ethcli yields --project aave
ethcli yields --project curve
ethcli yields --chain ethereum
```

**Sources**: DefiLlama, Curve Finance, Uniswap, Yearn

DefiLlama shuts down its free yields endpoints on 2026-11-13. From then on, its
yields need an API plan key: set `DEFILLAMA_API_KEY` or run
`echo $KEY | ethcli config set-defillama --stdin`. The other sources keep working
without it.

If a source fails, `ethcli yields` warns on stderr and lists it under
`failed_sources` in JSON output. If no yields come back and a source failed, it
exits non-zero, because the pools you asked for may be in the source that failed.

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

# Pass the wallet that will sign: transaction data is built for it and pays the
# output to it. Without --sender, CowSwap, LI.FI and Enso are priced for a
# placeholder address (marked in the output) and return no transaction data
ethcli quote best ETH USDC 1000000000000000000 --sender 0xYourAddress

# JSON output
ethcli quote compare ETH USDC 1000000000000000000 --format json
```

**Available Aggregators**:

| Alias | Aggregator | Notes |
|-------|------------|-------|
| `openocean`, `oo`, `de1` | De¹ (formerly OpenOcean) | Multi-chain DEX aggregator |
| `kyberswap`, `kyber` | KyberSwap | Dynamic routing |
| `0x`, `zerox` | 0x Protocol | Professional-grade liquidity |
| `1inch`, `oneinch` | 1inch | Pathfinder algorithm |
| `cowswap`, `cow` | CowSwap | MEV-protected trades |
| `li.fi`, `lifi` | LI.FI | Cross-chain aggregator |
| `velora`, `paraswap` | Velora/ParaSwap | Multi-protocol routing |
| `velora-delta`, `delta` | Velora Delta | Gasless order filled by solvers; output is net of gas, no transaction data |
| `enso`, `ensofi` | Enso Finance | DeFi shortcuts |
| `propamm`, `pamm` | Prop AMMs | Market-maker pools re-priced every block; Ethereum mainnet, major pairs. Conditional: shown but never the best quote |

**Reading the result**:

- **Outliers**: with three or more quotes, a source whose output is under half the median is listed in `aggregation.outliers` (shown as `LOW` in the table, `outlier: true` per quote) and left out of the average, spread and improvement figures. This catches an aggregator that only knows a thin pool for the token.
- **High quotes**: a source above twice the median is listed in `aggregation.high_outliers` (`HIGH` in the table) with a warning. It is still the best quote and still counts in the figures, so check it before using it.
- **Sender**: CowSwap, LI.FI and Enso cannot price without a sender. With no `--sender` they are priced for a placeholder address: `placeholder_sender` at the top level lists them, each such quote and the best quote carry `placeholder_sender: true`, and they never return transaction data. Pass `--sender` with the wallet that will sign to get transaction data; the zero address and the placeholder itself are rejected.
- **Velora**: token decimals are looked up over RPC and sent with every request, so tokens outside Velora's own list are quoted on mainnet too.
- **Prop AMMs**: `propamm` reports the best price from the market-maker pools that block builders re-price each block. The figure comes from a builder's snapshot and only holds if the trade lands in a block from a builder carrying that maker's update, so it is conditional: it is listed in `conditional_quotes` (`COND` in the table, unranked) and is never `best_quote` or part of the summary figures. Ethereum mainnet only, wrapped tokens (ETH is quoted as WETH), no sender needed.
- **Velora Delta**: a second Velora source. Delta is an order the user signs and solvers fill, so its output already has the gas fee taken out and it returns no transaction data; the other sources report output before gas. On larger trades in thinner pools it can beat Velora's Market route
- **Ties**: when sources return exactly the same output, a quote priced for your sender (or needing none) is preferred over one priced for the placeholder; otherwise the first source listed wins.
- **Not applicable**: a source that cannot quote this swap is not a failure. Prop AMMs are only asked on Ethereum mainnet; on mainnet, a pair no maker quotes is listed in `not_applicable` (`N/A` in the table) and is not counted in `total_sources`.
- **Failed sources**: `failed_sources` lists each source that returned no quote and why (a missing API key, an upstream error). `quote best` prints them under the summary line.
- **LI.FI fee**: LI.FI takes a fixed fee out of the input before swapping, so its output is already net of it. The quote reports it as `fee_amount`/`fee_token`.

---

## Direct API Commands

### Alchemy - Alchemy API

Requires `ALCHEMY_API_KEY` environment variable.

```bash
# NFT queries
ethcli alchemy nft get-nfts 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
ethcli alchemy nft metadata 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d 1
ethcli alchemy nft contracts-for-owner 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
ethcli alchemy nft contract-metadata 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d
ethcli alchemy nft is-holder 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d

# Token data
ethcli alchemy token balances 0x...
ethcli alchemy token metadata 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Transfers
ethcli alchemy transfers all 0x...

# Debug traces
ethcli alchemy debug trace-tx 0x...

# Notify (dashboard: Webhooks) - needs the Webhooks auth token
ethcli alchemy notify list-webhooks            # alias: ethcli alchemy webhooks ...
# Gas Manager (dashboard: Gas Sponsorship) - needs an access key
ethcli alchemy gas-manager list-policies       # alias: ethcli alchemy gas-sponsorship ...
```

`alchemy notify` and `alchemy gas-manager` do not accept the app API key:

- **Notify / Webhooks**: copy the Auth Token from the AUTH TOKEN button at the top
  right of the dashboard Webhooks page (sidebar Data -> Webhooks,
  https://dashboard.alchemy.com/webhooks). Set it with
  `ethcli config set-alchemy-notify-token --stdin` or `ALCHEMY_NOTIFY_TOKEN`.
- **Gas Manager / Gas Sponsorship**: create an Access Key under dashboard ->
  Security with Gas Manager permissions (billing/team admins only). Set it with
  `ethcli config set-alchemy-access-key --stdin` or `ALCHEMY_ACCESS_KEY`.
Alchemy retired several NFT API endpoints on 2026-09-30, and the matching
`ethcli alchemy nft` subcommands were removed:

| Removed | Use instead |
|---------|-------------|
| `collections-for-owner` | `contracts-for-owner` |
| `collection-metadata`, `search-contract-metadata` | `contract-metadata` (OpenSea data is in `openSeaMetadata`) |
| `spam-contracts` | `is-spam <contract>` |
| `summarize-attributes`, `compute-rarity` | no direct replacement; aggregate `nfts-for-contract` output |
| `invalidate-contract` | no replacement; `refresh-metadata` still refreshes single tokens |
| `is-airdrop`, `sales` | no replacement |

`is-holder` is kept and now uses `getNFTsForOwner` filtered to the contract.

The Alchemy Transaction Simulation API was retired on the same date, so
`ethcli alchemy simulation asset-changes|execution` were removed. Use
`ethcli simulate call ... --via tenderly` (rich decoded output),
`--via debug` (`debug_traceCall` on any RPC), `--via alchemy` (Alchemy's
`debug_traceCall`), or `ethcli alchemy debug trace-call`.

### Gecko - CoinGecko API

Optional `COINGECKO_API_KEY` for Pro API (higher rate limits).

```bash
# Coin data
ethcli gecko coins get bitcoin
ethcli gecko coins --format json get ethereum

# Price queries
ethcli gecko simple price bitcoin,ethereum --vs usd,eur
ethcli gecko coins chart bitcoin --days 30

# Market data
ethcli gecko coins markets usd
ethcli gecko global trending

# NFT data
ethcli gecko nfts get bored-ape-yacht-club

# Tokenized real-world assets (list/markets/get/issuers work without a key)
ethcli gecko rwas list --asset-type stock
ethcli gecko rwas markets --asset-type etf
ethcli gecko rwas get 1-800-flowers-com --full
ethcli gecko rwas issuers
ethcli gecko rwas chart <rwa-id> --days 90         # Basic plan or above

# Onchain wallets (Analyst plan or above)
ethcli gecko wallets balances 0x... --networks eth,base
ethcli gecko wallets transfers eth 0x... --from 2026-11-01 --to 2026-11-07
ethcli gecko wallets trades eth 0x...
ethcli gecko wallets pnl 0x... --networks eth,base

# News, insights, supply breakdown (Analyst plan or above)
ethcli gecko news --coin bitcoin
ethcli gecko insights
ethcli gecko coins supply-breakdown uniswap

# Onchain multi-network (Enterprise plan) and trade ranges (Analyst plan)
ethcli gecko onchain token-price-multi eth:0x...,base:0x... --market-cap
ethcli gecko onchain tokens-multi eth:0x... --top-pools
ethcli gecko onchain pool-trades eth 0x... --from 2026-11-01 --to 2026-11-02
ethcli gecko onchain token-trades eth 0x... --from 2026-11-01 --to 2026-11-02
```

### Llama - DefiLlama API

Optional `DEFILLAMA_API_KEY` (or `ethcli config set-defillama`) for Pro endpoints.
`llama yields` needs it from 2026-11-13, when the free yields endpoints shut down.

```bash
# TVL data
ethcli llama tvl protocol aave
ethcli llama tvl protocols
ethcli llama tvl chains

# Prices
ethcli llama coins current ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli llama coins current ethereum:0x...,ethereum:0x...
ethcli llama coins chart ethereum:0x... --period 1d

# Yields
ethcli llama yields pools

# Yields v2 (Pro key required)
ethcli llama yields earn --chain ethereum --protocol aave-v3 --stablecoin true --limit 50
ethcli llama yields earn-history <pool-id> --range 30d
ethcli llama yields verified
ethcli llama yields verified-pool <pool-id>
ethcli llama yields borrow-markets
ethcli llama yields borrow-history <market-id> --range 90d
ethcli llama yields borrow-routes
ethcli llama yields loops

# Stablecoins
ethcli llama stablecoins list
```

With a key, `pro_routes = true` under `[defillama]` (or `DEFILLAMA_PRO_ROUTES=1`)
sends the free TVL, coins, volumes, fees and stablecoins endpoints to the Pro API.

### Moralis - Moralis API

Requires `MORALIS_API_KEY` environment variable.

Moralis removed Fantom support on 2026-05-29; `ethcli moralis` blocks Fantom
chain aliases. Moralis deleted its Discovery, Volume and Market Data APIs and
several ERC20 helper endpoints on 2026-06-04, and
`GET /erc20/{address}/holders/historical` on 2026-07-31. The corresponding
commands have been removed:

| Removed command | Replacement |
|-----------------|-------------|
| `moralis market *`, `moralis volume *` | `moralis token trending`, `moralis analytics batch\|timeseries` (Token Analytics) |
| `moralis discovery rising-liquidity\|buying-pressure\|solid-performers\|experienced-buyers\|risky-bets\|blue-chip\|top-gainers\|top-losers\|trending\|filter\|token` | `moralis token search` (Token Search, `/tokens/search`), `moralis token trending` |
| `moralis token stats`, `moralis token pairs-stats` | `moralis discovery token-analytics` (`/tokens/{address}/analytics`), `moralis token pair-stats` |
| `moralis token by-symbols` | `moralis token search <symbol>` |
| `moralis token holders-historical` | `moralis token holders-summary`, `moralis token holders` (no historical replacement) |
| `moralis token exchange-new-tokens\|exchange-bonding-tokens\|exchange-graduated-tokens\|bonding-status\|pair-snipers` | none documented |

`moralis discovery token-analytics` and `moralis discovery token-score` remain
(token scores are EVM-only). `moralis defi pair-price|pair-reserves|pair-address`
still work but call endpoints Moralis has dropped from its OpenAPI spec.

```bash
# Wallet data
ethcli moralis wallet balance 0x...
ethcli moralis wallet tokens 0x...
ethcli moralis wallet transactions 0x...
ethcli moralis token transfers 0x...
ethcli moralis wallet insight 0x... --chains eth,base --chain-breakdown

# Token score history (1d, 7d, 30d)
ethcli moralis discovery token-score-history 0x... --timeframe 30d

# NFT data
ethcli moralis nft list 0x...
ethcli moralis nft metadata 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d 1

# Token data
ethcli moralis token price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli moralis token metadata 0x...

# DeFi positions
ethcli moralis defi wallet-positions 0x...
```

### Dune SIM (removed)

Dune Sim (sim.dune.com) was shut down on 2026-08-01, so `ethcli dsim`,
`--source dsim`, `ethcli config set-dune-sim` and the `DUNE_SIM_API_KEY`
variable were removed (see <https://github.com/yldfi/yldfi-rs/issues/64>).
Use `ethcli portfolio` / `ethcli nfts` (Alchemy, Moralis) instead. Dune
Analytics (`ethcli dune`, `DUNE_API_KEY`) is unaffected.

### Dune - Dune Analytics API

Requires `DUNE_API_KEY` environment variable.

```bash
# Run queries
ethcli dune query execute 1234567
ethcli dune query execute 1234567 --params '{"address": "0x..."}'

# Get results
ethcli dune query results 1234567
ethcli dune query csv 1234567

# Executions
ethcli dune execution status <execution-id>
ethcli dune execution results <execution-id>

# Tables
ethcli dune tables list
```

### Curve - Curve Finance API

```bash
# Pool data
ethcli curve pools all
ethcli curve pools list ethereum

# Volume
ethcli curve volumes total ethereum

# Chain USD volume from prices.curve.finance (default: last 30 days, daily)
ethcli curve prices volume ethereum --interval day

# Lending
ethcli curve lending all
ethcli curve lending chain ethereum

# Token data
ethcli curve tokens list ethereum
ethcli curve prices token ethereum 0x...

# Router - find optimal swap routes
ethcli curve router route <from_token> <to_token> --chain ethereum --limit 5
ethcli curve router encode <from> <to> <amount> <min_out> --chain ethereum
ethcli curve router stats --chain ethereum
ethcli curve router address ethereum
```

### Exchange Data (`cex`, alias `ccxt`)

Market data from centralised exchanges and perp venues, via the `cexm` crate.

```bash
# Ticker (--exchange / -e defaults to binance)
ethcli cex ticker BTC/USDT
ethcli cex ticker ETH/USDT --exchange okx
ethcli cex ticker ETH/USD -e coinbase
ethcli cex ticker ETH/USDC:USDC -e hyperliquid   # perpetuals: BASE/USDC:USDC

# Order book, candles, recent trades
ethcli cex order-book BTC/USDT --limit 10
ethcli cex ohlcv BTC/USDT --timeframe 1h --limit 100
ethcli cex trades ETH/USDT -e bybit --limit 20

# Markets, and one pair across every exchange
ethcli cex markets --exchange kraken --quote USD
ethcli cex compare BTC/USDT
```

**Supported exchanges**: Binance, Bitget, OKX, Bybit, Coinbase and Kraken (spot), Hyperliquid and Lighter
(perpetuals). Public data only, no API keys. Binance is queried via its market-data
host `data-api.binance.vision`, which isn't region-locked. `ethcli ccxt` is an alias.

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
# Defaults to mainnet (https://api.dataengine.chain.link / wss://ws.dataengine.chain.link).
# For testnet, set CHAINLINK_REST_URL / CHAINLINK_WS_URL or
# `ethcli config set-chainlink --rest-url ... --ws-url ...`.
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

# LP position valuation (on-chain, no API key; honours --chain)
ethcli uniswap position 1234567                         # v3 NFT
ethcli uniswap position 12345 --version v4              # v4 NFT
ethcli uniswap position --v2 <pair> --owner <address>   # v2-style pair, also Sushi and other forks
ethcli uniswap lp-positions --owner <address> --json    # all open v3 positions (--all for closed)
ethcli uniswap position 1234567 --block 21000000 --no-price

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

### Bytecode Analysis - Function Selectors & Security Patterns

Analyze contract bytecode without requiring source code or ABI. Uses evmole for function extraction and evm-disassembler for opcode analysis. This is native bytecode analysis, not Solidity source reconstruction.

```bash
# Extract function selectors, arguments, and state mutability
ethcli contract selectors 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli contract sel 0x... --lookup  # Lookup signatures from 4byte.directory
ethcli contract sel 0x... --follow-proxy --lookup  # Follow proxy to implementation

# Disassemble bytecode into opcodes
ethcli contract disassemble 0x... --limit 50
ethcli contract dis 0x...

# Get opcode frequency statistics
ethcli contract opcodes 0x...
ethcli contract ops 0x...

# Full security analysis
ethcli contract analyze 0x...
ethcli contract az 0x... --include-disassembly --lookup

# Unverified/proxy contract review
ethcli contract analyze 0x... --follow-proxy --lookup --dispatcher --checks
```

**Security patterns detected**:
- **SELFDESTRUCT** (Critical) - Contract can be destroyed
- **DELEGATECALL** (High) - Arbitrary code execution risk
- **CALLCODE** (High) - Deprecated dangerous call
- **ORIGIN** (Medium) - tx.origin auth, honeypot indicator
- **CREATE/CREATE2** (Medium) - Dynamic contract creation

**Output includes**:
- Function selectors with arguments and mutability
- Optional proxy following for selector extraction (`contract selectors --follow-proxy`)
- Optional selector -> handler offset dispatcher map (`--dispatcher`)
- Optional handler guard/check heuristics (`--checks`) for caller gates, calldata hash checks, storage status checks, and external calls; findings are emitted for side-effecting bodies
- Risk level (LOW, MEDIUM, HIGH, CRITICAL)
- Dangerous opcode counts
- Hardcoded address detection
- Opcode frequency statistics

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
ethcli solodit search "access control" --protocol "Aave" --language Solidity,Vyper
ethcli solodit search "price" --min-quality 3 --sort quality
ethcli solodit search "oracle" --protocol-category DeFi --forked "Compound"
ethcli solodit search --impact HIGH --reported 30          # keywords are optional
ethcli solodit search "reentrancy" --reported-after 2024-01-01

# Get a specific finding (slug, ID, or solodit.cyfrin.io/issues/<slug> URL).
# Best-effort: the API has no get-by-id endpoint, so this searches and matches exactly.
ethcli solodit get <finding-slug>

# Check API rate limit
ethcli solodit rate-limit

# List tags and firms
ethcli solodit tags
ethcli solodit firms
```

On a 401 the server message is shown ("Missing API key" vs "Invalid API key").
If your key is rejected, check it at solodit.cyfrin.io (Profile > API Keys):
regenerating a key invalidates the previous one.

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

**Etherscan API coverage:** Etherscan has ended API support for Scroll (534352,
2026-04-16), Moonbeam (1284) and Moonriver (1285) (2026-07-31), and Swell
(1923, 2026-02-25) — see the [Etherscan changelog](https://docs.etherscan.io/changelog).
On those chains, Etherscan-backed commands (`contract abi/source/creation/call`,
`account` history, `gas`) fail fast with an explanatory error; RPC-based
commands keep working. If your Etherscan plan does not cover a chain or
endpoint, the error is reported as an "Etherscan plan restriction".

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
| `ETHCLI_NO_PROXY` | Optional | Set to `1`/`true` to disable HTTP proxy auto-detection |
| `TENDERLY_ACCESS_KEY` | `ethcli tenderly` | Tenderly API access |
| `ALCHEMY_API_KEY` | `ethcli alchemy`, aggregation | Alchemy API access |
| `ALCHEMY_NOTIFY_TOKEN` | `ethcli alchemy notify` | Webhooks auth token (dashboard Data -> Webhooks, AUTH TOKEN button) |
| `ALCHEMY_ACCESS_KEY` | `ethcli alchemy gas-manager` | Access key with Gas Manager (Gas Sponsorship) permissions (dashboard -> Security) |
| `COINGECKO_API_KEY` | Optional | CoinGecko Pro API (higher rate limits) |
| `DEFILLAMA_API_KEY` | Pro endpoints; DefiLlama yields from 2026-11-13 | DefiLlama API plan key; or `ethcli config set-defillama --stdin` |
| `DEFILLAMA_PRO_ROUTES` | Optional | `1`/`true`: route `ethcli llama` free endpoints through the Pro API (needs a key) |
| `MORALIS_API_KEY` | `ethcli moralis` | Moralis API access |
| `DUNE_API_KEY` | `ethcli dune` | Dune Analytics queries |
| `THEGRAPH_API_KEY` | Uniswap subgraph | The Graph API access |
| `CHAINLINK_API_KEY` | `chainlink streams` | Chainlink Data Streams (premium) |
| `CHAINLINK_USER_SECRET` | `chainlink streams` | Chainlink Data Streams (premium) |
| `GOPLUS_APP_KEY` | Optional | GoPlus batch queries (higher rate limits) |
| `GOPLUS_APP_SECRET` | Optional | GoPlus batch queries (higher rate limits) |
| `SOLODIT_API_KEY` | `ethcli solodit` | Solodit vulnerability database |
| `PYTH_API_KEY` | `ethcli pyth price`, `price --source pyth` | Pyth Hermes (required since the Pyth Core upgrade; or `ethcli config set-pyth`) |
| `ONEINCH_API_KEY` / `1INCH_API_KEY` | `ethcli 1inch` | 1inch DEX Aggregator (required) |
| `ZEROX_API_KEY` / `0X_API_KEY` | `ethcli 0x` | 0x Protocol (optional, higher limits) |
| `LIFI_INTEGRATOR` | `ethcli lifi` | LI.FI analytics tracking (optional) |
| `KYBERSWAP_API_KEY` | `ethcli kyber`, `quote` | KyberSwap API gateway key (optional; without it the keyless 3 rps API is used) |
| `LIFI_API_KEY` | `ethcli lifi`, `quote` | LI.FI API key (optional; without it limits are per IP) |
| `PARASWAP_API_KEY` / `VELORA_API_KEY` | `ethcli velora` | ParaSwap (optional, higher limits) |
| `ENSO_API_KEY` | `ethcli enso` | Enso Finance (required) |
| `PROPAMM_DATA_URL` | Optional | Data endpoint for the `propamm` quote source (default: Titan Builder, EU) |

---

## Direct DEX Aggregator Commands

### 1inch - DEX Aggregator

Direct access to the 1inch Swap API v6.1. Requires `ONEINCH_API_KEY` or `1INCH_API_KEY` environment variable.

```bash
# Get swap quote (price estimation)
ethcli 1inch quote <src_token> <dst_token> <amount> --chain-id 1
ethcli 1inch quote 0xEee...EEeE 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 1000000000000000000 --include-gas

# Get swap transaction data
ethcli 1inch swap <src> <dst> <amount> <from_address> --chain-id 1 --slippage 1
ethcli 1inch swap 0xEee...EEeE 0xA0b8... 1000000000000000000 0xYourAddress --receiver 0x...

# Get supported tokens
ethcli 1inch tokens --chain-id 1

# Get liquidity sources
ethcli 1inch sources --chain-id 1

# Get approval spender address
ethcli 1inch spender --chain-id 1

# Check token allowance
ethcli 1inch allowance <token> <wallet> --chain-id 1

# Get approval transaction
ethcli 1inch approve <token> --chain-id 1 --amount 1000000
```

**Alias**: `ethcli oneinch`

### De¹ (formerly OpenOcean) - DEX Aggregator

Direct access to the De¹ (formerly OpenOcean) v4 API (`open-api.de1.exchange`). No API key required. Quote/swap amounts are
in smallest units (sent as `amountDecimals`); `--gas-price` is in gwei and is
converted to wei (`gasPriceDecimals`).

```bash
# Get swap quote
ethcli open-ocean quote <in_token> <out_token> <amount> --chain ethereum
ethcli open-ocean quote 0xEee...EEeE 0xA0b8... 1000000000000000000 --slippage 1 --gas-price 30
ethcli open-ocean quote ETH USDC 1000000000000000000 --chain ethereum   # symbols resolve per chain

# Get swap transaction data (/v4/{chain}/swap)
ethcli open-ocean swap <in_token> <out_token> <amount> <account> --chain ethereum

# Get reverse quote (output amount is human-readable, e.g. 1 = 1 token)
ethcli open-ocean reverse-quote <in_token> <out_token> <out_amount> --chain ethereum

# Get supported tokens
ethcli open-ocean tokens --chain ethereum

# Get DEX sources
ethcli open-ocean dexes --chain ethereum
```

**Aliases**: `ethcli oo`, `ethcli de1`

### KyberSwap - DEX Aggregator

Direct access to KyberSwap API. No API key required.

```bash
# Get optimal swap routes
ethcli kyber-swap routes <token_in> <token_out> <amount_in> --chain ethereum
ethcli kyber-swap routes 0xC02...Cc2 0xA0b8... 1000000000000000000 --save-gas

# Get route with full data
ethcli kyber-swap route-data <token_in> <token_out> <amount_in> --chain ethereum

# Build swap transaction from route
ethcli kyber-swap build <token_in> <token_out> <amount_in> <sender> <recipient> \
  --chain ethereum --slippage-bps 50 --route-summary '<json>'

# Route + calldata in one call (KyberSwap restricts /swap to partners:
# pass --client-id or set KYBERSWAP_CLIENT_ID; recipient defaults to sender)
ethcli kyber-swap swap <token_in> <token_out> <amount_in> <sender> [recipient] \
  --chain ethereum --slippage-bps 50 --client-id <partner-id>
```

**Alias**: `ethcli kyber`

### 0x Protocol - DEX Aggregator

Direct access to 0x API. Requires an API key: `[zerox]` in config, or `ZEROX_API_KEY` / `0X_API_KEY`.

```bash
# Get swap quote with transaction data
ethcli 0x quote <sell_token> <buy_token> <sell_amount> <taker> --chain ethereum
ethcli 0x quote 0xEee...EEeE 0xA0b8... 1000000000000000000 0xYourAddress --slippage-bps 100

# Get price estimate (lighter, no tx data)
ethcli 0x price <sell_token> <buy_token> <sell_amount> <taker> --chain ethereum

# Use the AllowanceHolder flow (plain ERC-20 approval) instead of Permit2
ethcli 0x quote <sell_token> <buy_token> <sell_amount> <taker> --allowance-holder
ethcli 0x price <sell_token> <buy_token> <sell_amount> <taker> --allowance-holder

# Get liquidity sources
ethcli 0x sources --chain ethereum

# Supported chains (Swap API, or Gasless API with --gasless)
ethcli 0x chains
ethcli 0x chains --gasless

# Gasless swaps (ERC-20 sell token; gas paid from the sell token)
ethcli 0x gasless-price <sell_token> <buy_token> <sell_amount> <taker> --chain base
ethcli 0x gasless-quote <sell_token> <buy_token> <sell_amount> <taker> --chain base
ethcli 0x gasless-status <trade_hash> --chain base
```

**Alias**: `ethcli zerox`

### CowSwap - MEV-Protected Trading

Direct access to CoW Protocol API. No API key required.

```bash
# Get MEV-protected swap quote
ethcli cow-swap quote <sell_token> <buy_token> <amount> <from> --chain ethereum
ethcli cow-swap quote 0xC02...Cc2 0xA0b8... 1000000000000000000 0xYourAddress --kind sell

# Get order by UID
ethcli cow-swap order <uid> --chain ethereum

# Get orders for an address
ethcli cow-swap orders <owner> --chain ethereum

# Order competition status (solvers that bid on it)
ethcli cow-swap order-status <uid> --chain ethereum

# Orders settled in a transaction / several orders by UID (max 128)
ethcli cow-swap orders-by-tx <tx_hash> --chain ethereum
ethcli cow-swap orders-by-uids --uid <uid1>,<uid2> --chain ethereum

# Full appData document for an appData hash
ethcli cow-swap app-data <app_data_hash> --chain ethereum

# Total surplus earned by a user (endpoint marked unstable by CoW)
ethcli cow-swap surplus <address> --chain ethereum

# Get trades for an address (paginated via /api/v2/trades; API default limit 10)
ethcli cow-swap trades <owner> --chain ethereum --limit 100 --offset 0

# Get trades for an order
ethcli cow-swap order-trades <uid> --chain ethereum

# Get current auction
ethcli cow-swap auction --chain ethereum

# Get solver competition (v2 API): by auction ID, by settlement tx, or latest
ethcli cow-swap competition <auction_id> --chain ethereum
ethcli cow-swap competition --tx-hash <settlement_tx> --chain ethereum
ethcli cow-swap competition --chain ethereum

# Get native token price
ethcli cow-swap native-price <token> --chain ethereum

# Cancel orders (signature over OrderCancellations(bytes[] orderUids))
ethcli cow-swap cancel-orders --uid <uid1>,<uid2> <signature> --chain ethereum
```

**Alias**: `ethcli cow`

### LI.FI - Cross-Chain Aggregator

Direct access to LI.FI API for cross-chain swaps and bridges. Optional `LIFI_INTEGRATOR` for analytics and `LIFI_API_KEY` for per-key rate limits.

```bash
# Get swap/bridge quote
ethcli lifi quote <from_chain> <from_token> <to_chain> <to_token> <amount> <from_address>
ethcli lifi quote 1 0xA0b8... 137 0x2791... 1000000000 0xYourAddress --slippage 0.5

# Quote by output amount: how much to send to receive exactly <to_amount>
ethcli lifi quote-to-amount <from_chain> <from_token> <to_chain> <to_token> <to_amount> <from_address>
ethcli lifi quote-to-amount 1 USDC 42161 USDC 10000000 0xYourAddress

# Bridge/swap, then run contract calls on the destination chain (beta)
ethcli lifi contract-calls <from_chain> <from_token> <to_chain> <to_token> <to_amount> <from_address> \
  --calls '[{"fromAmount":"...","fromTokenAddress":"0x...","toContractAddress":"0x...","toContractCallData":"0x...","toContractGasLimit":"..."}]'

# Get multiple routes
ethcli lifi routes <from_chain> <from_token> <to_chain> <to_token> <amount> <from_address>

# Get best route
ethcli lifi best-route <from_chain> <from_token> <to_chain> <to_token> <amount> <from_address>

# Get transaction status
ethcli lifi status <tx_hash> --from-chain 1 --to-chain 137

# Transfers sent or received by a wallet (last 30 days unless --from-timestamp;
# page with the response's next/previous cursors)
ethcli lifi transfers <wallet> --status all --limit 20
ethcli lifi transfers <wallet> --from-timestamp 1700000000 --next <cursor>

# LI.Fuel gas refuel status
ethcli lifi gas-status <tx_hash>

# List supported chains
ethcli lifi chains

# Get chain details
ethcli lifi chain <chain_id>

# List tokens for a chain
ethcli lifi tokens --chain-id 1

# Look up one token by symbol or address
ethcli lifi token 1 USDC

# List available tools (bridges/exchanges)
ethcli lifi tools

# List bridges
ethcli lifi bridges

# List exchanges
ethcli lifi exchanges

# Get gas prices
ethcli lifi gas <chain_id>

# List connections (available routes)
ethcli lifi connections --from-chain 1 --to-chain 137
```

**Alias**: `ethcli li.fi`

### Velora - ParaSwap DEX Aggregator

Direct access to the Velora (formerly ParaSwap) API at `api.velora.xyz`; prices request Augustus v6.2 routes. Optional `PARASWAP_API_KEY` or `VELORA_API_KEY` for higher rate limits.

```bash
# Get swap price/route
ethcli velora price <src_token> <dest_token> <amount> --chain ethereum --side SELL
ethcli velora price 0xC02...Cc2 0xA0b8... 1000000000000000000 --src-decimals 18 --dest-decimals 6
# --src-decimals/--dest-decimals are looked up over RPC when omitted (every chain),
# so tokens outside Velora's list work without them

# Build swap transaction
ethcli velora transaction <user_address> --chain ethereum --slippage 100 --price-route '<json>'

# Price route + transaction in one call (lower rate limits, no RFQ liquidity)
ethcli velora swap <src_token> <dest_token> <amount> <user_address> --chain ethereum --slippage 100

# List supported tokens
ethcli velora tokens --chain ethereum
```

**Alias**: `ethcli paraswap`

### Enso Finance - DeFi Aggregator

Direct access to Enso Finance API. Requires `ENSO_API_KEY` environment variable.

```bash
# Get swap/DeFi route
ethcli enso route <token_in> <token_out> <amount_in> <from_address> --chain-id 1
ethcli enso route 0xC02...Cc2 0xA0b8... 1000000000000000000 0xYourAddress --slippage 50

# Get token price in USD
ethcli enso price <token> --chain-id 1

# Get token balances for an address
ethcli enso balances <address> --chain-id 1

# Quote (output amount and route, no tx data); sender is optional
ethcli enso quote <token_in> <token_out> <amount_in> [from_address] --chain-id 1

# Prices of several tokens on one chain (max 100)
ethcli enso prices <token1> <token2> --chain-id 1

# Search tokens (filters: --chain-id, --project, --protocol, --address, --symbol,
# --underlying, --type defi|base, --apy-from/--apy-to, --tvl-from/--tvl-to, paging)
ethcli enso tokens --chain-id 1 --protocol aave-v3 --include-metadata

# Networks and protocols
ethcli enso networks
ethcli enso protocols --chain-id 1

# Approval transaction for Enso's router, and the Enso smart wallet of an address
ethcli enso approve <token> <amount> <from_address> --chain-id 1
ethcli enso wallet <from_address> --chain-id 1
```

### Pyth Network - Price Feeds

Direct access to the Pyth Network Hermes API (`https://pyth.dourolabs.app/hermes`).

**API key required**: since the Pyth Core upgrade (2026-08-26) Hermes requires an
API key on every request. Get one from the Pyth Terminal (<https://pythdata.app>),
then either run `ethcli config set-pyth <key>` (or `echo $KEY | ethcli config set-pyth --stdin`)
or set `PYTH_API_KEY`. The config file key takes precedence over the environment variable.

```bash
# Get latest price for one or more feeds
ethcli pyth price BTC/USD
ethcli pyth price ETH SOL AVAX

# Price at a past time (first update published at or after the unix timestamp)
ethcli pyth price ETH/USD --at 1704067200

# Search for price feeds
ethcli pyth search "BTC"
ethcli pyth search "ETH/USD"

# List all available price feed IDs
ethcli pyth feeds
ethcli pyth feeds --asset-type crypto

# Get well-known feed IDs
ethcli pyth known-feeds

# Latest publisher stake caps update
ethcli pyth stake-caps
```

**Notes**:
- Supports common symbols: BTC, ETH, SOL, USDC, USDT, DAI, AVAX, ARB, OP, LINK, UNI, AAVE, CRV, CVX, etc.,
  names and wrapped forms (`bitcoin`, `WBTC`, `WETH`) and pair forms (`ETH/USD`, `ETH-USD`, `ETHUSD`).
  `ethcli price --source pyth` uses the same mapping (see the pythc README for the full list)
- Use `ethcli pyth search` to find additional feeds by name
- `ethcli pyth price` fails with setup instructions when no key is configured; `known-feeds` works offline
- In `ethcli price` aggregation, Pyth is reported as `PYTH_API_KEY not configured` and skipped when no key is set

## License

MIT
