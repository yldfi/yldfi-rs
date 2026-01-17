//! Configuration management commands

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize a new config file with template and example endpoints
    Init {
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
    },

    /// Show config file path
    Path,

    /// Set Etherscan API key
    SetEtherscanKey {
        /// API key
        key: String,
    },

    /// Set Tenderly API credentials
    SetTenderly {
        /// Tenderly access key
        #[arg(long)]
        key: String,

        /// Tenderly account slug
        #[arg(long)]
        account: String,

        /// Tenderly project slug
        #[arg(long)]
        project: String,
    },

    /// Set Alchemy API key
    SetAlchemy {
        /// Alchemy API key
        key: String,

        /// Default network (e.g., eth-mainnet, polygon-mainnet)
        #[arg(long)]
        network: Option<String>,
    },

    /// Set Moralis API key
    SetMoralis {
        /// Moralis API key
        key: String,
    },

    /// Set Chainlink Data Streams credentials
    SetChainlink {
        /// Chainlink API key (client ID)
        #[arg(long)]
        key: String,

        /// Chainlink user secret (client secret)
        #[arg(long)]
        secret: String,

        /// REST API URL (defaults to mainnet)
        #[arg(long)]
        rest_url: Option<String>,

        /// WebSocket URL (defaults to mainnet)
        #[arg(long)]
        ws_url: Option<String>,
    },

    /// Set Dune Analytics API key
    SetDune {
        /// Dune API key
        key: String,
    },

    /// Set Dune SIM API key (separate from Dune Analytics)
    SetDuneSim {
        /// Dune SIM API key
        key: String,
    },

    /// Add a debug-capable RPC URL (for debug_traceCall, etc.)
    AddDebugRpc {
        /// RPC URL with debug namespace enabled
        url: String,
    },

    /// Remove a debug-capable RPC URL
    RemoveDebugRpc {
        /// RPC URL to remove
        url: String,
    },

    /// Show current config
    Show,
}
