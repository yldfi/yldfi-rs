# Changelog

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/oinch-v0.1.4...oinch-v0.2.0) (2026-10-21)


### ⚠ BREAKING CHANGES

* **oinch:** Chain::{Fantom, Klaytn, Aurora} are removed.

### Features

* **oinch:** match Chain to the 1inch Swap API's chain list ([f55bb9d](https://github.com/yldfi/yldfi-rs/commit/f55bb9d5aacb810f189c6799028999017f7ed389))


### Bug Fixes

* **oinch:** read and write logoURI ([0f33f28](https://github.com/yldfi/yldfi-rs/commit/0f33f281e5f653ab2e51f3533f02270e9357e076))
* **oinch:** sanitize the 402 response body like other API errors ([069c887](https://github.com/yldfi/yldfi-rs/commit/069c8871a6091158b3b68baf89283b598cdfe9b9))
* route API errors through yldfi-common and redact API keys ([f5595c5](https://github.com/yldfi/yldfi-rs/commit/f5595c5dc4a6a2d496e6a3565c4d6eb20d6a5232))
* smaller upstream API changes (0x, 1inch, Moralis, CoinGecko, Alchemy, LI.FI, KyberSwap) ([964be4c](https://github.com/yldfi/yldfi-rs/commit/964be4c1ba63778f9aac630b509886e0349123e7))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

## [0.1.4](https://github.com/yldfi/yldfi-rs/compare/oinch-v0.1.3...oinch-v0.1.4) (2026-09-25)


### Bug Fixes

* migrate DEX/aggregator clients off deprecated endpoints (cowp, oinch, vlra, openoc, crv, cgko, dllma) ([#74](https://github.com/yldfi/yldfi-rs/issues/74)) ([1981119](https://github.com/yldfi/yldfi-rs/commit/19811196ca836fa14a4364c8c758895da8411347))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/oinch-v0.1.2...oinch-v0.1.3) (2026-02-15)


### Bug Fixes

* rename colliding example filenames across crates ([d12f5b2](https://github.com/yldfi/yldfi-rs/commit/d12f5b2d977df3dc0a56c016a793d8bf37ed0c64))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/oinch-v0.1.1...oinch-v0.1.2) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/oinch-v0.1.0...oinch-v0.1.1) (2026-01-17)


### Bug Fixes

* update remaining old crate names in doc examples ([b746668](https://github.com/yldfi/yldfi-rs/commit/b746668ee41c6e657b9d0b9efe557a88490e5355))
