//! Test API calls against Dune Analytics
//!
//! Run with: DUNE_API_KEY=your_key cargo run --example test_api -p dune

use dnapi::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DUNE_API_KEY").expect("DUNE_API_KEY required");
    let client = Client::new(&api_key)?;

    // Test 1: Get usage stats
    println!("=== Usage Stats ===");
    let usage = client.usage().get().await?;
    println!("Bytes used: {:?}", usage.bytes_used);
    println!("Private queries: {:?}", usage.private_queries);
    if let Some(period) = usage.billing_periods.first() {
        println!(
            "Credits: {:.2} / {:.2}",
            period.credits_used.unwrap_or(0.0),
            period.credits_included.unwrap_or(0.0)
        );
    }

    // Test 2: Execute a simple SQL query
    println!("\n=== Execute SQL ===");
    let result = client
        .executions()
        .run_sql("SELECT 1 as value, 'hello' as message", Some(60))
        .await?;
    println!("State: {:?}", result.state);
    if let Some(data) = &result.result {
        println!(
            "Columns: {:?}",
            data.metadata.as_ref().map(|m| &m.column_names)
        );
        for row in &data.rows {
            println!("  Row: {:?}", row);
        }
    }

    // Test 3: Get cached results from a public query
    // Query 4496 is a simple "SELECT 1" query
    println!("\n=== Cached Query Results ===");
    match client.executions().query_results(4496).await {
        Ok(result) => {
            println!("State: {:?}", result.state);
            if let Some(data) = &result.result {
                println!(
                    "Row count: {:?}",
                    data.metadata.as_ref().and_then(|m| m.row_count)
                );
            }
        }
        Err(e) => println!("Could not get cached results: {}", e),
    }

    // Test 4: List tables
    println!("\n=== Tables ===");
    let tables = client.tables().list().await?;
    println!("Found {} tables", tables.tables.len());
    for table in tables.tables.iter().take(3) {
        println!("  - {:?}", table.full_name);
    }

    // Test 5: List materialized views
    println!("\n=== Materialized Views ===");
    let matviews = client.matviews().list().await?;
    println!(
        "Found {} materialized views",
        matviews.materialized_views.len()
    );
    for mv in matviews.materialized_views.iter().take(3) {
        println!("  - {:?} (query {})", mv.id, mv.query_id.unwrap_or(0));
    }

    println!("\n=== All tests passed! ===");
    Ok(())
}
