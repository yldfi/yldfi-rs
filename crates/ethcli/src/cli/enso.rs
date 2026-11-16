//! Direct Enso Finance API commands
//!
//! Provides 1:1 access to Enso Finance DeFi Aggregator API endpoints.

use crate::aggregator::get_cached_config;
use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use ensof::{BundleAction, BundleRequest, Client, QuoteRequest, RouteRequest, TokensRequest};
use secrecy::ExposeSecret;

#[derive(Args, Clone)]
pub struct EnsoArgs {
    #[command(subcommand)]
    pub action: EnsoCommands,

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
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Slippage in basis points (e.g., 50 = 0.5%)
        #[arg(long, default_value = "50")]
        slippage: u16,
        /// Routing strategy (router, delegate, ensowallet, router-legacy, delegate-legacy)
        #[arg(long)]
        routing_strategy: Option<String>,
    },

    /// Get a quote (output amount and route, no transaction data)
    Quote {
        /// Source token address
        token_in: String,
        /// Destination token address
        token_out: String,
        /// Amount in smallest units (wei)
        amount_in: String,
        /// Sender address (optional for quotes)
        from_address: Option<String>,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Routing strategy (router, delegate, ensowallet, router-legacy, delegate-legacy)
        #[arg(long)]
        routing_strategy: Option<String>,
        /// Aggregators to leave out (comma-separated, e.g. 1inch)
        #[arg(long, value_delimiter = ',')]
        ignore_aggregators: Vec<String>,
        /// Latency/quality trade-off
        #[arg(long, value_parser = ["light", "medium", "high", "max"])]
        effort: Option<String>,
        /// Swaps the quote may use
        #[arg(long, value_parser = ["any", "rfq", "amm"])]
        swap_type: Option<String>,
    },

    /// Get token price in USD
    Price {
        /// Token address
        token: String,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
    },

    /// Get token balances for an address
    Balances {
        /// Wallet address
        address: String,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
    },

    /// Bundle multiple DeFi actions into one transaction
    Bundle {
        /// Sender address
        from_address: String,
        /// Actions as JSON array: [{"protocol":"...","action":"...","args":{...}}]
        actions_json: String,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Routing strategy (router, delegate, ensowallet, router-legacy, delegate-legacy).
        /// Defaults to `router` (for EOAs); the API's own default is delegate
        /// mode, which is only executable from a smart wallet.
        #[arg(long, default_value = "router")]
        routing_strategy: Option<String>,
    },

    /// Get USD prices of several tokens on one chain (max 100)
    Prices {
        /// Token addresses
        #[arg(required = true, num_args = 1..)]
        tokens: Vec<String>,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
    },

    /// Search the tokens (base and DeFi) Enso knows, one page at a time
    Tokens {
        /// Only this chain (no default: omit to search every chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Only this project (e.g. aave)
        #[arg(long)]
        project: Option<String>,
        /// Only this protocol slug (e.g. aave-v3)
        #[arg(long)]
        protocol: Option<String>,
        /// Only these token addresses (comma-separated or repeated)
        #[arg(long, value_delimiter = ',')]
        address: Vec<String>,
        /// Only these symbols (comma-separated or repeated)
        #[arg(long, value_delimiter = ',')]
        symbol: Vec<String>,
        /// Only tokens with any of these underlying tokens (comma-separated or repeated)
        #[arg(long, value_delimiter = ',')]
        underlying: Vec<String>,
        /// Token type
        #[arg(long = "type", value_parser = ["defi", "base"])]
        token_type: Option<String>,
        /// Minimum APY
        #[arg(long)]
        apy_from: Option<f64>,
        /// Maximum APY
        #[arg(long)]
        apy_to: Option<f64>,
        /// Minimum TVL
        #[arg(long)]
        tvl_from: Option<f64>,
        /// Maximum TVL
        #[arg(long)]
        tvl_to: Option<f64>,
        /// Include symbol, name and logos
        #[arg(long)]
        include_metadata: bool,
        /// Include underlying tokens
        #[arg(long)]
        include_underlying: bool,
        /// Page number
        #[arg(long)]
        page: Option<u64>,
        /// Page size (max 1000)
        #[arg(long)]
        page_size: Option<u64>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<u64>,
    },

    /// List the networks Enso supports
    Networks,

    /// List protocols, optionally filtered by chain or slug
    Protocols {
        /// Only protocols on this chain (no default: omit for every chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Only the protocol with this slug
        #[arg(long)]
        slug: Option<String>,
    },

    /// Build a transaction approving Enso's router to spend a token
    Approve {
        /// Token address
        token: String,
        /// Amount in smallest units
        amount: String,
        /// Token owner (sender) address
        from_address: String,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Routing strategy (approve the smart wallet with ensowallet/delegate)
        #[arg(long)]
        routing_strategy: Option<String>,
    },

    /// Get the Enso smart wallet of an address
    Wallet {
        /// Owner address
        from_address: String,
        /// Chain ID (defaults to the global --chain)
        #[arg(long)]
        chain_id: Option<u64>,
        /// Routing strategy
        #[arg(long)]
        routing_strategy: Option<String>,
    },
}

