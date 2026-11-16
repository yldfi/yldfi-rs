//! Direct 0x API commands
//!
//! Provides 1:1 access to 0x DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use secrecy::ExposeSecret;
use zrxswap::{Client, PriceRequest, QuoteRequest};

#[derive(Args, Clone)]
pub struct ZeroXArgs {
    #[command(subcommand)]
    pub action: ZeroXCommands,

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
        /// Skip validation (no effect: the 0x v2 API ignores it)
        #[arg(long)]
        skip_validation: bool,
        /// Use the AllowanceHolder flow (approve `allowanceTarget`, no Permit2
        /// signature) instead of Permit2
        #[arg(long)]
        allowance_holder: bool,
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
        /// Use the AllowanceHolder flow instead of Permit2
        #[arg(long)]
        allowance_holder: bool,
    },

    /// Get list of liquidity sources
    Sources {
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// List the chains the Swap API (or, with --gasless, the Gasless API) supports
    Chains {
        /// List the Gasless API's chains instead
        #[arg(long)]
        gasless: bool,
    },

    /// Get an indicative price for a gasless swap (ERC-20 sell token only;
    /// gas is paid from the sell token)
    GaslessPrice {
        /// Source token address (ERC-20, not the native token)
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

    /// Get a firm gasless quote: the EIP-712 trade (and approval, if needed)
    /// to sign. Submitting the signatures is not supported.
    GaslessQuote {
        /// Source token address (ERC-20, not the native token)
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
        /// Exclude sources (comma-separated)
        #[arg(long)]
        exclude_sources: Option<String>,
    },

    /// Get the status of a submitted gasless trade
    GaslessStatus {
        /// Trade hash (`trade.hash` of the gasless quote)
        trade_hash: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
}

pub async fn run(args: ZeroXArgs, _chain: &str) -> anyhow::Result<()> {
    // Config first (same as the quote aggregator), then env vars. The 0x v2
    // API rejects unauthenticated requests, so fail fast without a key.
    let api_key = crate::aggregator::get_cached_config()
        .as_ref()
        .and_then(|c| c.zerox.as_ref())
        .map(|z| z.api_key.expose_secret().to_string())
        .or_else(|| std::env::var("ZEROX_API_KEY").ok())
        .or_else(|| std::env::var("0X_API_KEY").ok())
        .filter(|k| !k.is_empty());

    let Some(key) = api_key else {
        anyhow::bail!(
            "0x API key required. Add [zerox] to the config or set ZEROX_API_KEY / 0X_API_KEY.\n\
             Get an API key at: https://dashboard.0x.org"
        );
    };
    let client = Client::with_api_key(&key)?;

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
            allowance_holder,
        } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let mut request = QuoteRequest::sell(&sell_token, &buy_token, &sell_amount);
            request.taker = Some(taker);
            request.slippage_bps = Some(slippage_bps);
            if let Some(sources) = exclude_sources {
                request.excluded_sources = Some(sources);
            }
            if skip_validation {
                eprintln!("Warning: --skip-validation has no effect with the 0x v2 API");
            }

            let quote = if allowance_holder {
                client
                    .get_allowance_holder_quote(zrx_chain, &request)
                    .await?
            } else {
                client.get_quote(zrx_chain, &request).await?
            };
            output_json(&quote, args.format)?;
        }

        ZeroXCommands::Price {
            sell_token,
            buy_token,
            sell_amount,
            taker,
            chain,
            slippage_bps,
            allowance_holder,
        } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let mut request = PriceRequest::sell(&sell_token, &buy_token, &sell_amount);
            request.taker = Some(taker);
            request.slippage_bps = Some(slippage_bps);

            let price = if allowance_holder {
                client
                    .get_allowance_holder_price(zrx_chain, &request)
                    .await?
            } else {
                client.get_price(zrx_chain, &request).await?
            };
            output_json(&price, args.format)?;
        }

        ZeroXCommands::Sources { chain } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let sources = client.get_sources(zrx_chain).await?;
            output_json(&sources, args.format)?;
        }

        ZeroXCommands::Chains { gasless } => {
            let chains = if gasless {
                client.get_gasless_chains().await?
            } else {
                client.get_swap_chains().await?
            };
            output_json(&chains, args.format)?;
        }

        ZeroXCommands::GaslessPrice {
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

            let price = client.get_gasless_price(zrx_chain, &request).await?;
            output_json(&price, args.format)?;
        }

        ZeroXCommands::GaslessQuote {
            sell_token,
            buy_token,
            sell_amount,
            taker,
            chain,
            slippage_bps,
            exclude_sources,
        } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let mut request = QuoteRequest::sell(&sell_token, &buy_token, &sell_amount);
            request.taker = Some(taker);
            request.slippage_bps = Some(slippage_bps);
            request.excluded_sources = exclude_sources;

            let quote = client.get_gasless_quote(zrx_chain, &request).await?;
            output_json(&quote, args.format)?;
        }

        ZeroXCommands::GaslessStatus { trade_hash, chain } => {
            let zrx_chain = chain_name_to_zrx_chain(&chain)?;
            let status = client.get_gasless_status(zrx_chain, &trade_hash).await?;
            output_json(&status, args.format)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(clap::Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: ZeroXCommands,
    }

    fn parse(args: &[&str]) -> Result<ZeroXCommands, clap::Error> {
        use clap::Parser;
        TestCli::try_parse_from(std::iter::once("0x").chain(args.iter().copied()))
            .map(|c| c.command)
    }

    #[test]
    fn quote_and_price_default_to_permit2() {
        match parse(&["quote", "0xa", "0xb", "1", "0xt"]).unwrap() {
            ZeroXCommands::Quote {
                allowance_holder, ..
            } => assert!(!allowance_holder),
            _ => panic!("expected quote"),
        }
        match parse(&["price", "0xa", "0xb", "1", "0xt", "--allowance-holder"]).unwrap() {
            ZeroXCommands::Price {
                allowance_holder, ..
            } => assert!(allowance_holder),
            _ => panic!("expected price"),
        }
    }

    #[test]
    fn parses_chains() {
        assert!(matches!(
            parse(&["chains"]).unwrap(),
            ZeroXCommands::Chains { gasless: false }
        ));
        assert!(matches!(
            parse(&["chains", "--gasless"]).unwrap(),
            ZeroXCommands::Chains { gasless: true }
        ));
    }

    #[test]
    fn parses_gasless_commands() {
        assert!(matches!(
            parse(&["gasless-price", "0xa", "0xb", "1", "0xt", "--chain", "base"]).unwrap(),
            ZeroXCommands::GaslessPrice { .. }
        ));
        assert!(matches!(
            parse(&["gasless-quote", "0xa", "0xb", "1", "0xt"]).unwrap(),
            ZeroXCommands::GaslessQuote { .. }
        ));
        match parse(&["gasless-status", "0xhash", "--chain", "arbitrum"]).unwrap() {
            ZeroXCommands::GaslessStatus { trade_hash, chain } => {
                assert_eq!(trade_hash, "0xhash");
                assert_eq!(chain, "arbitrum");
            }
            _ => panic!("expected gasless-status"),
        }
    }
}
