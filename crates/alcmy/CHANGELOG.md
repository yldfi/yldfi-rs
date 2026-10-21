# Changelog

## [0.3.0](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.2.0...alcmy-v0.3.0) (2026-10-21)


### Bug Fixes

* **alcmy:** match Gas Manager, Solana DAS, Signer, portfolio and trace shapes ([756bb2c](https://github.com/yldfi/yldfi-rs/commit/756bb2cbb6dad59a16b74e7e869ed67c08842ae8))
* **alcmy:** match Notify, NFT and debug shapes; strip URLs from errors ([6714db9](https://github.com/yldfi/yldfi-rs/commit/6714db97ae98c2e9a274f75edf80c5d60da332fc))
* route API errors through yldfi-common and redact API keys ([f5595c5](https://github.com/yldfi/yldfi-rs/commit/f5595c5dc4a6a2d496e6a3565c4d6eb20d6a5232))
* smaller upstream API changes (0x, 1inch, Moralis, CoinGecko, Alchemy, LI.FI, KyberSwap) ([964be4c](https://github.com/yldfi/yldfi-rs/commit/964be4c1ba63778f9aac630b509886e0349123e7))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.6...alcmy-v0.2.0) (2026-09-25)


### ⚠ BREAKING CHANGES

* the removed alcmy methods/types, ethcli subcommands and MCP tools no longer exist. Use contracts-for-owner, contract-metadata (openSeaMetadata), is-spam and nfts-for-contract instead.

### Features

* remove sunset Alchemy NFT/Simulation APIs and Dune Sim integration ([#71](https://github.com/yldfi/yldfi-rs/issues/71)) ([416d973](https://github.com/yldfi/yldfi-rs/commit/416d97332c5e39696b10d522465a2b3e65b152b4))


### Bug Fixes

* provider client bugs found by live smoke test (alcmy, tndrly, mrls, gplus) ([#75](https://github.com/yldfi/yldfi-rs/issues/75)) ([8b50186](https://github.com/yldfi/yldfi-rs/commit/8b50186a012a7dbdec8409f0e93287ef1c88c1f7))

## [0.1.6](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.5...alcmy-v0.1.6) (2026-02-16)


### Bug Fixes

* **ethcli:** fix 6 critical CLI bugs — gas estimate, beacon auth, allowance args, CCXT monthly, chainlink quote, sim overrides ([31b1cff](https://github.com/yldfi/yldfi-rs/commit/31b1cff89a1c0d150031a9a943a420eadae96aee)), closes [#37](https://github.com/yldfi/yldfi-rs/issues/37)
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([#38](https://github.com/yldfi/yldfi-rs/issues/38)) ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))

## [0.1.5](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.4...alcmy-v0.1.5) (2026-02-15)


### Bug Fixes

* rename colliding example filenames across crates ([d12f5b2](https://github.com/yldfi/yldfi-rs/commit/d12f5b2d977df3dc0a56c016a793d8bf37ed0c64))

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.3...alcmy-v0.1.4) (2026-01-30)


### Bug Fixes

* update Enso API client for new endpoints ([1c631e9](https://github.com/yldfi/yldfi-rs/commit/1c631e9eacc6d43813cd5d8160f37a0eb27706f1))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.2...alcmy-v0.1.3) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.1...alcmy-v0.1.2) (2026-01-26)


### Features

* **ethcli:** add Uniswap V2/V3/V4 support and refactor API clients ([46df1e0](https://github.com/yldfi/yldfi-rs/commit/46df1e0b7bdf733f504b00931cd310ec429cc0c7))


### Bug Fixes

* address 44 issues from multi-agent code review ([5812bae](https://github.com/yldfi/yldfi-rs/commit/5812baea74461394a0f8a68ec490987b1cda168d))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/alcmy-v0.1.0...alcmy-v0.1.1) (2026-01-17)


### Features

* comprehensive codebase improvements and new API crates ([d0c2831](https://github.com/yldfi/yldfi-rs/commit/d0c2831dfe6242bfd8cff8f983fe6bf7c62e8af9))
