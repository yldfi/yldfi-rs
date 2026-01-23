//! Live API integration tests for Pyth Hermes client
//!
//! These tests hit the real Pyth Hermes API. Run with:
//! ```bash
//! cargo test -p pyth --test live_api -- --nocapture
//! ```

use pyth::{feed_ids, symbol_to_feed_id, Client, Config};

/// Test creating a mainnet client
#[tokio::test]
async fn test_client_creation() {
    let client = Client::new();
    assert!(client.is_ok(), "Failed to create mainnet client");
}

/// Test creating a testnet client
#[tokio::test]
async fn test_testnet_client_creation() {
    let client = Client::testnet();
    assert!(client.is_ok(), "Failed to create testnet client");
}

/// Test creating a client with custom config
#[tokio::test]
async fn test_custom_config_client() {
    let config = Config::mainnet().with_timeout(std::time::Duration::from_secs(60));

    let client = Client::with_config(config);
    assert!(client.is_ok(), "Failed to create client with custom config");
}

/// Test fetching a single price (ETH/USD)
#[tokio::test]
async fn test_get_latest_price_eth() {
    let client = Client::new().expect("Failed to create client");

    let result = client.get_latest_price(feed_ids::ETH_USD).await;
    assert!(
        result.is_ok(),
        "Failed to fetch ETH price: {:?}",
        result.err()
    );

    let feed = result.unwrap();
    assert!(feed.is_some(), "No ETH price data returned");

    let feed = feed.unwrap();
    assert_eq!(
        feed.id,
        feed_ids::ETH_USD.trim_start_matches("0x"),
        "Feed ID mismatch"
    );

    let price = feed.price_f64();
    assert!(price.is_some(), "Failed to convert price to f64");

    let price = price.unwrap();
    assert!(price > 0.0, "ETH price should be positive: {}", price);
    assert!(
        price < 100_000.0,
        "ETH price seems unreasonably high: {}",
        price
    );

    println!("ETH/USD: ${:.2}", price);
}

/// Test fetching BTC price
#[tokio::test]
async fn test_get_latest_price_btc() {
    let client = Client::new().expect("Failed to create client");

    let result = client.get_latest_price(feed_ids::BTC_USD).await;
    assert!(
        result.is_ok(),
        "Failed to fetch BTC price: {:?}",
        result.err()
    );

    let feed = result.unwrap().expect("No BTC price data");

    let price = feed.price_f64().expect("Failed to convert price");
    assert!(price > 0.0, "BTC price should be positive");
    assert!(price > 1000.0, "BTC price seems too low: {}", price);

    println!("BTC/USD: ${:.2}", price);
}

/// Test fetching SOL price
#[tokio::test]
async fn test_get_latest_price_sol() {
    let client = Client::new().expect("Failed to create client");

    let result = client.get_latest_price(feed_ids::SOL_USD).await;
    assert!(
        result.is_ok(),
        "Failed to fetch SOL price: {:?}",
        result.err()
    );

    let feed = result.unwrap().expect("No SOL price data");

    let price = feed.price_f64().expect("Failed to convert price");
    assert!(price > 0.0, "SOL price should be positive");

    println!("SOL/USD: ${:.2}", price);
}

/// Test fetching multiple prices at once
#[tokio::test]
async fn test_get_latest_prices_multiple() {
    let client = Client::new().expect("Failed to create client");

    let feed_ids_list = &[feed_ids::BTC_USD, feed_ids::ETH_USD, feed_ids::SOL_USD];

    let result = client.get_latest_prices(feed_ids_list).await;
    assert!(
        result.is_ok(),
        "Failed to fetch multiple prices: {:?}",
        result.err()
    );

    let feeds = result.unwrap();
    assert_eq!(
        feeds.len(),
        3,
        "Expected 3 price feeds, got {}",
        feeds.len()
    );

    for feed in &feeds {
        let price = feed.price_f64().expect("Failed to convert price");
        assert!(price > 0.0, "Price should be positive for feed {}", feed.id);
        println!("{}: ${:.2}", feed.id, price);
    }
}

