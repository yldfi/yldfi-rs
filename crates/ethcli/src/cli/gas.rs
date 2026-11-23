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

    /// Estimate the full cost of a transaction: L2 execution fee plus the L1
    /// data fee on rollups, in the native token and USD
    ///
    /// L2 gas comes from eth_estimateGas and the price from eth_gasPrice. The
    /// L1 data fee comes from the GasPriceOracle predeploy (`getL1Fee`) on
    /// OP-stack chains and from NodeInterface (`gasEstimateL1Component`) on
    /// Arbitrum. On L1 chains it is reported as not applicable.
    ///
    /// Examples:
    ///   ethcli gas cost 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 -s "transfer(address,uint256)" -a "0x000000000000000000000000000000000000dEaD 1" --from 0x... --chain base
    ///   ethcli gas cost 0x000000000000000000000000000000000000dEaD --value 1000000000000000 --chain optimism
    Cost {
        /// Target address
        #[arg(value_name = "TO")]
        to: String,

        /// Calldata (hex) - use this OR --sig
        #[arg(value_name = "DATA")]
        data: Option<String>,

        /// Function signature, e.g. "transfer(address,uint256)"
        #[arg(long, short, conflicts_with = "data", value_name = "SIG")]
        sig: Option<String>,

        /// Function arguments (use with --sig)
        #[arg(long, short = 'a', num_args = 1.., value_delimiter = ' ', value_name = "ARG")]
        args: Option<Vec<String>>,

        /// Sender; the estimate runs as this address (needed when the call
        /// checks balances or allowances)
        #[arg(long, value_name = "ADDRESS")]
        from: Option<String>,

        /// Value to send, in wei
        #[arg(long, default_value = "0", value_name = "WEI")]
        value: String,

        /// Skip the USD conversion
        #[arg(long)]
        no_usd: bool,

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
}

fn parse_address(s: &str) -> anyhow::Result<alloy::primitives::Address> {
    let resolved = crate::utils::address::resolve_label(s);
    resolved
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address '{s}': {e}"))
}

