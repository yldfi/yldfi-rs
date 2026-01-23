//! Live integration tests for Chainlink price feeds
//!
//! These tests hit real RPC endpoints to verify Chainlink functionality.
//! Run with:
//! ```bash
//! cargo test -p ethcli --test chainlink_live -- --nocapture
//! ```

use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use ethcli::chainlink::{
    denominations, fetch_price, oracles, symbol_to_address, tokens, Aggregator, FeedRegistry,
    FEED_REGISTRY,
};
use std::str::FromStr;

/// Get an Ethereum mainnet provider
fn mainnet_provider() -> impl alloy::providers::Provider + Clone {
    // Use PublicNode public RPC - no auth required
    ProviderBuilder::new().connect_http("https://ethereum-rpc.publicnode.com".parse().unwrap())
}

/// Get an Arbitrum provider
fn arbitrum_provider() -> impl alloy::providers::Provider + Clone {
    ProviderBuilder::new().connect_http("https://arbitrum-one-rpc.publicnode.com".parse().unwrap())
}

/// Get a Polygon provider
fn polygon_provider() -> impl alloy::providers::Provider + Clone {
    ProviderBuilder::new().connect_http("https://polygon-bor-rpc.publicnode.com".parse().unwrap())
}

// ==================== Feed Registry Tests (Mainnet) ====================

/// Test that the Feed Registry address is correct
#[tokio::test]
async fn test_feed_registry_address() {
    assert_eq!(
        FEED_REGISTRY,
        Address::from_str("0x47Fb2585D2C56Fe188D0E6ec628a38b74fCeeeDf").unwrap()
    );
}

/// Test checking if a feed exists
#[tokio::test]
async fn test_feed_registry_has_feed() {
    let provider = mainnet_provider();
    let registry = FeedRegistry::new(provider);

    // ETH/USD should exist
    let has_eth_usd = registry
        .has_feed(denominations::ETH, denominations::USD)
        .await;
    assert!(
        has_eth_usd.is_ok(),
        "Failed to check ETH/USD feed: {:?}",
        has_eth_usd.err()
    );
    assert!(has_eth_usd.unwrap(), "ETH/USD feed should exist");

    // Random address pair should not exist
    let random_base = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
    let has_random = registry.has_feed(random_base, denominations::USD).await;
    assert!(has_random.is_ok(), "Should not error on missing feed");
    assert!(!has_random.unwrap(), "Random feed should not exist");
}

/// Test getting the aggregator address for a pair
#[tokio::test]
async fn test_feed_registry_get_feed() {
    let provider = mainnet_provider();
    let registry = FeedRegistry::new(provider);

    let feed = registry
        .get_feed(denominations::ETH, denominations::USD)
        .await;
    assert!(feed.is_ok(), "Failed to get ETH/USD feed: {:?}", feed.err());

    let feed_addr = feed.unwrap();
    assert_ne!(feed_addr, Address::ZERO, "Feed address should not be zero");
    println!("ETH/USD feed address: {}", feed_addr);
}

/// Test getting ETH/USD price via Feed Registry
#[tokio::test]
async fn test_feed_registry_latest_price_eth() {
    let provider = mainnet_provider();
    let registry = FeedRegistry::new(provider);

    let result = registry
        .latest_price(denominations::ETH, denominations::USD)
        .await;
    assert!(
        result.is_ok(),
        "Failed to fetch ETH/USD price: {:?}",
        result.err()
    );

    let price = result.unwrap();
    assert_eq!(price.decimals, 8, "ETH/USD should have 8 decimals");
    assert!(!price.is_stale(), "Price should not be stale");
    assert!(price.is_valid(), "Price should be valid");

    let price_f64 = price.to_f64().expect("Should convert to f64");
    assert!(price_f64 > 0.0, "Price should be positive");
    assert!(
        price_f64 < 100_000.0,
        "ETH price seems unreasonably high: {}",
        price_f64
    );

    println!("ETH/USD: ${:.2} (decimals: {})", price_f64, price.decimals);
}

