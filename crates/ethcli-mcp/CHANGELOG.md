# Changelog

## Unreleased

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.14...ethcli-mcp-v0.2.0) (2026-05-04)

### Features

* **mcp:** expose contract dispatcher mapping and side-effecting handler guard/check heuristics on contract_analyze
* **mcp:** return JSON for token_balance and endpoints_health
* **mcp:** expose proxy following on contract_selectors

### Bug Fixes

* **mcp:** bound endpoints_health to one probe by default and expose the probes argument

## [0.1.14](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.13...ethcli-mcp-v0.1.14) (2026-04-22)


### Bug Fixes

* satisfy latest clippy lints ([c97038b](https://github.com/yldfi/yldfi-rs/commit/c97038b9129d8ddc901fbbf38fbcd8aa05af7d10))
* support explicit contract signatures and dynamic abi decode ([f453b86](https://github.com/yldfi/yldfi-rs/commit/f453b86ce319f74d8a582501574f2c2352c474c2))

## [0.1.13](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.12...ethcli-mcp-v0.1.13) (2026-02-16)


### Bug Fixes

* **mcp:** place -n/--network at correct subcommand level for Alchemy tools ([#43](https://github.com/yldfi/yldfi-rs/issues/43)) ([19c7a7d](https://github.com/yldfi/yldfi-rs/commit/19c7a7d9fcf45118ecb984f6b5b79baf5be0fd3c))

## [0.1.12](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.11...ethcli-mcp-v0.1.12) (2026-02-16)


### Bug Fixes

* **mcp:** restore --network flag for Alchemy and Tenderly tools ([ff62ab6](https://github.com/yldfi/yldfi-rs/commit/ff62ab664c3d4db0c7d74c0673f810c068f441ec))
* **mcp:** restore --network flag for Alchemy and Tenderly tools ([4997517](https://github.com/yldfi/yldfi-rs/commit/4997517c07281a1a02961b76c8e267e16c6f3e4b))
* **mcp:** restore --network flag for Alchemy and Tenderly tools ([#41](https://github.com/yldfi/yldfi-rs/issues/41)) ([ff62ab6](https://github.com/yldfi/yldfi-rs/commit/ff62ab664c3d4db0c7d74c0673f810c068f441ec))

## [0.1.11](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.10...ethcli-mcp-v0.1.11) (2026-02-16)


### Bug Fixes

* **mcp:** correct CLI flag names and placement for Alchemy and Moralis tools ([ff55e4d](https://github.com/yldfi/yldfi-rs/commit/ff55e4d074d01d6a6e578bbeae8b098b3d1c3f64))
* **mcp:** correct CLI flag names and placement for Alchemy and Moralis tools ([0ec24b8](https://github.com/yldfi/yldfi-rs/commit/0ec24b8e9b64969c1ae879f58bff695e56ccad1e))
* **mcp:** correct CLI flag names and placement for Alchemy and Moralis tools ([#39](https://github.com/yldfi/yldfi-rs/issues/39)) ([ff55e4d](https://github.com/yldfi/yldfi-rs/commit/ff55e4d074d01d6a6e578bbeae8b098b3d1c3f64))

## [0.1.10](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.9...ethcli-mcp-v0.1.10) (2026-02-16)


### Bug Fixes

* address Copilot review feedback on PR [#38](https://github.com/yldfi/yldfi-rs/issues/38) ([79c9900](https://github.com/yldfi/yldfi-rs/commit/79c99002ad070ec849364e4194d68d210f1f633f))
* **mcp:** add missing from_address param to enso_route ([#34](https://github.com/yldfi/yldfi-rs/issues/34)) ([88c4738](https://github.com/yldfi/yldfi-rs/commit/88c473835585bb4c731d5e1c8ee0728dc1766e2f))
* **mcp:** fix ~45 critical parameter mismatches across all MCP tool wrappers ([c1ee10b](https://github.com/yldfi/yldfi-rs/commit/c1ee10bca0ef977bd1a7a9f48387d5fe9f48b5cf)), closes [#36](https://github.com/yldfi/yldfi-rs/issues/36)
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([#38](https://github.com/yldfi/yldfi-rs/issues/38)) ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))

## [0.1.9](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.8...ethcli-mcp-v0.1.9) (2026-02-14)


### Bug Fixes

* address Copilot review feedback for merged PRs ([4bec07c](https://github.com/yldfi/yldfi-rs/commit/4bec07cc1974b1ecb841cad2455ebaf7dbb5cf30))
* address Copilot review feedback for merged PRs ([b497355](https://github.com/yldfi/yldfi-rs/commit/b4973555548a5d2dd7fbcd0cdfcdaa90cdd62a58))

## [0.1.8](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.7...ethcli-mcp-v0.1.8) (2026-02-14)


### Features

* add multi-chain support and chainlist.org integration ([48c3e09](https://github.com/yldfi/yldfi-rs/commit/48c3e0943105f6680e65ec5743362649165b0f52))

## [0.1.7](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.6...ethcli-mcp-v0.1.7) (2026-02-13)


### Features

* add 130+ CLI commands for Alchemy & Moralis, fix formatting ([b580a6e](https://github.com/yldfi/yldfi-rs/commit/b580a6edb67179787771d854bc44dcec5ddfc8c0))
* add 169 MCP tools, fix deserialization bugs across Moralis/CoinGecko/Curve ([92085a1](https://github.com/yldfi/yldfi-rs/commit/92085a1fa7639eb1e2d62f8a0f2a836af264a257))
* add 87 large-effort MCP tools — Curve, CoinGecko, Alchemy, Moralis ([380f7b9](https://github.com/yldfi/yldfi-rs/commit/380f7b9053292570a5e3b3321071d60fc938bcb4))
* add medium-effort API coverage — Tenderly batch, Dune CRUD, CowSwap orders ([5cdcf89](https://github.com/yldfi/yldfi-rs/commit/5cdcf892aab53199aa061021bac41d963be593ce))
* **tenderly:** full VNet integration with Admin RPC and simulation ([dac43e2](https://github.com/yldfi/yldfi-rs/commit/dac43e202028861fcb80474bf3b807560b6722bd))
* **tenderly:** full VNet integration with Admin RPC and simulation ([bb6d849](https://github.com/yldfi/yldfi-rs/commit/bb6d84985bf33a126c71fbcc0176c9605b219a85))


### Bug Fixes

* **ethcli-mcp:** require amount for set_erc20_balance tool ([47e2528](https://github.com/yldfi/yldfi-rs/commit/47e252830b47a8cea0963e3c75d270a9742e2f45))
* resolve clippy warnings for too_many_arguments and useless_format ([68ac6d0](https://github.com/yldfi/yldfi-rs/commit/68ac6d035360d87061507928db95dcaf346648ac))

## [0.1.6](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.5...ethcli-mcp-v0.1.6) (2026-02-02)


### Bug Fixes

* **mcp:** pass function signature as positional arg in contract_call ([1a2b72a](https://github.com/yldfi/yldfi-rs/commit/1a2b72a81073b9bdd99874eeb5de65271dc72e59)), closes [#15](https://github.com/yldfi/yldfi-rs/issues/15)

## [0.1.5](https://github.com/yldfi/yldfi-rs/compare/ethcli-mcp-v0.1.4...ethcli-mcp-v0.1.5) (2026-02-02)


### Features

* **ethcli-mcp:** expose missing CLI parameters for feature parity ([fb85ffe](https://github.com/yldfi/yldfi-rs/commit/fb85ffe9ad6252119de3c65e92b11b1fe26db3d0))

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
