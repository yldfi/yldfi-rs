# Changelog

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/dnapi-v0.1.3...dnapi-v0.2.0) (2026-09-25)


### ⚠ BREAKING CHANGES

* the `volume` and `market` modules, `Client::volume()`, `Client::market()`, the DiscoveryApi list/filter/token methods and their types, and TokenApi::{get_stats, get_exchange_*_tokens, get_by_symbols, get_holders_historical, get_pairs_stats, get_pair_snipers, get_bonding_status} (plus their response types) are removed. DiscoveryApi keeps get_token_analytics and get_token_score, which are still in the spec (scores are EVM-only).

### Bug Fixes

* remove dead Moralis/Tenderly endpoints, fix Dune API paths ([#73](https://github.com/yldfi/yldfi-rs/issues/73)) ([78db18e](https://github.com/yldfi/yldfi-rs/commit/78db18e22b7eadb8ad71ce3cefdbc21bf4ea117c))

## [0.1.3](https://github.com/yldfi/yldfi-rs/compare/dnapi-v0.1.2...dnapi-v0.1.3) (2026-02-15)


### Bug Fixes

* rename colliding example filenames across crates ([d12f5b2](https://github.com/yldfi/yldfi-rs/commit/d12f5b2d977df3dc0a56c016a793d8bf37ed0c64))

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/dnapi-v0.1.1...dnapi-v0.1.2) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/dnapi-v0.1.0...dnapi-v0.1.1) (2026-01-17)


### Bug Fixes

* update remaining old crate names in doc examples ([b746668](https://github.com/yldfi/yldfi-rs/commit/b746668ee41c6e657b9d0b9efe557a88490e5355))