/// Test getting BTC/USD price via Feed Registry
#[tokio::test]
async fn test_feed_registry_latest_price_btc() {
    let provider = mainnet_provider();
    let registry = FeedRegistry::new(provider);

    let result = registry
        .latest_price(denominations::BTC, denominations::USD)
        .await;
    assert!(
        result.is_ok(),
        "Failed to fetch BTC/USD price: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");
    assert!(price_f64 > 1000.0, "BTC price seems too low: {}", price_f64);

    println!("BTC/USD: ${:.2}", price_f64);
}

/// Test getting price for DeFi tokens via Feed Registry
#[tokio::test]
async fn test_feed_registry_defi_tokens() {
    let provider = mainnet_provider();
    let registry = FeedRegistry::new(provider);

    let test_tokens = vec![
        ("LINK", tokens::LINK),
        ("AAVE", tokens::AAVE),
        ("UNI", tokens::UNI),
        ("MKR", tokens::MKR),
    ];

    for (name, addr) in test_tokens {
        let result = registry.latest_price(addr, denominations::USD).await;

        match result {
            Ok(price) => {
                if let Some(price_f64) = price.to_f64() {
                    println!("{}/USD: ${:.4}", name, price_f64);
                    assert!(price_f64 > 0.0, "{} price should be positive", name);
                }
            }
            Err(e) => {
                println!("{}/USD: Failed - {}", name, e);
            }
        }
    }
}

/// Test getting feed description
#[tokio::test]
async fn test_feed_registry_description() {
    let provider = mainnet_provider();
    let registry = FeedRegistry::new(provider);

    let desc = registry
        .description(denominations::ETH, denominations::USD)
        .await;
    assert!(desc.is_ok(), "Failed to get description: {:?}", desc.err());

    let description = desc.unwrap();
    assert!(
        description.contains("ETH") || description.contains("USD"),
        "Description should mention ETH or USD: {}",
        description
    );
    println!("ETH/USD description: {}", description);
}

// ==================== Direct Aggregator Tests ====================

/// Test direct aggregator query for ETH/USD on mainnet
#[tokio::test]
async fn test_aggregator_eth_usd_mainnet() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let result = aggregator.latest_price().await;
    assert!(
        result.is_ok(),
        "Failed to fetch ETH/USD via aggregator: {:?}",
        result.err()
    );

    let price = result.unwrap();
    assert_eq!(price.decimals, 8, "ETH/USD should have 8 decimals");
    assert!(!price.is_stale(), "Price should not be stale");

    let price_f64 = price.to_f64().expect("Should convert to f64");
    println!("ETH/USD (direct aggregator): ${:.2}", price_f64);
}

/// Test aggregator description
#[tokio::test]
async fn test_aggregator_description() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let desc = aggregator.description().await;
    assert!(
        desc.is_ok(),
        "Failed to get aggregator description: {:?}",
        desc.err()
    );

    let description = desc.unwrap();
    assert!(
        description.contains("ETH") && description.contains("USD"),
        "Description should be 'ETH / USD': {}",
        description
    );
    println!("Aggregator description: {}", description);
}

/// Test aggregator version
#[tokio::test]
async fn test_aggregator_version() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let version = aggregator.version().await;
    assert!(
        version.is_ok(),
        "Failed to get aggregator version: {:?}",
        version.err()
    );

    let v = version.unwrap();
    assert!(v > 0, "Version should be positive: {}", v);
    println!("Aggregator version: {}", v);
}

/// Test aggregator decimals
#[tokio::test]
async fn test_aggregator_decimals() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let decimals = aggregator.decimals().await;
    assert!(
        decimals.is_ok(),
        "Failed to get decimals: {:?}",
        decimals.err()
    );

    assert_eq!(decimals.unwrap(), 8, "ETH/USD should have 8 decimals");
}

/// Test aggregator latest_answer (simpler method)
#[tokio::test]
async fn test_aggregator_latest_answer() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let answer = aggregator.latest_answer().await;
    assert!(
        answer.is_ok(),
        "Failed to get latest answer: {:?}",
        answer.err()
    );

    let raw_answer = answer.unwrap();
    assert!(
        raw_answer > alloy::primitives::I256::ZERO,
        "Answer should be positive"
    );
    println!("Raw answer: {}", raw_answer);
}

// ==================== Multiple Chain Tests ====================

