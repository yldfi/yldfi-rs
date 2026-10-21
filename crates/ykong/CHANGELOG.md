# Changelog

## [0.1.6](https://github.com/yldfi/yldfi-rs/compare/ykong-v0.1.5...ykong-v0.1.6) (2026-10-21)


### Bug Fixes

* handle dated upstream API changes (Etherscan, DefiLlama, Chainlink, Kong, CoW) ([b713e16](https://github.com/yldfi/yldfi-rs/commit/b713e1618c9a9ca3cd0d5d79cec0149c08498006))
* **ykong:** pass GraphQL arguments as variables ([6240247](https://github.com/yldfi/yldfi-rs/commit/6240247a8c467e6f35598ce5106299150bc8fd5f))
* **ykong:** require a chain for vault strategies, keep 5xx retryable ([8e23c79](https://github.com/yldfi/yldfi-rs/commit/8e23c799e87d607eaada928895a4b63141870bdb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

## [0.1.5](https://github.com/yldfi/yldfi-rs/compare/ykong-v0.1.4...ykong-v0.1.5) (2026-09-25)


### Bug Fixes

* DEX aggregator, Curve, Uniswap, yields and Kong bugs found by live smoke test ([#81](https://github.com/yldfi/yldfi-rs/issues/81)) ([730e9eb](https://github.com/yldfi/yldfi-rs/commit/730e9eb388557e434fd530c64d8fc1df17e7b9e8))

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/ykong-v0.1.3...ykong-v0.1.4) (2026-02-14)


### Bug Fixes

* **ykong:** deserialize Kong price/tvl block_number and timestamp from strings ([2766f41](https://github.com/yldfi/yldfi-rs/commit/2766f41d3101fcdbc4db65accede40bdc592893b))
* **ykong:** deserialize Kong price/tvl fields from strings ([4f25e5f](https://github.com/yldfi/yldfi-rs/commit/4f25e5f2e5886284989d45ae4c0d90b337b66b7d))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/ykong-v0.1.2...ykong-v0.1.3) (2026-02-02)


### Bug Fixes

* use assert! instead of assert_eq! with literal bool ([9712503](https://github.com/yldfi/yldfi-rs/commit/97125034e229b815e38f6dea8e061ac174104793))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/ykong-v0.1.1...ykong-v0.1.2) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/ykong-v0.1.0...ykong-v0.1.1) (2026-01-26)


### Features

* add ykong (Yearn Kong) and unswp improvements ([f065b43](https://github.com/yldfi/yldfi-rs/commit/f065b4313245716d0671df278441bc05f048f872))
