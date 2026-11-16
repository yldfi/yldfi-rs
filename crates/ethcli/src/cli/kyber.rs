//! Direct KyberSwap API commands
//!
//! Provides 1:1 access to KyberSwap Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use kybr::{BuildRouteRequest, Client, RouteRequest, SwapRequest};

#[derive(Args, Clone)]
pub struct KyberArgs {
    #[command(subcommand)]
    pub action: KyberCommands,

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
pub enum KyberCommands {
    /// Get optimal swap routes
    Routes {
        /// Source token address
        token_in: String,
        /// Destination token address
        token_out: String,
        /// Amount in smallest units (wei)
        amount_in: String,
        /// Chain name (ethereum, bsc, polygon, arbitrum, optimism, base, sonic, etc.)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Save gas (may reduce output)
        #[arg(long)]
        save_gas: bool,
        /// Include DEX sources (comma-separated)
        #[arg(long)]
        include_dexes: Option<String>,
        /// Exclude DEX sources (comma-separated)
        #[arg(long)]
        exclude_dexes: Option<String>,
    },

    /// Get route with full data
    RouteData {
        /// Source token address
        token_in: String,
        /// Destination token address
        token_out: String,
        /// Amount in smallest units (wei)
        amount_in: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Save gas
        #[arg(long)]
        save_gas: bool,
    },

    /// Build swap transaction from route
    Build {
        /// Source token address
        token_in: String,
        /// Destination token address
        token_out: String,
        /// Amount in smallest units (wei)
        amount_in: String,
        /// Sender address
        sender: String,
        /// Recipient address
        recipient: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
        #[arg(long, default_value = "50")]
        slippage_bps: u32,
        /// Deadline in seconds from now
        #[arg(long, default_value = "1200")]
        deadline: u64,
        /// Route summary JSON from the `routes` command, or the full
        /// `route-data` output (`{"routeSummary": ...}`). Sent unmodified.
        #[arg(long)]
        route_summary: String,
    },

    /// Get a route and its swap calldata in one call (`/api/v1/swap`)
    ///
    /// KyberSwap restricts this endpoint to partners (identify with
    /// --client-id or KYBERSWAP_CLIENT_ID); otherwise use `routes` + `build`.
    Swap {
        /// Source token address
        token_in: String,
        /// Destination token address
        token_out: String,
        /// Amount in smallest units (wei)
        amount_in: String,
        /// Sender address (input tokens are taken from it)
        sender: String,
        /// Recipient address (defaults to sender)
        recipient: Option<String>,
        /// Chain name (ethereum, bsc, polygon, arbitrum, optimism, base, sonic, etc.)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
        #[arg(long, default_value = "50")]
        slippage_bps: u32,
        /// Deadline in seconds from now (API default: 20 minutes)
        #[arg(long)]
        deadline: Option<u64>,
        /// Include DEX sources (comma-separated)
        #[arg(long)]
        include_dexes: Option<String>,
        /// Exclude DEX sources (comma-separated)
        #[arg(long)]
        exclude_dexes: Option<String>,
        /// Exclude RFQ sources
        #[arg(long)]
        exclude_rfq: bool,
        /// Partner client ID sent as `x-client-id` (falls back to KYBERSWAP_CLIENT_ID)
        #[arg(long)]
        client_id: Option<String>,
    },
}

pub async fn run(args: KyberArgs, _chain: &str) -> anyhow::Result<()> {
    let client = Client::with_config(kyber_config())?;

    match args.action {
        KyberCommands::Routes {
            token_in,
            token_out,
            amount_in,
            chain,
            save_gas,
            include_dexes,
            exclude_dexes,
        } => {
            let kyber_chain = chain_name_to_kybr_chain(&chain)?;
            let mut request = RouteRequest::new(&token_in, &token_out, &amount_in);
            request.save_gas = Some(save_gas);
            if let Some(dexes) = include_dexes {
                request.include_dexs = Some(dexes);
            }
            if let Some(dexes) = exclude_dexes {
                request.exclude_dexs = Some(dexes);
            }

            // Print the raw routeSummary so it can be passed to `build` unmodified
            let route_data = client.get_route_data(kyber_chain, &request).await?;
            output_json(&route_data.route_summary, args.format)?;
        }

        KyberCommands::RouteData {
            token_in,
            token_out,
            amount_in,
            chain,
            save_gas,
        } => {
            let kyber_chain = chain_name_to_kybr_chain(&chain)?;
            let mut request = RouteRequest::new(&token_in, &token_out, &amount_in);
            request.save_gas = Some(save_gas);

            let route_data = client.get_route_data(kyber_chain, &request).await?;
            output_json(&route_data, args.format)?;
        }

        KyberCommands::Build {
            token_in,
            token_out,
            amount_in,
            sender,
            recipient,
            chain,
            slippage_bps,
            deadline,
            route_summary,
        } => {
            let kyber_chain = chain_name_to_kybr_chain(&chain)?;

            if slippage_bps > 2000 {
                anyhow::bail!(
                    "--slippage-bps {slippage_bps} is too high (KyberSwap max 2000 = 20%)"
                );
            }
            let summary = extract_route_summary(&route_summary)?;
            check_summary_matches(&summary, &token_in, &token_out, &amount_in)?;
            let deadline = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() + deadline)
                .ok();

            let request = BuildRouteRequest {
                route_summary: summary,
                sender,
                recipient,
                slippage_tolerance_bps: Some(slippage_bps),
                deadline,
                enable_permit: None,
            };

            let build_data = client.build_route(kyber_chain, &request).await?;
            output_json(&build_data, args.format)?;
        }

        KyberCommands::Swap {
            token_in,
            token_out,
            amount_in,
            sender,
            recipient,
            chain,
            slippage_bps,
            deadline,
            include_dexes,
            exclude_dexes,
            exclude_rfq,
            client_id,
        } => {
            let kyber_chain = chain_name_to_kybr_chain(&chain)?;
            if slippage_bps > 2000 {
                anyhow::bail!(
                    "--slippage-bps {slippage_bps} is too high (KyberSwap max 2000 = 20%)"
                );
            }
            let recipient = recipient.unwrap_or_else(|| sender.clone());
            let mut request = SwapRequest::new(&token_in, &token_out, &amount_in)
                .with_sender(sender)
                .with_recipient(recipient);
            request.slippage_tolerance_bps = Some(slippage_bps);
            request.deadline = deadline.and_then(|secs| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() + secs)
                    .ok()
            });
            request.included_sources = include_dexes;
            request.excluded_sources = exclude_dexes;
            if exclude_rfq {
                request.exclude_rfq_sources = Some(true);
            }