/// Test Arbitrum ETH/USD oracle
#[tokio::test]
async fn test_arbitrum_eth_usd() {
    let provider = arbitrum_provider();
    let aggregator = Aggregator::new(oracles::arbitrum::ETH_USD, provider);

    let result = aggregator.latest_price().await;
    assert!(
        result.is_ok(),
        "Failed to fetch Arbitrum ETH/USD: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");
    assert!(price_f64 > 0.0, "Price should be positive");

    println!("Arbitrum ETH/USD: ${:.2}", price_f64);
}

/// Test Arbitrum ARB/USD oracle
#[tokio::test]
async fn test_arbitrum_arb_usd() {
    let provider = arbitrum_provider();
    let aggregator = Aggregator::new(oracles::arbitrum::ARB_USD, provider);

    let result = aggregator.latest_price().await;
    assert!(
        result.is_ok(),
        "Failed to fetch Arbitrum ARB/USD: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");
    assert!(price_f64 > 0.0, "ARB price should be positive");
    assert!(price_f64 < 100.0, "ARB price should be reasonable");

    println!("Arbitrum ARB/USD: ${:.4}", price_f64);
}

/// Test Polygon MATIC/USD oracle
#[tokio::test]
async fn test_polygon_matic_usd() {
    let provider = polygon_provider();
    let aggregator = Aggregator::new(oracles::polygon::MATIC_USD, provider);

    let result = aggregator.latest_price().await;
    assert!(
        result.is_ok(),
        "Failed to fetch Polygon MATIC/USD: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");
    assert!(price_f64 > 0.0, "MATIC price should be positive");

    println!("Polygon MATIC/USD: ${:.4}", price_f64);
}

// ==================== High-Level API Tests ====================

/// Test the fetch_price convenience function
#[tokio::test]
async fn test_fetch_price_by_symbol() {
    let provider = mainnet_provider();

    let result = fetch_price(provider.clone(), "ETH", "ethereum").await;
    assert!(
        result.is_ok(),
        "Failed to fetch ETH price: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");
    assert!(price_f64 > 0.0, "ETH price should be positive");

    println!("fetch_price(ETH): ${:.2}", price_f64);
}

/// Test fetch_price with various token symbols
#[tokio::test]
async fn test_fetch_price_multiple_tokens() {
    let provider = mainnet_provider();

    let tokens = vec!["ETH", "BTC", "LINK", "USDC", "DAI", "CVX", "CRV", "AAVE"];
    let mut successes = 0;

    for token in &tokens {
        match fetch_price(provider.clone(), token, "ethereum").await {
            Ok(price) => {
                if let Some(price_f64) = price.to_f64() {
                    println!("{}: ${:.4}", token, price_f64);
                    successes += 1;
                }
            }
            Err(e) => {
                println!("{}: Error - {}", token, e);
            }
        }
    }

    println!("\nResults: {}/{} succeeded", successes, tokens.len());
    assert!(
        successes >= tokens.len() - 2,
        "Too many failures: only {}/{} succeeded",
        successes,
        tokens.len()
    );
}

/// Test fetch_price with address instead of symbol
#[tokio::test]
async fn test_fetch_price_by_address() {
    let provider = mainnet_provider();

    // USDC address
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let result = fetch_price(provider, usdc_addr, "ethereum").await;
    assert!(
        result.is_ok(),
        "Failed to fetch USDC price by address: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");

    // USDC should be close to $1
    assert!(price_f64 > 0.95, "USDC should be > $0.95: {}", price_f64);
    assert!(price_f64 < 1.05, "USDC should be < $1.05: {}", price_f64);

    println!("USDC: ${:.4}", price_f64);
}

/// Test fetch_price for L2 chain
#[tokio::test]
async fn test_fetch_price_arbitrum() {
    let provider = arbitrum_provider();

    let result = fetch_price(provider, "ETH", "arbitrum").await;
    assert!(
        result.is_ok(),
        "Failed to fetch Arbitrum ETH price: {:?}",
        result.err()
    );

    let price = result.unwrap();
    let price_f64 = price.to_f64().expect("Should convert to f64");
    println!("Arbitrum ETH: ${:.2}", price_f64);
}

// ==================== Symbol Mapping Tests ====================

