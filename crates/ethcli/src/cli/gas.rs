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
        #[arg(
            long,
            short,
            visible_alias = "format",
            value_enum,
            default_value = "table"
        )]
        output: OutputFormat,
    },

    /// Estimate confirmation time for a given gas price
    Estimate {
        /// Gas price in gwei
        #[arg(value_name = "GWEI")]
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
    // Both gas subcommands use the Etherscan gas tracker.
    client.ensure_api_supported()?;

    match action {
        GasCommands::Oracle { output } => {
            if !quiet {
                eprintln!("Fetching gas oracle for {}...", chain.display_name());
            }

            let estimates = match client.gas_oracle().await {
                Ok(oracle) => GasEstimates {
                    safe: oracle.safe_gas_price,
                    standard: oracle.propose_gas_price,
                    fast: oracle.fast_gas_price,
                    base_fee: oracle.suggested_base_fee,
                    last_block: Some(oracle.last_block),
                    gas_used_ratio: Some(oracle.gas_used_ratio),
                    source: "etherscan",
                },
                Err(e) => {
                    // e.g. Etherscan free plan: "Free API access is not
                    // supported for this chain" on non-mainnet chains.
                    if !quiet {
                        eprintln!(
                            "Etherscan gas oracle unavailable ({}); using RPC eth_feeHistory",
                            crate::utils::url::redact_urls_in_text(&e.to_string())
                        );
                    }
                    rpc_gas_estimates(chain).await.map_err(|rpc_err| {
                        anyhow::anyhow!(
                            "Failed to fetch gas oracle: {} (RPC fallback also failed: {})",
                            e,
                            rpc_err
                        )
                    })?
                }
            };

            if output.is_json() {
                let mut json = serde_json::json!({
                    "safe_gas_price": estimates.safe.to_string(),
                    "propose_gas_price": estimates.standard.to_string(),
                    "fast_gas_price": estimates.fast.to_string(),
                    "suggested_base_fee": estimates.base_fee.to_string(),
                    "last_block": estimates.last_block,
                    "source": estimates.source,
                });
                if let Some(ratio) = &estimates.gas_used_ratio {
                    json["gas_used_ratio"] = serde_json::json!(ratio);
                }
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("Gas Prices ({})", chain.display_name());
                println!("{}", "─".repeat(40));
                println!("Safe:      {} gwei", format_gwei_3(estimates.safe));
                println!("Standard:  {} gwei", format_gwei_3(estimates.standard));
                println!("Fast:      {} gwei", format_gwei_3(estimates.fast));
                println!("Base Fee:  {} gwei", format_gwei_3(estimates.base_fee));
                if estimates.source != "etherscan" {
                    println!("(source: RPC eth_feeHistory)");
                }
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

            let gas_price = U256::from(*gwei) * U256::from(1_000_000_000u64);
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

/// Gas price tiers in wei
#[derive(Debug, Clone, PartialEq)]
struct GasEstimates {
    safe: U256,
    standard: U256,
    fast: U256,
    base_fee: U256,
    last_block: Option<u64>,
    gas_used_ratio: Option<Vec<f64>>,
    source: &'static str,
}

/// Format wei as gwei with 3 decimals
fn format_gwei_3(wei: U256) -> String {
    let gwei = U256::from(1_000_000_000u64);
    let whole = wei / gwei;
    let frac = (wei % gwei) * U256::from(1000) / gwei;
    format!("{}.{:03}", whole, frac)
}

/// Reward percentiles requested from eth_feeHistory (safe / standard / fast)
const FEE_PERCENTILES: [f64; 3] = [10.0, 50.0, 90.0];

/// Derive gas tiers from an eth_feeHistory response.
///
/// Base fee is the projected next-block base fee (last entry of
/// `baseFeePerGas`); each tier adds the average priority fee at the
/// corresponding percentile across the sampled blocks.
fn estimates_from_fee_history(
    base_fees: &[u128],
    rewards: Option<&[Vec<u128>]>,
    last_block: Option<u64>,
) -> Option<GasEstimates> {
    let base_fee = *base_fees.last()?;
    let avg_tip = |idx: usize| -> u128 {
        let Some(rows) = rewards else { return 0 };
        let vals: Vec<u128> = rows.iter().filter_map(|r| r.get(idx).copied()).collect();
        if vals.is_empty() {
            0
        } else {
            vals.iter().sum::<u128>() / vals.len() as u128
        }
    };
    Some(GasEstimates {
        safe: U256::from(base_fee + avg_tip(0)),
        standard: U256::from(base_fee + avg_tip(1)),
        fast: U256::from(base_fee + avg_tip(2)),
        base_fee: U256::from(base_fee),
        last_block,
        gas_used_ratio: None,
        source: "rpc",
    })
}

/// Gas tiers from RPC (eth_feeHistory, falling back to eth_gasPrice),
/// failing over across configured endpoints.
async fn rpc_gas_estimates(chain: Chain) -> anyhow::Result<GasEstimates> {
    use alloy::eips::BlockNumberOrTag;
    use alloy::providers::Provider;

    let candidates = crate::rpc::candidate_endpoints(
        chain,
        &crate::rpc::SelectionOptions::default(),
        crate::rpc::MAX_FAILOVER_ATTEMPTS,
    )?;
    crate::rpc::with_failover(candidates, |endpoint| async move {
        let provider = endpoint.provider();
        match provider
            .get_fee_history(20, BlockNumberOrTag::Latest, &FEE_PERCENTILES)
            .await
        {
            Ok(history) => {
                let last_block =
                    (history.oldest_block + history.base_fee_per_gas.len() as u64).checked_sub(2);
                if let Some(est) = estimates_from_fee_history(
                    &history.base_fee_per_gas,
                    history.reward.as_deref(),
                    last_block,
                ) {
                    return Ok(est);
                }
            }
            Err(e) => {
                let msg = e.to_string();
                if crate::rpc::is_failover_error(&msg) {
                    return Err(anyhow::anyhow!("eth_feeHistory failed: {msg}"));
                }
                // Chain without EIP-1559 / method unsupported: use gasPrice
            }
        }
        let price = provider
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("eth_gasPrice failed: {e}"))?;
        let price = U256::from(price);
        Ok(GasEstimates {
            safe: price,
            standard: price,
            fast: price,
            base_fee: price,
            last_block: None,
            gas_used_ratio: None,
            source: "rpc",
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_history_tiers() {
        let base = [100u128, 110, 120];
        let rewards = vec![vec![1u128, 5, 9], vec![3, 7, 11]];
        let est = estimates_from_fee_history(&base, Some(&rewards), Some(42)).unwrap();
        assert_eq!(est.base_fee, U256::from(120u64));
        assert_eq!(est.safe, U256::from(122u64));
        assert_eq!(est.standard, U256::from(126u64));
        assert_eq!(est.fast, U256::from(130u64));
        assert_eq!(est.last_block, Some(42));
        assert_eq!(est.source, "rpc");
    }

    #[test]
    fn fee_history_without_rewards_or_blocks() {
        let est = estimates_from_fee_history(&[7], None, None).unwrap();
        assert_eq!(est.fast, U256::from(7u64));
        assert!(estimates_from_fee_history(&[], None, None).is_none());
    }

    #[test]
    fn gwei_formatting() {
        assert_eq!(format_gwei_3(U256::from(180_000_000u64)), "0.180");
        assert_eq!(format_gwei_3(U256::from(25_123_456_789u64)), "25.123");
    }
}
