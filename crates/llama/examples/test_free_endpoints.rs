use llama::coins::Token;
use llama::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Test 1: TVL - Get protocols
    println!("=== Testing TVL: protocols ===");
    let protocols = client.tvl().protocols().await?;
    println!("Found {} protocols", protocols.len());
    for p in protocols.iter().take(3) {
        println!(
            "  - {}: ${:.0}M TVL",
            p.name,
            p.tvl.unwrap_or(0.0) / 1_000_000.0
        );
    }

    // Test 2: TVL - Get chains
    println!("\n=== Testing TVL: chains ===");
    let chains = client.tvl().chains().await?;
    println!("Found {} chains", chains.0.len());
    for c in chains.0.iter().take(3) {
        let name = c
            .name
            .as_deref()
            .or(c.gecko_id.as_deref())
            .unwrap_or("Unknown");
        println!("  - {}: ${:.0}B TVL", name, c.tvl / 1_000_000_000.0);
    }

    // Test 3: Coins - Current prices
    println!("\n=== Testing Coins: current prices ===");
    let tokens = vec![Token::coingecko("ethereum"), Token::coingecko("bitcoin")];
    let prices = client.coins().current(&tokens).await?;
    for (id, data) in &prices.coins {
        println!("  - {}: ${:.2}", id, data.price);
    }

    // Test 4: Stablecoins - List
    println!("\n=== Testing Stablecoins: list ===");
    let stables = client.stablecoins().list().await?;
    println!("Found {} stablecoins", stables.pegged_assets.len());
    for s in stables.pegged_assets.iter().take(3) {
        println!("  - {}: {}", s.name, s.symbol);
    }

    // Test 5: Yields - Pools (free)
    println!("\n=== Testing Yields: pools ===");
    let pools = client.yields().pools().await?;
    println!("Found {} yield pools", pools.len());
    for p in pools.iter().take(3) {
        println!(
            "  - {} on {}: {:.2}% APY",
            p.symbol,
            p.chain,
            p.apy.unwrap_or(0.0)
        );
    }

    // Test 6: Volumes - DEX overview
    println!("\n=== Testing Volumes: DEX overview ===");
    let dex = client.volumes().dex_overview().await?;
    println!(
        "24h DEX volume: ${:.0}B",
        dex.total24h.unwrap_or(0.0) / 1_000_000_000.0
    );
    println!("Top DEXes:");
    for p in dex.protocols.iter().take(3) {
        println!(
            "  - {}: ${:.0}M",
            p.name,
            p.total24h.unwrap_or(0.0) / 1_000_000.0
        );
    }

    // Test 7: Fees - Overview
    println!("\n=== Testing Fees: overview ===");
    let fees = client.fees().overview().await?;
    println!(
        "24h fees: ${:.0}M",
        fees.total24h.unwrap_or(0.0) / 1_000_000.0
    );

    // Test 8: Open Interest
    println!("\n=== Testing Volumes: open interest ===");
    let oi = client.volumes().open_interest().await?;
    println!("Found {} protocols with open interest", oi.protocols.len());
    for p in oi.protocols.iter().take(3) {
        println!(
            "  - {}: ${:.0}B OI",
            p.name.as_deref().unwrap_or("?"),
            p.total24h.unwrap_or(0.0) / 1_000_000_000.0
        );
    }

    println!("\n✅ All free endpoint tests passed!");
    Ok(())
}
