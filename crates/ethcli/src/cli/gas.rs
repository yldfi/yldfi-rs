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

        /// Use this RPC endpoint instead of the configured ones (e.g. for a
        /// chain with no configured endpoint)
        #[arg(long, visible_alias = "rpc", value_name = "URL")]
        rpc_url: Option<String>,

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
            rpc_url,
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
            // With --rpc-url the endpoint's own chain id is used
            let chain = match rpc_url {
                Some(_) => {
                    use alloy::providers::Provider;
                    let endpoint = cost_endpoints(chain, rpc_url.as_deref())?.remove(0);
                    let id = endpoint
                        .provider()
                        .get_chain_id()
                        .await
                        .map_err(|e| anyhow::anyhow!("eth_chainId failed: {e}"))?;
                    resolve_rpc_chain(chain, id)?
                }
                None => chain,
            };
            if !quiet {
                eprintln!("Estimating transaction cost on {}...", chain.display_name());
            }
            let mut cost =
                estimate_cost(chain, from, to, calldata, value, rpc_url.as_deref()).await?;
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
                    "chain_id": chain.chain_id(),
                    "native_symbol": cost.native_symbol,
                    "fees": cost,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("Estimated cost on {}", chain.display_name());
                for line in cost.lines(&cost.native_symbol) {
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
    function getOperatorFee(uint256 gasUsed) external view returns (uint256);
    function gasEstimateL1Component(address to, bool contractCreation, bytes data)
        external payable returns (uint64 gasEstimateForL1, uint256 baseFee, uint256 l1BaseFeeEstimate);
}

/// Endpoints for `gas cost`: the one given with `--rpc-url`, else the
/// configured ones for the chain
fn cost_endpoints(
    chain: Chain,
    rpc_url: Option<&str>,
) -> anyhow::Result<Vec<crate::rpc::Endpoint>> {
    Ok(match rpc_url {
        Some(url) => vec![crate::rpc::Endpoint::new(
            crate::config::EndpointConfig::new(url.to_string()),
            30,
            None,
        )?],
        None => crate::rpc::candidate_endpoints(
            chain,
            &crate::rpc::SelectionOptions::default(),
            crate::rpc::MAX_FAILOVER_ATTEMPTS,
        )?,
    })
}

/// The chain to use with `--rpc-url`: the endpoint's own `eth_chainId`.
/// `--chain` left at its default (ethereum) gives way to it; any other
/// `--chain` that disagrees is an error.
fn resolve_rpc_chain(requested: Chain, endpoint_chain_id: u64) -> anyhow::Result<Chain> {
    if requested.chain_id() == endpoint_chain_id {
        return Ok(requested);
    }
    if requested == Chain::Ethereum {
        return Ok(Chain::from_chain_id(endpoint_chain_id));
    }
    anyhow::bail!(
        "--rpc-url serves chain {endpoint_chain_id}, but --chain is {} ({})",
        requested.name(),
        requested.chain_id()
    )
}

/// The EIP-1559 transaction as the L1 fee oracle expects it.
///
/// OP-stack's `getL1Fee` takes the unsigned RLP and allows for the
/// signature itself. Scroll's takes the signed transaction (checked: on a
/// live receipt `getL1Fee(raw signed tx)` equals `l1Fee` exactly), so it gets
/// a placeholder signature of full size.
#[allow(clippy::too_many_arguments)]
fn oracle_tx_bytes(
    signed: bool,
    chain_id: u64,
    nonce: u64,
    gas: u64,
    price: U256,
    to: alloy::primitives::Address,
    value: U256,
    data: Vec<u8>,
) -> alloy::primitives::Bytes {
    use alloy::consensus::{SignableTransaction, TxEip1559};
    use alloy::eips::eip2718::Encodable2718;
    let tx = TxEip1559 {
        chain_id,
        nonce,
        gas_limit: gas,
        max_fee_per_gas: u128::try_from(price).unwrap_or(u128::MAX),
        max_priority_fee_per_gas: 1_000_000,
        to: alloy::primitives::TxKind::Call(to),
        value,
        access_list: Default::default(),
        input: data.into(),
    };
    if !signed {
        return tx.encoded_for_signing().into();
    }
    let word = U256::from_be_bytes([0x7f; 32]);
    let signature = alloy::primitives::Signature::new(word, word, false);
    alloy::consensus::TxEnvelope::from(tx.into_signed(signature))
        .encoded_2718()
        .into()
}

