//! Mocked tests for the aggregator module
//!
//! These tests verify the aggregation logic without requiring external API calls.

use ethcli::aggregator::{
    AggregatedResult, NormalizedPrice, PriceAggregation, PriceSource, SourceResult,
};

// ==================== Symbol mapping tests ====================

#[test]
fn test_symbol_to_coingecko_id_common_tokens() {
    use ethcli::aggregator::price::symbol_to_coingecko_id;

    // Native tokens
    assert_eq!(symbol_to_coingecko_id("ETH"), "ethereum");
    assert_eq!(symbol_to_coingecko_id("BTC"), "bitcoin");
    assert_eq!(symbol_to_coingecko_id("SOL"), "solana");

    // Case insensitive
    assert_eq!(symbol_to_coingecko_id("eth"), "ethereum");
    assert_eq!(symbol_to_coingecko_id("Btc"), "bitcoin");

    // Stablecoins
    assert_eq!(symbol_to_coingecko_id("USDC"), "usd-coin");
    assert_eq!(symbol_to_coingecko_id("USDT"), "tether");
    assert_eq!(symbol_to_coingecko_id("DAI"), "dai");

    // DeFi tokens
    assert_eq!(symbol_to_coingecko_id("LINK"), "chainlink");
    assert_eq!(symbol_to_coingecko_id("UNI"), "uniswap");
    assert_eq!(symbol_to_coingecko_id("AAVE"), "aave");
    assert_eq!(symbol_to_coingecko_id("CRV"), "curve-dao-token");

    // Unknown tokens fallback to lowercase
    assert_eq!(symbol_to_coingecko_id("UNKNOWN"), "unknown");
}

#[test]
fn test_symbol_to_eth_address() {
    use ethcli::aggregator::price::symbol_to_eth_address;

    // Common tokens
    assert_eq!(
        symbol_to_eth_address("USDC"),
        Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    );
    assert_eq!(
        symbol_to_eth_address("WETH"),
        Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
    );
    assert_eq!(
        symbol_to_eth_address("LINK"),
        Some("0x514910771AF9Ca656af840dff83E8264EcF986CA")
    );

    // Case insensitive
    assert_eq!(
        symbol_to_eth_address("usdc"),
        Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    );

    // Unknown tokens
    assert_eq!(symbol_to_eth_address("UNKNOWN_TOKEN"), None);
}

