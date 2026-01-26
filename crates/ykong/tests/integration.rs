//! Integration tests for ykong client
//!
//! These tests verify the client can be created and configured correctly.
//! Tests marked with #[ignore] require network access to the Kong API.

use std::time::Duration;
use ykong::{Client, Config};

#[test]
fn test_client_creation() {
    let client = Client::new();
    assert!(client.is_ok(), "Client should be created successfully");
}

#[test]
fn test_client_with_config() {
    let config = Config::new()
        .with_timeout(Duration::from_secs(30))
        .with_rate_limit(10, Duration::from_secs(1));

    let client = Client::with_config(config);
    assert!(client.is_ok(), "Client with config should be created successfully");
}

#[test]
fn test_client_cloneable() {
    let client = Client::new().expect("Client creation failed");
    let cloned = client.clone();

    // Both should have the same base URL (verifies Arc sharing works)
    assert_eq!(
        std::ptr::eq(client.http(), cloned.http()),
        true,
        "Cloned clients should share the same HTTP client"
    );
}

#[test]
fn test_rate_limiter_config() {
    let config = Config::new().with_rate_limit(5, Duration::from_secs(1));
    let client = Client::with_config(config).expect("Client creation failed");

    let limiter = client.rate_limiter();
    assert!(limiter.is_some(), "Rate limiter should be configured");

    let limiter = limiter.unwrap();
    assert_eq!(limiter.max_requests(), 5);
    assert_eq!(limiter.window(), Duration::from_secs(1));
}

#[test]
fn test_no_rate_limiter_by_default() {
    let client = Client::new().expect("Client creation failed");
    assert!(
        client.rate_limiter().is_none(),
        "Rate limiter should be disabled by default"
    );
}

// Integration tests that require network access
// Run with: cargo test -p ykong --test integration -- --ignored

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_vaults() {
    let client = Client::new().expect("Client creation failed");

    // Fetch Ethereum mainnet vaults
    let vaults = client.vaults().by_chain(1).await;
    assert!(vaults.is_ok(), "Should fetch vaults successfully: {:?}", vaults.err());

    let vaults = vaults.unwrap();
    assert!(!vaults.is_empty(), "Should have at least one vault");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_rate_limited_requests() {
    // Create client with very restrictive rate limit
    let config = Config::new().with_rate_limit(2, Duration::from_secs(1));
    let client = Client::with_config(config).expect("Client creation failed");

    let start = std::time::Instant::now();

    // Make 3 requests - should take at least 1 second due to rate limiting
    for _ in 0..3 {
        let _ = client.vaults().by_chain(1).await;
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(900),
        "Rate limiting should have delayed requests, took {:?}",
        elapsed
    );
}
