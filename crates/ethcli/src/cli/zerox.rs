//! Direct 0x API commands
//!
//! Provides 1:1 access to 0x DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use zrxswap::{Client, PriceRequest, QuoteRequest};

#[derive(Args, Clone)]
pub struct ZeroXArgs {
    #[command(subcommand)]
    pub action: ZeroXCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum ZeroXCommands {
    /// Get swap quote with transaction data
    Quote {
        /// Source token address (use 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE for native token)
        sell_token: String,
        /// Destination token address
        buy_token: String,
        /// Amount to sell in smallest units (wei)
        sell_amount: String,
        /// Taker address (wallet executing the swap)
        taker: String,
        /// Chain name (ethereum, bsc, polygon, arbitrum, optimism, base, avalanche)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage in basis points (e.g., 100 = 1%)
        #[arg(long, default_value = "100")]
        slippage_bps: u32,
        /// Exclude sources (comma-separated)
        #[arg(long)]
        exclude_sources: Option<String>,
        /// Skip validation
        #[arg(long)]
        skip_validation: bool,
    },

    /// Get price estimate (lighter than quote, no tx data)
    Price {
        /// Source token address
        sell_token: String,
        /// Destination token address
        buy_token: String,
        /// Amount to sell in smallest units
        sell_amount: String,
        /// Taker address
        taker: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage in basis points
        #[arg(long, default_value = "100")]
        slippage_bps: u32,
    },

    /// Get list of liquidity sources
    Sources {
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
}

pub async fn run(args: ZeroXArgs, _chain: &str) -> anyhow::Result<()> {
    let api_key = std::env::var("ZEROX_API_KEY")
        .or_else(|_| std::env::var("0X_API_KEY"))
        .ok();

    let client = if let Some(key) = api_key {
        Client::with_api_key(&key)?
    } else {
        Client::new()?
    };

    match args.action {
        ZeroXCommands::Quote {
            sell_token,
            buy_token,
            sell_amount,
            taker,
            chain,
            slippage_bps,
            exclude_sources,
            skip_validation,
        } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let mut request = QuoteRequest::sell(&sell_token, &buy_token, &sell_amount);
            request.taker = Some(taker);
            request.slippage_bps = Some(slippage_bps);
            if let Some(sources) = exclude_sources {
                request.excluded_sources = Some(sources);
            }
            request.skip_validation = Some(skip_validation);

            let quote = client.get_quote(zrx_chain, &request).await?;
            output_json(&quote, args.format)?;
        }

        ZeroXCommands::Price {
            sell_token,
            buy_token,
            sell_amount,
            taker,
            chain,
            slippage_bps,
        } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let mut request = PriceRequest::sell(&sell_token, &buy_token, &sell_amount);
            request.taker = Some(taker);
            request.slippage_bps = Some(slippage_bps);

            let price = client.get_price(zrx_chain, &request).await?;
            output_json(&price, args.format)?;
        }

        ZeroXCommands::Sources { chain } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let sources = client.get_sources(zrx_chain).await?;
            output_json(&sources, args.format)?;
        }
    }

    Ok(())
}

fn chain_name_to_zrx_chain(name: &str) -> anyhow::Result<zrxswap::Chain> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => Ok(zrxswap::Chain::Ethereum),
        "bsc" | "bnb" | "binance" => Ok(zrxswap::Chain::Bsc),
        "polygon" | "matic" => Ok(zrxswap::Chain::Polygon),
        "arbitrum" | "arb" => Ok(zrxswap::Chain::Arbitrum),
        "optimism" | "op" => Ok(zrxswap::Chain::Optimism),
        "avalanche" | "avax" => Ok(zrxswap::Chain::Avalanche),
        "base" => Ok(zrxswap::Chain::Base),
        _ => anyhow::bail!(
            "Unsupported chain: {}. Supported: ethereum, bsc, polygon, arbitrum, optimism, avalanche, base",
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