pub async fn handle(
    action: &GasCommands,
    chain: Chain,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    // The Etherscan gas tracker; `oracle` falls back to RPC when it is
    // unavailable (unsupported or sunset chain, plan limits, ...)
    let etherscan = || -> anyhow::Result<Client> {
        let client = Client::new(chain, api_key.clone())?;
        client.ensure_api_supported()?;
        Ok(client)
    };

    match action {
        GasCommands::Oracle { output } => {
            if !quiet {
                eprintln!("Fetching gas oracle for {}...", chain.display_name());
            }

            let oracle = match etherscan() {
                Ok(client) => client.gas_oracle().await.map_err(anyhow::Error::from),
                Err(e) => Err(e),
            };
            let estimates = match oracle {
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

        GasCommands::Cost {
            to,
            data,
            sig,
            args,
            from,
            value,
            no_usd,
            output,
        } => {
            let to = parse_address(to)?;
            let from = from.as_deref().map(parse_address).transpose()?;
            let calldata: Vec<u8> = if let Some(signature) = sig {
                let func_args = args.as_deref().unwrap_or(&[]);
                let encoded = super::cast::abi_encode(signature, func_args)?;
                hex::decode(encoded.trim_start_matches("0x"))?
            } else if let Some(d) = data {
                hex::decode(d.trim_start_matches("0x"))
                    .map_err(|e| anyhow::anyhow!("Invalid calldata: {e}"))?
            } else {
                Vec::new()
            };
            let value: U256 = value
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid --value '{value}': {e}"))?;
            if !quiet {
                eprintln!("Estimating transaction cost on {}...", chain.display_name());
            }
            let mut cost = estimate_cost(chain, from, to, calldata, value).await?;
            if !*no_usd {
                match crate::tx::l2_fee::native_usd_price(chain, None).await {
                    Ok((price, source)) => cost.set_usd(price, source),
                    Err(e) => {
                        if !quiet {
                            eprintln!("USD price not available: {e}");
                        }
                    }
                }
            }
            if output.is_json() {
                let json = serde_json::json!({
                    "chain": chain.name(),
                    "native_symbol": chain.native_symbol(),
                    "fees": cost,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("Estimated cost on {}", chain.display_name());
                for line in cost.lines(chain.native_symbol()) {
                    println!("{line}");
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
            let seconds = etherscan()?
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

alloy::sol! {
    function getL1Fee(bytes data) external view returns (uint256);
    function gasEstimateL1Component(address to, bool contractCreation, bytes data)
        external payable returns (uint64 gasEstimateForL1, uint256 baseFee, uint256 l1BaseFeeEstimate);
}

/// L2 gas, gas price and L1 data fee for a transaction at the latest block
async fn estimate_cost(
    chain: Chain,
    from: Option<alloy::primitives::Address>,
    to: alloy::primitives::Address,
    data: Vec<u8>,
    value: U256,
) -> anyhow::Result<crate::tx::l2_fee::FeeBreakdown> {
    use crate::tx::l2_fee::{FeeBreakdown, FeeModel, ARB_NODE_INTERFACE, OP_GAS_PRICE_ORACLE};
    use alloy::consensus::{SignableTransaction, TxEip1559};
    use alloy::providers::Provider;
    use alloy::rpc::types::TransactionRequest;
    use alloy::sol_types::SolCall;

    let model = FeeModel::for_chain_id(chain.chain_id());
    let candidates = crate::rpc::candidate_endpoints(
        chain,
        &crate::rpc::SelectionOptions::default(),
        crate::rpc::MAX_FAILOVER_ATTEMPTS,
    )?;
    crate::rpc::with_failover(candidates, |endpoint| {
        let data = data.clone();
        async move {
            let provider = endpoint.provider();
            let mut req = TransactionRequest::default()
                .to(to)
                .value(value)
                .input(data.clone().into());
            if let Some(from) = from {
                req = req.from(from);
            }
            let gas = provider
                .estimate_gas(req)
                .await
                .map_err(|e| anyhow::anyhow!("eth_estimateGas failed: {e}"))?;
            let price = U256::from(
                provider
                    .get_gas_price()
                    .await
                    .map_err(|e| anyhow::anyhow!("eth_gasPrice failed: {e}"))?,
            );
            let exec = U256::from(gas) * price;
            match model {
                FeeModel::L1 => Ok(FeeBreakdown::new(model, gas, price, exec, None)),
                FeeModel::OpStack => {
                    // getL1Fee takes the unsigned, RLP-encoded transaction;
                    // the oracle allows for the signature itself
                    let nonce = match from {
                        Some(f) => provider.get_transaction_count(f).await.unwrap_or(0),
                        None => 0,
                    };
                    let tx = TxEip1559 {
                        chain_id: chain.chain_id(),
                        nonce,
                        gas_limit: gas,
                        max_fee_per_gas: u128::try_from(price).unwrap_or(u128::MAX),
                        max_priority_fee_per_gas: 1_000_000,
                        to: alloy::primitives::TxKind::Call(to),
                        value,
                        access_list: Default::default(),
                        input: data.into(),
                    };
                    let call = getL1FeeCall {
                        data: tx.encoded_for_signing().into(),
                    };
                    let out = provider
                        .call(
                            TransactionRequest::default()
                                .to(OP_GAS_PRICE_ORACLE)
                                .input(call.abi_encode().into()),
                        )
                        .await
                        .map_err(|e| anyhow::anyhow!("GasPriceOracle.getL1Fee failed: {e}"))?;
                    let l1 = getL1FeeCall::abi_decode_returns(&out)
                        .map_err(|e| anyhow::anyhow!("GasPriceOracle.getL1Fee: {e}"))?;
                    Ok(FeeBreakdown::new(model, gas, price, exec, Some(l1)))
                }
                FeeModel::Arbitrum => {
                    // eth_estimateGas on Arbitrum already includes the L2 gas
                    // charged for L1 data; NodeInterface says how much
                    let call = gasEstimateL1ComponentCall {
                        to,
                        contractCreation: false,
                        data: data.into(),
                    };
                    let mut req = TransactionRequest::default()
                        .to(ARB_NODE_INTERFACE)
                        .input(call.abi_encode().into());
                    if let Some(from) = from {
                        req = req.from(from);
                    }
                    let out = provider.call(req).await.map_err(|e| {
                        anyhow::anyhow!("NodeInterface.gasEstimateL1Component failed: {e}")
                    })?;
                    let r = gasEstimateL1ComponentCall::abi_decode_returns(&out).map_err(|e| {
                        anyhow::anyhow!("NodeInterface.gasEstimateL1Component: {e}")
                    })?;
                    let l1_gas = r.gasEstimateForL1.min(gas);
                    let exec_gas = gas - l1_gas;
                    let mut b = FeeBreakdown::new(
                        model,
                        exec_gas,
                        price,
                        U256::from(exec_gas) * price,
                        Some(U256::from(l1_gas) * price),
                    );
                    b.l1_gas_used = Some(l1_gas);
                    Ok(b)
                }
            }
        }
    })
    .await
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
