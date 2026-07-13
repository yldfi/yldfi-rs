# ethcli LLM Quick Reference

Condensed reference for LLM context. For full docs see CLAUDE.md.

## Global Flags
```
--chain <name>       Chain: ethereum|polygon|arbitrum|optimism|base|bsc|avalanche
--etherscan-key <k>  Etherscan API key (or ETHERSCAN_API_KEY env)
-v/-vv/-vvv          Verbosity level
-q/--quiet           Suppress progress output
```

## Core Commands

### Transaction Analysis
```bash
ethcli tx <hash>                    # Analyze transaction
ethcli tx <hash> --enrich           # With Etherscan enrichment
ethcli tx -f hashes.txt             # From file
ethcli tx --stdin                   # From stdin
ethcli tx <h1> <h2> --batch-size 10 # Parallel batch
```

### Account Operations
```bash
ethcli account balance <addr>       # ETH balance (supports ENS)
ethcli account balance <a1> <a2>    # Multiple (uses multicall)
ethcli account txs <addr>           # Transaction history
ethcli account erc20 <addr>         # ERC20 transfers
ethcli account erc721 <addr>        # NFT transfers
ethcli account info <addr>          # Comprehensive info
```

### Contract Operations
```bash
ethcli contract abi <addr>          # Download ABI (JSON)
ethcli contract source <addr>       # Download source code
ethcli contract creation <addr>     # Creation tx & deployer
ethcli contract verify-status <addr> # Verification status
```

### Bytecode Analysis
```bash
ethcli contract selectors <addr>    # Extract function selectors (evmole)
ethcli contract sel <addr> --lookup # With 4byte.directory lookup
ethcli contract sel <addr> --follow-proxy --lookup
                                     # Implementation selectors for proxies
ethcli contract disassemble <addr>  # Full opcode disassembly
ethcli contract dis <addr> --limit 50 # Limit output
ethcli contract opcodes <addr>      # Opcode frequency stats
ethcli contract analyze <addr>      # Combined security analysis
ethcli contract az <addr> --lookup  # With signature lookup
ethcli contract analyze <addr> --follow-proxy --lookup --dispatcher --checks
                                      # Unverified/proxy review: handler map + guard heuristics
```

### ENS Resolution
```bash
ethcli ens resolve <name>           # ENS to address
ethcli ens lookup <addr>            # Reverse lookup
ethcli ens namehash <name>          # Compute namehash
```

### Token Operations
```bash
ethcli token info <addr>            # Token metadata
ethcli token balance <token> <wallet> # Token balance
ethcli token holders <addr>         # Top holders
ethcli token supply <addr>          # Total supply
```

### Gas Oracle
```bash
ethcli gas oracle                   # Current gas prices
ethcli gas estimate --to <a> --value <v> # Estimate gas
ethcli gas history                  # Historical gas
```

### Signature Lookup
```bash
ethcli sig fn <selector>            # Function by 4-byte selector
ethcli sig event <topic>            # Event by topic hash
ethcli sig encode "transfer(address,uint256)" # Get selector
```

### Cast Utilities
```bash
ethcli cast to-wei 1.5 eth          # ETH to wei
ethcli cast from-wei <wei> eth      # Wei to ETH
ethcli cast to-hex 255              # Decimal to hex
ethcli cast from-hex 0xff           # Hex to decimal
ethcli cast keccak "text"           # Keccak256 hash
ethcli cast sig "fn(type,type)"     # Function selector
ethcli cast abi-encode "fn(t,t)" v1 v2  # ABI encode
ethcli cast abi-decode "fn(t,t)" <data> # ABI decode
ethcli cast checksum <addr>         # Checksum address
ethcli cast concat-hex 0x1 0x2      # Concatenate hex
```

### RPC Direct Calls
```bash
ethcli rpc block latest             # Get latest block
ethcli rpc block <num>              # Get block by number (decimal or hex)
ethcli rpc block 0x1406f40          # Hex block numbers supported
ethcli rpc call <to> <data>         # eth_call
ethcli rpc code <addr>              # Contract bytecode
ethcli rpc storage <addr> <slot>    # Storage slot
ethcli rpc receipt <hash>           # Transaction receipt
ethcli rpc nonce <addr>             # Account nonce
```

### Event Logs
```bash
ethcli logs -c <contract> -e "Transfer(address,address,uint256)" -f <from> -t <to>
ethcli logs -c <contract> --since 7d    # Last 7 days
ethcli logs -c <contract> --resume      # Resumable fetch
ethcli logs ... --format json|ndjson    # Output format
```

## Aggregation Commands

### Price (multi-source)
```bash
ethcli price ETH                    # By symbol
ethcli price <token_addr>           # By address
ethcli price ETH --sources gecko,llama # Specific sources
```

