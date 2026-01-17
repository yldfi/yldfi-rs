//! Alchemy Simulation API handlers

use super::{build_calldata, AlchemyArgs};
use alcmy::simulation::{ExecutionFormat, SimulationTransaction};

/// Simulate a transaction via Alchemy's simulateAssetChanges API
///
/// Returns a list of asset changes (transfers, approvals, etc.) that would occur
/// if the transaction were executed.
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_alchemy(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    gas: Option<u64>,
    gas_price: Option<u64>,
    alchemy: &AlchemyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client()?;

    // Build calldata from signature or raw data
    let calldata = build_calldata(sig, data, args)?;

    // Build the simulation transaction
    let tx = SimulationTransaction {
        to: to.to_string(),
        from: from.clone(),
        data: Some(calldata),
        value: if value == "0" {
            None
        } else {
            Some(format_value_hex(value)?)
        },
        gas: gas.map(|g| format!("0x{:x}", g)),
        gas_price: gas_price.map(|g| format!("0x{:x}", g)),
        ..Default::default()
    };

    if !quiet {
        eprintln!("Simulating transaction via Alchemy...");
        eprintln!("  To: {}", to);
        if let Some(ref f) = from {
            eprintln!("  From: {}", f);
        }
        if let Some(ref d) = tx.data {
            eprintln!("  Data: {}...", &d[..d.len().min(20)]);
        }
    }

    // Call the Alchemy simulation API
    let response = client
        .simulation()
        .simulate_asset_changes(&tx)
        .await
        .map_err(|e| anyhow::anyhow!("Alchemy simulation failed: {}", e))?;

    // Check for errors
    if let Some(ref err) = response.error {
        eprintln!("Simulation Error: {}", err.message);
        if let Some(ref reason) = err.revert_reason {
            eprintln!("Revert Reason: {}", reason);
        }
        return Ok(());
    }

    // Print results
    if response.changes.is_empty() {
        println!("No asset changes detected.");
    } else {
        println!("\nAsset Changes:");
        println!("{}", "=".repeat(80));

        for (i, change) in response.changes.iter().enumerate() {
            println!(
                "\n[{}] {} {}",
                i + 1,
                change.change_type.to_uppercase(),
                change.asset_type
            );
            println!("  From: {}", change.from);
            println!("  To:   {}", change.to);

            if let Some(ref amount) = change.amount {
                let symbol = change.symbol.as_deref().unwrap_or("tokens");
                println!("  Amount: {} {}", amount, symbol);
            }

            if let Some(ref token_id) = change.token_id {
                println!("  Token ID: {}", token_id);
            }

            if let Some(ref contract) = change.contract_address {
                println!("  Contract: {}", contract);
            }

            if let Some(ref name) = change.name {
                println!("  Name: {}", name);
            }
        }
    }

    if let Some(ref gas_used) = response.gas_used {
        println!("\nGas Used: {}", gas_used);
    }

    Ok(())
}

/// Simulate a transaction with full execution trace via Alchemy
#[allow(clippy::too_many_arguments)]
pub async fn simulate_execution_alchemy(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    gas: Option<u64>,
    gas_price: Option<u64>,
    block: &str,
    nested: bool,
    alchemy: &AlchemyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client()?;

    // Build calldata from signature or raw data
    let calldata = build_calldata(sig, data, args)?;

    // Build the simulation transaction
    let tx = SimulationTransaction {
        to: to.to_string(),
        from: from.clone(),
        data: Some(calldata),
        value: if value == "0" {
            None
        } else {
            Some(format_value_hex(value)?)
        },
        gas: gas.map(|g| format!("0x{:x}", g)),
        gas_price: gas_price.map(|g| format!("0x{:x}", g)),
        ..Default::default()
    };

    if !quiet {
        eprintln!("Simulating execution via Alchemy...");
    }

    let format = if nested {
        ExecutionFormat::Nested
    } else {
        ExecutionFormat::Flat
    };

    // Call the Alchemy simulation API
    let response = client
        .simulation()
        .simulate_execution(&tx, format, block)
        .await
        .map_err(|e| anyhow::anyhow!("Alchemy simulation failed: {}", e))?;

    // Pretty-print the result as JSON
    let output = serde_json::to_string_pretty(&response)?;
    println!("{}", output);

    Ok(())
}

/// Trace an existing transaction via Alchemy's debug API
pub async fn trace_tx_via_alchemy(
    hash: &str,
    alchemy: &AlchemyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client()?;

    if !quiet {
        eprintln!("Tracing transaction via Alchemy: {}", hash);
    }

    // Use the debug API to trace the transaction
    let trace = client
        .debug()
        .trace_transaction(hash)
        .await
        .map_err(|e| anyhow::anyhow!("Alchemy trace failed: {}", e))?;

    // Pretty-print the result as JSON
    let output = serde_json::to_string_pretty(&trace)?;
    println!("{}", output);

    Ok(())
}

/// Format a value string to hex
fn format_value_hex(value: &str) -> anyhow::Result<String> {
    // Handle different formats: decimal, hex, or with units
    let value = value.trim();

    if value.starts_with("0x") || value.starts_with("0X") {
        // Already hex
        Ok(value.to_string())
    } else if value.ends_with("eth") || value.ends_with("ETH") {
        // Convert ETH to wei
        let eth_str = value.trim_end_matches(|c: char| c.is_alphabetic()).trim();
        let eth: f64 = eth_str.parse()?;
        let wei = (eth * 1e18) as u128;
        Ok(format!("0x{:x}", wei))
    } else if value.ends_with("gwei") || value.ends_with("GWEI") {
        // Convert gwei to wei
        let gwei_str = value.trim_end_matches(|c: char| c.is_alphabetic()).trim();
        let gwei: f64 = gwei_str.parse()?;
        let wei = (gwei * 1e9) as u128;
        Ok(format!("0x{:x}", wei))
    } else {
        // Assume wei (decimal)
        let wei: u128 = value.parse()?;
        Ok(format!("0x{:x}", wei))
    }
}
