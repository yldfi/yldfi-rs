//! Direct OpenOcean API commands
//!
//! Provides 1:1 access to OpenOcean DEX Aggregator API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use openoc::{Client, QuoteRequest, SwapRequest};

#[derive(Args, Clone)]
pub struct OpenOceanArgs {
    #[command(subcommand)]
    pub action: OpenOceanCommands,

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
pub enum OpenOceanCommands {
    /// Get swap quote (price estimation)
    Quote {
        /// Source token address (use 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE for native token)
        in_token: String,
        /// Destination token address
        out_token: String,
        /// Amount in smallest units (wei)
        amount: String,
        /// Chain name (ethereum, bsc, polygon, arbitrum, optimism, base, avalanche, fantom, etc.)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Gas price in Gwei
        #[arg(long)]
        gas_price: Option<String>,
        /// Slippage tolerance in percent (e.g., 1 = 1%)
        #[arg(long, default_value = "1")]
        slippage: f64,
    },

    /// Get swap transaction data
    Swap {
        /// Source token address
        in_token: String,
        /// Destination token address
        out_token: String,
        /// Amount in smallest units (wei)
        amount: String,
        /// Wallet address that will execute the swap
        account: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage tolerance in percent
        #[arg(long, default_value = "1")]
        slippage: f64,
        /// Gas price in Gwei
        #[arg(long)]
        gas_price: Option<String>,
        /// Referrer address
        #[arg(long)]
        referrer: Option<String>,
    },

    /// Get reverse quote (specify output amount)
    ReverseQuote {
        /// Source token address
        in_token: String,
        /// Destination token address
        out_token: String,
        /// Desired output amount, human-readable (e.g. "1" for 1 token; the
        /// /reverseQuote endpoint only documents the legacy `amount` param)
        out_amount: String,
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get list of supported tokens
    Tokens {
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get list of DEX sources
    Dexes {
        /// Chain name
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
}

pub async fn run(args: OpenOceanArgs, _chain: &str) -> anyhow::Result<()> {
    let client = Client::new()?;

    match args.action {
        OpenOceanCommands::Quote {
            in_token,
            out_token,
            amount,
            chain,
            gas_price,
            slippage,
        } => {
            let oo_chain = chain_name_to_openoc_chain(&chain)?;
            let mut request = QuoteRequest::new(&in_token, &out_token, &amount);
            request.slippage = Some(slippage);
            if let Some(gp) = gas_price {
                request.gas_price = Some(gwei_to_wei(&gp)?);
            }

            let quote = client.get_quote(oo_chain, &request).await?;
            output_json(&quote, args.format)?;
        }

        OpenOceanCommands::Swap {
            in_token,
            out_token,
            amount,
            account,
            chain,
            slippage,
            gas_price,
            referrer,
        } => {
            let oo_chain = chain_name_to_openoc_chain(&chain)?;
            let mut request = SwapRequest::new(&in_token, &out_token, &amount, &account);
            request.slippage = Some(slippage);
            if let Some(gp) = gas_price {
                request.gas_price = Some(gwei_to_wei(&gp)?);
            }
            if let Some(r) = referrer {
                request.referrer = Some(r);
            }

            let swap = client.get_swap_quote(oo_chain, &request).await?;
            output_json(&swap, args.format)?;
        }

        OpenOceanCommands::ReverseQuote {
            in_token,
            out_token,
            out_amount,
            chain,
        } => {
            let oo_chain = chain_name_to_openoc_chain(&chain)?;
            let quote = client
                .get_reverse_quote(oo_chain, &in_token, &out_token, &out_amount)
                .await?;
            output_json(&quote, args.format)?;
        }

        OpenOceanCommands::Tokens { chain } => {
            let oo_chain = chain_name_to_openoc_chain(&chain)?;
            let tokens = client.get_token_list(oo_chain).await?;
            output_json(&tokens, args.format)?;
        }

        OpenOceanCommands::Dexes { chain } => {
            let oo_chain = chain_name_to_openoc_chain(&chain)?;
            let dexes = client.get_dex_list(oo_chain).await?;
            output_json(&dexes, args.format)?;
        }
    }

    Ok(())
}

fn chain_name_to_openoc_chain(name: &str) -> anyhow::Result<openoc::Chain> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => Ok(openoc::Chain::Eth),
        "bsc" | "bnb" | "binance" => Ok(openoc::Chain::Bsc),
        "polygon" | "matic" => Ok(openoc::Chain::Polygon),
        "arbitrum" | "arb" => Ok(openoc::Chain::Arbitrum),
        "optimism" | "op" => Ok(openoc::Chain::Optimism),
        "avalanche" | "avax" => Ok(openoc::Chain::Avax),
        "fantom" | "ftm" => Ok(openoc::Chain::Fantom),
        "base" => Ok(openoc::Chain::Base),
        "gnosis" | "xdai" => Ok(openoc::Chain::Gnosis),
        "linea" => Ok(openoc::Chain::Linea),
        "zksync" | "zksync-era" => Ok(openoc::Chain::Zksync),
        "scroll" => Ok(openoc::Chain::Scroll),
        "mantle" => Ok(openoc::Chain::Mantle),
        "blast" => Ok(openoc::Chain::Blast),
        _ => anyhow::bail!(
            "Unsupported chain: {}. Supported: ethereum, bsc, polygon, arbitrum, optimism, avalanche, fantom, base, gnosis, linea, zksync, scroll, mantle, blast",
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

/// Convert a gwei amount (e.g. "30" or "0.5") to a wei string for `gasPriceDecimals`
fn gwei_to_wei(gwei: &str) -> anyhow::Result<String> {
    let gwei = gwei.trim();
    let invalid = || anyhow::anyhow!("Invalid gas price in gwei: '{}'", gwei);
    let (whole, frac) = gwei.split_once('.').unwrap_or((gwei, ""));
    if frac.len() > 9 {
        anyhow::bail!("Gas price '{}' has more than 9 decimal places", gwei);
    }
    let whole: u128 = if whole.is_empty() {
        0
    } else {
        whole.parse().map_err(|_| invalid())?
    };
    let frac_wei: u128 = if frac.is_empty() {
        0
    } else {
        format!("{:0<9}", frac).parse().map_err(|_| invalid())?
    };
    let wei = whole
        .checked_mul(1_000_000_000)
        .and_then(|w| w.checked_add(frac_wei))
        .ok_or_else(|| anyhow::anyhow!("Gas price '{}' is too large", gwei))?;
    Ok(wei.to_string())
}

#[cfg(test)]
mod tests {
    use super::gwei_to_wei;

    #[test]
    fn converts_gwei_to_wei() {
        assert_eq!(gwei_to_wei("30").unwrap(), "30000000000");
        assert_eq!(gwei_to_wei("0.5").unwrap(), "500000000");
        assert_eq!(gwei_to_wei("1.000000001").unwrap(), "1000000001");
        assert!(gwei_to_wei("abc").is_err());
        assert!(gwei_to_wei("1.0000000001").is_err());
    }
}