### Portfolio
```bash
ethcli portfolio <wallet>           # All token balances
ethcli portfolio <wallet> --exclude-spam # Filter spam
```

### Quote (DEX aggregators)
```bash
ethcli quote best ETH USDC <amount> # Best quote
ethcli quote compare ETH USDC <amt> # Compare all
ethcli quote from openocean ETH USDC <amt> # Specific source
```

### Yields
```bash
ethcli yields                       # All DeFi yields
ethcli yields --protocol aave       # Filter by protocol
ethcli yields --chain ethereum      # Filter by chain
```

### NFTs (multi-source)
```bash
ethcli nfts <wallet>                # NFT holdings
ethcli nfts <wallet> --chain ethereum
```

## Direct API Commands

### Alchemy (requires ALCHEMY_API_KEY)
```bash
ethcli alchemy balances <addr>
ethcli alchemy nfts <addr>
ethcli alchemy transfers <addr> --category erc20
ethcli alchemy trace-tx <hash>
```

### CoinGecko (optional COINGECKO_API_KEY)
```bash
ethcli gecko coin bitcoin
ethcli gecko price bitcoin,ethereum --vs usd
ethcli gecko markets --per-page 100
```

### DefiLlama
```bash
ethcli llama tvl aave
ethcli llama price ethereum:<token>
ethcli llama yields --chain ethereum
```

### Moralis (requires MORALIS_API_KEY)
Fantom is blocked for Moralis calls after Moralis' 2026-05-29 removal notice.
Legacy Discovery, Volume, Market Data, selected ERC20 helper, and pair sniper
commands are blocked ahead of the 2026-06-04 endpoint removal.

```bash
ethcli moralis balance <addr>
ethcli moralis tokens <addr>
ethcli moralis defi-positions <addr>
```

### Chainlink (RPC-based, no key)
```bash
ethcli chainlink price ETH          # Current price
ethcli chainlink price ETH --block <n> # Historical
ethcli chainlink oracles            # List known oracles
```

### Dune (requires DUNE_API_KEY)
```bash
ethcli dune query <id>              # Run query
ethcli dune results <id>            # Get results
```

### Uniswap
```bash
ethcli uniswap pool <addr>          # Pool state (on-chain)
ethcli uniswap eth-price            # ETH price (subgraph)
ethcli uniswap top-pools 10         # Top pools by TVL
ethcli uniswap positions <addr>     # LP positions
```

### Yearn/Kong
```bash
ethcli kong vaults list
ethcli kong vaults get --chain-id 1 <addr>
ethcli kong strategies list
```

### GoPlus Security
```bash
ethcli goplus token <addr> --chain-id 1  # Token security
ethcli goplus address <addr> --chain-id 1 # Address security
```

### Solodit (requires SOLODIT_API_KEY)
```bash
ethcli solodit search "reentrancy" --impact HIGH
ethcli solodit get <slug>
```

