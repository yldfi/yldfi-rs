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

## Available Commands

### Core Ethereum Commands
```
ethcli logs       # Fetch historical logs from contracts
ethcli tx         # Analyze transaction(s)
ethcli account    # Account operations (balance, transactions, transfers)
ethcli address    # Address book (save and lookup addresses by label)
ethcli contract   # Contract operations (ABI, source, creation, bytecode analysis)
ethcli token      # Token operations (info, holders, balance)
ethcli gas        # Gas price oracle and estimates
ethcli sig        # Signature lookup (function selectors, event topics)
ethcli simulate   # Transaction simulation and tracing
ethcli cast       # Type conversions, hashing, encoding
ethcli rpc        # Direct RPC calls
ethcli ens        # ENS name resolution
ethcli endpoints  # Manage RPC endpoints
ethcli config     # Manage configuration
ethcli update     # Check for updates
ethcli upgrade    # Download and install the latest release (ethcli + ethcli-mcp; symlink/wrapper-safe with rollback since 0.23.5)
ethcli doctor     # Diagnose configuration and connectivity
ethcli chainlist  # Browse chains and discover RPC endpoints from chainlist.org
ethcli completions # Generate shell completions (bash, zsh, fish, powershell)
```

### Aggregation Commands (parallel queries to multiple APIs)
```
ethcli price      # Token prices from CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, exchanges
ethcli portfolio  # Portfolio balances from Alchemy, Moralis
ethcli nfts       # NFT holdings from Alchemy, CoinGecko, Moralis
ethcli yields     # DeFi yields from DefiLlama and Curve
ethcli quote      # Swap quotes from OpenOcean, KyberSwap, 0x, 1inch, CowSwap, LI.FI, Velora (Market and Delta), Enso, prop AMMs
```

### Direct API Access Commands
```
ethcli tenderly   # Tenderly API (vnets, wallets, contracts, alerts, actions)
ethcli alchemy    # Alchemy API (NFTs, prices, portfolio, transfers, debug)
ethcli gecko      # CoinGecko API (coins, prices, NFTs, onchain DEX data)
ethcli llama      # DefiLlama API (TVL, prices, yields, stablecoins)
ethcli moralis    # Moralis API (wallet, token, NFT, DeFi, transactions)
ethcli dune       # Dune Analytics API (queries, executions, tables)
ethcli curve      # Curve Finance API (pools, volumes, lending, tokens, router)
ethcli chainlink  # Chainlink price feeds (RPC-based, no API key needed)
ethcli cex       # Exchange data (Binance, Bitget, OKX, Bybit, Coinbase, Kraken, Hyperliquid, Lighter)
ethcli kong       # Yearn Kong API (vaults, strategies, prices, TVL, reports)
ethcli uniswap    # Uniswap V2/V3/V4 (on-chain lens + subgraph)
ethcli goplus     # GoPlus Security API (token/address/NFT/approval security)
ethcli solodit    # Solodit vulnerability database (smart contract security findings)
```

### Direct DEX Aggregator Commands
```
ethcli 1inch      # 1inch API (quote, swap, tokens, sources, approvals) - requires API key
ethcli open-ocean # De¹/OpenOcean API (quote, swap, reverse-quote, tokens, dexes)
ethcli kyber-swap # KyberSwap API (routes, route-data, build, swap)
ethcli 0x         # 0x Protocol API (quote, price, sources, chains, gasless) - requires API key
ethcli cow-swap   # CowSwap API (quote, orders, order status, trades, auctions, app data, surplus) - MEV protected
ethcli lifi       # LI.FI API (quote, quote-to-amount, routes, status, transfers, chains, tokens, bridges)
ethcli velora     # Velora/ParaSwap API (price, transaction, swap, tokens)
ethcli enso       # Enso Finance API (route, quote, prices, tokens, protocols, approve, wallet) - requires API key
ethcli pyth       # Pyth Network API (price feeds incl. historical, search, known-feeds, stake caps)
```

### Security & Token Analysis
```
ethcli blacklist  # Token blacklist management (spam/scam filtering)
```

## RPC Read Commands

Direct chain reads. These are the commands to reach for when a value is needed
from a contract or an account; they return decoded, machine-readable output so
nothing has to be converted or re-parsed afterwards.

```bash
# One call, decoded. Give the return type with -d
ethcli rpc call <addr> -s "balanceOf(address)" -a <holder> -d uint256
ethcli rpc call <addr> -s "totalSupply()" -d uint256 --block 21000000 --json

# Call by function name using the contract's ABI (fetched automatically)
ethcli contract call <addr> totalSupply
ethcli contract call <token> balanceOf <holder> --json   # adds decimals and the scaled amount

# Many calls at one block in a single request (Multicall3), in input order
ethcli rpc multicall "<addr> totalSupply()(uint256)" "<addr> balanceOf(address)(uint256) <holder>"
ethcli rpc multicall -f calls.txt --block 21000000 --json      # one call per line; "-" reads stdin
ethcli rpc multicall "<vault> pricePerShare()(uint256)" --block-range 21000000:21070000:7200

# Transaction, balance, nonce, storage, code
ethcli rpc tx <hash> --json
ethcli rpc receipt <hash> -o json
ethcli rpc balance <addr> --block 21000000 --json     # wei and ether
ethcli rpc nonce <addr>
ethcli rpc storage <addr> <slot> --json                # value as hex and decimal
ethcli rpc code <addr> --summary --json                # size, code hash, EIP-7702 delegate; no bytecode

# Block for a time, and a time in place of a block
ethcli rpc block-at 2024-01-31                          # latest block at or before that time
ethcli rpc balance <addr> --block 2024-01-31
ethcli contract call <addr> totalSupply --block 7d
```

### `tx`: one call for a transaction

`ethcli tx <hash>` returns the decoded call, events and token transfers, and also:

- `transaction_index`, `effective_gas_price`, `fee_wei` and `fee`;
- `fee_breakdown`: execution fee, L1 data fee and total. On OP-stack chains
  the receipt's `l1Fee` is added, so `fee_wei` is the total paid; on Arbitrum
  the L1 part (`gasUsedForL1` × price) is already in `gasUsed` and is split
  out; on L1 chains `l1_fee_wei` is null (not applicable). `--full` adds
  `fee_breakdown.usd` at the block's time. `rpc receipt -o json` returns the
  raw receipt (all chain fields) plus `feeBreakdown`. For an estimate before
  sending: `ethcli gas cost <to> [data|-s sig -a args] --from <addr>`;
- `balance_changes`: the net change per address and token over the whole
  transaction, signed, in raw units and scaled by decimals.
  - Token lines come from Transfer events, plus deposits into and withdrawals
    from the chain's wrapped native token (which mint and burn it without a
    Transfer event). They are complete.
  - Native-token lines are exact only with a call trace (`--full`). Without
    one they are built from the top-level value, wrapped-native deposits and
    withdrawals, and the fee, and each carries `"partial": true`: a transfer
    made inside the call, such as a router paying out ETH, is not seen, so a
    contract or the sender can be wrong. Do not treat a partial line as final.
  - NFT transfers and the zero address are left out.

`--full` adds, for up to three more requests:

- a call trace (`debug_traceTransaction`), when a configured endpoint supports
  tracing. Native-token lines then include transfers made inside the call and
  lose the `partial` flag. If no trace is available a note says so and the
  lines stay partial;
- `timestamp` and `time`, the block's time;
- `revert`, for a failed transaction: the decoded reason, with `revert_from`
  saying where it came from.
  - `"trace"`: the revert data of the transaction as it actually ran, read
    from the call trace. This is the real reason.
  - `"replay"`: used only when no trace is available. The call is re-run
    against the parent block's state, which is not the state the transaction
    ran in. If the failure depended on an earlier transaction in the same
    block (someone else taking an auction first) or on the block's time, the
    replay can fail for a different reason. Treat it as a lead, not the answer.

