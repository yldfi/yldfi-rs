# Changelog

## [0.24.0](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.6...ethcli-v0.24.0) (2026-10-21)


### Features

* **ethcli:** dune query execute --params ([29dbef4](https://github.com/yldfi/yldfi-rs/commit/29dbef47acc42ee45b8cf957511b4791c6ab4868))
* **ethcli:** ENS wildcard resolution and CCIP-read ([22131b9](https://github.com/yldfi/yldfi-rs/commit/22131b90feeeeec801c43d4deda84ac6eb0a2173))


### Bug Fixes

* **alcmy:** match Gas Manager, Solana DAS, Signer, portfolio and trace shapes ([756bb2c](https://github.com/yldfi/yldfi-rs/commit/756bb2cbb6dad59a16b74e7e869ed67c08842ae8))
* **alcmy:** match Notify, NFT and debug shapes; strip URLs from errors ([6714db9](https://github.com/yldfi/yldfi-rs/commit/6714db97ae98c2e9a274f75edf80c5d60da332fc))
* **cowp:** accept bare-string order UID and page owner orders ([01159f4](https://github.com/yldfi/yldfi-rs/commit/01159f40fde07768c81bfcff6a876768142e36d0))
* **cowp:** onchainOrder, optional quote from, ethsign ([8aace42](https://github.com/yldfi/yldfi-rs/commit/8aace425eaac6848d31df35cf5facbc81ce8e3d9))
* **dnapi:** pipeline and query-list types, results options, cancel on timeout ([ef2d682](https://github.com/yldfi/yldfi-rs/commit/ef2d6821f9c15450f5c10486c94f929d80ddd71e))
* **dnapi:** send sql, insert CSV/NDJSON, handle partial results ([38720cb](https://github.com/yldfi/yldfi-rs/commit/38720cb32d236205327abea489dcfbe24f117934))
* **ethcli-mcp:** annotate tools, redact errors, fix tool argv gaps ([89afed5](https://github.com/yldfi/yldfi-rs/commit/89afed500a4e0ebe2541a25dd911aaeb4425dc1e))
* **ethcli-mcp:** close subprocess stdin, block flag injection, gate side effects ([84a3c34](https://github.com/yldfi/yldfi-rs/commit/84a3c344cdb61ad5091f89a9aa867e33886f1e27))
* **ethcli:** 1inch --fee is a percent (0-3), not basis points ([4d5a894](https://github.com/yldfi/yldfi-rs/commit/4d5a89485ce69d879772f40791137ee1a6bc1bef))
* **ethcli:** abort self-update when the listed checksum can't be fetched ([3033883](https://github.com/yldfi/yldfi-rs/commit/30338830d69ab14cfad481f8fd3839daf12f4cf9))
* **ethcli:** accept decimal block numbers in alchemy debug commands ([c0c478d](https://github.com/yldfi/yldfi-rs/commit/c0c478dce9dab877b21a76a35700fea4ec48c6e6))
* **ethcli:** accept ENS names and labels in all account commands ([5a5b5f2](https://github.com/yldfi/yldfi-rs/commit/5a5b5f2c0d3fe0d943a9c138b0bf17fae6612bc6))
* **ethcli:** accept only global-unicast IPv6 for CCIP gateways ([52f1a81](https://github.com/yldfi/yldfi-rs/commit/52f1a81bedfd6eb9b20b0a6e6a280bd93dda8e3a))
* **ethcli:** accept the documented quote source names ([c27a464](https://github.com/yldfi/yldfi-rs/commit/c27a4642091e1a0ca3e2c150508c4fd27ea4c201))
* **ethcli:** alchemy commands use [alchemy] default_network ([67e73de](https://github.com/yldfi/yldfi-rs/commit/67e73de5396a936a7ade747aee46adac859fc383))
* **ethcli:** Alchemy portfolio rows get real metadata and chain names ([04a79f0](https://github.com/yldfi/yldfi-rs/commit/04a79f0f0748406000a01954dc33cd7bc09f8fc6))
* **ethcli:** alerts update keeps the target; --with-metadata can be false ([661676f](https://github.com/yldfi/yldfi-rs/commit/661676f1f58827c65ccde98d5744a8f4d9d04d8f))
* **ethcli:** blacklist scan rejects an unknown chain instead of using mainnet ([e028e1a](https://github.com/yldfi/yldfi-rs/commit/e028e1a4ccbcacc8574b152430b885675bbba323))
* **ethcli:** bound string lengths decoded from multicall results ([38b9272](https://github.com/yldfi/yldfi-rs/commit/38b927218b325421ddba3b6138fd36c74c79adca))
* **ethcli:** bytecode security scan resumes after INVALID at the next JUMPDEST ([8df0d2a](https://github.com/yldfi/yldfi-rs/commit/8df0d2a67d26780af4297e48d154d7eb9cd49612))
* **ethcli:** bytes32 isn't a complex arg; say --skip-validation does nothing ([2983d1a](https://github.com/yldfi/yldfi-rs/commit/2983d1a7f5b7079f20bee2d9afdcbbee1ebc37c3))
* **ethcli:** canonicalise signatures in cast sig and cast topic ([5adea58](https://github.com/yldfi/yldfi-rs/commit/5adea58815d4933e314e7ffdbcfaea79357db5fe))
* **ethcli:** ccxt compare queries exchanges in parallel and reports failures ([28e59a7](https://github.com/yldfi/yldfi-rs/commit/28e59a72a8c5bd714f3a33a14adcd8036ae55ea3))
* **ethcli:** ccxt tickers reports failures; account erc20 shows token units ([86287fb](https://github.com/yldfi/yldfi-rs/commit/86287fb602b1b97ad5e3cd2b0e5a423b1b72db48))
* **ethcli:** Chainlink quote handling, L2-only symbols and NoFeed exit ([42e50c2](https://github.com/yldfi/yldfi-rs/commit/42e50c257a4665d932e781f218a1b694a645af7d))
* **ethcli:** CoinGecko and DefiLlama platform ids for more chains ([06c3e49](https://github.com/yldfi/yldfi-rs/commit/06c3e49d3f135c26f9aa9a4594861ed5f562bd65))
* **ethcli:** compute best_source; read the Curve LP OHLC close by name ([a3f7c0a](https://github.com/yldfi/yldfi-rs/commit/a3f7c0a082601a8aa06d3cce5af90930a78caa95))
* **ethcli:** coordinate config writes on a stable lock file ([f509e65](https://github.com/yldfi/yldfi-rs/commit/f509e652a42c33c952513680f95919b20750a4af))
* **ethcli:** correct topic0 and indexed params for ABI-resolved events ([90e6a9f](https://github.com/yldfi/yldfi-rs/commit/90e6a9fe170adc0197c60dd63dfcbaca6f44f7e4))
* **ethcli:** correct tx topic0 constants and a malformed Convex address ([cc71cb5](https://github.com/yldfi/yldfi-rs/commit/cc71cb517811565b36b6780da3d15301bef219f7))
* **ethcli:** CSV escapes only text; SQLite adds columns to reused tables ([69a4e4c](https://github.com/yldfi/yldfi-rs/commit/69a4e4c08e6144e0fee1ade5f823e63beed81b76))
* **ethcli:** decode logs matched by topic-hash -e filters ([612172e](https://github.com/yldfi/yldfi-rs/commit/612172ed768cef278dfe4976faac3927daebe773))
* **ethcli:** delete every page of VNets with tenderly vnets delete --all ([d1cbb57](https://github.com/yldfi/yldfi-rs/commit/d1cbb5700077c4e26c514b21d82a838e13f5fad4))
* **ethcli:** don't overwrite an unparsable config in chainlist add ([786e117](https://github.com/yldfi/yldfi-rs/commit/786e117c85c413a05428a4782ee0518a910a770f))
* **ethcli:** don't panic on malformed trace calldata; drop dead cache.rs ([35e8791](https://github.com/yldfi/yldfi-rs/commit/35e87917ad01dc570f967c38fb98fee49d9ad17f))
* **ethcli:** don't report a 10,000-block range limit that was never found ([e3ad364](https://github.com/yldfi/yldfi-rs/commit/e3ad364b538a46108332bf436cc399a383d30311))
* **ethcli:** don't require Etherscan for RPC-only gas and contract work ([c46627f](https://github.com/yldfi/yldfi-rs/commit/c46627fa25765e7498fbc238f6c6cb57eb564299))
* **ethcli:** endpoints health probes concurrently and never substitutes localhost ([545d323](https://github.com/yldfi/yldfi-rs/commit/545d3237c05e33f991dd7d02444e84d31385dc84))
* **ethcli:** endpoints optimize keeps user settings and reports mismatches ([82f7dbf](https://github.com/yldfi/yldfi-rs/commit/82f7dbfd688e70ba2fc20c233fb4ab476a9f89ad))
* **ethcli:** ERC-721 transfers and canonical event signatures in tx analysis ([9c3ab9f](https://github.com/yldfi/yldfi-rs/commit/9c3ab9f9b7ab4fc22d90366d35506415e32085b7))
* **ethcli:** fail over between endpoints for ENS commands ([a5f0dde](https://github.com/yldfi/yldfi-rs/commit/a5f0ddef50b6d24acf562e4198ba8d470c3635b3))
* **ethcli:** give the KongVault price source a nested budget that fits ([d158953](https://github.com/yldfi/yldfi-rs/commit/d15895332d16631245d48b4bee90a10946622eea))
* **ethcli:** honour [proxy] enabled for RPC and refuse unsupported proxy modes ([3c2d0b4](https://github.com/yldfi/yldfi-rs/commit/3c2d0b49b27b42d732b49ce22ecaa810349f288c))
* **ethcli:** keep earlier results when resuming logs into a file ([1b4bdac](https://github.com/yldfi/yldfi-rs/commit/1b4bdac2871e01fba2fa05b576753b0b54269210))
* **ethcli:** keep logs checkpoints in step with the output ([c472329](https://github.com/yldfi/yldfi-rs/commit/c472329a3712bde5659f4418137a925082a3e971))
* **ethcli:** keep signature cache clears when saves merge with disk ([75e1959](https://github.com/yldfi/yldfi-rs/commit/75e1959621f45686831d479be9a35bc096c2c182))
* **ethcli:** kyberswap build checks its args; velora --side; owner ellipsis ([3e54827](https://github.com/yldfi/yldfi-rs/commit/3e54827e307999c4e07ff01bfaba48aa99c18aa5))
* **ethcli:** let log fetching react to typed RPC errors ([ffe79f7](https://github.com/yldfi/yldfi-rs/commit/ffe79f75dd7cca9e20a64261a3d7c1ec8db540a7))
* **ethcli:** mask SOCKS proxy credentials in config show ([1adc472](https://github.com/yldfi/yldfi-rs/commit/1adc4722e4a1f75191b1453d5c9c0e3d7ebe6afa))
* **ethcli:** match Yearn multicall results to the vaults queried ([2573e40](https://github.com/yldfi/yldfi-rs/commit/2573e40aa5058565cc0fdf2be6728a7a3fed2539))
* **ethcli:** name Cancun opcodes and keep disassembler output off stdout ([8c1dccf](https://github.com/yldfi/yldfi-rs/commit/8c1dccfac3c59ad48430e5e1ab52ef29ecd8983f))
* **ethcli:** never overwrite an unparsable address book or blacklist ([ec79be1](https://github.com/yldfi/yldfi-rs/commit/ec79be133a3e3ac997a1cbdf86a8e7a3fcc2172b))
* **ethcli:** normalise blacklist keys ([c6903bf](https://github.com/yldfi/yldfi-rs/commit/c6903bf8b4fe8a27b86697edc3693799aba88522))
* **ethcli:** one value/block parser for simulate; keep RPC URLs off argv ([8492f22](https://github.com/yldfi/yldfi-rs/commit/8492f22a58b8fb53d98e9fc9dcca2ff480a6f9f3))
* **ethcli:** only show normalised, displayable ENS reverse names ([10cdf92](https://github.com/yldfi/yldfi-rs/commit/10cdf9269587511fb2375c4772e7311aac0ac698))
* **ethcli:** page through Alchemy/Moralis NFTs and fetch chains concurrently ([377ee46](https://github.com/yldfi/yldfi-rs/commit/377ee460501b4a4d2662e6d0d5a5a6c56e1b9276))
* **ethcli:** parallel Chainlink registry reads; keep Data Streams error bodies ([6c23a41](https://github.com/yldfi/yldfi-rs/commit/6c23a412e4acffc68de02d8361c8948813c5dc3a))
* **ethcli:** parse eth/gwei amounts exactly and reject negative to-wei ([711a88d](https://github.com/yldfi/yldfi-rs/commit/711a88d5a24be54515af9f2f14e2d4e71a609270))
* **ethcli:** proxy handling in contract call and analyze --follow-proxy ([57e206d](https://github.com/yldfi/yldfi-rs/commit/57e206d85762f4e11cd2bce19de7bc7ce00af238))
* **ethcli:** quote exits non-zero on failure and parses amounts strictly ([469a3a8](https://github.com/yldfi/yldfi-rs/commit/469a3a833bb9ab7cceea18dc0b348f09ad42f803))
* **ethcli:** redact The Graph API key from all uniswap subgraph errors ([12893e0](https://github.com/yldfi/yldfi-rs/commit/12893e00986248255b408a2b9aa89a93dffa3c82))
* **ethcli:** redact The Graph API key from uniswap subgraph errors ([6f3e8bb](https://github.com/yldfi/yldfi-rs/commit/6f3e8bb1047ec80626f9831d930b5d9df86bce7f))
* **ethcli:** redact the token after an Authorization scheme ([68a2755](https://github.com/yldfi/yldfi-rs/commit/68a2755187bdbcda3dd15c89aa879ea854524d5e))
* **ethcli:** redact the whole Authorization/x-api-key header value ([f33bc1f](https://github.com/yldfi/yldfi-rs/commit/f33bc1f22c3967b1e85db88057890fb477557633))
* **ethcli:** redact URLs and bare secrets in sanitized error messages ([6f760a8](https://github.com/yldfi/yldfi-rs/commit/6f760a85ab693bd9b480c92f9d5670c41bc78828))
* **ethcli:** report price spread over all sources; fix Pyth confidence ([390fe63](https://github.com/yldfi/yldfi-rs/commit/390fe63f485894fa5c2d14d1ff55856c56056634))
* **ethcli:** report RPC errors instead of 'not found' during outages ([210ab77](https://github.com/yldfi/yldfi-rs/commit/210ab779f5c02fed5b20ec4a0f32cab6f3062fd3))
* **ethcli:** report Yearn portfolio failures per chain ([35a58a8](https://github.com/yldfi/yldfi-rs/commit/35a58a809e5cf9c830e923a33eeeef1868be87ff))
* **ethcli:** resolve price symbols on the requested chain ([e4d319b](https://github.com/yldfi/yldfi-rs/commit/e4d319b2c9104f2ee9580a0172ced23a9df4b373))
* **ethcli:** restrict CCIP-read gateway requests ([14cda09](https://github.com/yldfi/yldfi-rs/commit/14cda09115ad32825cd0dd923f80097361611207))
* **ethcli:** rpc prefers archive endpoints for old blocks; not found fails ([9ad9fed](https://github.com/yldfi/yldfi-rs/commit/9ad9fed5f6cf657f9adb56817874ecee45608359))
* **ethcli:** sanitise on-chain token symbols in the Alchemy portfolio ([0e70b56](https://github.com/yldfi/yldfi-rs/commit/0e70b5666031791e5c93e5eef47188c70a277505))
* **ethcli:** scale contract call --human output only for amount getters ([537368d](https://github.com/yldfi/yldfi-rs/commit/537368d714aa2e906c63fee82fd4a6fc5395c074))
* **ethcli:** scan the whole bytecode in security analysis ([9f653d6](https://github.com/yldfi/yldfi-rs/commit/9f653d6e7f1bdc2c353a2f2d2d9faa452a7f74c4))
* **ethcli:** send simulate overrides the way each backend expects ([2e6dca1](https://github.com/yldfi/yldfi-rs/commit/2e6dca15ddf1ab28595639bba81ff42cd7d54c35))
* **ethcli:** send the Etherscan key only to Etherscan API hosts ([cde9ffa](https://github.com/yldfi/yldfi-rs/commit/cde9ffacfcf81b72a2ec437ae779ef0b60b80e70))
* **ethcli:** signature cache reads hits, negative-caches only real misses ([1326738](https://github.com/yldfi/yldfi-rs/commit/132673801dfb595c99cada9a869cb81da63fb9c3))
* **ethcli:** simulate honours --chain and forwards gas settings ([57c38f6](https://github.com/yldfi/yldfi-rs/commit/57c38f67fc4a45bde64082f3df197e86de7e391b))
* **ethcli:** smart endpoint selection and stale Chainlink answers ([c507dc8](https://github.com/yldfi/yldfi-rs/commit/c507dc80f6d4e47dd18c9068aa512b4af90040f3))
* **ethcli:** stop capping account history at block 99,999,999 ([a6f5c17](https://github.com/yldfi/yldfi-rs/commit/a6f5c1762709218d046d7a2b4f57af4f533fafef))
* **ethcli:** token metadata cache expires, skips misses, keys by chain id ([658d0ff](https://github.com/yldfi/yldfi-rs/commit/658d0ffc42410c278b4062837a135dbacd3db887))
* **ethcli:** truncate yields/blacklist text by characters, not bytes ([7043400](https://github.com/yldfi/yldfi-rs/commit/704340071555f0b37e0df1f43f83b1677aade19b))
* **ethcli:** tx fails when nothing was analyzed; reject --batch-size 0 ([1908fd4](https://github.com/yldfi/yldfi-rs/commit/1908fd4a4d914d3a5fa07b6da3899a806791b1e4))
* **ethcli:** uniswap subgraph commands honour the chain and report failures ([e828f39](https://github.com/yldfi/yldfi-rs/commit/e828f39927fd83f3bc4cc36b2f5e41362b4443cf))
* **ethcli:** validate address imports; point key hints at real config ([53e4e5a](https://github.com/yldfi/yldfi-rs/commit/53e4e5af9eca9b78596cb7df5768e28e812c5bb4))
* **ethcli:** verify ENS reverse records and normalise names ([2a14159](https://github.com/yldfi/yldfi-rs/commit/2a1415977f94ffdd606c42a678f5e3d2b89dfd17))
* **ethcli:** whitelist blacklist-scan protocols by address, not name ([d7da2e8](https://github.com/yldfi/yldfi-rs/commit/d7da2e847b9abb5af7ba6fe8f91a4df7900f1a7e))
* **ethcli:** wrap native xDAI as WXDAI for CoW quotes on Gnosis ([41242d4](https://github.com/yldfi/yldfi-rs/commit/41242d499ad00ffa7d120999a91a5175b96f4ac8))
* **ethcli:** yields project prefilter, Yearn chains and partial Uniswap ([cc18cfd](https://github.com/yldfi/yldfi-rs/commit/cc18cfdfe2294d19703ffb6373085589dee8108b))
* handle dated upstream API changes (Etherscan, DefiLlama, Chainlink, Kong, CoW) ([b713e16](https://github.com/yldfi/yldfi-rs/commit/b713e1618c9a9ca3cd0d5d79cec0149c08498006))
* **kybr:** drop route fields v1 doesn't return ([b2d40b8](https://github.com/yldfi/yldfi-rs/commit/b2d40b8a33cba4734b2fc561055f52def63be272))
* **lfi:** fee is a fraction; status, connections and error fixes ([d967bb0](https://github.com/yldfi/yldfi-rs/commit/d967bb06363dd874b7b2501090fb8e1bbf891c59))
* **lfi:** use /v1/gas/prices/{chainId} ([23a2dec](https://github.com/yldfi/yldfi-rs/commit/23a2decc686a6304ddb9d3b5359dfed1ab2b2f8d))
* **mrls:** unwrap Moralis response envelopes and send required params ([59749f5](https://github.com/yldfi/yldfi-rs/commit/59749f5800378ab25f49a240e5b400b887c156fd))
* **mrls:** use Moralis error mapping and fix remaining response shapes ([60cfa9a](https://github.com/yldfi/yldfi-rs/commit/60cfa9af66f663f4230452e7f019bb17566dd3f0))
* smaller upstream API changes (0x, 1inch, Moralis, CoinGecko, Alchemy, LI.FI, KyberSwap) ([964be4c](https://github.com/yldfi/yldfi-rs/commit/964be4c1ba63778f9aac630b509886e0349123e7))
* **tndrly:** remodel actions, alerts, contracts and channels from the spec ([4929be9](https://github.com/yldfi/yldfi-rs/commit/4929be9cb52495937c3f5d05e2cc8ac88698ecba))
* **tndrly:** vnet callArgs, alert update merge, nested transaction_info ([c07bb48](https://github.com/yldfi/yldfi-rs/commit/c07bb484bde6314b47a0a23ca5180e870f90b414))
* **ykong:** require a chain for vault strategies, keep 5xx retryable ([8e23c79](https://github.com/yldfi/yldfi-rs/commit/8e23c799e87d607eaada928895a4b63141870bdb))
* **yldfi-common:** case-insensitive redaction, keep tx hashes, POL ([eace5ed](https://github.com/yldfi/yldfi-rs/commit/eace5ed073a90425b2be0f7ab3d9d5f63cccffdf))


### Performance Improvements

* **ethcli:** answer single signature lookups from the cache ([888feca](https://github.com/yldfi/yldfi-rs/commit/888feca2a1b0d6d10c652dfc5e7915b91dabab39))
* **ethcli:** look up 4byte signatures concurrently for --lookup ([a98161c](https://github.com/yldfi/yldfi-rs/commit/a98161c5afac5a7a66f5380523c5b758913c6e48))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * alcmy bumped from 0.2 to 0.3.0
    * cgko bumped from 0.1 to 0.2.0
    * dllma bumped from 0.1 to 0.2.0
    * mrls bumped from 0.2 to 0.3.0
    * dnapi bumped from 0.2 to 0.3.0
    * crv bumped from 0.1 to 0.2.0
    * pythc bumped from 0.2 to 0.2.1
    * tndrly bumped from 0.4 to 0.5.0
    * yldfi-common bumped from 0.1 to 0.1.6
    * openoc bumped from 0.1 to 0.2.0
    * cowp bumped from 0.1 to 0.2.0
    * kybr bumped from 0.1 to 0.2.0
    * zrxswap bumped from 0.1 to 0.2.0
    * oinch bumped from 0.1 to 0.2.0
    * lfi bumped from 0.1 to 0.2.0
    * vlra bumped from 0.1 to 0.2.0
    * ensof bumped from 0.1 to 0.2.0
    * unswp bumped from 0.1 to 0.1.7
    * ykong bumped from 0.1 to 0.1.6
    * gplus bumped from 0.1 to 0.2.0

## [0.23.6](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.5...ethcli-v0.23.6) (2026-10-20)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * sldt bumped from 0.2 to 0.2.1

## [0.23.5](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.4...ethcli-v0.23.5) (2026-10-13)


### Bug Fixes

* **ethcli:** upgrade ethcli-mcp in any same-user layout, with rollback ([#116](https://github.com/yldfi/yldfi-rs/issues/116)) ([b80af18](https://github.com/yldfi/yldfi-rs/commit/b80af18de193336d2c878fb28e82e9a91da15f25))

## [0.23.4](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.3...ethcli-v0.23.4) (2026-10-13)


### Bug Fixes

* **ethcli:** keep symlinked layouts intact on upgrade ([#91](https://github.com/yldfi/yldfi-rs/issues/91)) ([824c0cd](https://github.com/yldfi/yldfi-rs/commit/824c0cd6daacb3e54ecea9113c79fa6118b9d62c))

## [0.23.3](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.2...ethcli-v0.23.3) (2026-10-13)


### Bug Fixes

* **ethcli:** encode simulate --sig calldata natively ([#88](https://github.com/yldfi/yldfi-rs/issues/88)) ([5a9d116](https://github.com/yldfi/yldfi-rs/commit/5a9d116ebf091f157726f298f4cf0488bfbf87b9))

## [0.23.2](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.1...ethcli-v0.23.2) (2026-10-03)


### Features

* **ethcli:** add account history block bounds to CLI and MCP ([#86](https://github.com/yldfi/yldfi-rs/issues/86)) ([c0bf7e8](https://github.com/yldfi/yldfi-rs/commit/c0bf7e834647b7121cd5be033ee429a0129ec31b))

## [0.23.1](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.23.0...ethcli-v0.23.1) (2026-09-26)


### Bug Fixes

* **ethcli:** install upgraded binaries via temp file + rename ([#83](https://github.com/yldfi/yldfi-rs/issues/83)) ([804089e](https://github.com/yldfi/yldfi-rs/commit/804089e96bc33b4549f3067cadf67c9c1fc97faf))

## [0.23.0](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.22.4...ethcli-v0.23.0) (2026-09-25)


### ⚠ BREAKING CHANGES

* **sldt:** Error::Unauthorized and Error::RateLimited now carry data; ApiResponse.rate_limit is Option<RateLimit>; SearchResults gains fields.
* the `volume` and `market` modules, `Client::volume()`, `Client::market()`, the DiscoveryApi list/filter/token methods and their types, and TokenApi::{get_stats, get_exchange_*_tokens, get_by_symbols, get_holders_historical, get_pairs_stats, get_pair_snipers, get_bonding_status} (plus their response types) are removed. DiscoveryApi keeps get_token_analytics and get_token_score, which are still in the spec (scores are EVM-only).
* **pythc:** the default endpoint changed and price requests now fail with DomainError::Unauthorized unless an API key is configured.
* the removed alcmy methods/types, ethcli subcommands and MCP tools no longer exist. Use contracts-for-owner, contract-metadata (openSeaMetadata), is-spam and nfts-for-contract instead.

### Features

* remove sunset Alchemy NFT/Simulation APIs and Dune Sim integration ([#71](https://github.com/yldfi/yldfi-rs/issues/71)) ([416d973](https://github.com/yldfi/yldfi-rs/commit/416d97332c5e39696b10d522465a2b3e65b152b4))


### Bug Fixes

* DEX aggregator, Curve, Uniswap, yields and Kong bugs found by live smoke test ([#81](https://github.com/yldfi/yldfi-rs/issues/81)) ([730e9eb](https://github.com/yldfi/yldfi-rs/commit/730e9eb388557e434fd530c64d8fc1df17e7b9e8))
* **ethcli,unswp:** replace dead RPCs, fix V4 subgraph IDs, mainnet Data Streams, Etherscan sunset chains ([#72](https://github.com/yldfi/yldfi-rs/issues/72)) ([64ff968](https://github.com/yldfi/yldfi-rs/commit/64ff968a4d7166ab9a9c4e3f896bf8d816f53721))
* **ethcli:** stop leaking secrets, add RPC failover, fix bugs found by live smoke test ([#77](https://github.com/yldfi/yldfi-rs/issues/77)) ([ce2267b](https://github.com/yldfi/yldfi-rs/commit/ce2267b2d36712b9d883801eb8aead6fb271df06))
* market-data aggregators, DefiLlama/CoinGecko parsing, ccxt workarounds, MCP errors ([#78](https://github.com/yldfi/yldfi-rs/issues/78)) ([25af322](https://github.com/yldfi/yldfi-rs/commit/25af322ecce9f7bc291a2b685c1666ed855ee383))
* migrate DEX/aggregator clients off deprecated endpoints (cowp, oinch, vlra, openoc, crv, cgko, dllma) ([#74](https://github.com/yldfi/yldfi-rs/issues/74)) ([1981119](https://github.com/yldfi/yldfi-rs/commit/19811196ca836fa14a4364c8c758895da8411347))
* provider client bugs found by live smoke test (alcmy, tndrly, mrls, gplus) ([#75](https://github.com/yldfi/yldfi-rs/issues/75)) ([8b50186](https://github.com/yldfi/yldfi-rs/commit/8b50186a012a7dbdec8409f0e93287ef1c88c1f7))
* **pythc:** require API key for Pyth Hermes (Pyth Core upgrade) ([#70](https://github.com/yldfi/yldfi-rs/issues/70)) ([91d1d88](https://github.com/yldfi/yldfi-rs/commit/91d1d8861434b157804dc0b52199e2f33bc0ada4))
* remove dead Moralis/Tenderly endpoints, fix Dune API paths ([#73](https://github.com/yldfi/yldfi-rs/issues/73)) ([78db18e](https://github.com/yldfi/yldfi-rs/commit/78db18e22b7eadb8ad71ce3cefdbc21bf4ea117c))
* **sldt:** align Solodit client with the Findings API spec ([#76](https://github.com/yldfi/yldfi-rs/issues/76)) ([a04476e](https://github.com/yldfi/yldfi-rs/commit/a04476e9f20f42b034d0f6fa981dd34181e1765d))

## [0.22.4](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.22.3...ethcli-v0.22.4) (2026-07-13)


### Features

* **ethcli:** decode debug_traceTransaction traces into labeled call trees ([#65](https://github.com/yldfi/yldfi-rs/issues/65)) ([a3b7319](https://github.com/yldfi/yldfi-rs/commit/a3b731981d2f4d9c267c9af43f1a8e1f7c268475))


### Bug Fixes

* **dsim:** guard Dune Sim sunset, disable removed DeFi Positions ([#66](https://github.com/yldfi/yldfi-rs/issues/66)) ([00cddb0](https://github.com/yldfi/yldfi-rs/commit/00cddb0ae59dbfb87c48d44da5d87aa44b57d61c))
* **ethcli:** satisfy clippy 1.97 useless_borrows_in_formatting ([2a489e3](https://github.com/yldfi/yldfi-rs/commit/2a489e341bdd94c4da63e5cc2381a1c53610ef83))

## [0.22.2](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.22.1...ethcli-v0.22.2) (2026-05-10)


### Features

* add 'upgrade' command, keep 'update' for checking only ([e40ee32](https://github.com/yldfi/yldfi-rs/commit/e40ee325fefab3cb3795c05276bf362ede2a5ce7))
* add 130+ CLI commands for Alchemy & Moralis, fix formatting ([b580a6e](https://github.com/yldfi/yldfi-rs/commit/b580a6edb67179787771d854bc44dcec5ddfc8c0))
* add 169 MCP tools, fix deserialization bugs across Moralis/CoinGecko/Curve ([92085a1](https://github.com/yldfi/yldfi-rs/commit/92085a1fa7639eb1e2d62f8a0f2a836af264a257))
* add medium-effort API coverage — Tenderly batch, Dune CRUD, CowSwap orders ([5cdcf89](https://github.com/yldfi/yldfi-rs/commit/5cdcf892aab53199aa061021bac41d963be593ce))
* add multi-chain support and chainlist.org integration ([48c3e09](https://github.com/yldfi/yldfi-rs/commit/48c3e0943105f6680e65ec5743362649165b0f52))
* **ethcli:** add bytecode analysis and fix MCP bugs ([b74a17c](https://github.com/yldfi/yldfi-rs/commit/b74a17c2c0865fd335506f19fe4711a7c828ac4e))
* **ethcli:** add bytecode guard analysis and stabilize rpc selection ([7b4e3d1](https://github.com/yldfi/yldfi-rs/commit/7b4e3d1f5d96b11b3659d88d7d0f6399c099ec6e))
* **ethcli:** add direct DEX aggregator CLI commands ([f7ead70](https://github.com/yldfi/yldfi-rs/commit/f7ead7094e9f9e2ebad9ef93010f5e5a14b2aa73))
* **ethcli:** add Enso Finance as price source ([054e689](https://github.com/yldfi/yldfi-rs/commit/054e689b7caf177f747f6ba7f9bde4c3582affee))
* **ethcli:** add Enso Finance as price source in aggregator ([ff965d1](https://github.com/yldfi/yldfi-rs/commit/ff965d1d27423711e556e3d137936c7432239d4b))
* **ethcli:** add Kong vault-derived pricing source ([18ba09d](https://github.com/yldfi/yldfi-rs/commit/18ba09d8d9b504fbf234be75f9809816a24b9e4a))
* **ethcli:** add Kong vault-derived pricing source ([91a1eb4](https://github.com/yldfi/yldfi-rs/commit/91a1eb4fd5e6185c350d86c63807ff8ca7a85f3d))
* **simulate:** support Foundry 1.7 options ([541d308](https://github.com/yldfi/yldfi-rs/commit/541d3081c1d3011c63112cf5c8666b2c88c8e614))
* **tenderly:** full VNet integration with Admin RPC and simulation ([dac43e2](https://github.com/yldfi/yldfi-rs/commit/dac43e202028861fcb80474bf3b807560b6722bd))
* **tenderly:** full VNet integration with Admin RPC and simulation ([bb6d849](https://github.com/yldfi/yldfi-rs/commit/bb6d84985bf33a126c71fbcc0176c9605b219a85))


### Bug Fixes

* address Copilot review feedback for merged PRs ([4bec07c](https://github.com/yldfi/yldfi-rs/commit/4bec07cc1974b1ecb841cad2455ebaf7dbb5cf30))
* address Copilot review feedback for merged PRs ([b497355](https://github.com/yldfi/yldfi-rs/commit/b4973555548a5d2dd7fbcd0cdfcdaa90cdd62a58))
* address Copilot review feedback on PR [#38](https://github.com/yldfi/yldfi-rs/issues/38) ([79c9900](https://github.com/yldfi/yldfi-rs/commit/79c99002ad070ec849364e4194d68d210f1f633f))
* **crv:** update LendingVault struct to match Curve API response ([#19](https://github.com/yldfi/yldfi-rs/issues/19)) ([38a0bac](https://github.com/yldfi/yldfi-rs/commit/38a0bac6bdcae0772db212d5eae550ec25a8ec94))
* **ethcli-mcp:** harden MCP execution policy ([8f166a6](https://github.com/yldfi/yldfi-rs/commit/8f166a686cd22d8ecc721f290802d55f2cb8a7a1))
* **ethcli-mcp:** prevent data truncation in MCP tool outputs ([12da461](https://github.com/yldfi/yldfi-rs/commit/12da4612be2f9826971498a3c7a0c7a1c475e5f4))
* **ethcli:** fix 6 critical CLI bugs — gas estimate, beacon auth, allowance args, CCXT monthly, chainlink quote, sim overrides ([31b1cff](https://github.com/yldfi/yldfi-rs/commit/31b1cff89a1c0d150031a9a943a420eadae96aee)), closes [#37](https://github.com/yldfi/yldfi-rs/issues/37)
* **ethcli:** improve block ID parsing, node-type validation, and archive warnings ([0eddac0](https://github.com/yldfi/yldfi-rs/commit/0eddac0bf6fceb86a7679b6db92696437aab41e7))
* **ethcli:** improve Kong price source reliability ([#31](https://github.com/yldfi/yldfi-rs/issues/31)) ([4d8956a](https://github.com/yldfi/yldfi-rs/commit/4d8956a591e7d0e9a8bbe64dc4b7129c02195fdb))
* **ethcli:** improve price aggregation reliability and error reporting ([f6da4ae](https://github.com/yldfi/yldfi-rs/commit/f6da4aee814a8fb62c07ab9664b5b141c137b0a2))
* **ethcli:** increase download timeout for self-upgrade ([81a43a2](https://github.com/yldfi/yldfi-rs/commit/81a43a2b0330e1a0946b3523e9d1fb8f4a624658))
* **ethcli:** reduce false positives in bytecode security analysis ([2ca9421](https://github.com/yldfi/yldfi-rs/commit/2ca9421918fe27dbe32818834c3226138e539e31))
* **ethcli:** resolve clippy unnecessary_unwrap warning ([0b2934d](https://github.com/yldfi/yldfi-rs/commit/0b2934de93e308c9f3500ebd69381e852a74b862))
* **ethcli:** satisfy workspace clippy ([0296cbb](https://github.com/yldfi/yldfi-rs/commit/0296cbbbf988a5a4ed9d121a00d9c4b85be7bb22))
* replace stale chainlink streams sdk ([e2ec108](https://github.com/yldfi/yldfi-rs/commit/e2ec10801e7ff3ac9a115a73453177412230a853))
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([#38](https://github.com/yldfi/yldfi-rs/issues/38)) ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))
* resolve clippy print_literal warnings in chainlist output ([937fee1](https://github.com/yldfi/yldfi-rs/commit/937fee13f0466cd831edfb56360629b9ddf4fe2f))
* resolve clippy warnings ([3dcea82](https://github.com/yldfi/yldfi-rs/commit/3dcea82e8af3dcfe0f077365b90098e88f16a75b))
* resolve clippy warnings and doc test failure ([09ce397](https://github.com/yldfi/yldfi-rs/commit/09ce39721be834c176ce28229765752d722249a9))
* resolve clippy warnings in tests ([3da0cb0](https://github.com/yldfi/yldfi-rs/commit/3da0cb07d2dd1ba2d5febde875252dcab7efcf46))
* resolve clippy warnings in tx analyzer ([cd283d2](https://github.com/yldfi/yldfi-rs/commit/cd283d2446b7e9c670357e1af14e5ec6ea20dfaf))
* satisfy remaining rust 1.95 clippy lint ([0a6c25f](https://github.com/yldfi/yldfi-rs/commit/0a6c25f18e9a9e872b59d4dda3bf41b00d3c004b))
* satisfy rust 1.95 clippy lints ([4729526](https://github.com/yldfi/yldfi-rs/commit/4729526d86dd54b4d286261da3d4e86241aa5acf))
* satisfy rust 1.95 priority sort lint ([c2e98d9](https://github.com/yldfi/yldfi-rs/commit/c2e98d9004eeb8212d190043af697da190db85b4))
* support explicit contract signatures and dynamic abi decode ([f453b86](https://github.com/yldfi/yldfi-rs/commit/f453b86ce319f74d8a582501574f2c2352c474c2))
* **tests:** update chainlink_live tests for new fetch_price denomination param ([80a9960](https://github.com/yldfi/yldfi-rs/commit/80a9960e4bc8145925558f8c3e767d7dac393f4d))
* update Enso API client for new endpoints ([1c631e9](https://github.com/yldfi/yldfi-rs/commit/1c631e9eacc6d43813cd5d8160f37a0eb27706f1))
* use struct initializer syntax for Dune options (clippy::field_reassign_with_default) ([590551d](https://github.com/yldfi/yldfi-rs/commit/590551d37a6f3ced92086a78af9190952f509cc6))

## [0.22.1](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.22.0...ethcli-v0.22.1) (2026-05-10)

### Features

* **simulate:** pass Foundry 1.7 trace, replay, RPC, and Anvil fork options through to cast/anvil

### Dependencies

* **ethcli:** upgrade direct Alloy dependency to 2.x and foundry-block-explorers to 0.23

## [0.22.0](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.25...ethcli-v0.22.0) (2026-05-04)

### Features

* **contract:** add optional dispatcher mapping and side-effecting handler guard/check heuristics for unverified contract review
* **contract:** add proxy following to selector extraction
* **rpc:** prefer tested full/archive RPC endpoints over unknown endpoints for single-endpoint selection

### Bug Fixes

* **rpc:** keep restricted relay RPCs such as Flashbots as fallback-only for general read calls

## [0.21.25](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.24...ethcli-v0.21.25) (2026-04-22)


### Bug Fixes

* replace stale chainlink streams sdk ([e2ec108](https://github.com/yldfi/yldfi-rs/commit/e2ec10801e7ff3ac9a115a73453177412230a853))
* satisfy remaining rust 1.95 clippy lint ([0a6c25f](https://github.com/yldfi/yldfi-rs/commit/0a6c25f18e9a9e872b59d4dda3bf41b00d3c004b))
* satisfy rust 1.95 clippy lints ([4729526](https://github.com/yldfi/yldfi-rs/commit/4729526d86dd54b4d286261da3d4e86241aa5acf))
* satisfy rust 1.95 priority sort lint ([c2e98d9](https://github.com/yldfi/yldfi-rs/commit/c2e98d9004eeb8212d190043af697da190db85b4))
* support explicit contract signatures and dynamic abi decode ([f453b86](https://github.com/yldfi/yldfi-rs/commit/f453b86ce319f74d8a582501574f2c2352c474c2))

## [0.21.24](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.23...ethcli-v0.21.24) (2026-02-16)


### Bug Fixes

* **ethcli:** increase download timeout for self-upgrade ([81a43a2](https://github.com/yldfi/yldfi-rs/commit/81a43a2b0330e1a0946b3523e9d1fb8f4a624658))

## [0.21.23](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.22...ethcli-v0.21.23) (2026-02-16)


### Bug Fixes

* address Copilot review feedback on PR [#38](https://github.com/yldfi/yldfi-rs/issues/38) ([79c9900](https://github.com/yldfi/yldfi-rs/commit/79c99002ad070ec849364e4194d68d210f1f633f))
* **ethcli:** fix 6 critical CLI bugs — gas estimate, beacon auth, allowance args, CCXT monthly, chainlink quote, sim overrides ([31b1cff](https://github.com/yldfi/yldfi-rs/commit/31b1cff89a1c0d150031a9a943a420eadae96aee)), closes [#37](https://github.com/yldfi/yldfi-rs/issues/37)
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))
* resolve ~45 MCP parameter mismatches + 6 critical CLI bugs ([#38](https://github.com/yldfi/yldfi-rs/issues/38)) ([5d86ba6](https://github.com/yldfi/yldfi-rs/commit/5d86ba6384323977b49ea4c05342f92c5d1489a6))
* **tests:** update chainlink_live tests for new fetch_price denomination param ([80a9960](https://github.com/yldfi/yldfi-rs/commit/80a9960e4bc8145925558f8c3e767d7dac393f4d))

## [0.21.22](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.21...ethcli-v0.21.22) (2026-02-15)


### Bug Fixes

* **ethcli:** improve price aggregation reliability and error reporting ([f6da4ae](https://github.com/yldfi/yldfi-rs/commit/f6da4aee814a8fb62c07ab9664b5b141c137b0a2))

## [0.21.21](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.20...ethcli-v0.21.21) (2026-02-15)


### Bug Fixes

* **ethcli:** improve Kong price source reliability ([#31](https://github.com/yldfi/yldfi-rs/issues/31)) ([4d8956a](https://github.com/yldfi/yldfi-rs/commit/4d8956a591e7d0e9a8bbe64dc4b7129c02195fdb))

## [0.21.20](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.19...ethcli-v0.21.20) (2026-02-14)


### Features

* **ethcli:** add Enso Finance as price source ([054e689](https://github.com/yldfi/yldfi-rs/commit/054e689b7caf177f747f6ba7f9bde4c3582affee))
* **ethcli:** add Enso Finance as price source in aggregator ([ff965d1](https://github.com/yldfi/yldfi-rs/commit/ff965d1d27423711e556e3d137936c7432239d4b))
* **ethcli:** add Kong vault-derived pricing source ([18ba09d](https://github.com/yldfi/yldfi-rs/commit/18ba09d8d9b504fbf234be75f9809816a24b9e4a))
* **ethcli:** add Kong vault-derived pricing source ([91a1eb4](https://github.com/yldfi/yldfi-rs/commit/91a1eb4fd5e6185c350d86c63807ff8ca7a85f3d))


### Bug Fixes

* address Copilot review feedback for merged PRs ([4bec07c](https://github.com/yldfi/yldfi-rs/commit/4bec07cc1974b1ecb841cad2455ebaf7dbb5cf30))
* address Copilot review feedback for merged PRs ([b497355](https://github.com/yldfi/yldfi-rs/commit/b4973555548a5d2dd7fbcd0cdfcdaa90cdd62a58))

## [0.21.19](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.18...ethcli-v0.21.19) (2026-02-14)


### Features

* add multi-chain support and chainlist.org integration ([48c3e09](https://github.com/yldfi/yldfi-rs/commit/48c3e0943105f6680e65ec5743362649165b0f52))


### Bug Fixes

* resolve clippy print_literal warnings in chainlist output ([937fee1](https://github.com/yldfi/yldfi-rs/commit/937fee13f0466cd831edfb56360629b9ddf4fe2f))

## [0.21.18](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.17...ethcli-v0.21.18) (2026-02-13)


### Features

* add 130+ CLI commands for Alchemy & Moralis, fix formatting ([b580a6e](https://github.com/yldfi/yldfi-rs/commit/b580a6edb67179787771d854bc44dcec5ddfc8c0))
* add 169 MCP tools, fix deserialization bugs across Moralis/CoinGecko/Curve ([92085a1](https://github.com/yldfi/yldfi-rs/commit/92085a1fa7639eb1e2d62f8a0f2a836af264a257))
* add medium-effort API coverage — Tenderly batch, Dune CRUD, CowSwap orders ([5cdcf89](https://github.com/yldfi/yldfi-rs/commit/5cdcf892aab53199aa061021bac41d963be593ce))
* **tenderly:** full VNet integration with Admin RPC and simulation ([dac43e2](https://github.com/yldfi/yldfi-rs/commit/dac43e202028861fcb80474bf3b807560b6722bd))
* **tenderly:** full VNet integration with Admin RPC and simulation ([bb6d849](https://github.com/yldfi/yldfi-rs/commit/bb6d84985bf33a126c71fbcc0176c9605b219a85))


### Bug Fixes

* use struct initializer syntax for Dune options (clippy::field_reassign_with_default) ([590551d](https://github.com/yldfi/yldfi-rs/commit/590551d37a6f3ced92086a78af9190952f509cc6))

## [0.21.17](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.16...ethcli-v0.21.17) (2026-02-04)


### Bug Fixes

* **crv:** update LendingVault struct to match Curve API response ([#19](https://github.com/yldfi/yldfi-rs/issues/19)) ([38a0bac](https://github.com/yldfi/yldfi-rs/commit/38a0bac6bdcae0772db212d5eae550ec25a8ec94))

## [0.21.16](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.15...ethcli-v0.21.16) (2026-02-02)


### Features

* add 'upgrade' command, keep 'update' for checking only ([e40ee32](https://github.com/yldfi/yldfi-rs/commit/e40ee325fefab3cb3795c05276bf362ede2a5ce7))
* **ethcli:** add bytecode analysis and fix MCP bugs ([b74a17c](https://github.com/yldfi/yldfi-rs/commit/b74a17c2c0865fd335506f19fe4711a7c828ac4e))
* **ethcli:** add direct DEX aggregator CLI commands ([f7ead70](https://github.com/yldfi/yldfi-rs/commit/f7ead7094e9f9e2ebad9ef93010f5e5a14b2aa73))


### Bug Fixes

* **ethcli-mcp:** prevent data truncation in MCP tool outputs ([12da461](https://github.com/yldfi/yldfi-rs/commit/12da4612be2f9826971498a3c7a0c7a1c475e5f4))
* **ethcli:** improve block ID parsing, node-type validation, and archive warnings ([0eddac0](https://github.com/yldfi/yldfi-rs/commit/0eddac0bf6fceb86a7679b6db92696437aab41e7))
* **ethcli:** reduce false positives in bytecode security analysis ([2ca9421](https://github.com/yldfi/yldfi-rs/commit/2ca9421918fe27dbe32818834c3226138e539e31))
* **ethcli:** resolve clippy unnecessary_unwrap warning ([0b2934d](https://github.com/yldfi/yldfi-rs/commit/0b2934de93e308c9f3500ebd69381e852a74b862))
* resolve clippy warnings ([3dcea82](https://github.com/yldfi/yldfi-rs/commit/3dcea82e8af3dcfe0f077365b90098e88f16a75b))
* resolve clippy warnings and doc test failure ([09ce397](https://github.com/yldfi/yldfi-rs/commit/09ce39721be834c176ce28229765752d722249a9))
* resolve clippy warnings in tests ([3da0cb0](https://github.com/yldfi/yldfi-rs/commit/3da0cb07d2dd1ba2d5febde875252dcab7efcf46))
* resolve clippy warnings in tx analyzer ([cd283d2](https://github.com/yldfi/yldfi-rs/commit/cd283d2446b7e9c670357e1af14e5ec6ea20dfaf))
* update Enso API client for new endpoints ([1c631e9](https://github.com/yldfi/yldfi-rs/commit/1c631e9eacc6d43813cd5d8160f37a0eb27706f1))

## [0.21.15](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.14...ethcli-v0.21.15) (2026-02-02)


### Bug Fixes

* resolve clippy warnings in tests ([3da0cb0](https://github.com/yldfi/yldfi-rs/commit/3da0cb07d2dd1ba2d5febde875252dcab7efcf46))

## [0.21.14](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.13...ethcli-v0.21.14) (2026-01-30)


### Bug Fixes

* **ethcli:** reduce false positives in bytecode security analysis ([2ca9421](https://github.com/yldfi/yldfi-rs/commit/2ca9421918fe27dbe32818834c3226138e539e31))

## [0.21.13](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.12...ethcli-v0.21.13) (2026-01-30)


### Features

* add 'upgrade' command, keep 'update' for checking only ([e40ee32](https://github.com/yldfi/yldfi-rs/commit/e40ee325fefab3cb3795c05276bf362ede2a5ce7))
* **ethcli:** add bytecode analysis and fix MCP bugs ([b74a17c](https://github.com/yldfi/yldfi-rs/commit/b74a17c2c0865fd335506f19fe4711a7c828ac4e))


### Bug Fixes

* resolve clippy warnings ([3dcea82](https://github.com/yldfi/yldfi-rs/commit/3dcea82e8af3dcfe0f077365b90098e88f16a75b))

## [0.21.12](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.11...ethcli-v0.21.12) (2026-01-30)


### Bug Fixes

* update Enso API client for new endpoints ([1c631e9](https://github.com/yldfi/yldfi-rs/commit/1c631e9eacc6d43813cd5d8160f37a0eb27706f1))

## [0.21.11](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.10...ethcli-v0.21.11) (2026-01-29)


### Bug Fixes

* **ethcli-mcp:** prevent data truncation in MCP tool outputs ([12da461](https://github.com/yldfi/yldfi-rs/commit/12da4612be2f9826971498a3c7a0c7a1c475e5f4))
* resolve clippy warnings in tx analyzer ([cd283d2](https://github.com/yldfi/yldfi-rs/commit/cd283d2446b7e9c670357e1af14e5ec6ea20dfaf))

## [0.21.10](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.9...ethcli-v0.21.10) (2026-01-28)


### Features

* **ethcli:** add direct DEX aggregator CLI commands ([f7ead70](https://github.com/yldfi/yldfi-rs/commit/f7ead7094e9f9e2ebad9ef93010f5e5a14b2aa73))


### Bug Fixes

* resolve clippy warnings and doc test failure ([09ce397](https://github.com/yldfi/yldfi-rs/commit/09ce39721be834c176ce28229765752d722249a9))

## [0.21.9](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.8...ethcli-v0.21.9) (2026-01-27)


### Features

* **gplus:** add GoPlus Security API client crate ([e7d8505](https://github.com/yldfi/yldfi-rs/commit/e7d85053f49d3988affb68d8e6adbc94762e89f6))
* **sldt:** add Solodit vulnerability database client ([0593418](https://github.com/yldfi/yldfi-rs/commit/05934189ea2cce326caf7887d58669e883e641ca))

## [0.21.8](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.7...ethcli-v0.21.8) (2026-01-26)


### Features

* add ykong (Yearn Kong) and unswp improvements ([f065b43](https://github.com/yldfi/yldfi-rs/commit/f065b4313245716d0671df278441bc05f048f872))
* **ethcli:** add Uniswap V2/V3/V4 support and refactor API clients ([46df1e0](https://github.com/yldfi/yldfi-rs/commit/46df1e0b7bdf733f504b00931cd310ec429cc0c7))


### Bug Fixes

* address 14 issues from multi-agent code review ([d1e8de0](https://github.com/yldfi/yldfi-rs/commit/d1e8de0428b0380572cd01ffd1053649bc94840f))
* address 44 issues from multi-agent code review ([5812bae](https://github.com/yldfi/yldfi-rs/commit/5812baea74461394a0f8a68ec490987b1cda168d))
* address 8 issues from follow-up code review ([97a3f18](https://github.com/yldfi/yldfi-rs/commit/97a3f18b7e6b17e2984949d3daa968adf225e027))
* address 9 issues from fresh code review ([ee275fe](https://github.com/yldfi/yldfi-rs/commit/ee275fe54a46e5b93ec1aae91a8afef38fe2c967))
* **ethcli:** address 16 issues from multi-agent code review ([09f50be](https://github.com/yldfi/yldfi-rs/commit/09f50be38184ce4b0f35302da49965d66910c3b3))


### Performance Improvements

* **ethcli:** optimize HTTP client pooling and reduce allocations ([6f7886e](https://github.com/yldfi/yldfi-rs/commit/6f7886ea89f013398099e824fc5576985e675eaa))

## [0.21.7](https://github.com/yldfi/yldfi-rs/compare/ethcli-v0.21.6...ethcli-v0.21.7) (2026-01-23)


### Features

* **ethcli:** add SecretString for API key protection and chainlink/pyth modules ([58ee636](https://github.com/yldfi/yldfi-rs/commit/58ee63625b76a83d55a8f27611f2f62c2e00e241))
* implement review agent recommendations ([f8c9f42](https://github.com/yldfi/yldfi-rs/commit/f8c9f42813785c96a8bd5f694bbcf56fb6c16b49))


### Bug Fixes

* handle uint80 round IDs in Chainlink price feeds ([28c391e](https://github.com/yldfi/yldfi-rs/commit/28c391e7466c78a6f303b6d64fca8b55c84085aa))
* resolve clippy warnings in chainlink.rs ([3be41f5](https://github.com/yldfi/yldfi-rs/commit/3be41f54f3f4aed154c73e980c568e10a8051baf))

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
