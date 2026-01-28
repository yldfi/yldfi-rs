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
ethcli rpc block <num>              # Get block by number
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
ethcli simulate tx <hash>           # Trace existing tx
```

## Tenderly (requires TENDERLY_ACCESS_KEY)

```bash
ethcli tenderly vnets list --project <p> --account <a>
ethcli tenderly vnets create --slug <s> --network-id 1 ...
ethcli tenderly vnets admin --vnet <id> set-balance <addr> 10eth ...
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
ethcli endpoints add <url>          # Add RPC endpoint
ethcli endpoints list               # List endpoints
ethcli endpoints optimize --all     # Optimize all
ethcli endpoints health             # Check health
ethcli doctor                       # Diagnose issues
```

## Environment Variables

| Variable | Required For | Description |
|----------|-------------|-------------|
| ETHERSCAN_API_KEY | Optional | Higher rate limits |
| ALCHEMY_API_KEY | alchemy commands | Alchemy API |
| MORALIS_API_KEY | moralis commands | Moralis API |
| COINGECKO_API_KEY | Optional | CoinGecko Pro |
| DUNE_API_KEY | dune commands | Dune Analytics |
| DUNE_SIM_API_KEY | dsim commands | Dune SIM |
| TENDERLY_ACCESS_KEY | tenderly commands | Tenderly API |
| THEGRAPH_API_KEY | uniswap subgraph | The Graph |
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
