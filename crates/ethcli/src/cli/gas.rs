//! Gas oracle and estimation commands

use super::OutputFormat;
use crate::config::Chain;
use crate::etherscan::Client;
use alloy::primitives::U256;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum GasCommands {
    /// Get current gas prices from the gas oracle
    Oracle {
        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Estimate confirmation time for a given gas price
    Estimate {
        /// Gas price in gwei
        gwei: u64,
    },
}

pub async fn handle(
    action: &GasCommands,
    chain: Chain,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = Client::new(chain, api_key)?;

    match action {
        GasCommands::Oracle { output } => {
            if !quiet {
                eprintln!("Fetching gas oracle for {}...", chain.display_name());
            }

            let oracle = client
                .gas_oracle()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch gas oracle: {}", e))?;

            if output.is_json() {
                // Manually construct JSON since GasOracle doesn't implement Serialize
                let json = serde_json::json!({
                    "safe_gas_price": oracle.safe_gas_price.to_string(),
                    "propose_gas_price": oracle.propose_gas_price.to_string(),
                    "fast_gas_price": oracle.fast_gas_price.to_string(),
                    "suggested_base_fee": oracle.suggested_base_fee.to_string(),
                    "last_block": oracle.last_block,
                    "gas_used_ratio": oracle.gas_used_ratio
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                // Gas oracle values are in Wei, convert to Gwei for display
                let gwei = U256::from(1_000_000_000u64);
                let safe_gwei = oracle.safe_gas_price / gwei;
                let standard_gwei = oracle.propose_gas_price / gwei;
                let fast_gwei = oracle.fast_gas_price / gwei;
                let base_fee_gwei = oracle.suggested_base_fee / gwei;

                // Also get fractional part for more precision
                let safe_frac = (oracle.safe_gas_price % gwei) * U256::from(1000) / gwei;
                let standard_frac = (oracle.propose_gas_price % gwei) * U256::from(1000) / gwei;
                let fast_frac = (oracle.fast_gas_price % gwei) * U256::from(1000) / gwei;
                let base_fee_frac = (oracle.suggested_base_fee % gwei) * U256::from(1000) / gwei;

                println!("Gas Prices ({})", chain.display_name());
                println!("{}", "─".repeat(40));
                println!("Safe:      {}.{:03} gwei", safe_gwei, safe_frac);
                println!("Standard:  {}.{:03} gwei", standard_gwei, standard_frac);
                println!("Fast:      {}.{:03} gwei", fast_gwei, fast_frac);
                println!("Base Fee:  {}.{:03} gwei", base_fee_gwei, base_fee_frac);
            }
        }

        GasCommands::Estimate { gwei } => {
            if !quiet {
                eprintln!(
                    "Estimating confirmation time for {} gwei on {}...",
                    gwei,
                    chain.display_name()
                );
            }

            let gas_price = U256::from(*gwei);
            let seconds = client
                .gas_estimate(gas_price)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to estimate gas: {}", e))?;

            let time_str = if seconds < 60 {
                format!("{} seconds", seconds)
            } else if seconds < 3600 {
                format!("{} minutes", seconds / 60)
            } else {
                format!("{} hours", seconds / 3600)
            };

            println!("Estimated confirmation time: {}", time_str);
        }
    }

    Ok(())
}