### Dune SIM (requires DUNE_SIM_API_KEY)
Dune Sim shuts down 2026-08-01 (yldfi-rs issue #64); dsim commands print a
sunset warning to stderr. `ethcli dsim defi` is blocked (DeFi Positions was
deprecated 2026-06-01). DUNE_API_KEY is no longer accepted as a fallback.

```bash
ethcli dsim balances <addr>         # Wallet balances
ethcli dsim activity <addr>         # Wallet activity
ethcli dsim collectibles <addr>     # NFTs
```

### Curve
```bash
ethcli curve pools                  # List pools
ethcli curve pool <addr>            # Pool details
ethcli curve volumes                # Volume data
ethcli curve router route <from> <to> # Find swap route
```

### CCXT (Exchange Data)
```bash
ethcli ccxt ticker binance BTC/USDT # Get ticker
ethcli ccxt orderbook binance BTC/USDT # Order book
ethcli ccxt ohlcv binance BTC/USDT --timeframe 1h # Candles
ethcli ccxt exchanges               # List exchanges
```

### Blacklist (Spam Token Filtering)
```bash
ethcli blacklist scan <token>       # Scan token security
ethcli blacklist scan-portfolio <wallet> --auto-blacklist
ethcli blacklist list               # List blacklisted tokens
ethcli blacklist add <token> --reason "Scam"
ethcli blacklist check <token>      # Check if blacklisted
```

## DEX Aggregator Commands

### 1inch (requires 1INCH_API_KEY)
```bash
ethcli 1inch quote <src> <dst> <amt>
ethcli 1inch swap <src> <dst> <amt> <from>
```

### OpenOcean
```bash
ethcli openocean quote <in> <out> <amt>
ethcli openocean swap <in> <out> <amt> <account>
```

### KyberSwap
```bash
ethcli kyberswap routes <in> <out> <amt>
```

### 0x (optional ZEROX_API_KEY)
```bash
ethcli 0x quote <sell> <buy> <amt> <taker>
ethcli 0x price <sell> <buy> <amt> <taker>
```

### CowSwap (MEV-protected)
```bash
ethcli cowswap quote <sell> <buy> <amt> <from>
ethcli cowswap order <uid>
```

### LI.FI (cross-chain)
```bash
ethcli lifi quote <from_chain> <token> <to_chain> <token> <amt> <addr>
ethcli lifi chains
```

### Velora/ParaSwap
```bash
ethcli velora price <src> <dst> <amt>
```

### Enso (requires ENSO_API_KEY)
```bash
ethcli enso route <in> <out> <amt> <from>
```

### Pyth
```bash
ethcli pyth price BTC/USD
ethcli pyth search "ETH"
```

## Simulation & Tracing

```bash
ethcli simulate call <contract> --sig "fn(types)" <args>
ethcli simulate call ... --via tenderly|anvil|debug|trace
ethcli simulate call ... --trace --decode-internal --label 0x...:name
ethcli simulate call ... --via anvil --fork-url <rpc> --fork-block-number -10
ethcli simulate tx <hash> --decode-internal --trace-depth 6
ethcli simulate tx <hash> --via debug            # decoded call tree (names, args, events); no Foundry needed
ethcli simulate tx <hash> --via debug --raw      # raw callTracer JSON
```

## Tenderly (requires TENDERLY_ACCESS_KEY)

```bash
ethcli tenderly vnets list --project <p> --account <a>
ethcli tenderly vnets create --slug <s> --network-id 1 ...
ethcli tenderly vnets admin --vnet <id> set-balance <addr> 10eth ...
ethcli tenderly vnets admin --vnet <id> simulate-tx --from <addr> --to <addr> --data 0x... ...
ethcli tenderly vnets admin --vnet <id> simulate-bundle '[{"from":"0x...","to":"0x..."}]' ...
ethcli tenderly wallets list ...
ethcli tenderly contracts add <addr> --network 1 ...
```

## Configuration

```bash
ethcli config init                  # Create config file
ethcli config path                  # Show config path
ethcli config show                  # Display config
ethcli config validate              # Validate config
ethcli config set-etherscan-key <k> # Set API key
ethcli config set-tenderly --key <k> --account <a> --project <p>
ethcli endpoints add <url>          # Add RPC endpoint (auto-detects)
ethcli endpoints add <url> --node-type archive  # Mark as archive node
ethcli endpoints add <url> --node-type full --has-debug --priority 10
ethcli endpoints list               # List endpoints
ethcli endpoints list --archive     # Filter archive nodes only
ethcli endpoints optimize --all     # Optimize all
ethcli endpoints health --probes 1  # Check endpoint health quickly
ethcli doctor                       # Diagnose issues
```

## Environment Variables

| Variable | Required For | Description |
|----------|-------------|-------------|
| ETHERSCAN_API_KEY | Optional | Higher rate limits |
| ETHCLI_NO_PROXY | Optional | Disable HTTP proxy auto-detection |
| ALCHEMY_API_KEY | alchemy commands | Alchemy API |
| MORALIS_API_KEY | moralis commands | Moralis API |
| COINGECKO_API_KEY | Optional | CoinGecko Pro |
| DUNE_API_KEY | dune commands | Dune Analytics |
| DUNE_SIM_API_KEY | dsim commands | Dune SIM (sunset 2026-08-01) |
| TENDERLY_ACCESS_KEY | tenderly commands | Tenderly API |
| THEGRAPH_API_KEY | uniswap subgraph | The Graph |
| GOPLUS_APP_KEY | Optional | GoPlus batch queries |
| GOPLUS_APP_SECRET | Optional | GoPlus batch queries |
| 1INCH_API_KEY | 1inch commands | 1inch API |
| ZEROX_API_KEY | Optional | 0x higher limits |
| ENSO_API_KEY | enso commands | Enso Finance |
| SOLODIT_API_KEY | solodit commands | Solodit DB |
| CHAINLINK_API_KEY | chainlink streams | Data Streams |
| CHAINLINK_USER_SECRET | chainlink streams | Data Streams |

## Output Formats

Most commands support: `--output json|table|ndjson` or `-o json|table|ndjson`

## Aliases

| Alias | Command |
|-------|---------|
| t | tx |
| acc | account |
| addr | address |
| c | contract |
| tok | token |
| g | gas |
| ep | endpoints |
| cfg | config |
| log | logs |
| p | price |
| pf | portfolio |
| nft | nfts |
| q | quote |
| y | yields |
| uni | uniswap |
| cex | ccxt |
| yearn | kong |
| gp | goplus |
| sld | solodit |
| bl | blacklist |
| oneinch | 1inch |
| oo | openocean |
| kyber | kyberswap |
| zerox | 0x |
| cow | cowswap |
| li.fi | lifi |
| paraswap | velora |
