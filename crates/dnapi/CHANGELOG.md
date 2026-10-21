# Changelog

## [0.3.0](https://github.com/yldfi/yldfi-rs/compare/dnapi-v0.2.0...dnapi-v0.3.0) (2026-10-21)


### Bug Fixes

* **dnapi:** pipeline and query-list types, results options, cancel on timeout ([ef2d682](https://github.com/yldfi/yldfi-rs/commit/ef2d6821f9c15450f5c10486c94f929d80ddd71e))
* **dnapi:** send sql, insert CSV/NDJSON, handle partial results ([38720cb](https://github.com/yldfi/yldfi-rs/commit/38720cb32d236205327abea489dcfbe24f117934))
* route API errors through yldfi-common and redact API keys ([f5595c5](https://github.com/yldfi/yldfi-rs/commit/f5595c5dc4a6a2d496e6a3565c4d6eb20d6a5232))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1 to 0.1.6

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