#[test]
fn test_eth_address_to_symbol() {
    use ethcli::aggregator::price::eth_address_to_symbol;

    // Common tokens (lowercase addresses)
    assert_eq!(
        eth_address_to_symbol("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
        Some("USDC")
    );
    assert_eq!(
        eth_address_to_symbol("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
        Some("WETH")
    );

    // Mixed case should work
    assert_eq!(
        eth_address_to_symbol("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        Some("USDC")
    );

    // Unknown addresses
    assert_eq!(
        eth_address_to_symbol("0x0000000000000000000000000000000000000001"),
        None
    );
}

#[test]
fn test_is_token_address() {
    use ethcli::aggregator::price::is_token_address;

    // Valid addresses
    assert!(is_token_address(
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    ));
    assert!(is_token_address(
        "0x0000000000000000000000000000000000000001"
    ));

    // Invalid - wrong length
    assert!(!is_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB4")); // 41 chars
    assert!(!is_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48a")); // 43 chars

    // Invalid - not hex
    assert!(!is_token_address("ETH"));
    assert!(!is_token_address("USDC"));

    // Invalid - no 0x prefix
    assert!(!is_token_address(
        "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    ));
}

// ==================== SourceResult tests ====================

#[test]
fn test_source_result_success() {
    let price = NormalizedPrice::new(1500.0);
    let result = SourceResult::success("test_source", price.clone(), 100);

    assert!(result.is_success());
    assert_eq!(result.source, "test_source");
    assert_eq!(result.data.unwrap().usd, 1500.0);
    assert_eq!(result.latency_ms, 100);
    assert!(result.error.is_none());
}

#[test]
fn test_source_result_error() {
    let result: SourceResult<NormalizedPrice> =
        SourceResult::error("test_source", "connection failed", 50);

    assert!(!result.is_success());
    assert_eq!(result.source, "test_source");
    assert!(result.data.is_none());
    assert_eq!(result.error, Some("connection failed".to_string()));
    assert_eq!(result.latency_ms, 50);
}

#[test]
fn test_source_result_with_raw() {
    let price = NormalizedPrice::new(100.0);
    let raw = serde_json::json!({"price": 100.0, "currency": "USD"});
    let result = SourceResult::success_with_raw("test", price, raw.clone(), 75);

    assert!(result.is_success());
    assert_eq!(result.raw, Some(raw));
}

// ==================== NormalizedPrice tests ====================

#[test]
fn test_normalized_price_new() {
    let price = NormalizedPrice::new(1500.0);
    assert_eq!(price.usd, 1500.0);
    assert!(price.confidence.is_none());
    assert!(price.change_24h_pct.is_none());
}

#[test]
fn test_normalized_price_with_confidence() {
    let price = NormalizedPrice::new(1500.0).with_confidence(0.95);
    assert_eq!(price.usd, 1500.0);
    assert_eq!(price.confidence, Some(0.95));
}

#[test]
fn test_normalized_price_with_change() {
    let price = NormalizedPrice::new(1500.0).with_change_24h(-2.5);
    assert_eq!(price.usd, 1500.0);
    assert_eq!(price.change_24h_pct, Some(-2.5));
}

// ==================== PriceAggregation tests ====================

#[test]
fn test_price_aggregation_from_single_price() {
    let prices = vec![1500.0];
    let agg = PriceAggregation::from_prices(&prices).unwrap();

    assert_eq!(agg.median_usd, 1500.0);
    assert_eq!(agg.mean_usd, 1500.0);
    assert_eq!(agg.min_usd, 1500.0);
    assert_eq!(agg.max_usd, 1500.0);
    assert_eq!(agg.spread_pct, 0.0);
    assert!(agg.sources_agreed);
}

#[test]
fn test_price_aggregation_from_multiple_prices() {
    let prices = vec![1500.0, 1505.0, 1510.0];
    let agg = PriceAggregation::from_prices(&prices).unwrap();

    assert_eq!(agg.median_usd, 1505.0);
    assert!((agg.mean_usd - 1505.0).abs() < 0.01);
    assert_eq!(agg.min_usd, 1500.0);
    assert_eq!(agg.max_usd, 1510.0);
    // Spread = (1510 - 1500) / 1505 * 100 ≈ 0.66%
    assert!(agg.spread_pct > 0.0 && agg.spread_pct < 1.0);
    assert!(agg.sources_agreed); // Within 1% spread
}

#[test]
fn test_price_aggregation_sources_disagree() {
    let prices = vec![100.0, 200.0]; // 100% spread
    let agg = PriceAggregation::from_prices(&prices).unwrap();

    assert!(!agg.sources_agreed);
    assert!(agg.spread_pct > 1.0);
}

#[test]
fn test_price_aggregation_empty_prices() {
    let prices: Vec<f64> = vec![];
    let agg = PriceAggregation::from_prices(&prices);

    assert!(agg.is_none());
}

#[test]
fn test_price_aggregation_even_number_of_prices() {
    let prices = vec![100.0, 200.0, 300.0, 400.0];
    let agg = PriceAggregation::from_prices(&prices).unwrap();

    // Median of [100, 200, 300, 400] = average of 200 and 300 = 250
    assert_eq!(agg.median_usd, 250.0);
    assert_eq!(agg.mean_usd, 250.0);
}

// ==================== AggregatedResult tests ====================

#[test]
fn test_aggregated_result_all_succeeded() {
    let sources = vec![
        SourceResult::success("source_a", NormalizedPrice::new(100.0), 10),
        SourceResult::success("source_b", NormalizedPrice::new(101.0), 15),
        SourceResult::success("source_c", NormalizedPrice::new(99.0), 12),
    ];

    let agg = PriceAggregation::from_prices(&[100.0, 101.0, 99.0]).unwrap();
    let result = AggregatedResult::new(agg, sources, 20);

    assert_eq!(result.sources_queried, 3);
    assert_eq!(result.sources_succeeded, 3);
    assert!(result.all_succeeded());
    assert!(result.any_succeeded());
    assert!((result.success_rate() - 100.0).abs() < 0.01);
}

#[test]
fn test_aggregated_result_partial_success() {
    let sources: Vec<SourceResult<NormalizedPrice>> = vec![
        SourceResult::success("source_a", NormalizedPrice::new(100.0), 10),
        SourceResult::error("source_b", "failed", 15),
        SourceResult::success("source_c", NormalizedPrice::new(99.0), 12),
    ];

    let agg = PriceAggregation::from_prices(&[100.0, 99.0]).unwrap();
    let result = AggregatedResult::new(agg, sources, 20);

    assert_eq!(result.sources_queried, 3);
    assert_eq!(result.sources_succeeded, 2);
    assert!(!result.all_succeeded());
    assert!(result.any_succeeded());
    assert!((result.success_rate() - 66.66).abs() < 1.0);
}

#[test]
fn test_aggregated_result_all_failed() {
    let sources: Vec<SourceResult<NormalizedPrice>> = vec![
        SourceResult::error("source_a", "error 1", 10),
        SourceResult::error("source_b", "error 2", 15),
    ];

    let agg = PriceAggregation {
        median_usd: 0.0,
        mean_usd: 0.0,
        min_usd: 0.0,
        max_usd: 0.0,
        spread_pct: 0.0,
        sources_agreed: false,
        best_source: None,
    };

    let result = AggregatedResult::new(agg, sources, 20);

    assert_eq!(result.sources_queried, 2);
    assert_eq!(result.sources_succeeded, 0);
    assert!(!result.all_succeeded());
    assert!(!result.any_succeeded());
    assert!((result.success_rate() - 0.0).abs() < 0.01);
}

// ==================== PriceSource tests ====================

#[test]
fn test_price_source_from_str() {
    assert_eq!("gecko".parse::<PriceSource>().unwrap(), PriceSource::Gecko);
    assert_eq!(
        "coingecko".parse::<PriceSource>().unwrap(),
        PriceSource::Gecko
    );
    assert_eq!("llama".parse::<PriceSource>().unwrap(), PriceSource::Llama);
    assert_eq!(
        "defillama".parse::<PriceSource>().unwrap(),
        PriceSource::Llama
    );
    assert_eq!(
        "alchemy".parse::<PriceSource>().unwrap(),
        PriceSource::Alchemy
    );
    assert_eq!("ccxt".parse::<PriceSource>().unwrap(), PriceSource::Ccxt);
    assert_eq!(
        "chainlink".parse::<PriceSource>().unwrap(),
        PriceSource::Chainlink
    );
    assert_eq!("pyth".parse::<PriceSource>().unwrap(), PriceSource::Pyth);
    assert!("invalid".parse::<PriceSource>().is_err());
}

#[test]
fn test_price_source_display() {
    assert_eq!(PriceSource::Gecko.to_string(), "gecko");
    assert_eq!(PriceSource::Llama.to_string(), "llama");
    assert_eq!(PriceSource::All.to_string(), "all");
    assert_eq!(PriceSource::Chainlink.to_string(), "chainlink");
    assert_eq!(PriceSource::Pyth.to_string(), "pyth");
}

// ==================== Chain mapping tests ====================

#[test]
fn test_chain_normalization_for_gecko() {
    use ethcli::aggregator::chain_map::normalize_chain_for_source;

    assert_eq!(normalize_chain_for_source("gecko", "ethereum"), "ethereum");
    assert_eq!(normalize_chain_for_source("gecko", "eth"), "ethereum");
    assert_eq!(normalize_chain_for_source("gecko", "polygon"), "polygon-pos");
    assert_eq!(
        normalize_chain_for_source("gecko", "arbitrum"),
        "arbitrum-one"
    );
    // "op" maps to optimistic-ethereum, but "optimism" falls through to lowercase
    assert_eq!(
        normalize_chain_for_source("gecko", "op"),
        "optimistic-ethereum"
    );
}

#[test]
fn test_chain_normalization_for_llama() {
    use ethcli::aggregator::chain_map::normalize_chain_for_source;

    assert_eq!(normalize_chain_for_source("llama", "ethereum"), "ethereum");
    assert_eq!(normalize_chain_for_source("llama", "polygon"), "polygon");
    assert_eq!(normalize_chain_for_source("llama", "arbitrum"), "arbitrum");
    assert_eq!(normalize_chain_for_source("llama", "bsc"), "bsc");
}

#[test]
fn test_chain_normalization_for_alchemy() {
    use ethcli::aggregator::chain_map::normalize_chain_for_source;

    assert_eq!(normalize_chain_for_source("alchemy", "ethereum"), "eth-mainnet");
    assert_eq!(
        normalize_chain_for_source("alchemy", "polygon"),
        "polygon-mainnet"
    );
    assert_eq!(
        normalize_chain_for_source("alchemy", "arbitrum"),
        "arb-mainnet"
    );
    assert_eq!(
        normalize_chain_for_source("alchemy", "optimism"),
        "opt-mainnet"
    );
}

#[test]
fn test_chain_normalization_for_moralis() {
    use ethcli::aggregator::chain_map::normalize_chain_for_source;

    assert_eq!(normalize_chain_for_source("moralis", "ethereum"), "eth");
    assert_eq!(normalize_chain_for_source("moralis", "polygon"), "polygon");
    assert_eq!(normalize_chain_for_source("moralis", "bsc"), "bsc");
}

// ==================== Serialization tests ====================

#[test]
fn test_source_result_json_serialization() {
    let price = NormalizedPrice::new(1500.0).with_confidence(0.95);
    let result = SourceResult::success("gecko", price, 100);

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("gecko"));
    assert!(json.contains("1500"));
    assert!(json.contains("0.95"));

    // Deserialize back
    let parsed: SourceResult<NormalizedPrice> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.source, "gecko");
    assert_eq!(parsed.data.unwrap().usd, 1500.0);
}

#[test]
fn test_source_result_error_json_serialization() {
    let result: SourceResult<NormalizedPrice> =
        SourceResult::error("alchemy", "API rate limited", 50);

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("alchemy"));
    assert!(json.contains("API rate limited"));

    let parsed: SourceResult<NormalizedPrice> = serde_json::from_str(&json).unwrap();
    assert!(!parsed.is_success());
    assert_eq!(parsed.error.unwrap(), "API rate limited");
}

#[test]
fn test_aggregated_result_json_serialization() {
    let sources = vec![
        SourceResult::success("gecko", NormalizedPrice::new(100.0), 10),
        SourceResult::success("llama", NormalizedPrice::new(101.0), 15),
    ];
    let agg = PriceAggregation::from_prices(&[100.0, 101.0]).unwrap();
    let result = AggregatedResult::new(agg, sources, 20);

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("sources_queried"));
    assert!(json.contains("sources_succeeded"));
    assert!(json.contains("gecko"));
    assert!(json.contains("llama"));

    let parsed: AggregatedResult<NormalizedPrice, PriceAggregation> =
        serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.sources_queried, 2);
    assert_eq!(parsed.sources_succeeded, 2);
}
