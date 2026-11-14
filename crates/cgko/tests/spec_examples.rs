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

yldfi_spec_test::spec_tests! {
    file: "spec_examples.json",
    checks: {
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
    },
    // Spec operations this crate does not implement yet. Adding an endpoint
    // means moving it from here into `checks`.
    not_implemented: [
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
    ],
}
