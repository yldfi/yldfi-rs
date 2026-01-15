//! RPC endpoint management commands

use clap::Subcommand;

#[derive(Subcommand)]
pub enum EndpointCommands {
    /// List all configured endpoints
    List {
        /// Show only archive nodes
        #[arg(long)]
        archive: bool,

        /// Show only nodes with debug namespace
        #[arg(long)]
        debug: bool,

        /// Filter by chain (ethereum, polygon, arbitrum, etc.)
        #[arg(long)]
        chain: Option<String>,

        /// Show detailed information
        #[arg(short = 'd', long)]
        detailed: bool,
    },

    /// Add a new RPC endpoint (auto-optimizes)
    Add {
        /// RPC URL to add
        url: String,

        /// Chain this endpoint serves (auto-detected if not specified)
        #[arg(long)]
        chain: Option<String>,

        /// Skip optimization (just add with defaults)
        #[arg(long)]
        no_optimize: bool,
    },

    /// Remove an RPC endpoint
    Remove {
        /// RPC URL to remove
        url: String,
    },

    /// Optimize endpoint(s) by testing capabilities
    Optimize {
        /// RPC URL to optimize (if not specified, optimizes all)
        url: Option<String>,

        /// Optimize all endpoints
        #[arg(long)]
        all: bool,

        /// Filter by chain when using --all
        #[arg(long)]
        chain: Option<String>,

        /// Timeout for each test in seconds
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Test an endpoint for archive support (quick test)
    Test {
        /// RPC URL to test
        url: String,
    },

    /// Enable a disabled endpoint
    Enable {
        /// RPC URL to enable
        url: String,
    },

    /// Disable an endpoint (keeps config but excludes from pool)
    Disable {
        /// RPC URL to disable
        url: String,
    },
}
