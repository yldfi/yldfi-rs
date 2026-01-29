# Changelog

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.1...ethcli-mcp-v0.1.2) (2026-01-29)


### Features

* **ethcli-mcp:** full feature parity with CLI ([1e5f704](https://github.com/yldfi/yldfi-rs/commit/1e5f704892550f5d95ec9d785964b89bd18ac657))


### Bug Fixes

* add clippy allow for too_many_arguments ([926fab1](https://github.com/yldfi/yldfi-rs/commit/926fab171926c54bd88d104daf18f6b60f65db67))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.0...ethcli-mcp-v0.1.1) (2026-01-29)


### Bug Fixes

* **ethcli-mcp:** prevent data truncation in MCP tool outputs ([12da461](https://github.com/yldfi/yldfi-rs/commit/12da4612be2f9826971498a3c7a0c7a1c475e5f4))

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