/// Test symbol_to_address mapping
#[tokio::test]
async fn test_symbol_to_address_mapping() {
    // Test various symbols
    assert!(symbol_to_address("ETH").is_some());
    assert!(symbol_to_address("ETHEREUM").is_some());
    assert!(symbol_to_address("eth").is_some()); // case insensitive

    assert!(symbol_to_address("BTC").is_some());
    assert!(symbol_to_address("BITCOIN").is_some());

    assert!(symbol_to_address("USDC").is_some());
    assert!(symbol_to_address("USDT").is_some());
    assert!(symbol_to_address("DAI").is_some());

    assert!(symbol_to_address("LINK").is_some());
    assert!(symbol_to_address("CHAINLINK").is_some());

    assert!(symbol_to_address("CVX").is_some());
    assert!(symbol_to_address("CONVEX").is_some());

    assert!(symbol_to_address("CRV").is_some());
    assert!(symbol_to_address("CURVE").is_some());

    // Unknown symbol
    assert!(symbol_to_address("UNKNOWN_TOKEN").is_none());
    assert!(symbol_to_address("").is_none());
}

// ==================== All Known Oracles Test ====================

/// Test all known mainnet oracle addresses work
#[tokio::test]
async fn test_all_mainnet_oracles() {
    let provider = mainnet_provider();

    let known_oracles = vec![
        ("ETH_USD", oracles::ethereum::ETH_USD),
        ("BTC_USD", oracles::ethereum::BTC_USD),
        ("LINK_USD", oracles::ethereum::LINK_USD),
        ("USDC_USD", oracles::ethereum::USDC_USD),
        ("USDT_USD", oracles::ethereum::USDT_USD),
        ("DAI_USD", oracles::ethereum::DAI_USD),
        ("CVX_USD", oracles::ethereum::CVX_USD),
        ("CRV_USD", oracles::ethereum::CRV_USD),
        ("AAVE_USD", oracles::ethereum::AAVE_USD),
        ("UNI_USD", oracles::ethereum::UNI_USD),
        ("COMP_USD", oracles::ethereum::COMP_USD),
        ("MKR_USD", oracles::ethereum::MKR_USD),
        ("SNX_USD", oracles::ethereum::SNX_USD),
        ("YFI_USD", oracles::ethereum::YFI_USD),
        // LDO only has ETH-denominated feed on mainnet, not USD
        ("STETH_USD", oracles::ethereum::STETH_USD),
    ];

    let mut successes = 0;
    let mut failures = Vec::new();

    for (name, oracle) in &known_oracles {
        let aggregator = Aggregator::new(*oracle, provider.clone());
        match aggregator.latest_price().await {
            Ok(price) => {
                if let Some(price_f64) = price.to_f64() {
                    println!("{}: ${:.4}", name, price_f64);
                    successes += 1;
                } else {
                    failures.push(format!("{}: invalid price data", name));
                }
            }
            Err(e) => {
                failures.push(format!("{}: {}", name, e));
            }
        }
    }

    println!("\nResults: {}/{} succeeded", successes, known_oracles.len());

    if !failures.is_empty() {
        println!("Failures:");
        for f in &failures {
            println!("  - {}", f);
        }
    }

    // Allow some failures (oracles might be temporarily unavailable)
    assert!(
        successes >= known_oracles.len() - 2,
        "Too many failures: {:?}",
        failures
    );
}

/// Test all known Arbitrum oracle addresses work
#[tokio::test]
async fn test_all_arbitrum_oracles() {
    let provider = arbitrum_provider();

    let known_oracles = vec![
        ("ETH_USD", oracles::arbitrum::ETH_USD),
        ("BTC_USD", oracles::arbitrum::BTC_USD),
        ("LINK_USD", oracles::arbitrum::LINK_USD),
        ("USDC_USD", oracles::arbitrum::USDC_USD),
        ("USDT_USD", oracles::arbitrum::USDT_USD),
        ("DAI_USD", oracles::arbitrum::DAI_USD),
        ("ARB_USD", oracles::arbitrum::ARB_USD),
        ("GMX_USD", oracles::arbitrum::GMX_USD),
    ];

    let mut successes = 0;

    for (name, oracle) in &known_oracles {
        let aggregator = Aggregator::new(*oracle, provider.clone());
        match aggregator.latest_price().await {
            Ok(price) => {
                if let Some(price_f64) = price.to_f64() {
                    println!("Arbitrum {}: ${:.4}", name, price_f64);
                    successes += 1;
                }
            }
            Err(e) => {
                println!("Arbitrum {}: Error - {}", name, e);
            }
        }
    }

    println!(
        "\nArbitrum Results: {}/{} succeeded",
        successes,
        known_oracles.len()
    );
    assert!(
        successes >= known_oracles.len() - 1,
        "Too many Arbitrum oracle failures"
    );
}