pub async fn run(args: EnsoArgs, chain: &str) -> anyhow::Result<()> {
    // Get API key from config first, then fall back to env var
    let config = get_cached_config();
    let api_key = config
        .as_ref()
        .and_then(|c| c.enso.as_ref())
        .map(|e| e.api_key.expose_secret().to_string())
        .or_else(|| std::env::var("ENSO_API_KEY").ok());

    let client = if let Some(key) = api_key {
        Client::with_api_key(&key)?
    } else {
        anyhow::bail!(
            "Enso API key required. Set the ENSO_API_KEY environment variable or add api_key under [enso] in the config file (see: ethcli config path).\n\
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
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
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

        EnsoCommands::Quote {
            token_in,
            token_out,
            amount_in,
            from_address,
            chain_id,
            routing_strategy,
            ignore_aggregators,
            effort,
            swap_type,
        } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            let mut request = QuoteRequest::new(chain_id, &token_in, &token_out, &amount_in)
                .with_ignore_aggregators(ignore_aggregators);
            if let Some(from) = from_address {
                request = request.with_from_address(from);
            }
            if let Some(strategy) = routing_strategy {
                request = request.with_routing_strategy(parse_routing_strategy(&strategy)?);
            }
            if let Some(effort) = effort {
                request = request.with_effort(effort);
            }
            if let Some(swap_type) = swap_type {
                request = request.with_swap_type(swap_type);
            }

            let quote = client.get_quote(&request).await?;
            output_json(&quote, args.format)?;
        }

        EnsoCommands::Price { token, chain_id } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            let price = client.get_token_price(chain_id, &token).await?;
            output_json(&price, args.format)?;
        }

        EnsoCommands::Balances { address, chain_id } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            let balances = client.get_balances(chain_id, &address).await?;
            output_json(&balances, args.format)?;
        }

        EnsoCommands::Bundle {
            from_address,
            actions_json,
            chain_id,
            routing_strategy,
        } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            let actions: Vec<BundleAction> = serde_json::from_str(&actions_json)
                .map_err(|e| anyhow::anyhow!("Invalid actions JSON: {}", e))?;
            let mut request = BundleRequest::new(chain_id, &from_address, actions);
            if let Some(strategy) = routing_strategy {
                request = request.with_routing_strategy(parse_routing_strategy(&strategy)?);
            }
            let result = client.bundle(&request).await?;
            output_json(&result, args.format)?;
        }

        EnsoCommands::Prices { tokens, chain_id } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            if tokens.len() > 100 {
                anyhow::bail!("At most 100 token addresses per request");
            }
            let prices = client.get_token_prices(chain_id, &tokens).await?;
            output_json(&prices, args.format)?;
        }

        EnsoCommands::Tokens {
            chain_id,
            project,
            protocol,
            address,
            symbol,
            underlying,
            token_type,
            apy_from,
            apy_to,
            tvl_from,
            tvl_to,
            include_metadata,
            include_underlying,
            page,
            page_size,
            cursor,
        } => {
            let mut request = TokensRequest::new()
                .with_addresses(address)
                .with_symbols(symbol)
                .with_underlying_tokens(underlying);
            request.chain_id = chain_id;
            request.project = project;
            request.protocol_slug = protocol;
            request.token_type = token_type;
            request.apy_from = apy_from;
            request.apy_to = apy_to;
            request.tvl_from = tvl_from;
            request.tvl_to = tvl_to;
            request.include_metadata = include_metadata.then_some(true);
            request.include_underlying = include_underlying.then_some(true);
            request.page = page;
            request.page_size = page_size;
            request.cursor = cursor;

            let tokens = client.get_tokens(&request).await?;
            output_json(&tokens, args.format)?;
        }

        EnsoCommands::Networks => {
            let networks = client.get_networks().await?;
            output_json(&networks, args.format)?;
        }

        EnsoCommands::Protocols { chain_id, slug } => {
            let protocols = client.get_protocols(chain_id, slug.as_deref()).await?;
            output_json(&protocols, args.format)?;
        }

        EnsoCommands::Approve {
            token,
            amount,
            from_address,
            chain_id,
            routing_strategy,
        } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            let strategy = routing_strategy
                .as_deref()
                .map(parse_routing_strategy)
                .transpose()?;
            let tx = client
                .get_approve_transaction(chain_id, &from_address, &token, &amount, strategy)
                .await?;
            output_json(&tx, args.format)?;
        }

        EnsoCommands::Wallet {
            from_address,
            chain_id,
            routing_strategy,
        } => {
            let chain_id = super::resolve_chain_id(chain_id, chain)?;
            let strategy = routing_strategy
                .as_deref()
                .map(parse_routing_strategy)
                .transpose()?;
            let wallet = client.get_wallet(chain_id, &from_address, strategy).await?;
            output_json(&wallet, args.format)?;
        }
    }

    Ok(())
}

