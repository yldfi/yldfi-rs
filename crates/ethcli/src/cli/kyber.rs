//! Direct KyberSwap API commands
//!
//! Provides 1:1 access to KyberSwap Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use kybr::{BuildRouteRequest, Client, RouteRequest};

#[derive(Args, Clone)]
pub struct KyberArgs {
    #[command(subcommand)]
    pub action: KyberCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
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
        /// Chain name (ethereum, bsc, polygon, arbitrum, optimism, base, avalanche, fantom, linea, scroll, zksync)
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
        /// Route summary from routes command (JSON)
        #[arg(long)]
        route_summary: String,
    },
}

pub async fn run(args: KyberArgs, _chain: &str) -> anyhow::Result<()> {
    let client = Client::new()?;

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

            let routes = client.get_routes(kyber_chain, &request).await?;
            output_json(&routes, args.format)?;
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
            token_in: _,
            token_out: _,
            amount_in: _,
            sender,
            recipient,
            chain,
            slippage_bps,
            deadline: _deadline,
            route_summary,
        } => {
            let kyber_chain = chain_name_to_kybr_chain(&chain)?;

            // Parse route summary from JSON
            let summary: kybr::RouteSummary = serde_json::from_str(&route_summary)?;

            let request = BuildRouteRequest {
                route_summary: summary,
                sender,
                recipient,
                slippage_tolerance_bps: Some(slippage_bps),
                deadline: None,
                enable_permit: None,
            };

            let build_data = client.build_route(kyber_chain, &request).await?;
            output_json(&build_data, args.format)?;
        }
    }

    Ok(())
}

fn chain_name_to_kybr_chain(name: &str) -> anyhow::Result<kybr::Chain> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => Ok(kybr::Chain::Ethereum),
        "bsc" | "bnb" | "binance" => Ok(kybr::Chain::Bsc),
        "polygon" | "matic" => Ok(kybr::Chain::Polygon),
        "arbitrum" | "arb" => Ok(kybr::Chain::Arbitrum),
        "optimism" | "op" => Ok(kybr::Chain::Optimism),
        "avalanche" | "avax" => Ok(kybr::Chain::Avalanche),
        "fantom" | "ftm" => Ok(kybr::Chain::Fantom),
        "base" => Ok(kybr::Chain::Base),
        "linea" => Ok(kybr::Chain::Linea),
        "scroll" => Ok(kybr::Chain::Scroll),
        "zksync" | "zksync-era" => Ok(kybr::Chain::Zksync),
        _ => anyhow::bail!(
            "Unsupported chain: {}. Supported: ethereum, bsc, polygon, arbitrum, optimism, avalanche, fantom, base, linea, scroll, zksync",
            name
        ),
    }
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