```bash
ethcli tx <hash> --full
ethcli tx <hash> --full --json
```

This replaces `tx` followed by `rpc receipt` and `rpc block`.

With a trace, `native_transfer_list` lists every native-token transfer in the
transaction. `--graph` prints a Mermaid flowchart instead of the analysis: one
node per address (the sender first), one edge per token transfer, wrap or
unwrap, and native transfer. Without a trace, native transfers made inside the
call are missing and a comment in the diagram says so.

### Account history and identity

```bash
ethcli account txs <addr> --all --json          # whole history; --max-pages N caps requests (default 10)
ethcli account internal-txs <addr> --all --json
ethcli account erc20 <addr> --all --json
ethcli account funded-by <addr>                 # first funder
ethcli account labels <addr> <addr> ...         # up to 100
```

- `--all` reads 1,000 rows per request and walks past the explorer's
  10,000-row window by moving the block range, re-reading the boundary block
  so nothing is missed or repeated. It stops early at `--max-pages` (default
  10, so 10,000 rows) or when a single block holds more than 10,000 rows.
- With `--all` the JSON is an object, not a bare array:
  `{"complete": bool, "requests": n, "stopped": "...", "count": n, "rows": [...]}`.
  Check `complete` before treating the rows as the whole history; with
  `-o ndjson` the last line is `{"summary": true, "complete": ...}`.
- `--csv` on `txs`, `internal-txs` and `erc20` prints CSV with a header line;
  nested fields become `parent.child` columns. CSV cannot carry the `--all`
  completeness flag, so an incomplete walk is reported on stderr only; use
  `-o json` when completeness matters. `rpc multicall --csv` prints one row
  per call and block with each decoded output in its own column
  (`output_0`, `output_1`, ...).
- Cells a spreadsheet would run as a formula (starting `=`, `+`, `-`, `@`)
  get a leading `'`, except plain numbers, since values can come from
  contracts.
- `funded-by` uses Etherscan's `fundedby` endpoint (paid plan, EOAs only). On
  a free plan, or when that fails, it reports the earliest incoming
  native-token transfer among the first 100 normal and 100 internal
  transactions (`source: first_incoming_transfer`); token transfers are not
  considered.
- `labels` combines Etherscan name tags and labels (`getaddresstag`, Pro Plus
  plan) with the address book and ethcli's built-in labels. Without that plan
  a note says why the explorer part is missing.

### `contract inspect`

One call that answers "what is this address":

```bash
ethcli contract inspect <addr>
ethcli contract inspect <addr> --json
```

- **Kind:** EOA, contract, or an EOA that delegates its code under EIP-7702.
- **Proxy:** kind (EIP-1167, EIP-1967, beacon, OpenZeppelin legacy, Safe proxy, or
  an `implementation()` getter), implementation, admin and beacon.
- **Explorer facts:** verified name and creator.
- **Standard getters,** read together at one block: token name, symbol, decimals
  and supply; `owner()`; Safe owners and threshold; ERC-4626 `asset()`.
- **Functions:** from the verified ABI of the code that runs (the implementation,
  for a proxy). When there is no verified ABI the selectors are recovered from
  bytecode and named from the signature database; those names are guesses, and
  `interface.source` is `bytecode`.
- Anything that could not be fetched is listed under `notes`; the command does
  not fail because one part is missing.

It replaces the usual chain of `rpc code`, `contract selectors`, `contract abi`,
`contract creation` and `token info`.

### `rpc multicall`

- Each call is `<address> <signature> [args...]`. Return types in the signature
  (`balanceOf(address)(uint256)`) are what get decoded; without them the raw
  data is returned. Arguments are split on whitespace.
- `latest`, `safe` and `finalized` are resolved to a block number once, so every
  call reads the same state.
- `--block a,b,c` or `--block-range FROM:TO:STEP` repeats the same calls at each
  block (at most 1,000 blocks).
- Calls go through Multicall3, so `msg.sender` is that contract. Where
  Multicall3 is not available (a block before its deployment, or a chain
  without it) each call is sent separately; the output's `via` field says which.
- Each result has `success`, `raw`, decoded `outputs`, and a `revert` object when
  the call reverted.

### JSON output

- `--json` works on every command. It selects the command's own JSON output
  (`-o json`); on a command with no output option it is ignored with a note on
  stderr.
- Integers are decimal strings, since a uint256 does not fit a JSON number.
- `account` subcommands accept `-o ndjson` for one object per line.

### Times in place of blocks

`--block` on `rpc call`, `storage`, `code`, `nonce`, `balance`, `block`,
`multicall` and on `contract call` accepts:

| Form | Example |
|---|---|
| Date (midnight UTC) | `2024-01-31` |
| RFC 3339 | `2024-01-31T12:00:00Z` |
| Date and time (UTC) | `2024-01-31 12:00` |
| Unix timestamp | `@1706700000` |
| Age | `90s`, `30m`, `12h`, `7d`, `2w` |

The block used is the latest one at or before that time, found by binary search
over block headers, so it is exact. A bare number is still a block number.

A plain date is the **start** of that day in UTC. `--block 2024-04-05` reads the
state before anything that happened on 5 April, so it is the close of 4 April.
For an end-of-day or year-end figure, pass the next day's date or a full time
such as `2024-04-05T23:59:59Z`. All times are UTC: local midnight in a zone one
hour ahead is `23:00:00Z` on the previous day. The resolved block and its time
are printed on stderr (and `rpc block-at --json` returns them), so the choice
can be checked.

### Reverts

A reverted `rpc call`, `contract call` or `rpc multicall` entry reports why:

```
Error: Call reverted: "ERC20: transfer amount exceeds balance"
Error: Call reverted: AllowanceExpired(deadline=0)
Error: Call reverted: AllowanceExpired(0) (name from the signature database, not the contract)
```

With `--json` the output has `"reverted": true` and a `revert` object with
`kind`, `source`, `selector`, `signature`, `args`, `message` and `raw`.

- `source` says where the name came from: `builtin` (`Error(string)`,
  `Panic(uint256)`, no data), `abi` (the contract's own ABI, used by
  `contract call`), `signature_database`, or `none`.
- A `signature_database` name is a best guess. Selectors collide and the
  database holds junk entries; `other_candidates` lists other signatures whose
  types also decode the data.
- `ethcli sig error <data>` decodes revert data given on the command line.

### Endpoints

No `--rpc-url` is needed for past blocks or traces: an archive or debug-capable
endpoint is chosen automatically, in priority order from the config.
`--rpc-url` (on `rpc`, `contract call`, `tx`, `logs`, `simulate`) is for a local
fork or another specific endpoint. `ethcli endpoints list` shows hostnames and
capabilities; it does not print credentials, so a URL copied from it may not
work.

### Small helpers

```bash
ethcli sig lookup <hex>                       # 4-byte selector or 32-byte topic, picked by length
ethcli sig error <revert data>                # decode Error, Panic or a custom error
ethcli cast index <key_type> <key> <slot>     # storage slot of a mapping entry
ethcli tx <hash> --rpc-url <url>              # analyze against one endpoint
```

The JSON-RPC method names are accepted as aliases of the `rpc` subcommands
(`rpc eth_getTransactionReceipt <hash>`), as are `cast selector`, `cast
from-hex`, `cast calldata`, `account transactions` and `address lookup`.

## Bytecode Analysis Commands

Analyze contract bytecode without requiring source code or ABI. Uses evmole for function extraction and evm-disassembler for opcode analysis. This does not reconstruct Solidity source.

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

# Full security analysis (combines all above)
ethcli contract analyze 0x...
ethcli contract az 0x... --include-disassembly --lookup

# Analyze proxy's implementation contract instead of proxy bytecode
ethcli contract analyze 0x... --follow-proxy

