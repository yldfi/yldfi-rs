use crate::config::ConfigFile;
use clap::{Args, ValueEnum};
use tndrly::simulation::SimulationType as TndrlySimulationType;

/// Tenderly API credentials - shared across multiple subcommands
#[derive(Args, Clone, Debug)]
pub struct TenderlyArgs {
    /// Tenderly API key (or use TENDERLY_ACCESS_KEY env)
    #[arg(long, env = "TENDERLY_ACCESS_KEY")]
    pub tenderly_key: Option<String>,

    /// Tenderly account slug
    #[arg(long, env = "TENDERLY_ACCOUNT")]
    pub tenderly_account: Option<String>,

    /// Tenderly project slug
    #[arg(long, env = "TENDERLY_PROJECT")]
    pub tenderly_project: Option<String>,
}

impl TenderlyArgs {
    /// Get resolved Tenderly credentials (api_key, account, project) from args/env/config
    pub fn get_credentials(&self) -> anyhow::Result<(String, String, String)> {
        let config = ConfigFile::load_default().ok().flatten();
        let tenderly_config = config.as_ref().and_then(|c| c.tenderly.as_ref());

        let api_key = self
            .tenderly_key
            .clone()
            .or_else(|| tenderly_config.map(|t| t.access_key.clone()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Tenderly API key required. Set via --tenderly-key, TENDERLY_ACCESS_KEY env, or config file"
                )
            })?;

        let account = self
            .tenderly_account
            .clone()
            .or_else(|| tenderly_config.map(|t| t.account.clone()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Tenderly account required. Set via --tenderly-account, TENDERLY_ACCOUNT env, or config file"
                )
            })?;

        let project = self
            .tenderly_project
            .clone()
            .or_else(|| tenderly_config.map(|t| t.project.clone()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Tenderly project required. Set via --tenderly-project, TENDERLY_PROJECT env, or config file"
                )
            })?;

        Ok((api_key, account, project))
    }

    /// Create a tndrly::Client from args/env/config credentials
    pub fn create_client(&self) -> anyhow::Result<tndrly::Client> {
        let (api_key, account, project) = self.get_credentials()?;
        let config = tndrly::Config::new(api_key, account, project);
        tndrly::Client::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to create Tenderly client: {}", e))
    }
}

/// Output format for dry-run mode
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum DryRunFormat {
    /// Raw JSON request payload
    #[default]
    Json,
    /// curl command
    Curl,
    /// Node.js fetch snippet
    Fetch,
    /// PowerShell Invoke-RestMethod
    Powershell,
    /// Just the endpoint URL
    Url,
    /// Python requests
    Python,
    /// HTTPie command
    Httpie,
    /// wget command
    Wget,
    /// Go net/http
    Go,
    /// Rust reqwest
    Rust,
    /// Node.js axios
    Axios,
}

/// Simulation backend
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum SimulateVia {
    /// Use cast call --trace (default, works everywhere)
    #[default]
    Cast,
    /// Use Anvil fork (stateful, supports multiple txs)
    Anvil,
    /// Use Tenderly API (rich decoded output)
    Tenderly,
    /// Use debug_traceCall RPC (requires Geth-compatible node)
    Debug,
    /// Use trace_call RPC (requires Erigon/OpenEthereum-compatible node)
    Trace,
}

/// Simulation type (for Tenderly API)
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum SimulationType {
    /// Full simulation with decoded results
    #[default]
    Full,
    /// Quick simulation with less data (faster)
    Quick,
    /// ABI-only simulation (decode only)
    Abi,
}

impl SimulationType {
    /// Convert to tndrly SimulationType
    pub fn to_tndrly(self) -> TndrlySimulationType {
        match self {
            SimulationType::Full => TndrlySimulationType::Full,
            SimulationType::Quick => TndrlySimulationType::Quick,
            SimulationType::Abi => TndrlySimulationType::Abi,
        }
    }
}
