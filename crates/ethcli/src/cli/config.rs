//! Configuration management commands

use clap::Subcommand;
use std::io::{self, BufRead};

/// Read a value from stdin (first non-empty line, trimmed)
pub fn read_from_stdin() -> io::Result<String> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "No input provided on stdin",
    ))
}

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
    #[command(
        after_help = "To avoid exposing the key in shell history:\n  echo $KEY | ethcli config set-etherscan-key --stdin"
    )]
    SetEtherscanKey {
        /// API key (omit if using --stdin)
        key: Option<String>,

        /// Read API key from stdin (avoids shell history exposure)
        #[arg(long)]
        stdin: bool,
    },

    /// Set Tenderly API credentials
    #[command(
        after_help = "To avoid exposing credentials in shell history:\n  echo $KEY | ethcli config set-tenderly --stdin --account <acc> --project <proj>"
    )]
    SetTenderly {
        /// Tenderly access key (omit if using --stdin)
        #[arg(long)]
        key: Option<String>,

        /// Read access key from stdin
        #[arg(long)]
        stdin: bool,

        /// Tenderly account slug
        #[arg(long)]
        account: String,

        /// Tenderly project slug
        #[arg(long)]
        project: String,
    },

    /// Set Alchemy API key
    #[command(
        after_help = "To avoid exposing the key in shell history:\n  echo $KEY | ethcli config set-alchemy --stdin"
    )]
    SetAlchemy {
        /// Alchemy API key (omit if using --stdin)
        key: Option<String>,

        /// Read API key from stdin
        #[arg(long)]
        stdin: bool,

        /// Default network (e.g., eth-mainnet, polygon-mainnet)
        #[arg(long)]
        network: Option<String>,
    },

    /// Set Moralis API key
    #[command(
        after_help = "To avoid exposing the key in shell history:\n  echo $KEY | ethcli config set-moralis --stdin"
    )]
    SetMoralis {
        /// Moralis API key (omit if using --stdin)
        key: Option<String>,

        /// Read API key from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// Set Chainlink Data Streams credentials
    #[command(
        after_help = "To avoid exposing credentials in shell history:\n  echo \"$KEY:$SECRET\" | ethcli config set-chainlink --stdin"
    )]
    SetChainlink {
        /// Chainlink API key (client ID) - omit if using --stdin
        #[arg(long)]
        key: Option<String>,

        /// Chainlink user secret (client secret) - omit if using --stdin
        #[arg(long)]
        secret: Option<String>,

        /// Read key:secret from stdin (format: KEY:SECRET on one line)
        #[arg(long)]
        stdin: bool,

        /// REST API URL (defaults to mainnet)
        #[arg(long)]
        rest_url: Option<String>,

        /// WebSocket URL (defaults to mainnet)
        #[arg(long)]
        ws_url: Option<String>,
    },

    /// Set Dune Analytics API key
    #[command(
        after_help = "To avoid exposing the key in shell history:\n  echo $KEY | ethcli config set-dune --stdin"
    )]
    SetDune {
        /// Dune API key (omit if using --stdin)
        key: Option<String>,

        /// Read API key from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// Set Dune SIM API key (separate from Dune Analytics)
    #[command(
        after_help = "To avoid exposing the key in shell history:\n  echo $KEY | ethcli config set-dune-sim --stdin"
    )]
    SetDuneSim {
        /// Dune SIM API key (omit if using --stdin)
        key: Option<String>,

        /// Read API key from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// Set Solodit API key (smart contract vulnerability database)
    #[command(
        after_help = "To avoid exposing the key in shell history:\n  echo $KEY | ethcli config set-solodit --stdin\n\nGet an API key at: https://solodit.cyfrin.io (Profile > API Keys)"
    )]
    SetSolodit {
        /// Solodit API key (omit if using --stdin)
        key: Option<String>,

        /// Read API key from stdin
        #[arg(long)]
        stdin: bool,
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

    /// Validate config file syntax and structure
    Validate,
}