# Unverified/proxy review with selector handler mapping and guard heuristics
ethcli contract analyze 0x... --follow-proxy --lookup --dispatcher --checks

# JSON output
ethcli contract analyze 0x... --format json
```

### Proxy Detection

The analyze command automatically detects proxy contracts:
- **EIP-1167**: Minimal proxy clones (implementation embedded in bytecode)
- **EIP-1967**: Transparent proxies (implementation in storage slot)
- **EIP-1822**: UUPS proxies
- **OpenZeppelin Legacy**: Pre-EIP-1967 patterns

Use `--follow-proxy` to analyze the implementation contract instead of the proxy bytecode.

### Security Patterns Detected

| Opcode | Risk | Description |
|--------|------|-------------|
| SELFDESTRUCT | Critical | Contract can be destroyed, potentially locking user funds |
| DELEGATECALL | High | Arbitrary code execution - can modify contract state via external code |
| CALLCODE | High | Deprecated opcode with similar risks to DELEGATECALL |
| ORIGIN | Medium | Uses tx.origin for authorization - honeypot indicator |
| CREATE | Medium | Dynamically creates contracts |
| CREATE2 | Medium | Deterministic contract creation |

### Output Includes

- **Functions**: Selectors, arguments, state mutability (pure/view/nonpayable/payable)
- **Risk Level**: LOW, MEDIUM, HIGH, or CRITICAL based on detected patterns
- **Dangerous Opcodes**: Count of SELFDESTRUCT, DELEGATECALL, CALLCODE
- **Hardcoded Addresses**: Count of PUSH20 operations (embedded addresses)
- **Opcode Statistics**: Frequency of each opcode type
- **Proxy Info**: Detected proxy type and implementation address (if applicable)

### Notes

- No API key required - works directly with RPC bytecode
- Aliases: `sel`, `dis`, `ops`, `az`, `an`
- Works on unverified contracts (no source/ABI needed)

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

# JSON output (use --format or -o)
ethcli goplus token 0x... --chain-id 1 --format json
ethcli goplus token 0x... --chain-id 1 -o ndjson
```

### Notes

- **Free API**: No API key required for basic usage
- **Authenticated mode**: Set `GOPLUS_APP_KEY` and `GOPLUS_APP_SECRET` for batch queries and higher rate limits (without them `token-batch` queries each address individually and warns about addresses with no data)
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

# Show the blacklist file path
ethcli blacklist path
```

### Notes

- **Storage**: Blacklist stored in `~/.config/ethcli/blacklist.toml`
- **Security checks**: Uses GoPlus API + Etherscan verification status
- **Known protocols**: a fixed list of major token addresses (stETH, DAI, CRV, ...) skips the proxy warnings; names and symbols are never trusted, and verification status is never overridden
- **Alias**: `ethcli bl` works as an alias for `ethcli blacklist`

## Solodit Commands

Search and explore smart contract security vulnerability findings from the Solodit database (by Cyfrin).

```bash
# Search for vulnerability findings
ethcli solodit search "reentrancy"
ethcli sld search "oracle manipulation" --impact HIGH,MEDIUM
ethcli sld search "flash loan" --firm "Trail of Bits" --tag Reentrancy

# Filter by various criteria
ethcli sld search "access control" --protocol "Aave" --language Solidity
ethcli sld search "price" --min-quality 3 --sort quality

# Advanced filtering
ethcli sld search "oracle" --protocol-category DeFi,Bridge
ethcli sld search "reentrancy" --finder "username" --min-finders 1 --max-finders 3
ethcli sld search "flash loan" --reported 30                    # Last 30 days
ethcli sld search "access control" --reported-after 2024-01-01  # After specific date
ethcli sld search "price" --min-quality 3 --min-rarity 2
ethcli sld search "oracle" --sort quality --sort-dir asc

# Get a specific finding by slug
ethcli sld get <finding-slug>
ethcli sld get <finding-slug> --format json

# Check API rate limit status
ethcli sld rate-limit

# List common vulnerability tags
ethcli sld tags

# List common audit firms
ethcli sld firms

# Pagination
ethcli sld search "reentrancy" --page 2 --limit 50
```

### Search Filters

| Flag | Description | Example |
|------|-------------|---------|
| `--impact`, `-i` | Impact level(s) | `--impact HIGH,MEDIUM` |
| `--firm`, `-f` | Audit firm(s) | `--firm "Cyfrin,Sherlock"` |
| `--tag`, `-t` | Vulnerability tag(s) | `--tag Reentrancy,Oracle` |
| `--protocol` | Protocol name (partial) | `--protocol Aave` |
| `--protocol-category` | Protocol category(ies) | `--protocol-category DeFi` |
| `--forked` | Forked protocol(s) | `--forked Compound` |
| `--language` | Programming language(s) | `--language Solidity,Vyper` |
| `--finder` | Auditor handle (partial) | `--finder username` |
| `--min-finders` | Min number of finders | `--min-finders 1` |
| `--max-finders` | Max number of finders | `--max-finders 5` |
| `--reported` | Report period (30/60/90/alltime/after) | `--reported 30` |
| `--reported-after` | Reports after date (ISO) | `--reported-after 2024-01-01` |
| `--min-quality` | Min quality score (0-5) | `--min-quality 3` |
| `--min-rarity` | Min rarity score (0-5) | `--min-rarity 2` |
| `--sort` | Sort field | `--sort quality` |
| `--sort-dir` | Sort direction (asc/desc) | `--sort-dir asc` |
| `--page` | Page number (>= 1) | `--page 2` |
| `--limit` | Results per page (1-100) | `--limit 50` |
| `--format`, `-o` | Output format (table, json, ndjson) | `--format json` |

### Notes

- **API key required**: Set `SOLODIT_API_KEY` or use `ethcli config set-solodit <key>`
- **Get API key**: https://solodit.cyfrin.io (Profile > API Keys)
- **Rate limit**: 20 requests per 60 seconds
- **Alias**: `ethcli sld` works as an alias for `ethcli solodit`
- **Impact levels**: HIGH, MEDIUM, LOW, GAS (invalid values are rejected)
- **Keywords optional**: `ethcli sld search --impact HIGH` works without keywords
- **`get` is best-effort**: the API has only `POST /findings`; `get` searches by slug text and matches slug/ID exactly
- **401 errors** show the server message ("Missing API key" vs "Invalid API key"); regenerating a key at solodit.cyfrin.io invalidates the old one
- **Upstream quirks**: `report_date` may be `{}`; `protocols_protocolcategoryscore` is always `[]` (solodit/solodit_content#153)
- **Sort options**: recency (default), quality, rarity

## Simulation Commands

```bash
# Simulate a contract call (uses cast by default)
ethcli simulate call <contract> --sig "balanceOf(address)" <address> --rpc-url https://eth.llamarpc.com

# Simulate with trace (requires debug-capable node)
ethcli simulate call <contract> --sig "transfer(address,uint256)" <to> <amount> --trace

# Trace an existing transaction
ethcli simulate tx <tx_hash> --rpc-url https://eth.llamarpc.com

# Decoded call tree from debug_traceTransaction (contract names, function
# args, events — no Foundry/cast binary required); --raw for callTracer JSON
ethcli simulate tx <tx_hash> --via debug