/// Test fetching empty list returns empty
#[tokio::test]
async fn test_get_latest_prices_empty() {
    let client = Client::new().expect("Failed to create client");

    let result = client.get_latest_prices(&[]).await;
    assert!(result.is_ok(), "Empty request should succeed");

    let feeds = result.unwrap();
    assert!(feeds.is_empty(), "Empty request should return empty list");
}

/// Test confidence interval
#[tokio::test]
async fn test_confidence_interval() {
    let client = Client::new().expect("Failed to create client");

    let feed = client
        .get_latest_price(feed_ids::ETH_USD)
        .await
        .expect("Failed to fetch price")
        .expect("No price data");

    let confidence = feed.confidence_f64();
    assert!(confidence.is_some(), "Confidence should be available");

    let conf = confidence.unwrap();
    assert!(conf >= 0.0, "Confidence should be non-negative");
    assert!(
        conf < feed.price_f64().unwrap_or(0.0) * 0.1,
        "Confidence should be less than 10% of price"
    );

    println!(
        "ETH price: ${:.2} +/- ${:.4}",
        feed.price_f64().unwrap_or(0.0),
        conf
    );
}

/// Test stale detection
#[tokio::test]
async fn test_stale_detection() {
    let client = Client::new().expect("Failed to create client");

    let feed = client
        .get_latest_price(feed_ids::ETH_USD)
        .await
        .expect("Failed to fetch price")
        .expect("No price data");

    // Fresh data should not be stale (60 second threshold)
    assert!(
        !feed.is_stale(60),
        "Fresh price data should not be stale (60s threshold)"
    );

    // Very short threshold should mark as stale
    // Note: This might occasionally fail if the price was just updated
    // so we use a 0 second threshold which should always be stale
    assert!(
        feed.is_stale(0),
        "Price data should be stale with 0s threshold"
    );
}

/// Test getting all price feed IDs
#[tokio::test]
async fn test_get_price_feed_ids() {
    let client = Client::new().expect("Failed to create client");

    let result = client.get_price_feed_ids().await;
    assert!(result.is_ok(), "Failed to get feed IDs: {:?}", result.err());

    let feeds = result.unwrap();
    assert!(!feeds.is_empty(), "Should have at least some feeds");
    assert!(
        feeds.len() > 100,
        "Should have many feeds, got {}",
        feeds.len()
    );

    // Check that feeds have IDs
    for feed in feeds.iter().take(5) {
        assert!(!feed.id.is_empty(), "Feed ID should not be empty");
        println!("Feed: {} - {:?}", feed.id, feed.attributes.symbol);
    }

    println!("Total feeds available: {}", feeds.len());
}

/// Test searching for feeds
#[tokio::test]
async fn test_search_feeds() {
    let client = Client::new().expect("Failed to create client");

    let result = client.search_feeds("BTC").await;
    assert!(result.is_ok(), "Failed to search feeds: {:?}", result.err());

    let feeds = result.unwrap();
    assert!(!feeds.is_empty(), "Should find at least one BTC feed");

    // Should find BTC/USD at minimum
    let has_btc = feeds.iter().any(|f| {
        f.attributes
            .symbol
            .as_ref()
            .map(|s| s.contains("BTC"))
            .unwrap_or(false)
            || f.attributes
                .base
                .as_ref()
                .map(|b| b.contains("BTC"))
                .unwrap_or(false)
    });
    assert!(has_btc, "Should find a BTC-related feed");

    println!("Found {} BTC-related feeds", feeds.len());
}

/// Test filtering by asset type
#[tokio::test]
async fn test_get_feeds_by_asset_type() {
    let client = Client::new().expect("Failed to create client");

    let result = client.get_feeds_by_asset_type("crypto").await;
    assert!(
        result.is_ok(),
        "Failed to filter by asset type: {:?}",
        result.err()
    );

    let feeds = result.unwrap();
    assert!(!feeds.is_empty(), "Should find crypto feeds");
    assert!(
        feeds.len() > 50,
        "Should have many crypto feeds, got {}",
        feeds.len()
    );

    println!("Found {} crypto feeds", feeds.len());
}

