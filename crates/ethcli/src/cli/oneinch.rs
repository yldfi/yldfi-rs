//! Direct 1inch API commands
//!
//! Provides 1:1 access to 1inch DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use oinch::{Client, QuoteRequest, SwapRequest};

#[derive(Args, Clone)]
pub struct OneInchArgs {
    #[command(subcommand)]
    pub action: OneInchCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum OneInchCommands {
    /// Get swap quote (price estimation without tx data)
    Quote {
        /// Source token address (use 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE for native token)
        src: String,
        /// Destination token address
        dst: String,
        /// Amount in smallest units (wei)
        amount: String,
        /// Chain ID (1=Ethereum, 56=BSC, 137=Polygon, etc.)
        #[arg(long, default_value = "1")]
        chain_id: u64,
        /// Include gas estimation
        #[arg(long)]
        include_gas: bool,
        /// Connector tokens for finding best route
        #[arg(long)]
        connector_tokens: Option<String>,
        /// Fee percentage (in basis points, e.g., 100 = 1%)
        #[arg(long)]
        fee: Option<f64>,
    },

    /// Get swap transaction data
    Swap {
        /// Source token address
        src: String,
        /// Destination token address
        dst: String,
        /// Amount in smallest units (wei)
        amount: String,
        /// Wallet address that will execute the swap
        from: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
        /// Slippage tolerance in percent (e.g., 1 = 1%)
        #[arg(long, default_value = "1")]
        slippage: f64,
        /// Recipient address (defaults to from address)
        #[arg(long)]
        receiver: Option<String>,
        /// Referrer address for fee sharing
        #[arg(long)]
        referrer: Option<String>,
        /// Disable estimate (skip simulation)
        #[arg(long)]
        disable_estimate: bool,
        /// Allow partial fill
        #[arg(long)]
        allow_partial_fill: bool,
    },

    /// Get list of supported tokens
    Tokens {
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
    },

    /// Get list of liquidity sources (DEXes)
    Sources {
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
    },

    /// Get approval spender address (router contract)
    Spender {
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
    },

    /// Check token approval allowance
    Allowance {
        /// Token address
        token: String,
        /// Wallet address
        wallet: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
    },

    /// Get approval transaction data
    Approve {
        /// Token address to approve
        token: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: u64,
        /// Amount to approve (omit for unlimited)
        #[arg(long)]
        amount: Option<String>,
    },
}

pub async fn run(args: OneInchArgs, _chain: &str) -> anyhow::Result<()> {
    let api_key = std::env::var("ONEINCH_API_KEY")
        .or_else(|_| std::env::var("1INCH_API_KEY"))
        .ok();

    let Some(key) = api_key else {
        anyhow::bail!(
            "1inch API key required. Set ONEINCH_API_KEY or 1INCH_API_KEY environment variable.\n\
             Get an API key at: https://portal.1inch.dev"
        );
    };
    let client = Client::new(&key)?;

    match args.action {
        OneInchCommands::Quote {
            src,
            dst,
            amount,
            chain_id,
            include_gas,
            connector_tokens,
            fee,
        } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let mut request = QuoteRequest::new(&src, &dst, &amount);
            request.include_gas = Some(include_gas);
            if let Some(tokens) = connector_tokens {
                request.connector_tokens = Some(tokens);
            }
            if let Some(f) = fee {
                request.fee = Some(f);
            }

            let quote = client.get_quote(chain, &request).await?;
            output_json(&quote, args.format)?;
        }

        OneInchCommands::Swap {
            src,
            dst,
            amount,
            from,
            chain_id,
            slippage,
            receiver,
            referrer,
            disable_estimate,
            allow_partial_fill,
        } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let mut request = SwapRequest::new(&src, &dst, &amount, &from, slippage);
            request.dest_receiver = receiver;
            request.referrer = referrer;
            request.disable_estimate = Some(disable_estimate);
            request.allow_partial_fill = Some(allow_partial_fill);

            let swap = client.get_swap(chain, &request).await?;
            output_json(&swap, args.format)?;
        }

        OneInchCommands::Tokens { chain_id } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let tokens = client.get_tokens(chain).await?;
            output_json(&tokens, args.format)?;
        }

        OneInchCommands::Sources { chain_id } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let sources = client.get_liquidity_sources(chain).await?;
            output_json(&sources, args.format)?;
        }

        OneInchCommands::Spender { chain_id } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let spender = client.get_approve_spender(chain).await?;
            if args.format.is_json() {
                println!("{{\"spender\": \"{}\"}}", spender);
            } else {
                println!("{}", spender);
            }
        }

        OneInchCommands::Allowance {
            token,
            wallet,
            chain_id,
        } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let allowance = client.get_approve_allowance(chain, &token, &wallet).await?;
            if args.format.is_json() {
                println!("{{\"allowance\": \"{}\"}}", allowance);
            } else {
                println!("{}", allowance);
            }
        }

        OneInchCommands::Approve {
            token,
            chain_id,
            amount,
        } => {
            let chain = chain_id_to_oinch_chain(chain_id)?;
            let tx = client
                .get_approve_transaction(chain, &token, amount.as_deref())
                .await?;
            output_json(&tx, args.format)?;
        }
    }

    Ok(())
}

fn chain_id_to_oinch_chain(chain_id: u64) -> anyhow::Result<oinch::Chain> {
    match chain_id {
        1 => Ok(oinch::Chain::Ethereum),
        56 => Ok(oinch::Chain::Bsc),
        137 => Ok(oinch::Chain::Polygon),
        10 => Ok(oinch::Chain::Optimism),
        42161 => Ok(oinch::Chain::Arbitrum),
        43114 => Ok(oinch::Chain::Avalanche),
        100 => Ok(oinch::Chain::Gnosis),
        250 => Ok(oinch::Chain::Fantom),
        8453 => Ok(oinch::Chain::Base),
        324 => Ok(oinch::Chain::ZkSync),
        59144 => Ok(oinch::Chain::Linea),
        1313161554 => Ok(oinch::Chain::Aurora),
        _ => anyhow::bail!("Unsupported chain ID: {}. Supported: 1 (Ethereum), 56 (BSC), 137 (Polygon), 10 (Optimism), 42161 (Arbitrum), 43114 (Avalanche), 100 (Gnosis), 250 (Fantom), 8453 (Base), 324 (zkSync), 59144 (Linea), 1313161554 (Aurora)", chain_id),
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
