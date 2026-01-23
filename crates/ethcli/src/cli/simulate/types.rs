use crate::config::ConfigFile;
use clap::{Args, ValueEnum};
use tndrly::simulation::SimulationType as TndrlySimulationType;

/// Alchemy API credentials - shared across multiple subcommands
#[derive(Args, Clone, Debug)]
pub struct AlchemyArgs {
    /// Alchemy API key (or use ALCHEMY_API_KEY env)
    #[arg(long, env = "ALCHEMY_API_KEY")]
    pub alchemy_key: Option<String>,

    /// Alchemy network (e.g., eth-mainnet, polygon-mainnet)
    #[arg(long, default_value = "eth-mainnet")]
    pub alchemy_network: String,
}

impl AlchemyArgs {
    /// Get resolved Alchemy API key from args/env/config
    pub fn get_api_key(&self) -> anyhow::Result<String> {
        use secrecy::ExposeSecret;

        let config = ConfigFile::load_default().ok().flatten();
        let alchemy_config = config.as_ref().and_then(|c| c.alchemy.as_ref());

        self.alchemy_key
            .clone()
            .or_else(|| alchemy_config.map(|a| a.api_key.expose_secret().to_string()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Alchemy API key required. Set via --alchemy-key, ALCHEMY_API_KEY env, or config file"
                )
            })
    }

    /// Get the network, falling back to config if not specified
    pub fn get_network(&self) -> String {
        let config = ConfigFile::load_default().ok().flatten();
        let alchemy_config = config.as_ref().and_then(|c| c.alchemy.as_ref());

        // Use CLI arg if different from default, otherwise check config
        if self.alchemy_network != "eth-mainnet" {
            self.alchemy_network.clone()
        } else {
            alchemy_config
                .and_then(|a| a.default_network.clone())
                .unwrap_or_else(|| self.alchemy_network.clone())
        }
    }

    /// Parse network string to alcmy::Network enum
    pub fn parse_network(network: &str) -> alcmy::Network {
        match network.to_lowercase().as_str() {
            "eth-mainnet" | "ethereum" | "mainnet" => alcmy::Network::EthMainnet,
            "eth-sepolia" | "sepolia" => alcmy::Network::EthSepolia,
            "eth-holesky" | "holesky" => alcmy::Network::EthHolesky,
            "polygon-mainnet" | "polygon" | "matic" => alcmy::Network::PolygonMainnet,
            "polygon-amoy" | "amoy" => alcmy::Network::PolygonAmoy,
            "arb-mainnet" | "arbitrum" => alcmy::Network::ArbitrumMainnet,
            "arb-sepolia" => alcmy::Network::ArbitrumSepolia,
            "opt-mainnet" | "optimism" => alcmy::Network::OptMainnet,
            "opt-sepolia" => alcmy::Network::OptSepolia,
            "base-mainnet" | "base" => alcmy::Network::BaseMainnet,
            "base-sepolia" => alcmy::Network::BaseSepolia,
            "zksync-mainnet" | "zksync" => alcmy::Network::ZksyncMainnet,
            "zksync-sepolia" => alcmy::Network::ZksyncSepolia,
            "linea-mainnet" | "linea" => alcmy::Network::LineaMainnet,
            "scroll-mainnet" | "scroll" => alcmy::Network::ScrollMainnet,
            "blast-mainnet" | "blast" => alcmy::Network::BlastMainnet,
            "mantle-mainnet" | "mantle" => alcmy::Network::MantleMainnet,
            "bnb-mainnet" | "bnb" | "bsc" => alcmy::Network::Bnb,
            "avax-mainnet" | "avalanche" | "avax" => alcmy::Network::Avalanche,
            "fantom-mainnet" | "fantom" | "ftm" => alcmy::Network::Fantom,
            "gnosis-mainnet" | "gnosis" | "xdai" => alcmy::Network::Gnosis,
            _ => alcmy::Network::EthMainnet,
        }
    }

    /// Create an Alchemy client from args/env/config credentials
    pub fn create_client(&self) -> anyhow::Result<alcmy::Client> {
        let api_key = self.get_api_key()?;
        let network = self.get_network();
        let network_enum = Self::parse_network(&network);
        alcmy::Client::new(api_key, network_enum).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

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
        use secrecy::ExposeSecret;

        let config = ConfigFile::load_default().ok().flatten();
        let tenderly_config = config.as_ref().and_then(|c| c.tenderly.as_ref());

        let api_key = self
            .tenderly_key
            .clone()
            .or_else(|| tenderly_config.map(|t| t.access_key.expose_secret().to_string()))
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
    /// Use Alchemy Simulation API (asset changes, decoded traces)
    Alchemy,
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
