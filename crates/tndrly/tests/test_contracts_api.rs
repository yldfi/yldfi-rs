//! Tests for the Contracts API
//!
//! Run with: cargo test -p tndrly --test test_contracts_api -- --ignored

use tndrly::Client;

#[tokio::test]
#[ignore]
async fn test_contracts_list() {
    let client = Client::from_env().expect("Failed to create client");

    let contracts = client.contracts().list(None).await.unwrap();
    println!("Found {} contracts", contracts.len());

    for c in contracts.iter().take(3) {
        println!("  - ID: {}", c.id);
        println!("    Address: {}", c.address().unwrap_or("unknown"));
        println!("    Network: {}", c.network_id().unwrap_or("unknown"));
        println!("    Verified: {}", c.is_verified());
    }
}
