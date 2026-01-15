# Changelog

## [0.21.2](https://github.com/yldfi/ethcli/compare/v0.21.1...v0.21.2) (2026-01-13)


### Bug Fixes

* **tenderly:** update tndrly to 0.3.2 and fix contracts add ([e4ae597](https://github.com/yldfi/ethcli/commit/e4ae597a2c64e15bcf25eeb8a8326c46e6e0b48c))

## [0.21.1](https://github.com/yldfi/ethcli/compare/v0.21.0...v0.21.1) (2026-01-12)


### Bug Fixes

* **tenderly:** update for tndrly 0.3.1 API changes ([181984a](https://github.com/yldfi/ethcli/commit/181984adef32521ba6bc028b1a8dfbb3ad8dde62))

## [0.21.0](https://github.com/yldfi/ethcli/compare/v0.20.1...v0.21.0) (2026-01-12)


### Features

* **tenderly:** add admin RPC and delivery channels ([f5e16e3](https://github.com/yldfi/ethcli/commit/f5e16e31772698e86a4093e9955bebb529ec5ab5))

## [0.20.0](https://github.com/yldfi/ethcli/compare/v0.19.0...v0.20.0) (2026-01-12)


### Features

* add full Tenderly API support ([a2e5779](https://github.com/yldfi/ethcli/commit/a2e5779ee93b0a69d8323d0cc84f3c9bab8d2861))
* add state overrides for debug/trace backends and tndrly 0.2 params ([2e52910](https://github.com/yldfi/ethcli/commit/2e529109954c042659e1c53db467f92b52a31fc3))


### Bug Fixes

* add abi alias to contract abi command ([268c2e2](https://github.com/yldfi/ethcli/commit/268c2e2c8e8d471d0efe1af72c286d4ade50cfcb))

## [0.19.0](https://github.com/yldfi/ethcli/compare/v0.18.0...v0.19.0) (2026-01-10)


### Features

* add negative caching for signature lookups (24h TTL) ([9cfd108](https://github.com/yldfi/ethcli/commit/9cfd108))
* add OutputFormat enum with clap ValueEnum derive ([9cfd108](https://github.com/yldfi/ethcli/commit/9cfd108))
* add command aliases: bal, src, info, abi ([9cfd108](https://github.com/yldfi/ethcli/commit/9cfd108))


### Security

* set 0600 permissions on config, addressbook, and cache files ([9cfd108](https://github.com/yldfi/ethcli/commit/9cfd108))

## [0.18.0](https://github.com/yldfi/ethcli/compare/v0.17.0...v0.18.0) (2026-01-10)


### Features

* enhance token balance with multiple holders and ETH support ([23cff93](https://github.com/yldfi/ethcli/commit/23cff93276f4428fb563639641f62bbf13798ecd))
* support multiple tokens in token balance command ([499060b](https://github.com/yldfi/ethcli/commit/499060b76fa46eaa1d4b6d36aae46e41f8b87fd7))

## [0.17.0](https://github.com/yldfi/ethcli/compare/v0.16.0...v0.17.0) (2026-01-10)


### Features

* add --tag option to token balance command ([85ca095](https://github.com/yldfi/ethcli/commit/85ca095c3767cc2f02d7ed185127beac95b3fcd8))

## [0.16.0](https://github.com/yldfi/ethcli/compare/v0.15.0...v0.16.0) (2026-01-10)


### Features

* implement token balance command ([0f54a2a](https://github.com/yldfi/ethcli/commit/0f54a2afa94cf84f38fbc06649022155e8903e33))

## [0.15.0](https://github.com/yldfi/ethcli/compare/v0.14.1...v0.15.0) (2026-01-10)


### Features

* resolve address labels in simulate commands ([bd7da93](https://github.com/yldfi/ethcli/commit/bd7da932f3370da64b312ad26316f1de22e890bf))

## [0.14.1](https://github.com/yldfi/ethcli/compare/v0.14.0...v0.14.1) (2026-01-10)


### Bug Fixes

* filter debug/trace endpoints by chain ([6e7e99e](https://github.com/yldfi/ethcli/commit/6e7e99e9d6bfade96635afe68a54f77284e27c08))

## [0.14.0](https://github.com/yldfi/ethcli/compare/v0.13.0...v0.14.0) (2026-01-10)


### Features

* add --dry-run flag to simulate commands ([6535b32](https://github.com/yldfi/ethcli/commit/6535b326eb68438dbf4b6eaa6c57492149e0216f))
* expand dry-run formats and add --show-secrets flag ([0ff9cc8](https://github.com/yldfi/ethcli/commit/0ff9cc856653c2c79a52730462ed78c1e73612f3))

## [0.2.0](https://github.com/yldfi/ethcli/compare/v0.1.0...v0.2.0) (2026-01-10)


### Features

* add --since option and fix tuple[] ABI parsing ([2bf15e6](https://github.com/yldfi/ethcli/commit/2bf15e6c7f8e509b4e80effcea66e2c656667996))
* add address book, Multicall3 batching, and smart endpoint selection ([ec8c31c](https://github.com/yldfi/ethcli/commit/ec8c31cec1283b4e7e33c95fab9872f21086e594))
* add cast, rpc, and ens commands ([4da02b2](https://github.com/yldfi/ethcli/commit/4da02b243d6e831c2f70cbb9ebc3cf1d4a382b34))
* add comprehensive Etherscan API commands via foundry-block-explorers ([c07c803](https://github.com/yldfi/ethcli/commit/c07c80384d93d6f6eb26c9212f260641cf8e2d22))
* add multiple events, auto from_block, and topic hash support ([2685628](https://github.com/yldfi/ethcli/commit/2685628afe7c561e4af1690138e3d23d5ad08b82))
* add proxy detection, human-readable output, and explorer links ([ae4093d](https://github.com/yldfi/ethcli/commit/ae4093de70b0602aec60177aa783d62f277a27bd))
* add release-please for automated versioning ([c085cff](https://github.com/yldfi/ethcli/commit/c085cff513972e04c87be254543a2a99db2d862b))
* add tx command for transaction analysis with hybrid signature caching ([e4514a1](https://github.com/yldfi/ethcli/commit/e4514a1e25863d1ad08553db0755b8ecf93ce802))
* **contract:** add contract call with auto-ABI fetch ([a4d91e3](https://github.com/yldfi/ethcli/commit/a4d91e392f0fb84bffd6e898f5c2825feb5be408))
* improve CLI usability with ENS resolution, signature prioritization, and output ordering ([99dd5d0](https://github.com/yldfi/ethcli/commit/99dd5d0a12b4210c63ed4e1bae5ea0fa40b358c6))
* **logs:** add --timestamps flag to include block timestamps ([ae39749](https://github.com/yldfi/ethcli/commit/ae39749bab569f02f1145961d7c6d4338261a220))
* **rpc:** add signature support to rpc call command ([c576a68](https://github.com/yldfi/ethcli/commit/c576a6840b35c942f8422e1b08ae08c522232fc9))
* unified RPC endpoint management with auto-optimization ([28b1a56](https://github.com/yldfi/ethcli/commit/28b1a56de86af252d5754e66b121be617bdfe7a0))


### Bug Fixes

* add filename length limit and Windows reserved name check ([837cdbc](https://github.com/yldfi/ethcli/commit/837cdbce48aec57f50dc6e5d5ee56ba6cfc0b4af))
* address critical issues from second code review ([d41f0db](https://github.com/yldfi/ethcli/commit/d41f0db649dd1ae460c454fb9a14738050e97b83))
* address security and performance issues from code review ([74baef7](https://github.com/yldfi/ethcli/commit/74baef78e5cb0a55eccad1ed36b257aa49f804ba))
* address security and robustness issues from code review ([853ee84](https://github.com/yldfi/ethcli/commit/853ee848799c4705dd04df48cb526be6daa64907))
* case-insensitive event name matching ([5d860cb](https://github.com/yldfi/ethcli/commit/5d860cb196dc357199a5cb76d0c8861c0c6ba306))
* event filter in raw mode and add event name lookup ([1e7ee9e](https://github.com/yldfi/ethcli/commit/1e7ee9e2397e67ae141e17e44e8fcbe97913cf2a))
* read etherscan_api_key from config file for all commands ([acf568f](https://github.com/yldfi/ethcli/commit/acf568fce1f9abe67b8b955d9d920b3f7c481316))
* release workflow packages ethcli binary instead of eth-log-fetch ([d55f9d8](https://github.com/yldfi/ethcli/commit/d55f9d8f54481587e31685d10c68875abf7cd4e5))
* remove trailing_var_arg to fix CLI flag parsing in simulate ([35ca014](https://github.com/yldfi/ethcli/commit/35ca014606b24d5f92492774c6244232005c7743))
* resolve clippy and formatting issues ([4e5441d](https://github.com/yldfi/ethcli/commit/4e5441d5b28302517da52df1d59e516c18e45e8c))
* security hardening from multi-round AI code reviews ([11dac38](https://github.com/yldfi/ethcli/commit/11dac38c0f2e8d7582fe68ff23b7421295521017))
* try multiple RPC endpoints when tx/receipt not found ([f69e677](https://github.com/yldfi/ethcli/commit/f69e67717b14597126efd6b3c5031cae5c782c54))
* update release workflow to use ethcli binary name ([8d1d5e4](https://github.com/yldfi/ethcli/commit/8d1d5e45805cbe2352bf5e6a3161834779c79da7))
* update to connect_http for alloy 1.2 compatibility ([9b28877](https://github.com/yldfi/ethcli/commit/9b288779ed49108d6ab5c2725bc673126b8c40b2))
* use ASCII-only chars in filename sanitization ([aa13b3f](https://github.com/yldfi/ethcli/commit/aa13b3f244df23dc9a4e9f8228799aa8b128d83b))


### Performance Improvements

* parallelize transaction and receipt fetching ([ea2cbed](https://github.com/yldfi/ethcli/commit/ea2cbed01d9655242e3c36d567a85762dc87b237))

## [0.12.0](https://github.com/yldfi/ethcli/compare/v0.11.1...v0.12.0) (2025-12-30)


### Features

* add --since option and fix tuple[] ABI parsing ([c3ba737](https://github.com/yldfi/ethcli/commit/c3ba7374240b07ff1be6ec4b8e4adc33e23f0b10))

## [0.11.1](https://github.com/yldfi/ethcli/compare/v0.11.0...v0.11.1) (2025-12-30)


### Bug Fixes

* read etherscan_api_key from config file for all commands ([876ad37](https://github.com/yldfi/ethcli/commit/876ad371fda7337807f49c79743ce0d3cc64460b))
* resolve clippy and formatting issues ([baf938b](https://github.com/yldfi/ethcli/commit/baf938ba98a7505b8c358b51b2e5cca6462468be))

## [0.11.0](https://github.com/yldfi/ethcli/compare/v0.10.0...v0.11.0) (2025-12-26)


### Features

* add address book, Multicall3 batching, and smart endpoint selection ([fc7d8c0](https://github.com/yldfi/ethcli/commit/fc7d8c0f448b967979eb0d4866d8d98c50558f0f))


### Performance Improvements

* parallelize transaction and receipt fetching ([ccf9e19](https://github.com/yldfi/ethcli/commit/ccf9e19912bea2da0f0dd515ef4e497ba55574d8))

## [0.10.0](https://github.com/yldfi/ethcli/compare/v0.9.0...v0.10.0) (2025-12-25)


### Features

* improve CLI usability with ENS resolution, signature prioritization, and output ordering ([7040982](https://github.com/yldfi/ethcli/commit/704098231f0f56f75fb75db4e9352751cc60baee))

## [0.9.0](https://github.com/yldfi/ethcli/compare/v0.8.0...v0.9.0) (2025-12-24)


### Features

* add proxy detection, human-readable output, and explorer links ([7511fc9](https://github.com/yldfi/ethcli/commit/7511fc964301a015e844175703b515c39f63b540))

## [0.8.0](https://github.com/yldfi/ethcli/compare/v0.7.0...v0.8.0) (2025-12-24)


### Features

* **contract:** add contract call with auto-ABI fetch ([75ce4cb](https://github.com/yldfi/ethcli/commit/75ce4cb8db61643cb5805b09c083f31862df2dbd))
* **logs:** add --timestamps flag to include block timestamps ([a371faa](https://github.com/yldfi/ethcli/commit/a371faa7a5dc14b2481fd90c1e0a4791c17a98fe))


### Bug Fixes

* add filename length limit and Windows reserved name check ([e84c3d8](https://github.com/yldfi/ethcli/commit/e84c3d8cf892119508789bd416a4c3d8444a12ea))
* address critical issues from second code review ([0559ffd](https://github.com/yldfi/ethcli/commit/0559ffd0197eacb07f9b094f067edde072bf93f0))
* address security and performance issues from code review ([5feb14e](https://github.com/yldfi/ethcli/commit/5feb14ee6491d6a00b595e3ff71ed40d2e4bb93e))
* use ASCII-only chars in filename sanitization ([80405a8](https://github.com/yldfi/ethcli/commit/80405a81bbea09552c2e0b4b7e59340f41bdf748))

## [0.7.0](https://github.com/yldfi/ethcli/compare/v0.6.0...v0.7.0) (2025-12-24)


### Features

* **rpc:** add signature support to rpc call command ([bd43d24](https://github.com/yldfi/ethcli/commit/bd43d241d82e0430ce291df4e84818e1c1e66564))

## [0.6.0](https://github.com/yldfi/ethcli/compare/v0.5.1...v0.6.0) (2025-12-24)


### Features

* unified RPC endpoint management with auto-optimization ([4d71d0b](https://github.com/yldfi/ethcli/commit/4d71d0bc29fe8fdb646eaba531e1fea74bd6db93))

## [0.5.1](https://github.com/yldfi/ethcli/compare/v0.5.0...v0.5.1) (2025-12-23)


### Bug Fixes

* release workflow packages ethcli binary instead of eth-log-fetch ([e70c62a](https://github.com/yldfi/ethcli/commit/e70c62afd6524d04819862f37b2ed872665b5c02))

## [0.5.0](https://github.com/yldfi/ethcli/compare/v0.4.0...v0.5.0) (2025-12-23)


### Features

* add cast, rpc, and ens commands ([6d6a784](https://github.com/yldfi/ethcli/commit/6d6a784625a1ba688df0ad9879721917e2b608b3))


### Bug Fixes

* update release workflow to use ethcli binary name ([0e25f1b](https://github.com/yldfi/ethcli/commit/0e25f1ba4dd74b6165ed7e46bba4de075fe7ed75))

## [0.4.0](https://github.com/michaeldim/eth-log-fetcher/compare/v0.3.2...v0.4.0) (2025-12-23)


### Features

* add comprehensive Etherscan API commands via foundry-block-explorers ([df9ad06](https://github.com/michaeldim/eth-log-fetcher/commit/df9ad06b202acf001ae01f8f82ba9475660d7973))


### Bug Fixes

* update to connect_http for alloy 1.2 compatibility ([6bd1d7e](https://github.com/michaeldim/eth-log-fetcher/commit/6bd1d7eb5be3fdf9f43f63dce529c25cc7f15c2b))

## [0.2.0](https://github.com/michaeldim/eth-log-fetcher/compare/v0.1.0...v0.2.0) (2025-12-19)


### Features

* add release-please for automated versioning ([c085cff](https://github.com/michaeldim/eth-log-fetcher/commit/c085cff513972e04c87be254543a2a99db2d862b))


### Bug Fixes

* security hardening from multi-round AI code reviews ([11dac38](https://github.com/michaeldim/eth-log-fetcher/commit/11dac38c0f2e8d7582fe68ff23b7421295521017))
