use cgko::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Test 1: Ping
    println!("=== Testing ping ===");
    let ping = client.global().ping().await?;
    println!("API says: {}", ping.gecko_says);

    // Test 2: Simple price
    println!("\n=== Testing simple price ===");
    let prices = client
        .simple()
        .price(&["bitcoin", "ethereum"], &["usd"])
        .await?;
    for (coin, data) in &prices {
        if let Some(usd) = data.get("usd") {
            println!("  {}: ${}", coin, usd);
        }
    }

    // Test 3: Supported currencies
    println!("\n=== Testing supported currencies ===");
    let currencies = client.simple().supported_vs_currencies().await?;
    println!("Found {} supported currencies", currencies.len());
    println!("  First 5: {:?}", &currencies[..5.min(currencies.len())]);

    // Test 4: Trending
    println!("\n=== Testing trending ===");
    let trending = client.global().trending().await?;
    println!("Trending coins:");
    for item in trending.coins.iter().take(5) {
        println!(
            "  #{}: {} ({})",
            item.item.score.unwrap_or(0) + 1,
            item.item.name,
            item.item.symbol
        );
    }

    // Test 5: Global data
    println!("\n=== Testing global data ===");
    let global = client.global().data().await?;
    println!(
        "Active cryptocurrencies: {:?}",
        global.data.active_cryptocurrencies
    );
    println!("Markets: {:?}", global.data.markets);

    // Test 6: Coin markets
    println!("\n=== Testing coin markets ===");
    let markets = client.coins().markets("usd").await?;
    println!("Top coins by market cap:");
    for coin in markets.iter().take(5) {
        println!("  {}: ${:.2}", coin.name, coin.current_price.unwrap_or(0.0));
    }

    // Test 7: Search
    println!("\n=== Testing search ===");
    let search = client.global().search("uniswap").await?;
    println!("Search results for 'uniswap':");
    for coin in search.coins.iter().take(3) {
        println!(
            "  {} ({}) - rank #{:?}",
            coin.name, coin.symbol, coin.market_cap_rank
        );
    }

    // Test 8: Onchain networks
    println!("\n=== Testing onchain networks ===");
    let networks = client.onchain().networks().await?;
    println!("Found {} networks", networks.data.len());
    for net in networks.data.iter().take(5) {
        let name = net
            .attributes
            .as_ref()
            .and_then(|a| a.name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or(&net.id);
        println!("  {}", name);
    }

    // Test 9: Trending pools
    println!("\n=== Testing trending pools ===");
    let pools = client.onchain().trending_pools().await?;
    println!("Trending pools:");
    for pool in pools.data.iter().take(3) {
        let name = pool
            .attributes
            .as_ref()
            .and_then(|a| a.name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("?");
        let reserve = pool
            .attributes
            .as_ref()
            .and_then(|a| a.reserve_in_usd.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("0");
        println!("  {} - ${}", name, reserve);
    }

    println!("\n✅ All tests passed!");
    Ok(())
}