/// L2 gas, gas price, L1 data fee and operator fee for a transaction at the
/// latest block
async fn estimate_cost(
    chain: Chain,
    from: Option<alloy::primitives::Address>,
    to: alloy::primitives::Address,
    data: Vec<u8>,
    value: U256,
    rpc_url: Option<&str>,
) -> anyhow::Result<crate::tx::l2_fee::FeeBreakdown> {
    use crate::tx::l2_fee::{
        FeeBreakdown, FeeModel, ARB_NODE_INTERFACE, OP_GAS_PRICE_ORACLE, SCROLL_GAS_PRICE_ORACLE,
    };
    use alloy::providers::Provider;
    use alloy::rpc::types::TransactionRequest;
    use alloy::sol_types::SolCall;

    if FeeModel::known(chain.chain_id()) == Some(FeeModel::Unsupported) {
        anyhow::bail!(
            "L1 data fee estimate is not supported on chain {} (Mantle prices it in MNT \
             through a token ratio); `tx` and `rpc receipt` still read it from receipts",
            chain.chain_id()
        );
    }
    let candidates = cost_endpoints(chain, rpc_url)?;
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
            let model = crate::tx::l2_fee::detect_model(chain.chain_id(), provider).await;
            let view = |to_addr, input: Vec<u8>| {
                provider.call(
                    TransactionRequest::default()
                        .to(to_addr)
                        .input(input.into()),
                )
            };
            let mut b = match model {
                FeeModel::Unsupported => anyhow::bail!(
                    "L1 data fee estimate is not supported on chain {}",
                    chain.chain_id()
                ),
                FeeModel::L1 | FeeModel::IncludedInGas | FeeModel::Unknown => {
                    FeeBreakdown::new(model, gas, price, exec, None)
                }
                FeeModel::OpStack | FeeModel::Scroll => {
                    let nonce = match from {
                        Some(f) => provider.get_transaction_count(f).await.unwrap_or(0),
                        None => 0,
                    };
                    let rlp = oracle_tx_bytes(
                        model == FeeModel::Scroll,
                        chain.chain_id(),
                        nonce,
                        gas,
                        price,
                        to,
                        value,
                        data,
                    );
                    let oracle = if model == FeeModel::Scroll {
                        SCROLL_GAS_PRICE_ORACLE
                    } else {
                        OP_GAS_PRICE_ORACLE
                    };
                    let out = view(oracle, getL1FeeCall { data: rlp }.abi_encode())
                        .await
                        .map_err(|e| anyhow::anyhow!("L1 fee oracle getL1Fee failed: {e}"))?;
                    let l1 = getL1FeeCall::abi_decode_returns(&out)
                        .map_err(|e| anyhow::anyhow!("L1 fee oracle getL1Fee: {e}"))?;
                    let mut b = FeeBreakdown::new(model, gas, price, exec, Some(l1));
                    if model == FeeModel::OpStack {
                        // Operator fee (Isthmus and later). Chains before
                        // Isthmus have no getOperatorFee and charge none
                        let call = getOperatorFeeCall {
                            gasUsed: U256::from(gas),
                        };
                        if let Ok(out) = view(OP_GAS_PRICE_ORACLE, call.abi_encode()).await {
                            if let Ok(fee) = getOperatorFeeCall::abi_decode_returns(&out) {
                                if !fee.is_zero() {
                                    b.add_operator_fee(fee);
                                }
                            }
                        }
                    }
                    b
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
                    b
                }
            };
            b.native_symbol = crate::tx::l2_fee::gas_symbol(chain.chain_id());
            Ok(b)
        }
    })
    .await
}

#[cfg(test)]
mod cost_tests {
    use super::*;

    #[test]
    fn rpc_chain_resolution() {
        // Default --chain gives way to the endpoint
        assert_eq!(
            resolve_rpc_chain(Chain::Ethereum, 8453).unwrap(),
            Chain::Base
        );
        assert_eq!(resolve_rpc_chain(Chain::Base, 8453).unwrap(), Chain::Base);
        // An explicit, different --chain is an error
        assert!(resolve_rpc_chain(Chain::Optimism, 8453).is_err());
    }
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