# Use different backends
ethcli simulate call ... --via cast      # Default: uses cast call
ethcli simulate call ... --via anvil     # Forks mainnet with Anvil
ethcli simulate call ... --via tenderly  # Uses Tenderly API (rich output)
ethcli simulate call ... --via debug     # Uses debug_traceCall RPC
ethcli simulate call ... --via trace     # Uses trace_call RPC (Erigon/OpenEthereum)
ethcli simulate call ... --via alchemy   # Uses Alchemy debug_traceCall (Simulation API retired 2026-09-30)
```

`--sig` calldata is ABI-encoded natively, so `--via tenderly|debug|trace|alchemy`
work without Foundry. Foundry is still needed for `--via cast` (the default) and
`--via anvil`, and for signatures ethcli can't encode itself (parameter names,
`returns` clauses such as `balanceOf(address)(uint256)`); those fall back to
`cast calldata`. Pass raw calldata with `--data` to skip encoding entirely.

## Tenderly Commands

Requires `TENDERLY_ACCESS_KEY`, `TENDERLY_ACCOUNT` and `TENDERLY_PROJECT` (environment variables, matching `--tenderly-*` flags, or `ethcli config set-tenderly`). The `--tenderly-key`/`--tenderly-account`/`--tenderly-project` flags belong to the group command and go before the subcommand, e.g. `ethcli tenderly vnets --tenderly-project <slug> list`.

```bash
# Virtual TestNets (VNets)
ethcli tenderly vnets list
ethcli tenderly vnets create --slug <slug> --name "My VNet" --network-id 1
ethcli tenderly vnets get <vnet-id>
ethcli tenderly vnets delete <vnet-id>
ethcli tenderly vnets delete <id1> <id2> <id3>
ethcli tenderly vnets delete --all
ethcli tenderly vnets rpc <vnet-id>

# VNet Admin RPC - Balance Management
ethcli tenderly vnets admin --vnet <id> set-balance <address> 10eth
ethcli tenderly vnets admin --vnet <id> add-balance <address> 1eth
ethcli tenderly vnets admin --vnet <id> set-erc20-balance --token <token> --wallet <wallet> <amount>
ethcli tenderly vnets admin --vnet <id> set-max-erc20-balance --token <token> --wallet <wallet>

# VNet Admin RPC - Time Manipulation
ethcli tenderly vnets admin --vnet <id> increase-time 3600
ethcli tenderly vnets admin --vnet <id> set-timestamp <epoch>
ethcli tenderly vnets admin --vnet <id> increase-blocks 10

# VNet Admin RPC - State Management
ethcli tenderly vnets admin --vnet <id> snapshot
ethcli tenderly vnets admin --vnet <id> revert <snapshot-id>

# VNet Admin RPC - Storage/Code (slot/value accept decimal or hex, auto-padded to 32 bytes)
ethcli tenderly vnets admin --vnet <id> set-storage --address <addr> --slot 0 --value 1
ethcli tenderly vnets admin --vnet <id> set-code --address <addr> --code <bytecode>

# VNet Admin RPC - Transactions
ethcli tenderly vnets admin --vnet <id> send-tx --from <addr> --to <addr> --value 0x1
ethcli tenderly vnets admin --vnet <id> get-latest

# VNet Admin RPC - Simulation (tenderly_simulateTransaction / tenderly_simulateBundle)
ethcli tenderly vnets admin --vnet <id> simulate-tx --from <addr> --to <addr> --data 0x...
ethcli tenderly vnets admin --vnet <id> simulate-tx --from <addr> --to <addr> --value 0x1 --block latest --state-overrides '{"0x...":{"balance":"0x..."}}'
ethcli tenderly vnets admin --vnet <id> simulate-bundle '[{"from":"0x...","to":"0x...","data":"0x..."}]'
ethcli tenderly vnets admin --vnet <id> simulate-bundle '[...]' --state-overrides '{"0x...":{"balance":"0x..."}}' --block-overrides '{"number":"0x..."}'

# Virtual Wallets
ethcli tenderly wallets list
ethcli tenderly wallets add <address>
ethcli tenderly wallets get <address> --network 1

# Contracts
ethcli tenderly contracts list
ethcli tenderly contracts get <address> --network 1
ethcli tenderly contracts add <address> --network 1
ethcli tenderly contracts rename <address> --network 1 --name <name>
ethcli tenderly contracts encode-state --network 1 '{"0xToken": {"value": {"balances[0xHolder]": "1000"}}}'

# Alerts
ethcli tenderly alerts list
ethcli tenderly alerts create --name "Alert" --alert-type successful_transaction --network 1
ethcli tenderly alerts delete <alert-id>
ethcli tenderly alerts webhooks list
ethcli tenderly alerts webhooks create --name "Hook" --url https://...

# Web3 Actions
ethcli tenderly actions list
ethcli tenderly actions get <action-id>
ethcli tenderly actions stop <action-id>
ethcli tenderly actions resume <action-id>
ethcli tenderly actions get-call <action-id> <execution-id>

# Networks
ethcli tenderly networks list
ethcli tenderly networks get <network-id>

# Delivery Channels (Slack, Discord, Email, etc.)
ethcli tenderly channels list
ethcli tenderly channels account
ethcli tenderly channels project

# Simulation (alias to ethcli simulate --via tenderly; always uses Tenderly)
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

## Quote Commands

Get swap quotes from multiple DEX aggregators in parallel. Finds the best swap route across OpenOcean, KyberSwap, 0x, 1inch, CowSwap, LI.FI, Velora, and Enso.

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

# Set slippage tolerance (basis points, default 50 = 0.5%; applied to 0x, LI.FI and Enso tx data)
ethcli quote best ETH USDC 1000000000000000000 --slippage 100

# `quote from all` behaves like `quote compare`

# Pass the wallet that will sign: transaction data is built for it and pays the
# output to it. Without --sender, CowSwap, LI.FI and Enso are priced for a
# placeholder address (marked in the output) and return no transaction data
ethcli quote best ETH USDC 1000000000000000000 --sender 0xYourAddress

# JSON output
ethcli quote compare ETH USDC 1000000000000000000 --format json
```

### Available Sources

| Alias | Aggregator | Notes |
|-------|------------|-------|
| `openocean`, `oo` | OpenOcean | Multi-chain DEX aggregator |
| `kyberswap`, `kyber` | KyberSwap | Dynamic routing |
| `0x`, `zerox` | 0x Protocol | Professional-grade liquidity |
| `1inch`, `oneinch` | 1inch | Pathfinder algorithm |
| `cowswap`, `cow` | CowSwap | MEV-protected trades |
| `li.fi`, `lifi` | LI.FI | Cross-chain aggregator |
| `velora`, `paraswap` | Velora/ParaSwap | Multi-protocol routing |
| `velora-delta`, `delta` | Velora Delta | Gasless order filled by solvers; output is net of gas, no transaction data |
| `enso`, `ensofi` | Enso Finance | DeFi shortcuts |
| `propamm`, `pamm` | Prop AMMs | Market-maker pools re-priced every block; Ethereum mainnet, major pairs. Conditional: shown but never the best quote |

### Notes

- **No API keys required** for basic usage (some sources may have higher rate limits with keys)
- **MEV protection**: CowSwap uses batch auctions to protect against MEV
- **Amount format**: Pass raw amounts (wei) or use `--decimals` for human-readable input
- **Prop AMMs**: `propamm` reports the best price from the market-maker pools that block builders re-price each block. The figure comes from a builder's snapshot and only holds if the trade lands in a block from a builder carrying that maker's update, so it is conditional: it appears in `conditional_quotes` (table: `COND`, unranked) and is never `best_quote` or part of the summary figures. Mainnet only, wrapped tokens (ETH is quoted as WETH), no sender needed. `PROPAMM_DATA_URL` overrides the data endpoint
- **Velora Delta**: a second Velora source. Delta is an order the user signs and solvers fill, so its output already has the gas fee taken out and it returns no transaction data; the other sources report output before gas. On larger trades in thinner pools it can beat Velora's Market route
- **Outliers**: with three or more quotes, a source under half the median is listed in `aggregation.outliers` (table: `LOW`) and left out of the average, spread and improvement; a source above twice the median is listed in `high_outliers` (table: `HIGH`) with a warning but still counts
- **Sender**: CowSwap, LI.FI and Enso need a sender. Without `--sender` they are priced for a placeholder address (`placeholder_sender` in the output) and return no transaction data; the zero address and the placeholder are rejected as `--sender`
- **Not applicable**: prop AMMs are only asked on Ethereum mainnet; a pair no maker quotes is listed in `not_applicable`, not `failed_sources`, and is not counted in `total_sources`
- **Failed sources**: `failed_sources` in the output lists each source that returned no quote and why; `quote best` prints them too
- **LI.FI fee**: LI.FI's output is net of its own fixed fee, reported as `fee_amount`/`fee_token`
- **Symbols**: ETH (native), WETH, USDC, USDT, DAI, WBTC, ... resolve per chain (ethereum, optimism, bsc, polygon, base, arbitrum, avalanche); otherwise pass addresses
- **Chains**: Supports Ethereum, Polygon, Arbitrum, Optimism, Base, and more

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

# Data Streams (requires API credentials). Defaults to mainnet
# (api.dataengine.chain.link / ws.dataengine.chain.link); override with
# CHAINLINK_REST_URL / CHAINLINK_WS_URL or [chainlink] rest_url / ws_url for testnet.
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

# User vault positions are not available from Kong; use `ethcli portfolio`.

# List strategies
ethcli kong strategies list --chain-id 1
ethcli kong strategies list --vault 0x... -c 1 # Strategies for a vault (vaultStrategies)
ethcli kong strategies get --chain-id 1 0x...  # Get strategy details

# Token prices (contract addresses only; Kong currently returns no price data
# for most tokens - the command errors instead of printing null)
ethcli kong prices current --chain-id 1 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli kong prices historical --chain-id 1 0x... 1700000000  # At timestamp

# TVL data
ethcli kong tvl current --chain-id 1 0x...
ethcli kong tvl history --chain-id 1 0x... --period day --limit 30   # most recent 30 points

# Vault/strategy reports (harvests)
ethcli kong reports vault --chain-id 1 0x...
ethcli kong reports strategy --chain-id 1 0x...
```

