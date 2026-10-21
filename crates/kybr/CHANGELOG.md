# Changelog

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/kybr-v0.1.4...kybr-v0.2.0) (2026-10-21)


### ⚠ BREAKING CHANGES

* **kybr:** Chain::{Fantom, Scroll, Zksync, Blast, Mantle, PolygonZkEvm} are removed.

### Features

* **kybr:** match Chain to KyberSwap's active chains ([8481a61](https://github.com/yldfi/yldfi-rs/commit/8481a6106c3a1cbef045690183435138062d5ef1))


### Bug Fixes

* **kybr:** drop route fields v1 doesn't return ([b2d40b8](https://github.com/yldfi/yldfi-rs/commit/b2d40b8a33cba4734b2fc561055f52def63be272))
* **kybr:** send includedSources/excludedSources/gasInclude, add X-Client-Id ([187ce0e](https://github.com/yldfi/yldfi-rs/commit/187ce0e108e5e5b2327ced1ea988f0b73baa772d))
* smaller upstream API changes (0x, 1inch, Moralis, CoinGecko, Alchemy, LI.FI, KyberSwap) ([964be4c](https://github.com/yldfi/yldfi-rs/commit/964be4c1ba63778f9aac630b509886e0349123e7))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/kybr-v0.1.3...kybr-v0.1.4) (2026-09-25)


### Bug Fixes

* DEX aggregator, Curve, Uniswap, yields and Kong bugs found by live smoke test ([#81](https://github.com/yldfi/yldfi-rs/issues/81)) ([730e9eb](https://github.com/yldfi/yldfi-rs/commit/730e9eb388557e434fd530c64d8fc1df17e7b9e8))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/kybr-v0.1.2...kybr-v0.1.3) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/kybr-v0.1.1...kybr-v0.1.2) (2026-01-26)


### Bug Fixes

* address 44 issues from multi-agent code review ([5812bae](https://github.com/yldfi/yldfi-rs/commit/5812baea74461394a0f8a68ec490987b1cda168d))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/kybr-v0.1.0...kybr-v0.1.1) (2026-01-17)


### Bug Fixes

* update remaining old crate names in doc examples ([b746668](https://github.com/yldfi/yldfi-rs/commit/b746668ee41c6e657b9d0b9efe557a88490e5355))
