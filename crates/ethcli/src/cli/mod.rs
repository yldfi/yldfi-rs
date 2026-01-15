//! CLI command modules
//!
//! Each subcommand has its own module with argument definitions and handlers.

pub mod account;
pub mod address;
pub mod cast;
pub mod config;
pub mod contract;
pub mod doctor;
pub mod endpoints;
pub mod ens;
pub mod gas;
pub mod logs;
pub mod rpc;
pub mod sig;
pub mod simulate;
pub mod tenderly;
pub mod token;
pub mod tx;
pub mod update;

use clap::{Parser, Subcommand, ValueEnum};
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
    #[arg(long, env = "ETHERSCAN_API_KEY", global = true)]
    pub etherscan_key: Option<String>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress progress output
    #[arg(short, long, global = true)]
    pub quiet: bool,
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

    /// Check for updates and optionally install latest version
    Update {
        /// Automatically download and install the update
        #[arg(long)]
        install: bool,
    },

    /// Check configuration and endpoint health
    Doctor,
}
