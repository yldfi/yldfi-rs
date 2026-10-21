# Changelog

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/cowp-v0.1.5...cowp-v0.2.0) (2026-10-21)


### Features

* **cowp:** add Base, Polygon, Avalanche, BNB, Linea, Plasma and Ink ([4748952](https://github.com/yldfi/yldfi-rs/commit/47489521979534e6b619dd9b99e9f1a8c2aefb50))


### Bug Fixes

* **cowp:** accept bare-string order UID and page owner orders ([01159f4](https://github.com/yldfi/yldfi-rs/commit/01159f40fde07768c81bfcff6a876768142e36d0))
* **cowp:** onchainOrder, optional quote from, ethsign ([8aace42](https://github.com/yldfi/yldfi-rs/commit/8aace425eaac6848d31df35cf5facbc81ce8e3d9))
* handle dated upstream API changes (Etherscan, DefiLlama, Chainlink, Kong, CoW) ([b713e16](https://github.com/yldfi/yldfi-rs/commit/b713e1618c9a9ca3cd0d5d79cec0149c08498006))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

## [0.1.5](https://github.com/yldfi/yldfi-rs/compare/cowp-v0.1.4...cowp-v0.1.5) (2026-09-25)


### Bug Fixes

* DEX aggregator, Curve, Uniswap, yields and Kong bugs found by live smoke test ([#81](https://github.com/yldfi/yldfi-rs/issues/81)) ([730e9eb](https://github.com/yldfi/yldfi-rs/commit/730e9eb388557e434fd530c64d8fc1df17e7b9e8))
* migrate DEX/aggregator clients off deprecated endpoints (cowp, oinch, vlra, openoc, crv, cgko, dllma) ([#74](https://github.com/yldfi/yldfi-rs/issues/74)) ([1981119](https://github.com/yldfi/yldfi-rs/commit/19811196ca836fa14a4364c8c758895da8411347))

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/cowp-v0.1.3...cowp-v0.1.4) (2026-02-15)


### Bug Fixes

* rename colliding example filenames across crates ([d12f5b2](https://github.com/yldfi/yldfi-rs/commit/d12f5b2d977df3dc0a56c016a793d8bf37ed0c64))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/cowp-v0.1.2...cowp-v0.1.3) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/cowp-v0.1.1...cowp-v0.1.2) (2026-01-26)


### Bug Fixes

* address 44 issues from multi-agent code review ([5812bae](https://github.com/yldfi/yldfi-rs/commit/5812baea74461394a0f8a68ec490987b1cda168d))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/cowp-v0.1.0...cowp-v0.1.1) (2026-01-17)


### Bug Fixes

* update remaining old crate names in doc examples ([b746668](https://github.com/yldfi/yldfi-rs/commit/b746668ee41c6e657b9d0b9efe557a88490e5355))
