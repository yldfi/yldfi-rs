//! Direct LI.FI API commands
//!
//! Provides 1:1 access to LI.FI cross-chain DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use lfi::{Client, QuoteRequest, RoutesRequest, StatusRequest};

#[derive(Args, Clone)]
pub struct LiFiArgs {
    #[command(subcommand)]
    pub action: LiFiCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
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
        /// Slippage in percent (e.g., 0.5)
        #[arg(long, default_value = "0.5")]
        slippage: f64,
        /// Integrator identifier
        #[arg(long)]
        integrator: Option<String>,
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
        /// Slippage in percent
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

    /// List all connections (available routes)
    Connections {
        /// Source chain ID
        #[arg(long)]
        from_chain: Option<u64>,
        /// Destination chain ID
        #[arg(long)]
        to_chain: Option<u64>,
        /// Source token address
        #[arg(long)]
        from_token: Option<String>,
        /// Destination token address
        #[arg(long)]
        to_token: Option<String>,
    },
}

pub async fn run(args: LiFiArgs, _chain: &str) -> anyhow::Result<()> {
    let integrator = std::env::var("LIFI_INTEGRATOR").ok();
    let client = if let Some(int) = &integrator {
        Client::with_integrator(int)?
    } else {
        Client::new()?
    };

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
            request.slippage = Some(slippage);
            if let Some(int) = int_override.or(integrator) {
                request.integrator = Some(int);
            }

            let quote = client.get_quote(&request).await?;
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
            options.slippage = Some(slippage);
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
                match gas {
                    Some(g) => output_json(&g, args.format)?,
                    None => anyhow::bail!("No gas prices found for chain ID {chain_id}"),
                }
            } else {
                let gas = client.get_all_gas_prices().await?;
                output_json(&gas, args.format)?;
            }
        }

        LiFiCommands::Connections {
            from_chain,
            to_chain,
            from_token,
            to_token,
        } => {
            if from_chain.is_none()
                && to_chain.is_none()
                && from_token.is_none()
                && to_token.is_none()
            {
                let connections = client.get_all_connections().await?;
                output_json(&connections, args.format)?;
            } else {
                let request = lfi::ConnectionsRequest {
                    from_chain,
                    to_chain,
                    from_token,
                    to_token,
                    ..Default::default()
                };
                let connections = client.get_connections(&request).await?;
                output_json(&connections, args.format)?;
            }
        }
    }

    Ok(())
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
