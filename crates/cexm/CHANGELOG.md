# Changelog

## [0.1.2](https://github.com/yldfi/yldfi-rs/compare/cexm-v0.1.1...cexm-v0.1.2) (2026-11-17)


### Bug Fixes

* replace unmaintained chrono with time in cexm and ethcli ([#256](https://github.com/yldfi/yldfi-rs/issues/256)) ([113e640](https://github.com/yldfi/yldfi-rs/commit/113e64014957808f9fd4e6df612397553d3ab7ed))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/cexm-v0.1.0...cexm-v0.1.1) (2026-11-09)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * yldfi-common bumped from 0.1.6 to 0.1.7

## 0.1.0 (2026-11-04)


### ⚠ BREAKING CHANGES

* `ethcli ccxt` drops the `--testnet` flag, and ticker and order book JSON `timestamp` is null when the venue doesn't report one (Binance order books, Kraken).

### Features

* add cexm exchange market data crate; replace ccxt-rust in ethcli ([#148](https://github.com/yldfi/yldfi-rs/issues/148)) ([d994d3e](https://github.com/yldfi/yldfi-rs/commit/d994d3eb6c4dc9bf03379e9ee315de3ba6c1c1d0))