            let client_id = client_id.or_else(|| {
                std::env::var("KYBERSWAP_CLIENT_ID")
                    .ok()
                    .filter(|id| !id.is_empty())
            });
            let client = match client_id {
                Some(id) => client.with_client_id(id),
                None => client,
            };

            let swap = client.get_swap(kyber_chain, &request).await?;
            output_json(&swap, args.format)?;
        }
    }

    Ok(())
}

/// The route summary is sent as-is, so the positional token/amount
/// arguments must agree with it (they were silently ignored before)
fn check_summary_matches(
    summary: &serde_json::Value,
    token_in: &str,
    token_out: &str,
    amount_in: &str,
) -> anyhow::Result<()> {
    for (field, given) in [
        ("tokenIn", token_in),
        ("tokenOut", token_out),
        ("amountIn", amount_in),
    ] {
        if let Some(actual) = summary.get(field).and_then(|v| v.as_str()) {
            if !actual.eq_ignore_ascii_case(given.trim()) {
                anyhow::bail!(
                    "{field} in --route-summary is {actual}, but the command says {given}; \
                     rebuild the route or fix the arguments"
                );
            }
        }
    }
    Ok(())
}

/// Accept either a bare routeSummary or the full route-data output.
fn extract_route_summary(json: &str) -> anyhow::Result<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| anyhow::anyhow!("Invalid route summary JSON: {e}"))?;
    match value {
        serde_json::Value::Object(mut map) => Ok(map
            .remove("routeSummary")
            .unwrap_or(serde_json::Value::Object(map))),
        _ => anyhow::bail!("Invalid route summary JSON: expected an object"),
    }
}

/// KyberSwap config: the API gateway when `KYBERSWAP_API_KEY` is set,
/// otherwise the keyless (rate-limited) aggregator API
pub(crate) fn kyber_config() -> kybr::Config {
    match std::env::var("KYBERSWAP_API_KEY") {
        Ok(key) if !key.is_empty() => kybr::gateway_config(key),
        _ => kybr::default_config(),
    }
}

fn chain_name_to_kybr_chain(name: &str) -> anyhow::Result<kybr::Chain> {
    name.parse().map_err(|_| {
        let supported: Vec<&str> = kybr::Chain::all().iter().map(|c| c.as_str()).collect();
        anyhow::anyhow!(
            "Unsupported chain: {}. Supported: {}",
            name,
            supported.join(", ")
        )
    })
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

    #[test]
    fn build_args_must_match_the_route_summary() {
        let summary = serde_json::json!({"tokenIn": "0xAA", "tokenOut": "0xbb", "amountIn": "100"});
        assert!(check_summary_matches(&summary, "0xaa", "0xBB", "100").is_ok());
        assert!(check_summary_matches(&summary, "0xaa", "0xbb", "5").is_err());
    }

    use super::*;

    #[derive(clap::Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: KyberCommands,
    }

    fn parse(args: &[&str]) -> Result<KyberCommands, clap::Error> {
        use clap::Parser;
        TestCli::try_parse_from(std::iter::once("kyber-swap").chain(args.iter().copied()))
            .map(|c| c.command)
    }

    #[test]
    fn parses_swap() {
        match parse(&["swap", "0xin", "0xout", "100", "0xsender"]).unwrap() {
            KyberCommands::Swap {
                sender,
                recipient,
                slippage_bps,
                chain,
                ..
            } => {
                assert_eq!(sender, "0xsender");
                assert!(recipient.is_none());
                assert_eq!(slippage_bps, 50);
                assert_eq!(chain, "ethereum");
            }
            _ => panic!("expected swap"),
        }
        match parse(&[
            "swap",
            "0xin",
            "0xout",
            "100",
            "0xsender",
            "0xrecipient",
            "--chain",
            "base",
            "--slippage-bps",
            "10",
            "--client-id",
            "example-app",
            "--exclude-rfq",
        ])
        .unwrap()
        {
            KyberCommands::Swap {
                recipient,
                client_id,
                exclude_rfq,
                ..
            } => {
                assert_eq!(recipient.as_deref(), Some("0xrecipient"));
                assert_eq!(client_id.as_deref(), Some("example-app"));
                assert!(exclude_rfq);
            }
            _ => panic!("expected swap"),
        }
        assert!(parse(&["swap", "0xin", "0xout", "100"]).is_err());
    }

    #[test]
    fn extract_route_summary_accepts_both_shapes() {
        let full = r#"{"routeSummary":{"checksum":"1","routeID":"x"},"routerAddress":"0x1"}"#;
        let bare = r#"{"checksum":"1","routeID":"x"}"#;
        assert_eq!(extract_route_summary(full).unwrap()["routeID"], "x");
        assert_eq!(extract_route_summary(bare).unwrap()["checksum"], "1");
        assert!(extract_route_summary("1").is_err());
    }
}
