//! Direct LI.FI API commands
//!
//! Provides 1:1 access to LI.FI cross-chain DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use lfi::{
    Client, ContractCall, ContractCallsRequest, QuoteRequest, RoutesRequest, StatusRequest,
    ToAmountQuoteRequest, TransferStatusFilter, TransfersRequest,
};

#[derive(Args, Clone)]
pub struct LiFiArgs {
    #[command(subcommand)]
    pub action: LiFiCommands,

    /// Output format
    #[arg(
        long,
        short = 'o',
        visible_alias = "output",
        default_value = "json",
        global = true
    )]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum LiFiCommands {
    /// Get swap/bridge quote
    Quote {
        /// Source chain ID
        from_chain: u64,
        /// Source token address
        from_token: String,
        /// Destination chain ID
        to_chain: u64,
        /// Destination token address
        to_token: String,
        /// Amount in smallest units (wei)
        from_amount: String,
        /// Sender address
        from_address: String,
        /// Receiver address (defaults to sender)
        #[arg(long)]
        to_address: Option<String>,
        /// Slippage tolerance in percent (e.g., 0.5 = 0.5%). Converted to the
        /// decimal fraction LI.FI expects before sending.
        #[arg(long, default_value = "0.5")]
        slippage: f64,
        /// Integrator identifier
        #[arg(long)]
        integrator: Option<String>,
    },

    /// Get a quote by output amount: how much to send to receive exactly
    /// `to_amount` (`/quote/toAmount`)
    QuoteToAmount {
        /// Source chain ID
        from_chain: u64,
        /// Source token address
        from_token: String,
        /// Destination chain ID
        to_chain: u64,
        /// Destination token address
        to_token: String,
        /// Amount to receive, in the destination token's smallest units
        to_amount: String,
        /// Sender address
        from_address: String,
        /// Receiver address (defaults to sender)
        #[arg(long)]
        to_address: Option<String>,
        /// Slippage tolerance in percent (e.g., 0.5 = 0.5%). Converted to the
        /// decimal fraction LI.FI expects before sending.
        #[arg(long, default_value = "0.5")]
        slippage: f64,
        /// Integrator identifier
        #[arg(long)]
        integrator: Option<String>,
    },

    /// Quote a bridge/swap followed by contract calls on the destination
    /// chain (`/quote/contractCalls`, beta)
    ContractCalls {
        /// Source chain ID
        from_chain: u64,
        /// Source token address
        from_token: String,
        /// Destination chain ID
        to_chain: u64,
        /// Token the contract calls need on the destination chain
        to_token: String,
        /// Amount of `to_token` the calls need, in smallest units
        to_amount: String,
        /// Sender address
        from_address: String,
        /// Contract calls as a JSON array: [{"fromAmount":"...","fromTokenAddress":"0x...",
        /// "toContractAddress":"0x...","toContractCallData":"0x...","toContractGasLimit":"..."}]
        #[arg(long)]
        calls: String,
        /// Where bridged tokens go if a call fails (defaults to sender)
        #[arg(long)]
        fallback_address: Option<String>,
        /// Token the calls output, to send back to the sender
        #[arg(long)]
        outputs_token: Option<String>,
        /// Slippage tolerance in percent (e.g., 0.5 = 0.5%). Converted to the
        /// decimal fraction LI.FI expects before sending.
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// Get multiple routes
    Routes {
        /// Source chain ID
        from_chain: u64,
        /// Source token address
        from_token: String,
        /// Destination chain ID
        to_chain: u64,
        /// Destination token address
        to_token: String,
        /// Amount in smallest units (wei)
        from_amount: String,
        /// Sender address
        from_address: String,
        /// Receiver address
        #[arg(long)]
        to_address: Option<String>,
        /// Slippage tolerance in percent (e.g., 0.5 = 0.5%). Converted to the
        /// decimal fraction LI.FI expects before sending.
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// Get best route
    BestRoute {
        /// Source chain ID
        from_chain: u64,
        /// Source token address
        from_token: String,
        /// Destination chain ID
        to_chain: u64,
        /// Destination token address
        to_token: String,
        /// Amount in smallest units (wei)
        from_amount: String,
        /// Sender address
        from_address: String,
    },

    /// Get transaction status
    Status {
        /// Transaction hash
        tx_hash: String,
        /// Source chain ID
        #[arg(long)]
        from_chain: Option<u64>,
        /// Destination chain ID
        #[arg(long)]
        to_chain: Option<u64>,
        /// Bridge name
        #[arg(long)]
        bridge: Option<String>,
    },

    /// List supported chains
    Chains,

    /// Get chain details
    Chain {
        /// Chain ID
        chain_id: u64,
    },

    /// List tokens for a chain
    Tokens {
        /// Chain ID (omit for all chains)
        #[arg(long)]
        chain_id: Option<u64>,
    },

    /// List available tools (bridges/exchanges)
    Tools,

    /// List bridges
    Bridges,

    /// List exchanges
    Exchanges,

    /// Get gas prices for a chain (or all chains)
    Gas {
        /// Chain ID (if omitted, returns all chains)
        chain_id: Option<u64>,
    },

    /// Get just the transaction data from a quote (convenience wrapper)
    GetTransaction {
        /// Source chain ID
        from_chain: u64,
        /// Source token address
        from_token: String,
        /// Destination chain ID
        to_chain: u64,
        /// Destination token address
        to_token: String,
        /// Amount in smallest units (wei)
        from_amount: String,
        /// Sender address
        from_address: String,
        /// Slippage tolerance in percent (e.g., 0.5 = 0.5%). Converted to the
        /// decimal fraction LI.FI expects before sending.
        #[arg(long, default_value = "0.5")]
        slippage: f64,
    },

    /// Get updated transaction data for a route step (pass step JSON)
    StepTransaction {
        /// Step JSON (from a route response)
        step_json: String,
    },

    /// List connections (available routes) between two chains
    Connections {
        /// Source chain ID (required by LI.FI)
        #[arg(long)]
        from_chain: u64,
        /// Destination chain ID (required by LI.FI)
        #[arg(long)]
        to_chain: u64,
        /// Source token address
        #[arg(long)]
        from_token: Option<String>,
        /// Destination token address
        #[arg(long)]
        to_token: Option<String>,
    },

    /// Look up one token by address or symbol (`/token`)
    Token {
        /// Chain ID
        chain_id: u64,
        /// Token address or symbol (e.g. USDC)
        token: String,
    },

    /// Status of a LI.Fuel gas refuel (`/gas/status`)
    GasStatus {
        /// Transaction hash that started the refuel
        tx_hash: String,
    },

    /// List cross-chain transfers sent or received by a wallet
    /// (`/v2/analytics/transfers`; last 30 days unless --from-timestamp is set)
    Transfers {
        /// Sending or receiving wallet address
        wallet: String,
        /// Transfer status: all, done, pending, failed (API default: done)
        #[arg(long, value_parser = ["all", "done", "pending", "failed"])]
        status: Option<String>,
        /// Page size (API default: 10)
        #[arg(long)]
        limit: Option<u32>,
        /// Cursor from a previous response's `next`
        #[arg(long, conflicts_with = "previous")]
        next: Option<String>,
        /// Cursor from a previous response's `previous`
        #[arg(long)]
        previous: Option<String>,
        /// Oldest timestamp to include (unix seconds)
        #[arg(long)]
        from_timestamp: Option<u64>,
        /// Newest timestamp to include (unix seconds)
        #[arg(long)]
        to_timestamp: Option<u64>,
        /// Source chain ID
        #[arg(long)]
        from_chain: Option<u64>,
        /// Destination chain ID
        #[arg(long)]
        to_chain: Option<u64>,
    },
}

/// Optional LI.FI API key (`LIFI_API_KEY`); without it LI.FI rate limits by IP
pub(crate) fn lifi_api_key() -> Option<String> {
    std::env::var("LIFI_API_KEY").ok().filter(|k| !k.is_empty())
}

pub async fn run(args: LiFiArgs, _chain: &str) -> anyhow::Result<()> {
    let integrator = std::env::var("LIFI_INTEGRATOR").ok();
    let mut config = lfi::Config::new().with_optional_api_key(lifi_api_key());
    if let Some(int) = &integrator {
        config = config.with_integrator(int);
    }
    let client = Client::with_config(config)?;

    match args.action {
        LiFiCommands::Quote {
            from_chain,
            from_token,
            to_chain,
            to_token,
            from_amount,
            from_address,
            to_address,
            slippage,
            integrator: int_override,
        } => {
            let mut request = QuoteRequest::new(
                from_chain,
                to_chain,
                &from_token,
                &to_token,
                &from_amount,
                &from_address,
            );
            request.to_address = to_address;
            request.slippage = Some(percent_to_fraction(slippage)?);
            if let Some(int) = int_override.or(integrator) {
                request.integrator = Some(int);
            }

            let quote = client.get_quote(&request).await?;
            output_json(&quote, args.format)?;
        }

        LiFiCommands::QuoteToAmount {
            from_chain,
            from_token,
            to_chain,
            to_token,
            to_amount,
            from_address,
            to_address,
            slippage,
            integrator: int_override,
        } => {
            let mut request = ToAmountQuoteRequest::new(
                from_chain,
                to_chain,
                &from_token,
                &to_token,
                &to_amount,
                &from_address,
            );
            request.to_address = to_address;
            request.slippage = Some(percent_to_fraction(slippage)?);
            if let Some(int) = int_override.or(integrator) {
                request.integrator = Some(int);
            }

            let quote = client.get_quote_to_amount(&request).await?;
            output_json(&quote, args.format)?;
        }

        LiFiCommands::ContractCalls {
            from_chain,
            from_token,
            to_chain,
            to_token,
            to_amount,
            from_address,
            calls,
            fallback_address,
            outputs_token,
            slippage,
        } => {
            let calls = parse_contract_calls(&calls)?;
            let mut request = ContractCallsRequest::new(
                from_chain,
                &from_token,
                &from_address,
                to_chain,
                &to_token,
                &to_amount,
                calls,
            );
            request.to_fallback_address = fallback_address;
            request.contract_outputs_token = outputs_token;
            request.slippage = Some(percent_to_fraction(slippage)?);
            if let Some(int) = integrator {
                request.integrator = Some(int);
            }

            let quote = client.get_contract_calls_quote(&request).await?;
            output_json(&quote, args.format)?;
        }

        LiFiCommands::Routes {
            from_chain,
            from_token,
            to_chain,
            to_token,
            from_amount,
            from_address,
            to_address,
            slippage,
        } => {
            let mut request = RoutesRequest::new(
                from_chain,
                &from_token,
                &from_amount,
                &from_address,
                to_chain,
                &to_token,
            );
            request.to_address = to_address;
            let mut options = lfi::RoutesOptions::new();
            options.slippage = Some(percent_to_fraction(slippage)?);
            request.options = Some(options);

            let routes = client.get_routes(&request).await?;
            output_json(&routes, args.format)?;
        }

        LiFiCommands::BestRoute {
            from_chain,
            from_token,
            to_chain,
            to_token,
            from_amount,
            from_address,
        } => {
            let request = RoutesRequest::new(
                from_chain,
                &from_token,
                &from_amount,
                &from_address,
                to_chain,
                &to_token,
            );

            let route = client.get_best_route(&request).await?;
            output_json(&route, args.format)?;
        }

        LiFiCommands::Status {
            tx_hash,
            from_chain,
            to_chain,
            bridge,
        } => {
            let mut request = StatusRequest::new(&tx_hash);
            request.from_chain = from_chain;
            request.to_chain = to_chain;
            request.bridge = bridge;

            let status = client.get_status(&request).await?;
            output_json(&status, args.format)?;
        }

        LiFiCommands::Chains => {
            let chains = client.get_chains().await?;
            output_json(&chains, args.format)?;
        }

        LiFiCommands::Chain { chain_id } => {
            let chain = client.get_chain(chain_id).await?;
            output_json(&chain, args.format)?;
        }

        LiFiCommands::Tokens { chain_id } => {
            if let Some(id) = chain_id {
                let tokens = client.get_chain_tokens(id).await?;
                output_json(&tokens, args.format)?;
            } else {
                let tokens = client.get_all_tokens().await?;
                output_json(&tokens, args.format)?;
            }
        }

        LiFiCommands::Tools => {
            let tools = client.get_tools().await?;
            output_json(&tools, args.format)?;
        }

        LiFiCommands::Bridges => {
            let bridges = client.get_bridges().await?;
            output_json(&bridges, args.format)?;
        }

        LiFiCommands::Exchanges => {
            let exchanges = client.get_exchanges().await?;
            output_json(&exchanges, args.format)?;
        }

        LiFiCommands::Gas { chain_id } => {
            if let Some(chain_id) = chain_id {
                let gas = client.get_gas_prices(chain_id).await?;
                output_json(&gas, args.format)?;
            } else {
                let gas = client.get_all_gas_prices().await?;
                output_json(&gas, args.format)?;
            }
        }

        LiFiCommands::GetTransaction {
            from_chain,
            from_token,
            to_chain,
            to_token,
            from_amount,
            from_address,
            slippage,
        } => {
            let mut request = QuoteRequest::new(
                from_chain,
                to_chain,
                &from_token,
                &to_token,
                &from_amount,
                &from_address,
            );
            request.slippage = Some(percent_to_fraction(slippage)?);
            if let Some(int) = integrator {
                request.integrator = Some(int);
            }

            let tx = client.get_transaction(&request).await?;
            output_json(&tx, args.format)?;
        }

        LiFiCommands::StepTransaction { step_json } => {
            // Pass the step through untouched: LI.FI validates the full step
            // (e.g. token priceUSD), and a typed round-trip drops fields.
            let step: serde_json::Value = serde_json::from_str(&step_json)
                .map_err(|e| anyhow::anyhow!("Invalid step JSON: {}", e))?;
            if !step.is_object() {
                anyhow::bail!("Invalid step JSON: expected an object");
            }
            let updated_step = client.get_step_transaction_raw(&step).await?;
            output_json(&updated_step, args.format)?;
        }

        LiFiCommands::Connections {
            from_chain,
            to_chain,
            from_token,
            to_token,
        } => {
            let request = lfi::ConnectionsRequest {
                from_token,
                to_token,
                ..lfi::ConnectionsRequest::between(from_chain, to_chain)
            };
            let connections = client.get_connections(&request).await?;
            output_json(&connections, args.format)?;
        }

        LiFiCommands::Token { chain_id, token } => {
            let token = client.get_token(chain_id, &token).await?;
            output_json(&token, args.format)?;
        }

        LiFiCommands::GasStatus { tx_hash } => {
            let status = client.get_gas_status(&tx_hash).await?;
            output_json(&status, args.format)?;
        }

        LiFiCommands::Transfers {
            wallet,
            status,
            limit,
            next,
            previous,
            from_timestamp,
            to_timestamp,
            from_chain,
            to_chain,
        } => {
            let mut request =
                TransfersRequest::for_wallet(&wallet).with_time_range(from_timestamp, to_timestamp);
            if let Some(status) = status {
                request = request.with_status(parse_transfer_status(&status)?);
            }
            if let Some(chain) = from_chain {
                request = request.with_from_chain(chain);
            }
            if let Some(chain) = to_chain {
                request = request.with_to_chain(chain);
            }
            request.limit = limit;
            request.next = next;
            request.previous = previous;

            let transfers = client.get_transfers(&request).await?;
            output_json(&transfers, args.format)?;
        }
    }

    Ok(())
}

/// Maximum slippage (in percent) accepted by the CLI.
const MAX_SLIPPAGE_PERCENT: f64 = 50.0;

/// Convert a user-facing slippage percentage into LI.FI's decimal fraction.
fn percent_to_fraction(percent: f64) -> anyhow::Result<f64> {
    if percent > MAX_SLIPPAGE_PERCENT {
        anyhow::bail!(
            "--slippage {percent}% is too high (max {MAX_SLIPPAGE_PERCENT}%); \
             note --slippage is in percent, e.g. 0.5 = 0.5%"
        );
    }
    lfi::slippage_percent_to_fraction(percent).map_err(|e| anyhow::anyhow!("--slippage: {e}"))
}

fn parse_transfer_status(status: &str) -> anyhow::Result<TransferStatusFilter> {
    match status.to_lowercase().as_str() {
        "all" => Ok(TransferStatusFilter::All),
        "done" => Ok(TransferStatusFilter::Done),
        "pending" => Ok(TransferStatusFilter::Pending),
        "failed" => Ok(TransferStatusFilter::Failed),
        _ => anyhow::bail!(
            "Invalid transfer status: {}. Use: all, done, pending, failed",
            status
        ),
    }
}

/// Parse `--calls`: a JSON array of contract calls (a single object is
/// accepted as a one-call array)
fn parse_contract_calls(json: &str) -> anyhow::Result<Vec<ContractCall>> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Invalid --calls JSON: {}", e))?;
    let value = match value {
        serde_json::Value::Object(_) => serde_json::Value::Array(vec![value]),
        other => other,
    };
    let calls: Vec<ContractCall> = serde_json::from_value(value)
        .map_err(|e| anyhow::anyhow!("Invalid --calls JSON: {}", e))?;
    if calls.is_empty() {
        anyhow::bail!("--calls must contain at least one contract call");
    }
    Ok(calls)
}

fn output_json<T: serde::Serialize>(value: &T, format: OutputFormat) -> anyhow::Result<()> {
    let json = if format.is_table() {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_slippage_is_half_a_percent() {
        let f = percent_to_fraction(0.5).unwrap();
        assert!((f - 0.005).abs() < 1e-12);
    }

    #[test]
    fn slippage_out_of_range_is_rejected() {
        assert!(percent_to_fraction(0.0).is_err());
        assert!(percent_to_fraction(-0.5).is_err());
        assert!(percent_to_fraction(51.0).is_err());
        assert!((percent_to_fraction(50.0).unwrap() - 0.5).abs() < 1e-12);
    }

    #[derive(clap::Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: LiFiCommands,
    }

    fn parse(args: &[&str]) -> Result<LiFiCommands, clap::Error> {
        use clap::Parser;
        TestCli::try_parse_from(std::iter::once("lifi").chain(args.iter().copied()))
            .map(|c| c.command)
    }

    #[test]
    fn parses_token() {
        match parse(&["token", "1", "USDC"]).unwrap() {
            LiFiCommands::Token { chain_id, token } => {
                assert_eq!(chain_id, 1);
                assert_eq!(token, "USDC");
            }
            _ => panic!("expected token"),
        }
    }

    #[test]
    fn parses_quote_to_amount() {
        match parse(&[
            "quote-to-amount",
            "1",
            "USDC",
            "42161",
            "USDC",
            "10000000",
            "0xabc",
            "--slippage",
            "1",
        ])
        .unwrap()
        {
            LiFiCommands::QuoteToAmount {
                to_amount,
                slippage,
                to_chain,
                ..
            } => {
                assert_eq!(to_amount, "10000000");
                assert_eq!(to_chain, 42161);
                assert!((slippage - 1.0).abs() < 1e-12);
            }
            _ => panic!("expected quote-to-amount"),
        }
    }

    #[test]
    fn parses_transfers() {
        match parse(&[
            "transfers",
            "0xabc",
            "--status",
            "all",
            "--limit",
            "5",
            "--next",
            "cur",
        ])
        .unwrap()
        {
            LiFiCommands::Transfers {
                wallet,
                status,
                limit,
                next,
                previous,
                ..
            } => {
                assert_eq!(wallet, "0xabc");
                assert_eq!(status.as_deref(), Some("all"));
                assert_eq!(limit, Some(5));
                assert_eq!(next.as_deref(), Some("cur"));
                assert!(previous.is_none());
            }
            _ => panic!("expected transfers"),
        }
        assert!(parse(&["transfers", "0xabc", "--status", "bogus"]).is_err());
        assert!(parse(&["transfers", "0xabc", "--next", "a", "--previous", "b"]).is_err());
        assert_eq!(
            parse_transfer_status("PENDING").unwrap(),
            TransferStatusFilter::Pending
        );
    }

    #[test]
    fn parses_gas_status() {
        assert!(matches!(
            parse(&["gas-status", "0xdead"]).unwrap(),
            LiFiCommands::GasStatus { .. }
        ));
    }

    #[test]
    fn parses_contract_calls() {
        let call = r#"{"fromAmount":"1","fromTokenAddress":"0x0","toContractAddress":"0x1","toContractCallData":"0x","toContractGasLimit":"100000"}"#;
        assert!(matches!(
            parse(&[
                "contract-calls",
                "42161",
                "USDC",
                "1",
                "ETH",
                "1",
                "0xabc",
                "--calls",
                call
            ])
            .unwrap(),
            LiFiCommands::ContractCalls { .. }
        ));
        assert_eq!(parse_contract_calls(call).unwrap().len(), 1);
        assert_eq!(
            parse_contract_calls(&format!("[{call},{call}]"))
                .unwrap()
                .len(),
            2
        );
        assert!(parse_contract_calls("[]").is_err());
        assert!(parse_contract_calls(r#"[{"fromAmount":"1"}]"#).is_err());
        assert!(parse(&["contract-calls", "42161", "USDC", "1", "ETH", "1", "0xabc"]).is_err());
    }
}