### Notes

- **Chain IDs**: 1=Ethereum, 137=Polygon, 42161=Arbitrum, 10=Optimism, 8453=Base
- **Alias**: `ethcli yearn` works as an alias for `ethcli kong`
- **No API key**: Kong API is free and public

## Alchemy Commands

Direct access to Alchemy API. Requires `ALCHEMY_API_KEY` environment variable.

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

# Transfers (from / to / all)
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

## Gecko (CoinGecko) Commands

Direct access to CoinGecko API. Optional `COINGECKO_API_KEY` for Pro API (higher rate limits).

```bash
# Coin data
ethcli gecko coins get bitcoin

# Price queries
ethcli gecko simple price bitcoin,ethereum --vs usd,eur
ethcli gecko coins chart bitcoin --days 30

# Market data
ethcli gecko coins markets usd
ethcli gecko global trending

# NFT data
ethcli gecko nfts get bored-ape-yacht-club

# Tokenized real-world assets (list/markets/get/issuers work keyless)
ethcli gecko rwas list --asset-type stock
ethcli gecko rwas markets --asset-type etf --per-page 20
ethcli gecko rwas get 1-800-flowers-com            # --full adds tokens and market data
ethcli gecko rwas issuers
ethcli gecko rwas issuer <issuer-id>
ethcli gecko rwas tickers <rwa-id> --depth         # Basic plan or above
ethcli gecko rwas chart <rwa-id> --days 90         # Basic plan or above

# Onchain wallets (Analyst plan or above); --networks must be all EVM or all non-EVM
ethcli gecko wallets balances 0x... --networks eth,base
ethcli gecko wallets transfers eth 0x... --from 2026-11-01 --to 2026-11-07 --direction in
ethcli gecko wallets trades eth 0x... --token 0x...
ethcli gecko wallets pnl 0x... --networks eth,base --sort unrealized_pnl_usd_desc

# News and insights (Analyst plan or above)
ethcli gecko news --coin bitcoin --type guides
ethcli gecko insights --from 2026-11-01 --to 2026-11-15

# Supply breakdown (Analyst plan or above)
ethcli gecko coins supply-breakdown uniswap

# Onchain: multi-network token prices/data (Enterprise plan), trades
ethcli gecko onchain token-price-multi eth:0xc02a...,base:0x4200... --market-cap
ethcli gecko onchain tokens-multi eth:0xc02a... --top-pools
ethcli gecko onchain pool-trades eth 0x... --min-volume-usd 1000          # keyless OK
ethcli gecko onchain pool-trades eth 0x... --from 2026-11-01 --to 2026-11-02   # Analyst plan
ethcli gecko onchain token-trades eth 0x... --from 2026-11-01 --to 2026-11-02  # Analyst plan
```

Commands that need a paid plan (wallets, news, insights, supply-breakdown,
RWA tickers/chart, multi-network onchain, trade ranges) fail before any request
without a Pro key (`COINGECKO_API_KEY` + `COINGECKO_PRO=1`, or `use_pro = true`
under `[coingecko]`). `-o/--format` goes before the subcommand for grouped
commands (`ethcli gecko rwas -o ndjson list`).

## Llama (DefiLlama) Commands

Direct access to DefiLlama API. Optional `DEFILLAMA_API_KEY` (or `ethcli config set-defillama`) for Pro endpoints. `llama yields` and the DefiLlama source in `ethcli yields` need it from 2026-11-13, when the free yields endpoints shut down.

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
ethcli llama yields earn --chain ethereum,base --protocol aave-v3 --stablecoin true --min-tvl 1000000 --limit 50
ethcli llama yields earn --all                     # every earn pool, unpaginated
ethcli llama yields earn-history <pool-id> --range 30d   # 30d | 90d | max (default: full series)
ethcli llama yields verified
ethcli llama yields verified-pool <pool-id>
ethcli llama yields verified-history <pool-id> --range 90d
ethcli llama yields borrow-markets
ethcli llama yields borrow-history <market-id> --range max
ethcli llama yields borrow-routes
ethcli llama yields loops

# Stablecoins
ethcli llama stablecoins list
```

With a key, `pro_routes = true` under `[defillama]` in the config file (or
`DEFILLAMA_PRO_ROUTES=1`) sends the TVL, coins, volumes, fees and stablecoins
commands to `pro-api.llama.fi` (Pro rate limits, uses credits; same responses).
Without a key it has no effect.

## Moralis Commands

Direct access to Moralis API. Requires `MORALIS_API_KEY` environment variable.

```bash
# Wallet data
ethcli moralis wallet balance 0x...
ethcli moralis wallet tokens 0x...
ethcli moralis wallet transactions 0x...
ethcli moralis token transfers 0x...

# Wallet insight (age, activity, counterparties, gas, swap volume); all chains by default
ethcli moralis wallet insight 0x...
ethcli moralis wallet insight 0x... --chains eth,base --chain-breakdown

# Token score and score history (EVM only; --chain goes before the subcommand)
ethcli moralis discovery token-score 0x...
ethcli moralis discovery --chain base token-score-history 0x... --timeframe 30d   # 1d | 7d | 30d

# NFT data
ethcli moralis nft list 0x...
ethcli moralis nft metadata 0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d 1

# Token data
ethcli moralis token price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
ethcli moralis token metadata 0x...

# DeFi positions
ethcli moralis defi wallet-positions 0x...
```

## Dune Sim (removed)

Dune Sim (sim.dune.com) was shut down on 2026-08-01, so `ethcli dsim`,
`--source dsim`, `ethcli config set-dune-sim` and the `DUNE_SIM_API_KEY`
variable were removed (see <https://github.com/yldfi/yldfi-rs/issues/64>).
Use `ethcli portfolio` / `ethcli nfts` (Alchemy, Moralis) instead. Dune
Analytics (`ethcli dune`, `DUNE_API_KEY`) is unaffected.

## Dune Commands

Direct access to Dune Analytics API. Requires `DUNE_API_KEY` environment variable.

```bash
# Run a saved query (waits for completion)
ethcli dune query execute 1234567
ethcli dune query execute 1234567 --params '{"address": "0x..."}'