/// Test symbol to feed ID mapping
#[tokio::test]
async fn test_symbol_to_feed_id() {
    // Test known mappings
    assert_eq!(symbol_to_feed_id("ETH"), Some(feed_ids::ETH_USD));
    assert_eq!(symbol_to_feed_id("ETHEREUM"), Some(feed_ids::ETH_USD));
    assert_eq!(symbol_to_feed_id("WETH"), Some(feed_ids::ETH_USD));
    assert_eq!(symbol_to_feed_id("eth"), Some(feed_ids::ETH_USD)); // case insensitive

    assert_eq!(symbol_to_feed_id("BTC"), Some(feed_ids::BTC_USD));
    assert_eq!(symbol_to_feed_id("BITCOIN"), Some(feed_ids::BTC_USD));

    assert_eq!(symbol_to_feed_id("SOL"), Some(feed_ids::SOL_USD));
    assert_eq!(symbol_to_feed_id("USDC"), Some(feed_ids::USDC_USD));
    assert_eq!(symbol_to_feed_id("USDT"), Some(feed_ids::USDT_USD));
    assert_eq!(symbol_to_feed_id("LINK"), Some(feed_ids::LINK_USD));
    assert_eq!(symbol_to_feed_id("CVX"), Some(feed_ids::CVX_USD));
    assert_eq!(symbol_to_feed_id("DOGE"), Some(feed_ids::DOGE_USD));
    assert_eq!(symbol_to_feed_id("AVAX"), Some(feed_ids::AVAX_USD));

    // Test unknown symbol
    assert_eq!(symbol_to_feed_id("UNKNOWN_TOKEN"), None);
    assert_eq!(symbol_to_feed_id(""), None);
    // MKR doesn't have a Pyth feed
    assert_eq!(symbol_to_feed_id("MKR"), None);
}

/// Test using symbol lookup with client
#[tokio::test]
async fn test_symbol_lookup_integration() {
    let client = Client::new().expect("Failed to create client");

    // Use symbol lookup to get feed ID, then fetch price
    let feed_id = symbol_to_feed_id("ETH").expect("ETH should be mapped");

    let result = client.get_latest_price(feed_id).await;
    assert!(result.is_ok(), "Failed to fetch using symbol lookup");

    let feed = result.unwrap().expect("No price data");
    let price = feed.price_f64().expect("Failed to convert price");
    assert!(price > 0.0, "Price should be positive");

    println!("ETH (via symbol lookup): ${:.2}", price);
}

/// Test feed ID normalization (with and without 0x prefix)
#[tokio::test]
async fn test_feed_id_normalization() {
    let client = Client::new().expect("Failed to create client");

    // Feed ID with 0x prefix
    let with_prefix = feed_ids::ETH_USD;

    // Feed ID without 0x prefix
    let without_prefix = feed_ids::ETH_USD.trim_start_matches("0x");

    // Both should work
    let result1 = client.get_latest_price(with_prefix).await;
    let result2 = client.get_latest_price(without_prefix).await;

    assert!(result1.is_ok(), "With prefix should work");
    assert!(result2.is_ok(), "Without prefix should work");

    let price1 = result1.unwrap().unwrap().price_f64().expect("Price 1");
    let price2 = result2.unwrap().unwrap().price_f64().expect("Price 2");

    // Prices should be very close (might differ slightly due to timing)
    let diff_pct = ((price1 - price2) / price1).abs() * 100.0;
    assert!(
        diff_pct < 1.0,
        "Prices should be within 1%: {} vs {}",
        price1,
        price2
    );
}

