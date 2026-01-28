//! Direct Velora (ParaSwap) API commands
//!
//! Provides 1:1 access to Velora/ParaSwap DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use vlra::{Client, PriceRequest, TransactionRequest};

#[derive(Args, Clone)]
pub struct VeloraArgs {
    #[command(subcommand)]
    pub action: VeloraCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum VeloraCommands {
    /// Get swap price/route
    Price {
        /// Source token address
        src_token: String,
        /// Destination token address
        dest_token: String,
        /// Amount in smallest units (wei)
        amount: String,
        /// Chain name (ethereum, bsc, polygon, arbitrum, optimism, base, avalanche, fantom)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Side: SELL or BUY
        #[arg(long, default_value = "SELL")]
        side: String,
        /// Source token decimals
        #[arg(long)]
        src_decimals: Option<u8>,
        /// Destination token decimals
        #[arg(long)]
        dest_decimals: Option<u8>,
        /// User address
        #[arg(long)]
        user_address: Option<String>,
        /// Exclude DEXs (comma-separated)
        #[arg(long)]
        exclude_dexs: Option<String>,
    },

    /// Build swap transaction
    Transaction {
        /// User address (wallet executing the swap)
        user_address: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage in basis points (e.g., 100 = 1%)
        #[arg(long, default_value = "100")]
        slippage: u32,
        /// Receiver address (defaults to user address)
        #[arg(long)]
        receiver: Option<String>,
        /// Price route JSON from price command
        #[arg(long)]
        price_route: String,
    },

    /// List supported tokens
    Tokens {
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
}

pub async fn run(args: VeloraArgs, _chain: &str) -> anyhow::Result<()> {
    let api_key = std::env::var("PARASWAP_API_KEY")
        .or_else(|_| std::env::var("VELORA_API_KEY"))
        .ok();

    let client = if let Some(key) = api_key {
        Client::with_api_key(&key)?
    } else {
        Client::new()?
    };

    match args.action {
        VeloraCommands::Price {
            src_token,
            dest_token,
            amount,
            chain,
            side,
            src_decimals,
            dest_decimals,
            user_address,
            exclude_dexs,
        } => {
            let vlra_chain = chain_name_to_vlra_chain(&chain)?;
            let mut request = match side.to_uppercase().as_str() {
                "BUY" => PriceRequest::buy(&src_token, &dest_token, &amount),
                _ => PriceRequest::sell(&src_token, &dest_token, &amount),
            };
            if let Some(d) = src_decimals {
                request = request.with_src_decimals(d);
            }
            if let Some(d) = dest_decimals {
                request = request.with_dest_decimals(d);
            }
            if let Some(addr) = user_address {
                request = request.with_user_address(addr);
            }
            if let Some(dexs) = exclude_dexs {
                request = request.with_exclude_dexs(dexs);
            }

            let price = client.get_price(vlra_chain, &request).await?;
            output_json(&price, args.format)?;
        }

        VeloraCommands::Transaction {
            user_address,
            chain,
            slippage,
            receiver,
            price_route,
        } => {
            let vlra_chain = chain_name_to_vlra_chain(&chain)?;

            // Parse price route from JSON
            let route: vlra::PriceRoute = serde_json::from_str(&price_route)?;

            let mut request = TransactionRequest::new(&route, &user_address, slippage);
            if let Some(recv) = receiver {
                request = request.with_receiver(recv);
            }

            let tx = client.build_transaction(vlra_chain, &request).await?;
            output_json(&tx, args.format)?;
        }

        VeloraCommands::Tokens { chain } => {
            let vlra_chain = chain_name_to_vlra_chain(&chain)?;
            let tokens = client.get_tokens(vlra_chain).await?;
            output_json(&tokens, args.format)?;
        }
    }

    Ok(())
}

fn chain_name_to_vlra_chain(name: &str) -> anyhow::Result<vlra::Chain> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => Ok(vlra::Chain::Ethereum),
        "bsc" | "bnb" | "binance" => Ok(vlra::Chain::Bsc),
        "polygon" | "matic" => Ok(vlra::Chain::Polygon),
        "arbitrum" | "arb" => Ok(vlra::Chain::Arbitrum),
        "optimism" | "op" => Ok(vlra::Chain::Optimism),
        "avalanche" | "avax" => Ok(vlra::Chain::Avalanche),
        "fantom" | "ftm" => Ok(vlra::Chain::Fantom),
        "base" => Ok(vlra::Chain::Base),
        _ => anyhow::bail!(
            "Unsupported chain: {}. Supported: ethereum, bsc, polygon, arbitrum, optimism, avalanche, fantom, base",
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
