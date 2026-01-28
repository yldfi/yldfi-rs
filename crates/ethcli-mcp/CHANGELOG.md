# Changelog

## [0.1.0](https://github.com/yldfi/yldfi-rs/releases/tag/ethcli-mcp-v0.1.0) (Unreleased)

### Features

* Initial release of ethcli-mcp
* MCP server exposing 236 ethcli tools for AI assistants
* Supports all ethcli commands: transaction analysis, account queries, contract operations, ENS, DEX aggregators, oracles, and more
* STDIO transport for integration with Claude Desktop, Claude Code, and other MCP clients
* Automatic config inheritance from ethcli (no separate configuration needed)

### Tool Categories

* **Core**: logs, tx, account, address, contract, token, gas, rpc, ens, sig
* **DeFi**: uniswap, curve, kong/yearn, yields, quote
* **DEX Aggregators**: 1inch, openocean, kyberswap, 0x, cowswap, lifi, velora, enso
* **Oracles**: chainlink, pyth
* **Data Providers**: alchemy, gecko, llama, moralis, dune, dsim, ccxt
* **Security**: goplus, solodit, blacklist
* **Infrastructure**: tenderly, simulate, config, endpoints
