//! Chainlist - browse and search EVM chain information
//!
//! Fetches chain data from chainlist.org/rpcs.json to discover
//! chains and their public RPC endpoints.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ChainlistCommands {
    /// Search chains by name or chain ID
    Search {
        /// Search query (chain name, short name, or chain ID)
        #[arg(value_name = "QUERY")]
        query: String,

        /// Show testnet chains too
        #[arg(long)]
        testnets: bool,
    },

    /// List public RPC endpoints for a chain
    Rpcs {
        /// Chain name or ID (e.g., "gnosis" or "100")
        #[arg(value_name = "CHAIN")]
        chain: String,
    },

    /// Add public RPC endpoints for a chain from chainlist.org
    Add {
        /// Chain name or ID (e.g., "gnosis" or "100")
        #[arg(value_name = "CHAIN")]
        chain: String,

        /// Maximum number of endpoints to add
        #[arg(long, default_value = "5")]
        max: usize,
    },

    /// List all known chains (from yldfi-common)
    List {
        /// Show testnet chains too
        #[arg(long)]
        testnets: bool,
    },
}
