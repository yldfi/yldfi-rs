//! Quote 10 WETH -> USDC across every prop AMM, then show the venues and the
//! top of each WETH/USDC book.
//!
//! Run with: `cargo run -p pamm --example quote`

const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[tokio::main]
async fn main() -> Result<(), pamm::Error> {
    let client = pamm::Client::new()?;

    let venues = client.venues().await?;
    println!("{} venues:", venues.venues.len());
    for venue in &venues.venues {
        println!(
            "  {:<10} {} ({} pairs)",
            venue.maker,
            venue.quote_address,
            venue.asset_pairs.len()
        );
    }

    let quote = client.quote(WETH, USDC, "10000000000000000000").await?;
    let maker = venues.maker_of(&quote.pamm).unwrap_or("unknown maker");
    println!(
        "\n10 WETH -> {} USDC units from {maker} ({}), block {}",
        quote.amount_out, quote.pamm, quote.block_number
    );

    let same = client
        .quote_venue(&quote.pamm, WETH, USDC, "10000000000000000000")
        .await?;
    println!("same venue asked directly: {} USDC units", same.amount_out);

    let books = client.books().await?;
    println!("\n{} books; WETH/USDC best levels:", books.books.len());
    for book in books.books.iter().filter(|b| {
        b.token1_address.eq_ignore_ascii_case(WETH) && b.token2_address.eq_ignore_ascii_case(USDC)
    }) {
        let bid = book.bids.first().map(|l| l.price);
        let ask = book.asks.first().map(|l| l.price);
        println!("  {:<10} bid {bid:?} ask {ask:?}", book.maker);
    }

    match client
        .quote(
            "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599", // WBTC
            WETH,
            "100000000",
        )
        .await
    {
        Ok(q) => println!("\n1 WBTC -> {} WETH units", q.amount_out),
        Err(e) => println!("\n1 WBTC -> WETH: {e}"),
    }

    Ok(())
}
