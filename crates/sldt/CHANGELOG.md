# Changelog

## [0.2.1](https://github.com/yldfi/yldfi-rs/compare/sldt-v0.2.0...sldt-v0.2.1) (2026-10-20)


### Bug Fixes

* **sldt:** mention Solodit-side outages in the invalid-key hint ([#129](https://github.com/yldfi/yldfi-rs/issues/129)) ([721325d](https://github.com/yldfi/yldfi-rs/commit/721325d059b79f318a6693ac2ec07791c71eb263))

## [0.2.0](https://github.com/yldfi/yldfi-rs/compare/sldt-v0.1.1...sldt-v0.2.0) (2026-09-25)


### ⚠ BREAKING CHANGES

* **sldt:** Error::Unauthorized and Error::RateLimited now carry data; ApiResponse.rate_limit is Option<RateLimit>; SearchResults gains fields.

### Bug Fixes

* **sldt:** align Solodit client with the Findings API spec ([#76](https://github.com/yldfi/yldfi-rs/issues/76)) ([a04476e](https://github.com/yldfi/yldfi-rs/commit/a04476e9f20f42b034d0f6fa981dd34181e1765d))

## [0.1.1](https://github.com/yldfi/yldfi-rs/compare/sldt-v0.1.0...sldt-v0.1.1) (2026-01-27)


### Features

* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))
