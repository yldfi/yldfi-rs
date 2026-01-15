//! Doctor command - diagnose configuration and connectivity issues

use crate::config::{ConfigFile, EndpointConfig};
use std::time::Duration;

/// Run diagnostic checks on the configuration
pub async fn handle(quiet: bool) -> anyhow::Result<()> {
    let mut warnings = 0;
    let mut errors = 0;

    // Check config file
    let config_path = ConfigFile::default_path();
    if config_path.exists() {
        match ConfigFile::load(&config_path) {
            Ok(config) => {
                println!("✓ Config file: {}", config_path.display());

                // Check Etherscan API key
                if config.etherscan_api_key.is_some() {
                    println!("✓ Etherscan API key: configured");
                } else {
                    println!("⚠ Etherscan API key: not set (optional, increases rate limits)");
                    warnings += 1;
                }

                // Check Tenderly
                if let Some(tenderly) = &config.tenderly {
                    println!("✓ Tenderly: configured (account: {})", tenderly.account);
                } else {
                    println!("- Tenderly: not configured (optional)");
                }

                // Check endpoints by chain
                let mut chain_endpoints: std::collections::HashMap<String, Vec<&EndpointConfig>> =
                    std::collections::HashMap::new();
                for ep in &config.endpoints {
                    if ep.enabled {
                        chain_endpoints
                            .entry(ep.chain.to_string())
                            .or_default()
                            .push(ep);
                    }
                }

                if chain_endpoints.is_empty() {
                    println!("✗ No RPC endpoints configured");
                    errors += 1;
                } else {
                    println!("\nRPC Endpoints:");
                    for (chain, endpoints) in &chain_endpoints {
                        println!("  {}: {} endpoint(s)", chain, endpoints.len());
                    }

                    // Test a few endpoints if not quiet
                    if !quiet {
                        println!("\nTesting endpoints...");
                        let client = reqwest::Client::builder()
                            .timeout(Duration::from_secs(5))
                            .build()?;

                        // Test first endpoint of each chain
                        for (chain, endpoints) in &chain_endpoints {
                            if let Some(ep) = endpoints.first() {
                                let result = test_endpoint(&client, &ep.url).await;
                                match result {
                                    Ok(latency) => {
                                        println!(
                                            "  ✓ {} ({}): {}ms",
                                            chain,
                                            truncate_url(&ep.url, 40),
                                            latency
                                        );
                                    }
                                    Err(e) => {
                                        println!(
                                            "  ✗ {} ({}): {}",
                                            chain,
                                            truncate_url(&ep.url, 40),
                                            e
                                        );
                                        warnings += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "✗ Config file: {} (parse error: {})",
                    config_path.display(),
                    e
                );
                errors += 1;
            }
        }
    } else {
        println!("⚠ Config file: not found at {}", config_path.display());
        println!("  Run: ethcli config init");
        warnings += 1;
    }

    // Summary
    println!();
    if errors > 0 {
        println!("Found {} error(s) and {} warning(s)", errors, warnings);
        std::process::exit(1);
    } else if warnings > 0 {
        println!("All checks passed ({} warning(s))", warnings);
    } else {
        println!("✓ All checks passed!");
    }

    Ok(())
}

async fn test_endpoint(client: &reqwest::Client, url: &str) -> anyhow::Result<u128> {
    let start = std::time::Instant::now();

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let response = client.post(url).json(&body).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }

    let json: serde_json::Value = response.json().await?;
    if json.get("error").is_some() {
        anyhow::bail!("RPC error");
    }

    Ok(start.elapsed().as_millis())
}

fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        // Try to show the domain
        if let Some(start) = url.find("://") {
            let domain_start = start + 3;
            if let Some(end) = url[domain_start..].find('/') {
                let domain = &url[domain_start..domain_start + end];
                if domain.len() < max_len - 3 {
                    return format!("{}...", domain);
                }
            }
        }
        format!("{}...", &url[..max_len - 3])
    }
}
