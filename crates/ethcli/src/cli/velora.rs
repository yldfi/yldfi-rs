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
        /// Source token decimals (auto-detected via RPC on non-mainnet chains)
        #[arg(long)]
        src_decimals: Option<u8>,
        /// Destination token decimals (auto-detected via RPC on non-mainnet chains)
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
        /// Chain name (the network recorded in the price route takes precedence)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Slippage in basis points (e.g., 100 = 1%)
        #[arg(long, default_value = "100")]
        slippage: u32,
        /// Receiver address (defaults to user address)
        #[arg(long)]
        receiver: Option<String>,
        /// Price route JSON from the price command: either the full output
        /// (`{"priceRoute": {...}}`) or the `priceRoute` object. It is sent
        /// back to Velora unmodified.
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
            let (src_decimals, dest_decimals) = resolve_decimals(
                vlra_chain,
                &src_token,
                &dest_token,
                src_decimals,
                dest_decimals,
            )
            .await;
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
            let raw_route = extract_price_route(&price_route)?;
            let vlra_chain = match raw_route.get("network").and_then(|n| n.as_u64()) {
                Some(id) => vlra::Chain::from_chain_id(id)
                    .ok_or_else(|| anyhow::anyhow!("Unsupported network {id} in price route"))?,
                None => chain_name_to_vlra_chain(&chain)?,
            };
            if !(1..=5000).contains(&slippage) {
                anyhow::bail!(
                    "--slippage is in basis points and must be between 1 and 5000 (got {slippage})"
                );
            }

            let mut request =
                TransactionRequest::from_raw_price_route(raw_route, &user_address, slippage)
                    .map_err(|e| anyhow::anyhow!("Invalid price route JSON: {e}"))?;
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

/// Parse the `--price-route` argument, accepting either the full `price`
/// output (`{"priceRoute": {...}}`) or the bare `priceRoute` object.
fn extract_price_route(json: &str) -> anyhow::Result<serde_json::Value> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Invalid price route JSON: {e}"))?;
    match value {
        serde_json::Value::Object(mut map) => {
            if let Some(inner) = map.remove("priceRoute") {
                Ok(inner)
            } else {
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => anyhow::bail!("Invalid price route JSON: expected an object"),
    }
}

/// Velora requires `srcDecimals`/`destDecimals` for tokens outside its list,
/// which is common off mainnet. Auto-fill missing values from RPC there.
async fn resolve_decimals(
    chain: vlra::Chain,
    src_token: &str,
    dest_token: &str,
    src_decimals: Option<u8>,
    dest_decimals: Option<u8>,
) -> (Option<u8>, Option<u8>) {
    if chain == vlra::Chain::Ethereum {
        return (src_decimals, dest_decimals);
    }
    let chain_id = chain.chain_id();
    let lookup = |token: String| async move {
        match crate::utils::token_meta::fetch_token_decimals(chain_id, &token).await {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("Warning: could not auto-detect decimals for {token}: {e}");
                None
            }
        }
    };
    let src = match src_decimals {
        Some(d) => Some(d),
        None => lookup(src_token.to_string()).await,
    };
    let dest = match dest_decimals {
        Some(d) => Some(d),
        None => lookup(dest_token.to_string()).await,
    };
    (src, dest)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_price_route_accepts_full_output_or_bare_route() {
        let full = r#"{"priceRoute":{"network":42161,"hmac":"abc"}}"#;
        let bare = r#"{"network":42161,"hmac":"abc"}"#;
        assert_eq!(extract_price_route(full).unwrap()["hmac"], "abc");
        assert_eq!(extract_price_route(bare).unwrap()["network"], 42161);
        assert!(extract_price_route("[1]").is_err());
    }

    #[tokio::test]
    async fn mainnet_decimals_are_left_untouched() {
        let (s, d) = resolve_decimals(vlra::Chain::Ethereum, "0xa", "0xb", None, Some(6)).await;
        assert_eq!((s, d), (None, Some(6)));
    }
}
