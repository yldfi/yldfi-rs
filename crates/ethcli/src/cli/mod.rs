//! CLI command modules
//!
//! Each subcommand has its own module with argument definitions and handlers.

pub mod account;
pub mod address;
pub mod alchemy;
pub mod blacklist;
pub mod cast;
pub mod cex;
pub mod chainlink;
pub mod chainlist;
pub mod config;
pub mod contract;
pub mod cowswap;
pub mod curve;
pub mod doctor;
pub mod dune_cli;
pub mod endpoints;
pub mod ens;
pub mod enso;
pub mod gas;
pub mod gecko;
pub mod goplus;
pub mod kong;
pub mod kyber;
pub mod lifi;
pub mod llama;
pub mod logs;
pub mod moralis;
pub mod nfts;
pub mod oneinch;
pub mod openocean;
pub mod portfolio;
pub mod price;
pub mod pyth;
pub mod quote;
pub mod rpc;
pub mod schema;
pub mod sig;
pub mod simulate;
pub mod solodit;
pub mod tenderly;
pub mod token;
pub mod tx;
pub mod uniswap;
pub mod update;
pub mod velora;
pub mod yields;
pub mod zerox;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::fmt;

/// Output format for command results
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// JSON output (default for most commands)
    #[default]
    Json,
    /// Human-readable table/pretty format
    #[value(alias = "pretty")]
    Table,
    /// Newline-delimited JSON (for streaming)
    Ndjson,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Ndjson => write!(f, "ndjson"),
        }
    }
}

impl OutputFormat {
    /// Check if this format is JSON
    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }

    /// Check if this format is table/pretty
    pub fn is_table(&self) -> bool {
        matches!(self, OutputFormat::Table)
    }

    /// Check if this format is NDJSON
    pub fn is_ndjson(&self) -> bool {
        matches!(self, OutputFormat::Ndjson)
    }
}

#[derive(Parser)]
#[command(name = "ethcli")]
#[command(
    version,
    about = "Comprehensive Ethereum CLI for logs, transactions, accounts, and contracts"
)]
#[command(after_help = r#"EXAMPLES:
    # Fetch Transfer events from USDC
    ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
                -e "Transfer(address,address,uint256)" \
                -f 21000000 -t 21000100

    # Analyze a transaction
    ethcli tx 0x123...

    # Get account balance
    ethcli account balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

    # Get contract ABI
    ethcli contract abi 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

    # Cast utilities
    ethcli cast to-wei 1.5 eth
    ethcli cast sig "transfer(address,uint256)"

    # RPC calls
    ethcli rpc block latest
    ethcli rpc call 0x... 0xa9059cbb...

    # ENS resolution
    ethcli ens resolve vitalik.eth

EXIT CODES:
    0    Success
    1    General error (network, API, invalid input)
    2    Invalid arguments (bad CLI usage)

ENVIRONMENT VARIABLES:
    ETHERSCAN_API_KEY    Etherscan API key (optional, increases rate limit)
"#)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Chain to query
    #[arg(long, default_value = "ethereum", global = true)]
    pub chain: String,

    /// Etherscan API key
    #[arg(long, env = "ETHERSCAN_API_KEY", hide_env_values = true, global = true)]
    pub etherscan_key: Option<String>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress progress output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

/// Resolve the chain ID for a command that has its own optional `--chain-id`
/// flag: an explicit `--chain-id` wins, otherwise the global `--chain`
/// (name or numeric ID) is used.
///
/// # Errors
///
/// Returns an error if the global chain cannot be parsed.
pub fn resolve_chain_id(explicit: Option<u64>, global_chain: &str) -> anyhow::Result<u64> {
    if let Some(id) = explicit {
        return Ok(id);
    }
    crate::config::Chain::from_str_or_id(global_chain)
        .map(|c| c.chain_id())
        .map_err(|e| anyhow::anyhow!("Invalid --chain '{global_chain}': {e}"))
}

/// Does `token` (e.g. `--block` or `-b`) name this argument?
fn arg_named(arg: &clap::Arg, token: &str) -> bool {
    if let Some(long) = token.strip_prefix("--") {
        let long = long.split('=').next().unwrap_or(long);
        arg.get_long() == Some(long)
            || arg
                .get_all_aliases()
                .is_some_and(|aliases| aliases.contains(&long))
    } else if let Some(short) = token.strip_prefix('-') {
        let mut chars = short.chars();
        matches!((chars.next(), chars.next()), (Some(c), None) if arg.get_short() == Some(c))
    } else {
        false
    }
}

