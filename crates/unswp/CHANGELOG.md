# Changelog

## [0.1.7](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.6...unswp-v0.1.7) (2026-10-21)


### Bug Fixes

* route API errors through yldfi-common and redact API keys ([f5595c5](https://github.com/yldfi/yldfi-rs/commit/f5595c5dc4a6a2d496e6a3565c4d6eb20d6a5232))
* **unswp:** pass subgraph filters as variables and page day stats ([d0de7c4](https://github.com/yldfi/yldfi-rs/commit/d0de7c43485a27fa82167feeaa67d3b5170dc06a))
* **unswp:** V2 day data, day-stats ranking window, exact V2 deltas ([a398bfb](https://github.com/yldfi/yldfi-rs/commit/a398bfb33b2609e080f2fb57bb1556505946a979))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

## [0.1.6](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.5...unswp-v0.1.6) (2026-09-25)


### Bug Fixes

* DEX aggregator, Curve, Uniswap, yields and Kong bugs found by live smoke test ([#81](https://github.com/yldfi/yldfi-rs/issues/81)) ([730e9eb](https://github.com/yldfi/yldfi-rs/commit/730e9eb388557e434fd530c64d8fc1df17e7b9e8))
* **ethcli,unswp:** replace dead RPCs, fix V4 subgraph IDs, mainnet Data Streams, Etherscan sunset chains ([#72](https://github.com/yldfi/yldfi-rs/issues/72)) ([64ff968](https://github.com/yldfi/yldfi-rs/commit/64ff968a4d7166ab9a9c4e3f896bf8d816f53721))

## [0.1.5](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.4...unswp-v0.1.5) (2026-05-10)


### Bug Fixes

* **crv:** update LendingVault struct to match Curve API response ([#19](https://github.com/yldfi/yldfi-rs/issues/19)) ([38a0bac](https://github.com/yldfi/yldfi-rs/commit/38a0bac6bdcae0772db212d5eae550ec25a8ec94))

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.3...unswp-v0.1.4) (2026-05-10)

### Dependencies

* **unswp:** upgrade direct Alloy dependency to 2.x and align uniswap-lens with 1.x

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.2...unswp-v0.1.3) (2026-02-04)


### Bug Fixes

* **crv:** update LendingVault struct to match Curve API response ([#19](https://github.com/yldfi/yldfi-rs/issues/19)) ([38a0bac](https://github.com/yldfi/yldfi-rs/commit/38a0bac6bdcae0772db212d5eae550ec25a8ec94))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.1...unswp-v0.1.2) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/unswp-v0.1.0...unswp-v0.1.1) (2026-01-26)


### Features

* add ykong (Yearn Kong) and unswp improvements ([f065b43](https://github.com/yldfi/yldfi-rs/commit/f065b4313245716d0671df278441bc05f048f872))
* **ethcli:** add Uniswap V2/V3/V4 support and refactor API clients ([46df1e0](https://github.com/yldfi/yldfi-rs/commit/46df1e0b7bdf733f504b00931cd310ec429cc0c7))
