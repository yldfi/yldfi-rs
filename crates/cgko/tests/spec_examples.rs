//! Deserializes every example response in `CoinGecko`'s OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! returns for that endpoint, so a type that no longer matches the spec fails
//! here instead of at runtime.
//!
//! The examples live in `spec_examples.json`. Refresh them with
//! `python3 scripts/spec-check/examples.py
//! https://raw.githubusercontent.com/coingecko/coingecko-api-oas/main/pro-api.json
//! crates/cgko/tests/spec_examples.json`.

use cgko::{categories, coins, derivatives, exchanges, global, nfts, onchain, simple, treasury};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;

type Check = fn(&Value) -> Result<(), String>;

fn parse<T: DeserializeOwned>(v: &Value) -> Result<(), String> {
    serde_json::from_value::<T>(v.clone())
        .map(drop)
        .map_err(|e| e.to_string())
}

/// operationId -> the crate's response type for it
fn checks() -> BTreeMap<&'static str, Check> {
    macro_rules! map {
        ($($op:literal => $ty:ty),* $(,)?) => {{
            let mut m: BTreeMap<&'static str, Check> = BTreeMap::new();
            $( m.insert($op, parse::<$ty>); )*
            m
        }};
    }
    map! {
        // Simple
        "simple-price" => simple::PricesResponse,
        "simple-supported-currencies" => simple::SupportedCurrencies,
        "simple-token-price" => simple::TokenPricesResponse,
        // Coins
        "coins-list" => Vec<coins::CoinListItem>,
        "coins-list-new" => Vec<coins::RecentlyAddedCoin>,
        "coins-markets" => Vec<coins::CoinMarket>,
        "coins-top-gainers-losers" => coins::TopMoversResponse,
        "coins-id" => coins::CoinData,
        "coins-id-tickers" => coins::CoinTickers,
        "coins-id-history" => coins::CoinHistory,
        "coins-id-market-chart" => coins::MarketChart,
        "coins-id-market-chart-range" => coins::MarketChart,
        "coins-id-ohlc" => coins::OhlcData,
        "coins-id-ohlc-range" => coins::OhlcData,
        "coins-id-circulating-supply-chart" => coins::SupplyChart,
        "coins-id-circulating-supply-chart-range" => coins::SupplyChart,
        "coins-id-total-supply-chart" => coins::SupplyChart,
        "coins-id-total-supply-chart-range" => coins::SupplyChart,
        "coins-contract-address" => coins::CoinContractData,
        "contract-address-market-chart" => coins::MarketChart,
        "contract-address-market-chart-range" => coins::MarketChart,
        // Categories
        "coins-categories-list" => Vec<categories::CategoryListItem>,
        "coins-categories" => Vec<categories::Category>,
        // Exchanges
        "exchanges" => Vec<exchanges::Exchange>,
        "exchanges-list" => Vec<exchanges::ExchangeListItem>,
        "exchanges-id" => exchanges::Exchange,
        "exchanges-id-tickers" => exchanges::ExchangeTickers,
        "exchanges-id-volume-chart" => exchanges::VolumeChart,
        "exchanges-id-volume-chart-range" => exchanges::VolumeChart,
        // Derivatives
        "derivatives-tickers" => Vec<derivatives::DerivativeTicker>,
        "derivatives-exchanges" => Vec<derivatives::DerivativeExchange>,
        "derivatives-exchanges-id" => derivatives::DerivativeExchangeDetail,
        "derivatives-exchanges-list" => Vec<derivatives::DerivativeExchangeListItem>,
        "derivatives-underlyings-list" => Vec<derivatives::UnderlyingAsset>,
        "derivatives-underlyings" => Vec<derivatives::UnderlyingAssetData>,
        // NFTs
        "nfts-list" => Vec<nfts::NftListItem>,
        "nfts-id" => nfts::NftCollection,
        "nfts-contract-address" => nfts::NftCollection,
        "nfts-markets" => Vec<nfts::NftMarketItem>,
        "nfts-id-market-chart" => nfts::NftMarketChart,
        "nfts-contract-address-market-chart" => nfts::NftMarketChart,
        "nfts-id-tickers" => nfts::NftTickersResponse,
        // Treasury
        "entities-list" => Vec<treasury::EntityListItem>,
        "companies-public-treasury" => treasury::PublicTreasuryByCoin,
        "public-treasury-entity" => treasury::PublicTreasuryByEntity,
        "public-treasury-entity-chart" => treasury::HoldingChart,
        "public-treasury-transaction-history" => treasury::TransactionHistory,
        // Global and general
        "ping-server" => global::PingResponse,
        "api-usage" => global::ApiKeyUsage,
        "search-data" => global::SearchResponse,
        "trending-search" => global::TrendingResponse,
        "asset-platforms-list" => Vec<global::AssetPlatform>,
        "token-lists" => global::TokenList,
        "exchange-rates" => global::ExchangeRatesResponse,
        "crypto-global" => global::GlobalResponse,
        "global-defi" => global::DefiGlobalResponse,
        "global-market-cap-chart" => global::MarketCapChart,
        "global-fear-and-greed-chart" => global::FearAndGreedChart,
        // Onchain
        "networks-list" => onchain::NetworksResponse,
        "dexes-list" => onchain::DexesResponse,
        "pool-address" => onchain::PoolResponse,
        "pools-addresses" => onchain::PoolsResponse,
        "trending-pools-list" => onchain::PoolsResponse,
        "trending-pools-network" => onchain::PoolsResponse,
        "top-pools-network" => onchain::PoolsResponse,
        "top-pools-dex" => onchain::PoolsResponse,
        "latest-pools-list" => onchain::PoolsResponse,
        "latest-pools-network" => onchain::PoolsResponse,
        "pools-megafilter" => onchain::PoolsResponse,
        "search-pools" => onchain::PoolsResponse,
        "trending-search-pools" => onchain::PoolsResponse,
        "pools-category" => onchain::PoolsResponse,
        "categories-list" => onchain::OnchainCategoriesResponse,
        "top-pools-contract-address" => onchain::PoolsResponse,
        "token-data-contract-address" => onchain::TokenResponse,
        "tokens-data-contract-addresses" => onchain::TokensResponse,
        "token-info-contract-address" => onchain::TokenInfoResponse,
        "pool-token-info-contract-address" => onchain::PoolInfoResponse,
        "tokens-info-recent-updated" => onchain::TokensResponse,
        "top-token-holders-token-address" => onchain::TokenHoldersResponse,
        "top-token-traders-token-address" => onchain::TokenTradersResponse,
        "token-holders-chart-token-address" => onchain::HoldersChartResponse,
        "pool-ohlcv-contract-address" => onchain::OhlcvResponse,
        "token-ohlcv-token-address" => onchain::TokenOhlcvResponse,
        "pool-trades-contract-address" => onchain::TradesResponse,
        "token-trades-contract-address" => onchain::TokenTradesResponse,
        "pool-liquidity-events-contract-address" => onchain::LiquidityEventsResponse,
        "onchain-simple-price" => onchain::TokenPriceResponse,
    }
}