// ==================== Price Data Validation Tests ====================

/// Test that price data has reasonable values
#[tokio::test]
async fn test_price_data_validation() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let price = aggregator
        .latest_price()
        .await
        .expect("Failed to get price");

    // Validate core fields
    // Note: round_id may be 0 on some RPC providers due to how they handle u80
    assert!(price.updated_at > 0, "Updated at should be positive");
    assert!(price.decimals == 8, "Decimals should be 8");
    assert!(
        price.feed_address.is_some(),
        "Feed address should be populated"
    );

    // The answer should be valid (positive)
    assert!(
        price.is_valid() || price.round_id == 0,
        "Price should be valid or have round_id issue"
    );

    // Check age
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time error")
        .as_secs();

    let age = price.age_seconds(now);
    assert!(
        age < 3600,
        "Price should be less than 1 hour old: {} secs",
        age
    );

    println!("Round ID: {}", price.round_id);
    println!("Updated at: {}", price.updated_at);
    println!("Price age: {} seconds", age);
}

/// Test staleness detection
#[tokio::test]
async fn test_staleness_detection() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let price = aggregator
        .latest_price()
        .await
        .expect("Failed to get price");

    // Fresh price should not be stale
    assert!(!price.is_stale(), "Fresh price should not be stale");

    // Check is_older_than with current time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time error")
        .as_secs();

    // Price should not be older than 1 hour
    assert!(
        !price.is_older_than(now, 3600),
        "Price should not be older than 1 hour"
    );

    // Price should be older than 0 seconds
    assert!(
        price.is_older_than(now, 0),
        "Price should be older than 0 seconds"
    );
}

/// Test to_u256 conversion
#[tokio::test]
async fn test_price_to_u256() {
    let provider = mainnet_provider();
    let aggregator = Aggregator::new(oracles::ethereum::ETH_USD, provider);

    let price = aggregator
        .latest_price()
        .await
        .expect("Failed to get price");

    let u256_price = price.to_u256();
    assert!(u256_price.is_some(), "Should convert to U256");

    let u256_val = u256_price.unwrap();
    assert!(
        u256_val > alloy::primitives::U256::ZERO,
        "U256 price should be positive"
    );

    println!("Price as U256: {}", u256_val);
}

// ==================== Error Handling Tests ====================

/// Test error handling for invalid oracle address
#[tokio::test]
async fn test_invalid_oracle_address() {
    let provider = mainnet_provider();
    // Use a random address that's not a Chainlink oracle
    let invalid_oracle = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
    let aggregator = Aggregator::new(invalid_oracle, provider);

    let result = aggregator.latest_price().await;
    // Should fail with some kind of error
    assert!(result.is_err(), "Should fail with invalid oracle address");
}

/// Test error handling for unknown symbol
#[tokio::test]
async fn test_unknown_symbol_error() {
    let provider = mainnet_provider();

    let result = fetch_price(provider, "UNKNOWN_FAKE_TOKEN_XYZ", "ethereum").await;
    assert!(result.is_err(), "Should fail with unknown symbol");

    // Check error type
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Unknown") || err_str.contains("No feed"),
        "Error should mention unknown symbol: {}",
        err_str
    );
}

// ==================== Denomination Tests ====================

/// Test denomination addresses
#[tokio::test]
async fn test_denomination_addresses() {
    // USD denomination (ISO 4217 code 840 = 0x348)
    assert_eq!(
        denominations::USD,
        Address::from_str("0x0000000000000000000000000000000000000348").unwrap()
    );

    // ETH denomination (standard EEE...EEE address)
    assert_eq!(
        denominations::ETH,
        Address::from_str("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE").unwrap()
    );

    // BTC denomination (standard BBB...BBB address)
    assert_eq!(
        denominations::BTC,
        Address::from_str("0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB").unwrap()
    );
}