/// Expand a `--json` given anywhere on the command line into the output
/// option of the command being run.
///
/// Commands spell their output option differently (`--format`, `--output`,
/// both with `-o`), and a few have their own `--json`. This lets a caller
/// write `--json` on any of them:
///
/// - the command has its own `--json`: left as it is;
/// - the command has a format option that accepts `json`: `--json` becomes
///   that option, unless the option was also given explicitly;
/// - the command has no output option: `--json` is dropped with a note on
///   stderr, so a command that already prints JSON still runs.
///
/// Everything after a bare `--` is left untouched.
pub fn expand_json_flag(args: Vec<String>) -> Vec<String> {
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    if !args[..end].iter().skip(1).any(|a| a == "--json") {
        return args;
    }

    let root = Cli::command();
    let mut chain: Vec<&clap::Command> = vec![&root];
    let mut i = 1;
    while i < end {
        let token = &args[i];
        if token.starts_with('-') {
            // Step over the value of an option written as two tokens
            let leaf = chain[chain.len() - 1];
            let takes_value = !token.contains('=')
                && leaf
                    .get_arguments()
                    .chain(
                        chain[..chain.len() - 1]
                            .iter()
                            .flat_map(|c| c.get_arguments().filter(|a| a.is_global_set())),
                    )
                    .any(|a| arg_named(a, token) && a.get_action().takes_values());
            if takes_value {
                i += 1;
            }
        } else if let Some(sub) = chain[chain.len() - 1].find_subcommand(token) {
            chain.push(sub);
        }
        i += 1;
    }
    let leaf = chain[chain.len() - 1];

    if leaf.get_arguments().any(|a| a.get_long() == Some("json")) {
        return args;
    }

    let format_arg = leaf.get_arguments().find(|a| {
        (arg_named(a, "--format") || arg_named(a, "--output"))
            && a.get_possible_values()
                .iter()
                .any(|v| v.get_name() == "json")
    });

    let mut out = Vec::with_capacity(args.len() + 1);
    match format_arg {
        Some(arg) => {
            let explicit = args[1..end].iter().any(|t| arg_named(arg, t));
            let long = arg.get_long().unwrap_or("format");
            // The option belongs to the leaf command, so it goes after it
            // (at the end, or just before a bare `--`), wherever --json was
            for (idx, token) in args.iter().enumerate() {
                if idx == end && !explicit {
                    out.push(format!("--{long}"));
                    out.push("json".to_string());
                }
                if !(idx < end && idx > 0 && token == "--json") {
                    out.push(token.clone());
                }
            }
            if end == args.len() && !explicit {
                out.push(format!("--{long}"));
                out.push("json".to_string());
            }
        }
        None => {
            let path: Vec<&str> = chain.iter().map(|c| c.get_name()).collect();
            eprintln!(
                "note: `{}` has no output format option; --json ignored",
                path.join(" ")
            );
            for (idx, token) in args.iter().enumerate() {
                if !(idx < end && idx > 0 && token == "--json") {
                    out.push(token.clone());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod json_flag_tests {
    use super::expand_json_flag;

    fn expand(args: &[&str]) -> Vec<String> {
        expand_json_flag(args.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn becomes_the_commands_own_format_option() {
        assert_eq!(
            expand(&["ethcli", "rpc", "block", "latest", "--json"]),
            ["ethcli", "rpc", "block", "latest", "--format", "json"]
        );
        // `tx` spells it --output
        assert_eq!(
            expand(&["ethcli", "--json", "tx", "0xabc"]),
            ["ethcli", "tx", "0xabc", "--output", "json"]
        );
        // An alias and a global option with a value before the subcommand
        assert_eq!(
            expand(&["ethcli", "--chain", "base", "acc", "txs", "0x1", "--json"]),
            ["ethcli", "--chain", "base", "acc", "txs", "0x1", "--output", "json"]
        );
    }

    #[test]
    fn explicit_format_wins_and_own_json_is_kept() {
        assert_eq!(
            expand(&["ethcli", "rpc", "block", "1", "-o", "table", "--json"]),
            ["ethcli", "rpc", "block", "1", "-o", "table"]
        );
        let health = ["ethcli", "endpoints", "health", "--json"];
        assert_eq!(expand(&health), health);
    }

    #[test]
    fn dropped_where_there_is_no_format_and_untouched_after_double_dash() {
        assert_eq!(
            expand(&["ethcli", "cast", "keccak", "x", "--json"]),
            ["ethcli", "cast", "keccak", "x"]
        );
        let passthrough = ["ethcli", "cast", "keccak", "--", "--json"];
        assert_eq!(expand(&passthrough), passthrough);
        let none = ["ethcli", "rpc", "block", "latest"];
        assert_eq!(expand(&none), none);
    }

    #[test]
    fn rewritten_lines_still_parse() {
        use clap::Parser;
        for args in [
            vec!["ethcli", "rpc", "call", "0x00", "0x", "--json"],
            vec!["ethcli", "rpc", "code", "0x00", "--summary", "--json"],
            vec![
                "ethcli",
                "contract",
                "call",
                "0x00",
                "totalSupply",
                "--json",
            ],
            vec!["ethcli", "rpc", "chain-id", "--json"],
            vec!["ethcli", "--json", "tx", "0xabc"],
        ] {
            let expanded = expand(&args);
            assert!(
                super::Cli::try_parse_from(&expanded).is_ok(),
                "failed to parse {expanded:?}"
            );
        }
    }

    #[test]
    fn command_tree_is_consistent() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetch historical logs from contracts
    #[command(visible_alias = "log")]
    Logs(Box<logs::LogsArgs>),

    /// Analyze transaction(s)
    #[command(visible_alias = "t")]
    Tx(tx::TxArgs),

    /// Account operations (balance, transactions, transfers)
    #[command(visible_alias = "acc")]
    Account {
        #[command(subcommand)]
        action: account::AccountCommands,
    },

    /// Address book (save and lookup addresses by label)
    #[command(visible_alias = "addr")]
    Address {
        #[command(subcommand)]
        action: address::AddressCommands,
    },

    /// Token blacklist (exclude spam/scam tokens from portfolio)
    #[command(visible_alias = "bl")]
    Blacklist(blacklist::BlacklistArgs),

    /// Contract operations (ABI, source, creation)
    #[command(visible_alias = "c")]
    Contract {
        #[command(subcommand)]
        action: contract::ContractCommands,
    },

    /// Token operations (info, holders, balance)
    #[command(visible_alias = "tok")]
    Token {
        #[command(subcommand)]
        action: token::TokenCommands,
    },

    /// Gas price oracle and estimates
    #[command(visible_alias = "g")]
    Gas {
        #[command(subcommand)]
        action: gas::GasCommands,
    },

    /// Signature lookup (function selectors, event topics)
    Sig {
        #[command(subcommand)]
        action: sig::SigCommands,
    },

    /// Manage RPC endpoints
    #[command(visible_alias = "ep")]
    Endpoints {
        #[command(subcommand)]
        action: endpoints::EndpointCommands,
    },

    /// Manage configuration
    #[command(visible_alias = "cfg")]
    Config {
        #[command(subcommand)]
        action: config::ConfigCommands,
    },

    /// Type conversions, hashing, and encoding utilities
    Cast {
        #[command(subcommand)]
        action: cast::CastCommands,
    },

    /// Direct RPC calls (call, block, storage, code)
    Rpc {
        #[command(subcommand)]
        action: rpc::RpcCommands,

        /// Custom RPC URL (overrides default)
        #[arg(long, global = true)]
        rpc_url: Option<String>,
    },

    /// ENS name resolution
    Ens {
        #[command(subcommand)]
        action: ens::EnsCommands,

        /// Custom RPC URL (overrides default)
        #[arg(long, global = true)]
        rpc_url: Option<String>,
    },

    /// Simulate transactions and trace execution
    Simulate {
        #[command(subcommand)]
        action: Box<simulate::SimulateCommands>,
    },

    /// Tenderly API (vnets, wallets, contracts, alerts, actions, networks)
    Tenderly {
        #[command(subcommand)]
        action: Box<tenderly::TenderlyCommands>,
    },

    /// Get aggregated token price from multiple sources
    #[command(visible_alias = "p")]
    Price(price::PriceArgs),

    /// Get aggregated portfolio/balances from multiple sources
    #[command(visible_alias = "pf")]
    Portfolio(portfolio::PortfolioArgs),

    /// Get aggregated NFT holdings from multiple sources
    #[command(visible_alias = "nft")]
    Nfts(nfts::NftsArgs),

    /// Check for available updates
    Update,

    /// Download and install the latest version
    Upgrade,

    /// Check configuration and endpoint health
    Doctor,

    /// Browse chains and discover RPC endpoints from chainlist.org
    #[command(visible_alias = "chains")]
    Chainlist {
        #[command(subcommand)]
        action: chainlist::ChainlistCommands,
    },

    /// Direct Alchemy API access
    Alchemy {
        #[command(subcommand)]
        action: alchemy::AlchemyCommands,
    },

    /// Direct CoinGecko API access
    Gecko {
        #[command(subcommand)]
        action: gecko::GeckoCommands,
    },

    /// GoPlus Security API (token, address, NFT, approval security)
    ///
    /// Free API for security analysis. Set GOPLUS_APP_KEY and GOPLUS_APP_SECRET
    /// for batch queries and higher rate limits.
    #[command(visible_alias = "gp")]
    Goplus(goplus::GoPlusArgs),

    /// Solodit vulnerability database (search smart contract security findings)
    ///
    /// Search and explore vulnerability findings from audits across the web3 ecosystem.
    /// Requires SOLODIT_API_KEY environment variable (get from solodit.cyfrin.io).
    #[command(visible_alias = "sld")]
    Solodit(solodit::SoloditArgs),

    /// Direct DefiLlama API access
    Llama {
        #[command(subcommand)]
        action: llama::LlamaCommands,
    },

    /// Direct Moralis API access
    Moralis {
        #[command(subcommand)]
        action: moralis::MoralisCommands,
    },

    /// Direct Dune Analytics API access
    Dune {
        #[command(subcommand)]
        action: dune_cli::DuneCommands,
    },

    /// Direct Curve Finance API access
    Curve {
        #[command(subcommand)]
        action: curve::CurveCommands,
    },

    /// Get aggregated DeFi yields from Curve and DefiLlama
    #[command(visible_alias = "y")]
    Yields(yields::YieldsArgs),

    /// Get aggregated swap quotes from multiple DEX aggregators
    #[command(visible_alias = "q")]
    Quote {
        #[command(subcommand)]
        action: quote::QuoteCommands,
    },

    /// Chainlink Data Streams (low-latency, cryptographically verifiable market data)
    ///
    /// Requires API credentials from https://chain.link/data-streams
    /// Users must accept Chainlink's Terms of Service: https://chainlinklabs.com/terms
    Chainlink {
        #[command(subcommand)]
        action: chainlink::ChainlinkCommands,
    },

    /// Exchange market data (Binance, Bitget, OKX, Bybit, Coinbase, Kraken, Hyperliquid, Lighter)
    #[command(visible_alias = "ccxt")]
    Cex {
        #[command(subcommand)]
        action: cex::CexCommands,
    },

    /// Uniswap V2/V3/V4 pool queries (on-chain and subgraph)
    ///
    /// On-chain queries (pool, liquidity, balance) require only an RPC URL.
    /// Subgraph queries (eth-price, top-pools, swaps, day-data) require a The Graph API key.
    #[command(visible_alias = "uni")]
    Uniswap {
        #[command(subcommand)]
        action: uniswap::UniswapCommands,
    },

    /// Yearn Kong GraphQL API (vaults, strategies, prices, TVL, reports)
    ///
    /// Access Yearn Finance vault and strategy data via the Kong API.
    /// No API key required.
    #[command(visible_alias = "yearn")]
    Kong {
        #[command(subcommand)]
        action: kong::KongCommands,
    },

    /// Direct 1inch DEX Aggregator API access
    ///
    /// Get swap quotes, prices, and liquidity sources from 1inch.
    /// Requires 1INCH_API_KEY environment variable.
    #[command(name = "1inch", visible_alias = "oneinch")]
    OneInch(oneinch::OneInchArgs),

    /// Direct De¹ (formerly OpenOcean) DEX Aggregator API access
    ///
    /// Get swap quotes and routes from De¹ (formerly OpenOcean).
    /// No API key required.
    #[command(visible_aliases = ["oo", "de1"])]
    OpenOcean(openocean::OpenOceanArgs),

    /// Direct KyberSwap DEX Aggregator API access
    ///
    /// Get swap routes and build transactions via KyberSwap.
    /// No API key required.
    #[command(visible_alias = "kyber")]
    KyberSwap(kyber::KyberArgs),

    /// Direct 0x Protocol DEX Aggregator API access
    ///
    /// Get swap quotes and prices from 0x.
    /// Requires ZEROX_API_KEY environment variable.
    #[command(name = "0x", visible_alias = "zerox")]
    ZeroX(zerox::ZeroXArgs),

    /// Direct CowSwap (CoW Protocol) API access
    ///
    /// MEV-protected trading via CoW Protocol batch auctions.
    /// No API key required.
    #[command(visible_alias = "cow")]
    CowSwap(cowswap::CowSwapArgs),

    /// Direct LI.FI Cross-Chain DEX Aggregator API access
    ///
    /// Cross-chain swaps and bridges via LI.FI.
    /// Optional LIFI_INTEGRATOR for analytics.
    #[command(name = "lifi", visible_alias = "li.fi")]
    LiFi(lifi::LiFiArgs),

    /// Direct Velora (ParaSwap) DEX Aggregator API access
    ///
    /// Get swap prices and build transactions via ParaSwap.
    /// Optional PARASWAP_API_KEY or VELORA_API_KEY for higher rate limits.
    #[command(visible_alias = "paraswap")]
    Velora(velora::VeloraArgs),

    /// Direct Enso Finance DeFi Aggregator API access
    ///
    /// DeFi shortcuts and bundled actions via Enso.
    /// Requires ENSO_API_KEY environment variable.
    Enso(enso::EnsoArgs),

    /// Direct Pyth Network Price Feeds API access
    ///
    /// Real-time price data from Pyth Network Hermes.
    /// Requires a Pyth API key (PYTH_API_KEY or `ethcli config set-pyth`);
    /// get one at https://pythdata.app.
    Pyth(pyth::PythArgs),

    /// Generate shell completions
    #[command(after_help = r#"EXAMPLES:
    # Generate Bash completions
    ethcli completions bash > ~/.local/share/bash-completion/completions/ethcli

    # Generate Zsh completions
    ethcli completions zsh > ~/.zfunc/_ethcli

    # Generate Fish completions
    ethcli completions fish > ~/.config/fish/completions/ethcli.fish
"#)]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

impl Cli {
    /// Generate shell completions to stdout
    pub fn generate_completions(shell: Shell) {
        clap_complete::generate(shell, &mut Cli::command(), "ethcli", &mut std::io::stdout());
    }
}

#[cfg(test)]
mod env_secret_tests {
    use super::Cli;
    use clap::CommandFactory;

    fn walk(cmd: &clap::Command, path: &str, offenders: &mut Vec<String>) {
        for arg in cmd.get_arguments() {
            if arg.get_env().is_some() && !arg.is_hide_env_values_set() {
                offenders.push(format!("{path} --{}", arg.get_id()));
            }
        }
        for sub in cmd.get_subcommands() {
            walk(sub, &format!("{path} {}", sub.get_name()), offenders);
        }
    }

    /// Every env-backed argument may carry a secret (API key, credentialed
    /// RPC URL). clap prints the live value in `--help` unless
    /// `hide_env_values = true` is set, so enforce it for the whole tree.
    #[test]
    fn all_env_args_hide_values() {
        let mut offenders = Vec::new();
        walk(&Cli::command(), "ethcli", &mut offenders);
        assert!(
            offenders.is_empty(),
            "env-backed args missing hide_env_values: {offenders:?}"
        );
    }
}

#[cfg(test)]
mod resolve_chain_tests {
    use super::resolve_chain_id;

    #[test]
    fn explicit_chain_id_wins() {
        assert_eq!(resolve_chain_id(Some(8453), "arbitrum").unwrap(), 8453);
    }

    #[test]
    fn falls_back_to_global_chain() {
        assert_eq!(resolve_chain_id(None, "ethereum").unwrap(), 1);
        assert_eq!(resolve_chain_id(None, "arbitrum").unwrap(), 42161);
        assert_eq!(resolve_chain_id(None, "base").unwrap(), 8453);
        assert_eq!(resolve_chain_id(None, "10").unwrap(), 10);
        assert!(resolve_chain_id(None, "not-a-chain").is_err());
    }
}
