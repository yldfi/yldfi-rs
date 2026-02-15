//! Test API calls against Dune SIM

use dnsim::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DSIM_API_KEY").expect("DSIM_API_KEY required");
    let client = Client::new(&api_key)?;

    // Test 1: Get supported chains
    println!("=== Supported Chains ===");
    let chains = client.chains().list().await?;
    println!("Found {} chains:", chains.chains.len());
    for chain in chains.chains.iter().take(5) {
        println!(
            "  - {} (id: {}), tags: {:?}",
            chain.name, chain.chain_id, chain.tags
        );
    }
    if chains.chains.len() > 5 {
        println!("  ... and {} more", chains.chains.len() - 5);
    }

    // Test 2: Get balances for Vitalik's address
    println!("\n=== Balances (vitalik.eth) ===");
    let balances = client
        .balances()
        .get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .await?;
    println!("Found {} token balances", balances.balances.len());
    for balance in balances.balances.iter().take(5) {
        let value = balance.value_usd.unwrap_or(0.0);
        println!(
            "  - {} {} on {} (${:.2})",
            balance.amount, balance.symbol, balance.chain, value
        );
    }

    // Test 3: Get activity
    println!("\n=== Activity (vitalik.eth) ===");
    let mut opts = dnsim::activity::ActivityOptions::new();
    opts.limit = Some(5);
    let activity = client
        .activity()
        .get_with_options("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", &opts)
        .await?;
    println!("Found {} activities", activity.activity.len());
    for item in &activity.activity {
        println!(
            "  - {} {:?} on chain {} (tx: {})",
            item.block_time.as_deref().unwrap_or("?"),
            item.activity_type.as_deref().unwrap_or("?"),
            item.chain_id,
            item.tx_hash
                .as_deref()
                .unwrap_or("?")
                .chars()
                .take(20)
                .collect::<String>()
        );
    }

    // Test 4: Get token info
    println!("\n=== Token Info (USDC on Ethereum) ===");
    let token_opts = dnsim::tokens::TokenInfoOptions::new("1");
    let token_info = client
        .tokens()
        .get("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", &token_opts)
        .await?;
    for token in &token_info.tokens {
        println!(
            "  {} ({}) - ${:.4}",
            token.name,
            token.symbol,
            token.price_usd.unwrap_or(0.0)
        );
    }

    // Test 5: Get transactions
    println!("\n=== Recent Transactions ===");
    let mut tx_opts = dnsim::transactions::TransactionsOptions::new();
    tx_opts.limit = Some(3);
    let txs = client
        .transactions()
        .get_with_options("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", &tx_opts)
        .await?;
    println!("Found {} transactions", txs.transactions.len());
    for tx in &txs.transactions {
        println!(
            "  - {} on {} (block {})",
            tx.hash.chars().take(20).collect::<String>(),
            tx.chain,
            tx.block_number
        );
    }

    // Test 6: Get collectibles (NFTs)
    println!("\n=== Collectibles (NFTs) ===");
    let mut nft_opts = dnsim::collectibles::CollectiblesOptions::new();
    nft_opts.limit = Some(3);
    let collectibles = client
        .collectibles()
        .get_with_options("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", &nft_opts)
        .await?;
    println!("Found {} NFTs", collectibles.entries.len());
    for nft in &collectibles.entries {
        println!(
            "  - {} #{} on {} ({})",
            nft.name.as_deref().unwrap_or("Unknown"),
            nft.token_id,
            nft.chain,
            nft.token_standard
        );
    }

    println!("\n=== All tests passed! ===");
    Ok(())
}