# Get cached results
ethcli dune query results 1234567
ethcli dune query csv 1234567

# Executions
ethcli dune execution status <execution-id>
ethcli dune execution results <execution-id>

# Raw SQL
ethcli dune sql execute "SELECT 1"

# Tables
ethcli dune tables list
```

## Curve Commands

Direct access to Curve Finance API. No API key required.

```bash
# Optional positional chain: omit for all chains
ethcli curve pools big [CHAIN]
ethcli curve crvusd markets [CHAIN]

# OHLC / trades need two pool coins
ethcli curve ohlc pool ethereum <pool> --main-token <addr> --reference-token <addr>
ethcli curve trades get ethereum <pool> --main-token <addr> --reference-token <addr>
ethcli curve ohlc lp-token ethereum <pool>          # start/end default to last 7 days

# CRV (not crvUSD) circulating supply
ethcli curve crvusd crv-circulating-supply

# Pool data
ethcli curve pools all
ethcli curve pools list ethereum

# Volume and TVL
ethcli curve volumes total ethereum
ethcli curve volumes pools ethereum

# Lending
ethcli curve lending all
ethcli curve lending chain ethereum

# Token data
ethcli curve tokens list ethereum
ethcli curve prices token ethereum 0x...
```

## Exchange Commands (`cex`, alias `ccxt`)

Public market data from centralised exchanges and perp venues via the `cexm`
crate (`crates/cexm`). No API keys.

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

### Supported Exchanges

Binance, Bitget, OKX, Bybit, Coinbase and Kraken (spot), Hyperliquid and Lighter
(perpetuals). Public data only, no API keys. Binance is queried via its market-data
host `data-api.binance.vision`, which isn't region-locked. `ethcli ccxt` is an alias.

Not every venue has every timeframe (e.g. OKX and Bitget have no 8h candles);
unsupported ones fail with a clear error. Bars of 6h and longer are UTC-aligned.

## Direct DEX Aggregator Commands

### 1inch Commands

Direct access to 1inch DEX Aggregator API. Requires an API key: `[oneinch]` in config, or `ONEINCH_API_KEY` / `1INCH_API_KEY`. `--chain-id` defaults to the global `--chain`.

```bash
# Get swap quote
ethcli 1inch quote <src> <dst> <amount> --chain-id 1 --include-gas

# Get swap transaction data
ethcli 1inch swap <src> <dst> <amount> <from> --chain-id 1 --slippage 1

# Get supported tokens and sources
ethcli 1inch tokens --chain-id 1
ethcli 1inch sources --chain-id 1

# Approvals
ethcli 1inch spender --chain-id 1
ethcli 1inch allowance <token> <wallet> --chain-id 1
ethcli 1inch approve <token> --chain-id 1
```

**Alias**: `ethcli oneinch`

### OpenOcean Commands

Direct access to the De¹ (formerly OpenOcean) DEX Aggregator API. No API key required.

```bash
# Get swap quote (tokens: address or symbol such as ETH/USDC; amount in wei)
ethcli open-ocean quote <in_token> <out_token> <amount> --chain ethereum --slippage 1
ethcli open-ocean quote ETH USDC 1000000000000000000 --chain ethereum

# Get swap transaction
ethcli open-ocean swap <in_token> <out_token> <amount> <account> --chain ethereum

# Get reverse quote: how much in_token to sell to receive out_amount (human units) of out_token
ethcli open-ocean reverse-quote <in_token> <out_token> <out_amount> --chain ethereum

# List tokens and DEXes
ethcli open-ocean tokens --chain ethereum
ethcli open-ocean dexes --chain ethereum
```

**Aliases**: `ethcli oo`, `ethcli de1`

### KyberSwap Commands

Direct access to KyberSwap Aggregator API. No API key required; set `KYBERSWAP_API_KEY` to use the API gateway instead of the rate-limited keyless API.

```bash
# Get optimal routes
ethcli kyber-swap routes <token_in> <token_out> <amount_in> --chain ethereum

# Get route with full data
ethcli kyber-swap route-data <token_in> <token_out> <amount_in> --chain ethereum

# Build swap transaction
# --route-summary takes the `routes` output or the full `route-data` output, sent unmodified
ethcli kyber-swap build <token_in> <token_out> <amount_in> <sender> <recipient> \
  --chain ethereum --slippage-bps 50 --route-summary '<json>'

# Route + calldata in one call (GET /api/v1/swap). KyberSwap restricts this
# endpoint to partners: pass --client-id or set KYBERSWAP_CLIENT_ID.
# recipient defaults to sender; --deadline is seconds from now (API default 20 min)
ethcli kyber-swap swap <token_in> <token_out> <amount_in> <sender> [recipient] \
  --chain ethereum --slippage-bps 50 --client-id <partner-id> [--exclude-rfq]
```

**Alias**: `ethcli kyber`

### 0x Protocol Commands

Direct access to 0x API. Requires an API key: `[zerox]` in config, or `ZEROX_API_KEY` / `0X_API_KEY`.

```bash
# Get swap quote with tx data
ethcli 0x quote <sell_token> <buy_token> <sell_amount> <taker> --chain ethereum --slippage-bps 100

# Get price estimate (lighter)
ethcli 0x price <sell_token> <buy_token> <sell_amount> <taker> --chain ethereum

# AllowanceHolder flow (approve `allowanceTarget`, no Permit2 signature)
ethcli 0x quote <sell_token> <buy_token> <sell_amount> <taker> --allowance-holder
ethcli 0x price <sell_token> <buy_token> <sell_amount> <taker> --allowance-holder

# Get liquidity sources
ethcli 0x sources --chain ethereum

# Supported chains (Swap API; --gasless for the Gasless API)
ethcli 0x chains
ethcli 0x chains --gasless

# Gasless swaps (ERC-20 sell token, gas paid from it). gasless-quote returns the
# EIP-712 trade/approval to sign; submitting signatures is not supported.
ethcli 0x gasless-price <sell_token> <buy_token> <sell_amount> <taker> --chain base
ethcli 0x gasless-quote <sell_token> <buy_token> <sell_amount> <taker> --chain base
ethcli 0x gasless-status <trade_hash> --chain base
```

**Alias**: `ethcli zerox`

### CowSwap Commands

Direct access to CoW Protocol API for MEV-protected trading. No API key required.

```bash
# Get MEV-protected quote
ethcli cow-swap quote <sell_token> <buy_token> <amount> <from> --chain ethereum --kind sell

# Order and trade queries
ethcli cow-swap order <uid> --chain ethereum
ethcli cow-swap orders <owner> --chain ethereum
ethcli cow-swap trades <owner> --chain ethereum --limit 100   # paginated, API default 10
ethcli cow-swap order-trades <uid> --chain ethereum
ethcli cow-swap order-status <uid> --chain ethereum            # competition status + solver bids
ethcli cow-swap orders-by-tx <tx_hash> --chain ethereum        # orders settled in a tx
ethcli cow-swap orders-by-uids --uid <uid1>,<uid2> --chain ethereum   # max 128; unknown UIDs left out

# App data and user surplus
ethcli cow-swap app-data <app_data_hash> --chain ethereum
ethcli cow-swap surplus <address> --chain ethereum             # endpoint marked unstable by CoW

# Auction data
ethcli cow-swap auction --chain ethereum
ethcli cow-swap competition <auction_id> --chain ethereum
ethcli cow-swap competition --tx-hash <tx> --chain ethereum
ethcli cow-swap competition --chain ethereum                  # latest

# Cancel orders (signature over OrderCancellations(bytes[] orderUids))
ethcli cow-swap cancel-orders --uid <uid1>,<uid2> <signature> --chain ethereum

