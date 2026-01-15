//! Live API tests for mrls
//!
//! Run with: MORALIS_API_KEY=your_key cargo test -p mrls --test live_api -- --ignored

use mrls::Client;

/// Vitalik's address for testing
const VITALIK: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
/// WETH address
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
/// BAYC contract
const BAYC: &str = "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D";

// =============================================================================
// Wallet API Tests
// =============================================================================

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
async fn test_get_wallet_stats() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let stats = client
        .wallet()
        .get_stats(VITALIK)
        .await
        .expect("Failed to get wallet stats");

    println!("Wallet stats for {:?}:", stats.address);
    println!("  NFTs owned: {:?}", stats.nfts_owned);
    println!("  Collections: {:?}", stats.collections_owned);
    println!("  Transactions: {:?}", stats.transactions_count);
}

// =============================================================================
// Token API Tests
// =============================================================================

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

#[tokio::test]
#[ignore]
async fn test_get_trending_tokens() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let trending = client
        .token()
        .get_trending(Some("eth"))
        .await
        .expect("Failed to get trending tokens");

    println!("Trending tokens:");
    for token in trending.iter().take(10) {
        println!(
            "  #{:?} {} ({}) - ${:?}",
            token.rank,
            token.token_name.as_deref().unwrap_or("?"),
            token.token_symbol.as_deref().unwrap_or("?"),
            token.usd_price
        );
    }
}

// =============================================================================
// NFT API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_get_nft_floor_price() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let floor = client
        .nft()
        .get_floor_price(BAYC, Some("eth"))
        .await
        .expect("Failed to get NFT floor price");

    println!("BAYC floor price: ${:?}", floor.floor_price_usd);
    println!("Marketplace: {:?}", floor.marketplace);
}

#[tokio::test]
#[ignore]
async fn test_get_collection_stats() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let stats = client
        .nft()
        .get_collection_stats(BAYC, Some("eth"))
        .await
        .expect("Failed to get collection stats");

    println!("BAYC stats:");
    println!("  Total tokens: {:?}", stats.total_tokens);
    println!("  Owners: {:?}", stats.owners);
    println!("  Floor price USD: ${:?}", stats.floor_price_usd);
    println!("  Market cap USD: ${:?}", stats.market_cap_usd);
}

#[tokio::test]
#[ignore]
async fn test_get_wallet_nfts() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let nfts = client
        .nft()
        .get_wallet_nfts(VITALIK, None)
        .await
        .expect("Failed to get wallet NFTs");

    println!("Vitalik's NFTs: {} found", nfts.result.len());
    for nft in nfts.result.iter().take(5) {
        println!(
            "  {} #{:?} - {:?}",
            nft.name.as_deref().unwrap_or("Unknown"),
            nft.token_id,
            nft.token_address
        );
    }
}

// =============================================================================
// Block API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_get_latest_block() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let latest = client
        .block()
        .get_latest_block_number("eth")
        .await
        .expect("Failed to get latest block");

    println!("Latest Ethereum block: {:?}", latest.block);
    assert!(latest.block.is_some());
}

#[tokio::test]
#[ignore]
async fn test_get_block() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let block = client
        .block()
        .get_block("18000000", None)
        .await
        .expect("Failed to get block");

    println!("Block 18000000:");
    println!("  Hash: {:?}", block.hash);
    println!("  Timestamp: {:?}", block.timestamp);
    println!("  Tx count: {:?}", block.transaction_count);
}

// =============================================================================
// DeFi API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_get_defi_summary() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let summary = client
        .defi()
        .get_wallet_defi_summary(VITALIK)
        .await
        .expect("Failed to get DeFi summary");

    println!("Vitalik's DeFi summary:");
    println!("  Total USD value: ${:?}", summary.total_usd_value);
    println!("  Active protocols: {:?}", summary.active_protocols);
    if let Some(protocols) = &summary.protocols {
        for p in protocols.iter().take(5) {
            println!(
                "    {} - ${:?}",
                p.protocol_name.as_deref().unwrap_or("?"),
                p.total_usd_value
            );
        }
    }
}

// =============================================================================
// Resolve API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_resolve_ens() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let resolved = client
        .resolve()
        .resolve_domain("vitalik.eth")
        .await
        .expect("Failed to resolve ENS");

    println!("vitalik.eth resolves to: {:?}", resolved.address);
    assert!(resolved.address.is_some());
}

#[tokio::test]
#[ignore]
async fn test_reverse_resolve() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let resolved = client
        .resolve()
        .reverse_resolve(VITALIK)
        .await
        .expect("Failed to reverse resolve");

    println!("{} resolves to: {:?}", VITALIK, resolved.name);
}

// =============================================================================
// Market Data API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_get_top_tokens() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let tokens = client
        .market()
        .get_top_tokens(None)
        .await
        .expect("Failed to get top tokens");

    println!("Top tokens by market cap:");
    for token in tokens.iter().take(10) {
        println!(
            "  {} ({}) - ${:?} (mcap: ${:?})",
            token.token_name.as_deref().unwrap_or("?"),
            token.token_symbol.as_deref().unwrap_or("?"),
            token.price_usd,
            token.market_cap_usd
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_global_market_cap() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let market_cap = client
        .market()
        .get_global_market_cap()
        .await
        .expect("Failed to get global market cap");

    println!(
        "Global crypto market cap: ${:?}",
        market_cap.total_market_cap_usd
    );
    println!("24h change: {:?}%", market_cap.market_cap_change_24h);
}

// =============================================================================
// Discovery API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_get_blue_chip_tokens() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let tokens = client
        .discovery()
        .get_blue_chip(None)
        .await
        .expect("Failed to get blue chip tokens");

    println!("Blue chip tokens: {} found", tokens.result.len());
    for token in tokens.result.iter().take(5) {
        println!(
            "  {} ({}) - ${:?}",
            token.token_name.as_deref().unwrap_or("?"),
            token.token_symbol.as_deref().unwrap_or("?"),
            token.price_usd
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_top_gainers() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let tokens = client
        .discovery()
        .get_top_gainers(None)
        .await
        .expect("Failed to get top gainers");

    println!("Top gainers: {} found", tokens.result.len());
    for token in tokens.result.iter().take(5) {
        println!(
            "  {} ({}) - ${:?} (+{:?}%)",
            token.token_name.as_deref().unwrap_or("?"),
            token.token_symbol.as_deref().unwrap_or("?"),
            token.price_usd,
            token.price_change_24h
        );
    }
}

// =============================================================================
// Entities API Tests
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_get_entity_categories() {
    let client = Client::from_env().expect("MORALIS_API_KEY must be set");
    let categories = client
        .entities()
        .get_categories()
        .await
        .expect("Failed to get entity categories");

    println!("Entity categories:");
    for cat in &categories.result {
        println!(
            "  {} - {} ({:?} entities)",
            cat.id.as_deref().unwrap_or("?"),
            cat.name.as_deref().unwrap_or("?"),
            cat.entity_count
        );
    }
}
