//! Live API tests for mrls
//!
//! Run with: MORALIS_API_KEY=your_key cargo test -p mrls --test live_api -- --ignored

use mrls::Client;

/// Vitalik's address for testing
const VITALIK: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
/// WETH address
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

#[tokio::test]
#[ignore]
async fn test_get_native_balance() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let balance = client
        .wallet()
        .get_native_balance(VITALIK, Some("eth"))
        .await
        .expect("Failed to get native balance");

    println!("Native balance: {} wei", balance.balance);
    assert!(!balance.balance.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_get_token_balances() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let balances = client
        .wallet()
        .get_token_balances(VITALIK, None)
        .await
        .expect("Failed to get token balances");

    println!("Found {} token balances", balances.len());
    for token in balances.iter().take(5) {
        println!(
            "  {} ({}) - {}",
            token.name.as_deref().unwrap_or("Unknown"),
            token.symbol.as_deref().unwrap_or("???"),
            token.balance
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_net_worth() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let net_worth = client
        .wallet()
        .get_net_worth(VITALIK)
        .await
        .expect("Failed to get net worth");

    println!("Total net worth: ${}", net_worth.total_networth_usd);
    for chain in &net_worth.chains {
        println!(
            "  {}: ${} (native: ${}, tokens: ${})",
            chain.chain, chain.networth_usd, chain.native_balance_usd, chain.token_balance_usd
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_active_chains() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let active = client
        .wallet()
        .get_active_chains(VITALIK)
        .await
        .expect("Failed to get active chains");

    println!(
        "Active chains for {}: {}",
        active.address,
        active.active_chains.len()
    );
    for chain in &active.active_chains {
        println!("  {} (chain_id: {})", chain.chain, chain.chain_id);
    }
}

#[tokio::test]
#[ignore]
async fn test_get_token_price() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let price = client
        .token()
        .get_price(WETH, Some("eth"))
        .await
        .expect("Failed to get token price");

    println!("WETH Price: ${:?}", price.usd_price);
    println!("24h change: {:?}%", price.percent_change_24h);
    println!("Exchange: {:?}", price.exchange_name);
    assert!(price.usd_price.is_some());
}

#[tokio::test]
#[ignore]
async fn test_get_token_holders() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let holders = client
        .token()
        .get_holders(WETH, Some("eth"))
        .await
        .expect("Failed to get token holders");

    println!("Top WETH holders:");
    for holder in holders.result.iter().take(5) {
        println!(
            "  {} - {} ({:.4}%)",
            holder.owner,
            holder
                .balance_formatted
                .as_deref()
                .unwrap_or(&holder.balance),
            holder.percentage_relative_to_total_supply.unwrap_or(0.0)
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_token_pairs() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let pairs = client
        .token()
        .get_pairs(WETH, Some("eth"))
        .await
        .expect("Failed to get token pairs");

    println!("WETH trading pairs: {}", pairs.len());
    for pair in pairs.iter().take(5) {
        println!(
            "  {:?} on {:?} - ${:?} (liq: ${:?})",
            pair.pair_label, pair.exchange_name, pair.usd_price, pair.liquidity_usd
        );
    }
}
