# Changelog

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.3...ethcli-mcp-v0.1.4) (2026-01-30)


### Features

* **ethcli:** add bytecode analysis and fix MCP bugs ([b74a17c](https://github.com/yldfi/yldfi-rs/commit/b74a17c2c0865fd335506f19fe4711a7c828ac4e))


### Bug Fixes

* resolve clippy warnings ([3dcea82](https://github.com/yldfi/yldfi-rs/commit/3dcea82e8af3dcfe0f077365b90098e88f16a75b))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.2...ethcli-mcp-v0.1.3) (2026-01-30)


### Bug Fixes

* update Enso API client for new endpoints ([1c631e9](https://github.com/yldfi/yldfi-rs/commit/1c631e9eacc6d43813cd5d8160f37a0eb27706f1))

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