# Get native token price
ethcli cow-swap native-price <token> --chain ethereum
```

**Alias**: `ethcli cow`

### LI.FI Commands

Direct access to LI.FI cross-chain aggregator API. Optional `LIFI_INTEGRATOR` for analytics and `LIFI_API_KEY` for per-key (instead of per-IP) rate limits.

```bash
# Cross-chain quote (--slippage is in percent, default 0.5 = 0.5%; converted to
# LI.FI's decimal fraction, max 50%)
ethcli lifi quote <from_chain> <from_token> <to_chain> <to_token> <amount> <from_address>

# Quote by output amount (/quote/toAmount): action.fromAmount is what to send
ethcli lifi quote-to-amount <from_chain> <from_token> <to_chain> <to_token> <to_amount> <from_address>
ethcli lifi quote-to-amount 1 USDC 42161 USDC 10000000 0xYourAddress

# Bridge/swap then run contract calls on the destination chain (/quote/contractCalls, beta);
# --calls is a JSON array of ContractCall objects (a single object is accepted)
ethcli lifi contract-calls <from_chain> <from_token> <to_chain> <to_token> <to_amount> <from_address> \
  --calls '[{"fromAmount":"...","fromTokenAddress":"0x...","toContractAddress":"0x...","toContractCallData":"0x...","toContractGasLimit":"..."}]'

# Routes
ethcli lifi routes <from_chain> <from_token> <to_chain> <to_token> <amount> <from_address>
ethcli lifi best-route <from_chain> <from_token> <to_chain> <to_token> <amount> <from_address>

# Transaction status
ethcli lifi status <tx_hash> --from-chain 1 --to-chain 137
ethcli lifi gas-status <tx_hash>                    # LI.Fuel refuel status (NOT_FOUND if unknown)

# Wallet transfer history (/v2/analytics/transfers). Last 30 days unless
# --from-timestamp; --status all|done|pending|failed (API default done);
# page with the response's `next`/`previous` cursors
ethcli lifi transfers <wallet> --status all --limit 20
ethcli lifi transfers <wallet> --from-timestamp 1700000000 --next <cursor>

# Discovery
ethcli lifi chains
ethcli lifi chain <chain_id>
ethcli lifi tokens --chain-id 1
ethcli lifi token 1 USDC                            # one token by symbol or address
ethcli lifi tools
ethcli lifi bridges
ethcli lifi exchanges
ethcli lifi gas <chain_id>
ethcli lifi connections --from-chain 1 --to-chain 137
```

**Alias**: `ethcli li.fi`

### Velora (ParaSwap) Commands

Direct access to ParaSwap API. Optional `PARASWAP_API_KEY` or `VELORA_API_KEY` for higher limits.

```bash
# Get swap price/route
ethcli velora price <src_token> <dest_token> <amount> --chain ethereum --side SELL

# Build swap transaction (--price-route: full `price` output or its priceRoute, sent unmodified;
# token decimals are auto-detected via RPC when omitted)
ethcli velora transaction <user_address> --chain ethereum --slippage 100 --price-route '<json>'

# Price route + transaction in one call (GET /swap; lower rate limits, no RFQ).
# Takes the `price` flags plus --slippage (bps, 1-5000) and --receiver
ethcli velora swap <src_token> <dest_token> <amount> <user_address> --chain ethereum --slippage 100

# List tokens
ethcli velora tokens --chain ethereum
```

**Alias**: `ethcli paraswap`

### Enso Finance Commands

Direct access to Enso Finance DeFi aggregator API. Requires `ENSO_API_KEY`.

```bash
# Get DeFi route
ethcli enso route <token_in> <token_out> <amount_in> <from_address> --chain-id 1 --slippage 50

# Get token price
ethcli enso price <token> --chain-id 1

# Get balances
ethcli enso balances <address> --chain-id 1

# Bundle actions (defaults to the EOA-executable `router` strategy)
ethcli enso bundle <from_address> '[{"protocol":"enso","action":"route","args":{...}}]'

# Quote: output amount and route, no tx data (sender optional)
ethcli enso quote <token_in> <token_out> <amount_in> [from_address] --chain-id 1 \
  [--ignore-aggregators 1inch,0x] [--effort light|medium|high|max] [--swap-type any|rfq|amm]

# Prices of several tokens (max 100; null for tokens Enso cannot price)
ethcli enso prices <token1> <token2> --chain-id 1

# Search tokens (filters: --chain-id, --project, --protocol, --address, --symbol,
# --underlying, --type defi|base, --apy-from/--apy-to, --tvl-from/--tvl-to,
# --include-metadata, --include-underlying, --page/--page-size/--cursor)
ethcli enso tokens --chain-id 1 --protocol aave-v3 --include-metadata

# Networks and protocols
ethcli enso networks
ethcli enso protocols --chain-id 1 --slug aave-v3

# Approval tx for Enso's router (or the smart wallet with --routing-strategy)
ethcli enso approve <token> <amount> <from_address> --chain-id 1

# Enso smart wallet of an address
ethcli enso wallet <from_address> --chain-id 1
```

`--chain-id` defaults to the global `--chain`, except on `tokens` and `protocols`,
where it is an optional filter (omit it to search every chain).

### Pyth Network Commands

Direct access to the Pyth Network Hermes API (`https://pyth.dourolabs.app/hermes`).
**API key required** since the Pyth Core upgrade (2026-08-26): set `PYTH_API_KEY` or run
`ethcli config set-pyth <key>` (config takes precedence). Keys: https://pythdata.app

```bash
# Get latest prices
ethcli pyth price BTC/USD
ethcli pyth price ETH SOL AVAX

# Price at a past time (first update at or after the unix timestamp)
ethcli pyth price ETH/USD --at 1704067200

# Search for feeds
ethcli pyth search "BTC"

# List feeds
ethcli pyth feeds
ethcli pyth feeds --asset-type crypto
ethcli pyth known-feeds

# Latest publisher stake caps update
ethcli pyth stake-caps
```

**Notes**:
- Supports common symbols: BTC, ETH, SOL, USDC, USDT, DAI, AVAX, ARB, OP, LINK, UNI, AAVE, CRV, CVX, etc.
- Without a key, `pyth price` and `pyth stake-caps` error with setup instructions and the `price` aggregator reports Pyth as `PYTH_API_KEY not configured`
- `pyth search`/`feeds` send the key when available

## Uniswap Commands

Query Uniswap V2, V3, and V4 pools via on-chain lens queries and The Graph subgraph.

```bash
# On-chain queries (no API key needed; honour --chain, e.g. --chain arbitrum)
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
ethcli uniswap positions <address> --format json    # JSON output

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
├── bytecode/         # Bytecode analysis (evmole, evm-disassembler)
│   ├── mod.rs        # Module exports
│   ├── types.rs      # ExtractedFunction, OpcodeStats, SecurityAnalysis, etc.
│   ├── selectors.rs  # Function selector extraction (evmole wrapper)
│   ├── disassemble.rs # Opcode disassembly (evm-disassembler wrapper)
│   ├── security.rs   # Security pattern detection (SELFDESTRUCT, DELEGATECALL, etc.)
│   └── analyze.rs    # Combined bytecode analysis
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
    ├── dune_cli.rs   # Direct Dune Analytics API
    ├── curve.rs      # Direct Curve Finance API
    ├── chainlink.rs  # Chainlink price feeds (RPC + Data Streams)
    ├── cex.rs       # Exchange data via cexm
    ├── kong.rs       # Direct Yearn Kong API
    ├── uniswap.rs    # Uniswap V2/V3/V4 queries
    ├── goplus.rs     # GoPlus Security API (token/address/NFT/approval)
    ├── solodit.rs    # Solodit vulnerability database
    ├── blacklist.rs  # Token blacklist management
    ├── oneinch.rs    # Direct 1inch DEX Aggregator API
    ├── openocean.rs  # Direct OpenOcean DEX Aggregator API
    ├── kyber.rs      # Direct KyberSwap Aggregator API
    ├── zerox.rs      # Direct 0x Protocol API
    ├── cowswap.rs    # Direct CowSwap/CoW Protocol API
    ├── lifi.rs       # Direct LI.FI Cross-Chain API
    ├── velora.rs     # Direct Velora/ParaSwap API
    ├── enso.rs       # Direct Enso Finance API
    └── pyth.rs       # Direct Pyth Network Price Feeds API
```