/// Spec operations this crate does not implement yet. Adding an endpoint
/// means moving it from here into `checks()`.
const NOT_IMPLEMENTED: &[&str] = &[
    "coins-id-supply-breakdown",
    "insights",
    "news",
    "onchain-simple-price-multi",
    "pool-trades-contract-address-range",
    "rwas-id",
    "rwas-id-market-chart",
    "rwas-id-tickers",
    "rwas-issuers-id",
    "rwas-issuers-list",
    "rwas-list",
    "rwas-markets",
    "token-trades-contract-address-range",
    "tokens-data-contract-addresses-multi",
    "wallet-pnl",
    "wallet-token-balances",
    "wallet-token-transfers",
    "wallet-trades",
];

/// operationId -> its `example`, `sample` and `sample2` responses
fn examples() -> BTreeMap<String, Value> {
    let text = include_str!("spec_examples.json");
    serde_json::from_str(text).expect("spec_examples.json")
}

fn check_all(kind: &str) {
    let examples = examples();
    let mut failures = Vec::new();
    for (op, check) in checks() {
        let Some(entry) = examples.get(op) else {
            failures.push(format!("{op}: not in spec_examples.json"));
            continue;
        };
        if let Some(value) = entry.get(kind) {
            if let Err(e) = check(value) {
                failures.push(format!("{op}: {e}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} spec {kind}(s) failed to deserialize:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// The spec's own example responses
#[test]
fn spec_examples_deserialize() {
    check_all("example");
}

/// Responses built from the schema, with every field present
#[test]
fn spec_schema_samples_deserialize() {
    check_all("sample");
}

/// The same, taking the last alternative where a field may be one of
/// several types
#[test]
fn spec_schema_alternative_samples_deserialize() {
    check_all("sample2");
}

#[test]
fn every_spec_operation_is_checked_or_listed() {
    let checks = checks();
    let unknown: Vec<_> = examples()
        .into_keys()
        .filter(|op| !checks.contains_key(op.as_str()) && !NOT_IMPLEMENTED.contains(&op.as_str()))
        .collect();
    assert!(
        unknown.is_empty(),
        "spec operations neither checked nor listed in NOT_IMPLEMENTED: {unknown:?}"
    );
}