fn parse_routing_strategy(s: &str) -> anyhow::Result<ensof::RoutingStrategy> {
    match s.to_lowercase().as_str() {
        "router" => Ok(ensof::RoutingStrategy::Router),
        "delegate" => Ok(ensof::RoutingStrategy::Delegate),
        "ensowallet" | "enso_wallet" | "ensowallet-v2" => Ok(ensof::RoutingStrategy::Ensowallet),
        "router-legacy" => Ok(ensof::RoutingStrategy::RouterLegacy),
        "delegate-legacy" => Ok(ensof::RoutingStrategy::DelegateLegacy),
        _ => anyhow::bail!(
            "Invalid routing strategy: {}. Use: router, delegate, ensowallet, router-legacy, delegate-legacy",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(clap::Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: EnsoCommands,
    }

    fn parse(args: &[&str]) -> Result<EnsoCommands, clap::Error> {
        use clap::Parser;
        TestCli::try_parse_from(std::iter::once("enso").chain(args.iter().copied()))
            .map(|c| c.command)
    }

    #[test]
    fn parses_quote_with_optional_sender() {
        match parse(&["quote", "0xin", "0xout", "100"]).unwrap() {
            EnsoCommands::Quote { from_address, .. } => assert!(from_address.is_none()),
            _ => panic!("expected quote"),
        }
        match parse(&[
            "quote",
            "0xin",
            "0xout",
            "100",
            "0xfrom",
            "--ignore-aggregators",
            "1inch,0x",
            "--effort",
            "high",
        ])
        .unwrap()
        {
            EnsoCommands::Quote {
                from_address,
                ignore_aggregators,
                effort,
                ..
            } => {
                assert_eq!(from_address.as_deref(), Some("0xfrom"));
                assert_eq!(ignore_aggregators, vec!["1inch", "0x"]);
                assert_eq!(effort.as_deref(), Some("high"));
            }
            _ => panic!("expected quote"),
        }
        assert!(parse(&["quote", "0xin", "0xout", "100", "--effort", "huge"]).is_err());
    }

    #[test]
    fn parses_prices() {
        match parse(&["prices", "0xa", "0xb", "--chain-id", "8453"]).unwrap() {
            EnsoCommands::Prices { tokens, chain_id } => {
                assert_eq!(tokens, vec!["0xa", "0xb"]);
                assert_eq!(chain_id, Some(8453));
            }
            _ => panic!("expected prices"),
        }
        assert!(parse(&["prices"]).is_err());
    }

    #[test]
    fn parses_tokens_filters() {
        match parse(&[
            "tokens",
            "--chain-id",
            "1",
            "--protocol",
            "aave-v3",
            "--symbol",
            "aUSDC,aWETH",
            "--type",
            "defi",
            "--include-metadata",
            "--page-size",
            "50",
        ])
        .unwrap()
        {
            EnsoCommands::Tokens {
                chain_id,
                protocol,
                symbol,
                token_type,
                include_metadata,
                page_size,
                ..
            } => {
                assert_eq!(chain_id, Some(1));
                assert_eq!(protocol.as_deref(), Some("aave-v3"));
                assert_eq!(symbol, vec!["aUSDC", "aWETH"]);
                assert_eq!(token_type.as_deref(), Some("defi"));
                assert!(include_metadata);
                assert_eq!(page_size, Some(50));
            }
            _ => panic!("expected tokens"),
        }
        assert!(parse(&["tokens", "--type", "other"]).is_err());
    }

    #[test]
    fn parses_networks_protocols_approve_wallet() {
        assert!(matches!(
            parse(&["networks"]).unwrap(),
            EnsoCommands::Networks
        ));
        assert!(matches!(
            parse(&["protocols", "--slug", "aave-v3"]).unwrap(),
            EnsoCommands::Protocols { .. }
        ));
        match parse(&["approve", "0xtoken", "1000", "0xfrom"]).unwrap() {
            EnsoCommands::Approve {
                token,
                amount,
                from_address,
                ..
            } => {
                assert_eq!(
                    (token.as_str(), amount.as_str(), from_address.as_str()),
                    ("0xtoken", "1000", "0xfrom")
                );
            }
            _ => panic!("expected approve"),
        }
        assert!(matches!(
            parse(&["wallet", "0xfrom", "--routing-strategy", "delegate"]).unwrap(),
            EnsoCommands::Wallet { .. }
        ));
    }
}
