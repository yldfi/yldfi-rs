//! Direct Enso Finance API commands
//!
//! Provides 1:1 access to Enso Finance DeFi Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use ensof::{Client, RouteRequest};

#[derive(Args, Clone)]
pub struct EnsoArgs {
    #[command(subcommand)]
    pub action: EnsoCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum EnsoCommands {
    /// Get swap/DeFi route
    Route {
        /// Source token address
        token_in: String,
        /// Destination token address
        token_out: String,
        /// Amount in smallest units (wei)
        amount_in: String,
        /// Sender address
        from_address: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
        /// Slippage in basis points (e.g., 50 = 0.5%)
        #[arg(long, default_value = "50")]
        slippage: u16,
        /// Routing strategy (router, delegate, ensowallet)
        #[arg(long)]
        routing_strategy: Option<String>,
    },

    /// Get token price in USD
    Price {
        /// Token address
        token: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
    },

    /// Get token balances for an address
    Balances {
        /// Wallet address
        address: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
    },
}

pub async fn run(args: EnsoArgs, _chain: &str) -> anyhow::Result<()> {
    let api_key = std::env::var("ENSO_API_KEY").ok();

    let client = if let Some(key) = api_key {
        Client::with_api_key(&key)?
    } else {
        anyhow::bail!(
            "Enso API key required. Set ENSO_API_KEY environment variable.\n\
             Get an API key at: https://enso.finance"
        );
    };

    match args.action {
        EnsoCommands::Route {
            token_in,
            token_out,
            amount_in,
            from_address,
            chain_id,
            slippage,
            routing_strategy,
        } => {
            let mut request = RouteRequest::new(
                chain_id,
                &from_address,
                &token_in,
                &token_out,
                &amount_in,
                slippage,
            );
            if let Some(strategy) = routing_strategy {
                request.routing_strategy = Some(parse_routing_strategy(&strategy)?);
            }

            let route = client.get_route(&request).await?;
            output_json(&route, args.format)?;
        }

        EnsoCommands::Price { token, chain_id } => {
            let price = client.get_token_price(chain_id, &token).await?;
            output_json(&price, args.format)?;
        }

        EnsoCommands::Balances { address, chain_id } => {
            let balances = client.get_balances(chain_id, &address).await?;
            output_json(&balances, args.format)?;
        }
    }

    Ok(())
}

fn parse_routing_strategy(s: &str) -> anyhow::Result<ensof::RoutingStrategy> {
    match s.to_lowercase().as_str() {
        "router" => Ok(ensof::RoutingStrategy::Router),
        "delegate" => Ok(ensof::RoutingStrategy::Delegate),
        "ensowallet" | "enso_wallet" => Ok(ensof::RoutingStrategy::Ensowallet),
        _ => anyhow::bail!(
            "Invalid routing strategy: {}. Use: router, delegate, ensowallet",
            s
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