## Key Dependencies

- **alloy 1.0**: Ethereum provider, types, ABI decoding
- **evmole**: Function selector extraction from bytecode
- **evm-disassembler**: EVM opcode disassembly
- **foundry-block-explorers**: Etherscan API client
- **chainlink-data-streams-sdk**: Chainlink Data Streams API (optional, requires API key)
- **pyth** (`pythc`): Pyth Network Hermes API client (API key required, `Authorization: Bearer`)
- **tndrly**: Tenderly API client
- **alcmy**: Alchemy API client
- **gecko**: CoinGecko API client
- **llama**: DefiLlama API client
- **mrls**: Moralis API client
- **dune**: Dune Analytics API client
- **crv**: Curve Finance API client
- **ykong**: Yearn Kong GraphQL API client
- **unswp**: Uniswap V2/V3/V4 client (on-chain lens + subgraph)
- **gplus**: GoPlus Security API client
- **sldt**: Solodit vulnerability database client
- **cexm**: Centralised exchange market data (Binance, Bitget, OKX, Bybit, Coinbase, Kraken, Hyperliquid, Lighter)
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
# ABI and source come from Etherscan (rate limits retried twice, waiting 1 s
# then 2 s). If Etherscan has no verified source or fails (no key, plan
# restriction, unsupported chain), Sourcify is tried, then Blockscout on
# Ethereum and Optimism; stderr says which one answered. This applies to
# every command that fetches an ABI (contract call, tx decoding, ...).
# Only part of it (verified sources can be 100 KB+): --list, --function <name>,
# --grep <regex>, --file <name>; --file narrows --function and --grep.
# A shared absolute path prefix from the deployer's machine is stripped.

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
ethcli yields --project aave

# Chainlink price feeds (RPC-based, no API key)
ethcli chainlink price ETH
ethcli chainlink price CVX --chain ethereum
ethcli chainlink oracles --chain arbitrum
```

## Environment Variables

**Note:** Some services have similar-named keys that serve different purposes:
- `CHAINLINK_API_KEY` + `CHAINLINK_USER_SECRET` are only for Data Streams (premium), not needed for RPC-based price feeds

| Variable | Required For | Description |
|----------|-------------|-------------|
| `ETHERSCAN_API_KEY` | Optional | Increases Etherscan rate limit |
| `TENDERLY_ACCESS_KEY` | `ethcli tenderly` | Tenderly API access |
| `ALCHEMY_API_KEY` | `ethcli alchemy`, `--via alchemy` | Alchemy API access |
| `ALCHEMY_NOTIFY_TOKEN` | `alchemy notify` (`webhooks`) | Webhooks auth token (dashboard Data -> Webhooks, AUTH TOKEN button); or `ethcli config set-alchemy-notify-token --stdin` |
| `ALCHEMY_ACCESS_KEY` | `alchemy gas-manager` (`gas-sponsorship`) | Access key with Gas Manager permissions (dashboard -> Security); or `ethcli config set-alchemy-access-key --stdin` |
| `COINGECKO_API_KEY` | Optional | CoinGecko Pro API (increases rate limit) |
| `DEFILLAMA_API_KEY` | Pro endpoints; DefiLlama yields from 2026-11-13 | DefiLlama API plan key; or `ethcli config set-defillama --stdin` |
| `DEFILLAMA_PRO_ROUTES` | Optional | `1`/`true`: route `ethcli llama` free endpoints through the Pro API (needs `DEFILLAMA_API_KEY`); or `pro_routes = true` under `[defillama]` |
| `MORALIS_API_KEY` | `ethcli moralis` | Moralis API access |
| `DUNE_API_KEY` | `ethcli dune` | Dune Analytics queries |
| `CHAINLINK_API_KEY` | `chainlink streams` only | Data Streams API key |
| `CHAINLINK_USER_SECRET` | `chainlink streams` only | Data Streams secret |
| `THEGRAPH_API_KEY` | Uniswap subgraph | The Graph API access |
| `GOPLUS_APP_KEY` | Optional | GoPlus batch queries (>1 token) |
| `GOPLUS_APP_SECRET` | Optional | GoPlus batch queries (>1 token) |
| `SOLODIT_API_KEY` | `ethcli solodit` | Solodit vulnerability database |
| `PYTH_API_KEY` | `ethcli pyth price`, Pyth price source | Pyth Hermes (required since the Pyth Core upgrade) |
| `ONEINCH_API_KEY` / `1INCH_API_KEY` | `ethcli 1inch` | 1inch DEX Aggregator (required) |
| `ZEROX_API_KEY` / `0X_API_KEY` | Optional | 0x Protocol (higher rate limits) |
| `LIFI_INTEGRATOR` | Optional | LI.FI analytics tracking |
| `KYBERSWAP_API_KEY` | Optional | KyberSwap API gateway (`api.kyberswap.com/swap`, `X-Api-Key`); without it the keyless, 3 rps aggregator API is used |
| `LIFI_API_KEY` | Optional | LI.FI API key (`x-lifi-api-key`; without it limits are per IP) |
| `PARASWAP_API_KEY` / `VELORA_API_KEY` | Optional | Velora/ParaSwap (higher rate limits) |
| `ENSO_API_KEY` | `ethcli enso` | Enso Finance (required) |
| `PROPAMM_DATA_URL` | Optional | Data endpoint for the `propamm` quote source (default: Titan Builder, EU) |

## Release Process

**Automated with release-please:**
- Use [Conventional Commits](https://www.conventionalcommits.org/) in commit messages
- `feat:` - triggers minor version bump
- `fix:` - triggers patch version bump
- `feat!:` or `BREAKING CHANGE:` - triggers major version bump
- Release-please creates a PR with version bump and CHANGELOG
- Merging the PR tags `ethcli-vX.Y.Z`, which triggers `release.yml`: binary builds for all platforms, the GitHub release, then `cargo publish`
- crates.io publishing uses Trusted Publishing (`rust-lang/crates-io-auth-action`, OIDC); there is no registry token secret. `ethcli` trusts `release.yml`, every other crate trusts `publish-crate.yml`, and all crates require trusted publishing

**Re-running a release:** `gh workflow run release.yml -f tag=ethcli-vX.Y.Z` (rebuilds the binaries; publishing is skipped if the version is already on crates.io).

## Pre-commit Hooks

Installed hooks run automatically on commit:
- `cargo fmt --check` - formatting check
- `cargo clippy -- -D warnings` - lint check

## Architecture Notes

- Uses user-configured RPC endpoints (add with `ethcli endpoints add <url>`)
- **Archive node routing**: Historical queries prefer archive nodes when configured
- Parallel requests with automatic failover on errors
- Health tracking disables failing endpoints temporarily
- Checkpoint system allows resuming interrupted fetches
- Etherscan API v2 for ABI fetching (works without API key, rate limited)
- foundry-block-explorers for account, contract, token, and gas commands
- **Block numbers**: Accept both decimal (`17382257`) and hex (`0x1093b71`) format

### Endpoint Management

```bash
# Add endpoints with node type for proper archive routing
ethcli endpoints add https://my-archive.com --node-type archive
ethcli endpoints add https://my-node.com --node-type full --has-debug --priority 10

# Node types: archive (full history), full (recent only), unknown (default)
# Historical queries (old blocks) automatically prefer archive nodes if configured
# General reads avoid restricted relay RPCs such as Flashbots when other endpoints exist
```