/// Test all known feed IDs work
#[tokio::test]
async fn test_all_known_feed_ids() {
    let client = Client::new().expect("Failed to create client");

    let known_feeds = vec![
        ("BTC", feed_ids::BTC_USD),
        ("ETH", feed_ids::ETH_USD),
        ("SOL", feed_ids::SOL_USD),
        ("USDC", feed_ids::USDC_USD),
        ("USDT", feed_ids::USDT_USD),
        ("LINK", feed_ids::LINK_USD),
        ("ARB", feed_ids::ARB_USD),
        ("OP", feed_ids::OP_USD),
        ("AAVE", feed_ids::AAVE_USD),
        ("UNI", feed_ids::UNI_USD),
        ("CRV", feed_ids::CRV_USD),
        ("CVX", feed_ids::CVX_USD),
        ("SNX", feed_ids::SNX_USD),
        ("LDO", feed_ids::LDO_USD),
        ("DAI", feed_ids::DAI_USD),
        ("DOGE", feed_ids::DOGE_USD),
        ("AVAX", feed_ids::AVAX_USD),
        ("ATOM", feed_ids::ATOM_USD),
        ("DOT", feed_ids::DOT_USD),
    ];

    let mut successes = 0;
    let mut failures = Vec::new();

    for (symbol, feed_id) in &known_feeds {
        match client.get_latest_price(*feed_id).await {
            Ok(Some(feed)) => {
                if let Some(price) = feed.price_f64() {
                    println!("{}: ${:.4}", symbol, price);
                    successes += 1;
                } else {
                    failures.push(format!("{}: failed to convert price", symbol));
                }
            }
            Ok(None) => {
                failures.push(format!("{}: no price data", symbol));
            }
            Err(e) => {
                failures.push(format!("{}: error - {}", symbol, e));
            }
        }
    }

    println!("\nResults: {}/{} succeeded", successes, known_feeds.len());

    if !failures.is_empty() {
        println!("Failures:");
        for f in &failures {
            println!("  - {}", f);
        }
    }

    // Allow some failures (feeds might be temporarily unavailable)
    assert!(
        successes >= known_feeds.len() - 2,
        "Too many failures: {:?}",
        failures
    );
}

/// Test error handling for invalid feed ID
#[tokio::test]
async fn test_invalid_feed_id() {
    let client = Client::new().expect("Failed to create client");

    // Invalid feed ID format (too short)
    let result = client.get_latest_price("0x1234").await;

    // Should either return Ok(None) or an error, but not panic
    match result {
        Ok(None) => println!("Invalid feed returned None (expected)"),
        Ok(Some(_)) => panic!("Should not return data for invalid feed"),
        Err(e) => println!("Invalid feed returned error (expected): {}", e),
    }
}

/// Test publish time is recent
#[tokio::test]
async fn test_publish_time_recent() {
    let client = Client::new().expect("Failed to create client");

    let feed = client
        .get_latest_price(feed_ids::ETH_USD)
        .await
        .expect("Failed to fetch price")
        .expect("No price data");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time error")
        .as_secs() as i64;

    let age_secs = now - feed.price.publish_time;

    assert!(age_secs >= 0, "Publish time should not be in the future");
    assert!(
        age_secs < 300,
        "Price data should be less than 5 minutes old, got {} seconds",
        age_secs
    );

    println!("Price age: {} seconds", age_secs);
}

/// Test EMA price is available
#[tokio::test]
async fn test_ema_price_available() {
    let client = Client::new().expect("Failed to create client");

    let feed = client
        .get_latest_price(feed_ids::ETH_USD)
        .await
        .expect("Failed to fetch price")
        .expect("No price data");

    assert!(feed.ema_price.is_some(), "EMA price should be available");

    let ema = feed.ema_price.as_ref().unwrap();
    let ema_price: f64 = ema.price.parse().expect("Failed to parse EMA price");
    let multiplier = 10f64.powi(ema.expo);
    let ema_usd = ema_price * multiplier;

    let spot_usd = feed.price_f64().unwrap_or(0.0);

    // EMA should be within 10% of spot
    let diff_pct = ((ema_usd - spot_usd) / spot_usd).abs() * 100.0;
    assert!(
        diff_pct < 10.0,
        "EMA should be within 10% of spot: EMA=${:.2}, Spot=${:.2}",
        ema_usd,
        spot_usd
    );

    println!("Spot: ${:.2}, EMA: ${:.2}", spot_usd, ema_usd);
}
