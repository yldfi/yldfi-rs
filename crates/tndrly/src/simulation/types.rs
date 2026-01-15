//! Types for transaction simulation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request for simulating a single transaction
#[derive(Debug, Clone, Serialize)]
pub struct SimulationRequest {
    /// Network ID (e.g., "1" for mainnet)
    pub network_id: String,

    /// Sender address
    pub from: String,

    /// Recipient/contract address
    pub to: String,

    /// Encoded calldata
    pub input: String,

    /// Value in wei (hex format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<u64>,

    /// Gas price in wei (legacy transactions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,

    /// Max fee per gas in wei (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,

    /// Max priority fee per gas in wei (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,

    /// Transaction type (0 = legacy, 1 = access list, 2 = EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub transaction_type: Option<u8>,

    /// Transaction nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,

    /// Block number to simulate at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,

    /// Whether to save the simulation
    #[serde(default)]
    pub save: bool,

    /// Whether to save even if the simulation fails
    #[serde(default)]
    pub save_if_fails: bool,

    /// Simulation type: "full", "quick", or "abi"
    #[serde(default = "default_simulation_type")]
    pub simulation_type: String,

    /// State overrides for accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_objects: Option<HashMap<String, StateOverride>>,

    /// Block header overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_header: Option<BlockHeaderOverride>,

    /// Index of the transaction within the block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<u64>,

    /// Enable precise gas estimation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_gas: Option<bool>,

    /// Return access list in response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_access_list: Option<bool>,

    /// EIP-2930 access list for gas optimization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<AccessListEntry>>,

    // L2/Optimism parameters
    /// Latest L1 block number known to L2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l1_block_number: Option<u64>,

    /// Timestamp of the latest L1 block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l1_timestamp: Option<u64>,

    /// Address of the sender of the latest L1 to L2 message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l1_message_sender: Option<String>,

    /// Indicates if transaction is a deposit from L1 to L2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_tx: Option<bool>,

    /// Indicates if transaction is a system-level operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tx: Option<bool>,

    /// Amount of token minted within L2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<u64>,

    /// Desired amount to be minted (string for large values)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_to_mint: Option<String>,
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_simulation_type() -> String {
    "full".to_string()
}

impl SimulationRequest {
    /// Create a new simulation request
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            network_id: "1".to_string(),
            from: from.into(),
            to: to.into(),
            input: input.into(),
            value: None,
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            transaction_type: None,
            nonce: None,
            block_number: None,
            save: false,
            save_if_fails: false,
            simulation_type: "full".to_string(),
            state_objects: None,
            block_header: None,
            transaction_index: None,
            estimate_gas: None,
            generate_access_list: None,
            access_list: None,
            l1_block_number: None,
            l1_timestamp: None,
            l1_message_sender: None,
            deposit_tx: None,
            system_tx: None,
            mint: None,
            amount_to_mint: None,
        }
    }

    /// Set the network ID
    #[must_use]
    pub fn network_id(mut self, id: impl Into<String>) -> Self {
        self.network_id = id.into();
        self
    }

    /// Set the value in wei
    #[must_use]
    pub fn value(mut self, wei: impl Into<String>) -> Self {
        self.value = Some(wei.into());
        self
    }

    /// Set the value in wei from u128
    #[must_use]
    pub fn value_wei(mut self, wei: u128) -> Self {
        self.value = Some(format!("0x{:x}", wei));
        self
    }

    /// Set the gas limit
    #[must_use]
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas = Some(gas);
        self
    }

    /// Set the gas price (legacy transactions)
    #[must_use]
    pub fn gas_price(mut self, price: u64) -> Self {
        self.gas_price = Some(format!("{}", price));
        self
    }

    /// Set max fee per gas (EIP-1559)
    #[must_use]
    pub fn max_fee_per_gas(mut self, fee: impl Into<String>) -> Self {
        self.max_fee_per_gas = Some(fee.into());
        self.transaction_type = Some(2);
        self
    }

    /// Set max fee per gas from u64 (EIP-1559)
    #[must_use]
    pub fn max_fee_per_gas_wei(mut self, fee: u64) -> Self {
        self.max_fee_per_gas = Some(format!("{}", fee));
        self.transaction_type = Some(2);
        self
    }

    /// Set max priority fee per gas (EIP-1559)
    #[must_use]
    pub fn max_priority_fee_per_gas(mut self, fee: impl Into<String>) -> Self {
        self.max_priority_fee_per_gas = Some(fee.into());
        self.transaction_type = Some(2);
        self
    }

    /// Set max priority fee per gas from u64 (EIP-1559)
    #[must_use]
    pub fn max_priority_fee_per_gas_wei(mut self, fee: u64) -> Self {
        self.max_priority_fee_per_gas = Some(format!("{}", fee));
        self.transaction_type = Some(2);
        self
    }

    /// Set the transaction type (0 = legacy, 1 = access list, 2 = EIP-1559)
    #[must_use]
    pub fn transaction_type(mut self, tx_type: u8) -> Self {
        self.transaction_type = Some(tx_type);
        self
    }

    /// Set the nonce
    #[must_use]
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set the block number
    #[must_use]
    pub fn block_number(mut self, block: u64) -> Self {
        self.block_number = Some(block);
        self
    }

    /// Set whether to save the simulation on success
    ///
    /// Note: This only controls saving on successful simulations.
    /// Use `.save_if_fails(true)` to also save failed simulations.
    #[must_use]
    pub fn save(mut self, save: bool) -> Self {
        self.save = save;
        self
    }

    /// Set whether to save the simulation even if it fails
    ///
    /// When `true`, failed simulations will be saved to your project.
    /// This is useful for debugging transaction failures.
    #[must_use]
    pub fn save_if_fails(mut self, save_if_fails: bool) -> Self {
        self.save_if_fails = save_if_fails;
        self
    }

    /// Convenience method to save simulation regardless of success/failure
    ///
    /// Equivalent to calling `.save(true).save_if_fails(true)`.
    #[must_use]
    pub fn save_always(mut self) -> Self {
        self.save = true;
        self.save_if_fails = true;
        self
    }

    /// Set simulation type
    #[must_use]
    pub fn simulation_type(mut self, sim_type: SimulationType) -> Self {
        self.simulation_type = sim_type.as_str().to_string();
        self
    }

    /// Add state overrides
    #[must_use]
    pub fn state_overrides(mut self, overrides: HashMap<String, StateOverride>) -> Self {
        self.state_objects = Some(overrides);
        self
    }

    /// Add a balance override for an address
    #[must_use]
    pub fn override_balance(
        mut self,
        address: impl Into<String>,
        balance: impl Into<String>,
    ) -> Self {
        let address = address.into().to_lowercase();
        let overrides = self.state_objects.get_or_insert_with(HashMap::new);
        let entry = overrides.entry(address).or_default();
        entry.balance = Some(balance.into());
        self
    }

    /// Add a storage override
    #[must_use]
    pub fn override_storage(
        mut self,
        address: impl Into<String>,
        slot: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        let address = address.into().to_lowercase();
        let overrides = self.state_objects.get_or_insert_with(HashMap::new);
        let entry = overrides.entry(address).or_default();
        let storage = entry.storage.get_or_insert_with(HashMap::new);
        storage.insert(slot.into(), value.into());
        self
    }

    /// Add a code override
    #[must_use]
    pub fn override_code(mut self, address: impl Into<String>, code: impl Into<String>) -> Self {
        let address = address.into().to_lowercase();
        let overrides = self.state_objects.get_or_insert_with(HashMap::new);
        let entry = overrides.entry(address).or_default();
        entry.code = Some(code.into());
        self
    }

    /// Override block timestamp
    #[must_use]
    pub fn block_timestamp(mut self, timestamp: u64) -> Self {
        let header = self
            .block_header
            .get_or_insert_with(BlockHeaderOverride::default);
        header.timestamp = Some(format!("0x{:x}", timestamp));
        self
    }

    /// Set the transaction index within the block
    #[must_use]
    pub fn transaction_index(mut self, index: u64) -> Self {
        self.transaction_index = Some(index);
        self
    }

    /// Enable precise gas estimation
    #[must_use]
    pub fn estimate_gas(mut self, enable: bool) -> Self {
        self.estimate_gas = Some(enable);
        self
    }

    /// Generate access list in response
    #[must_use]
    pub fn generate_access_list(mut self, enable: bool) -> Self {
        self.generate_access_list = Some(enable);
        self
    }

    /// Set EIP-2930 access list for gas optimization
    #[must_use]
    pub fn access_list(mut self, list: Vec<AccessListEntry>) -> Self {
        self.access_list = Some(list);
        self.transaction_type = Some(1); // EIP-2930
        self
    }

    /// Add an access list entry
    #[must_use]
    pub fn add_access_list_entry(mut self, entry: AccessListEntry) -> Self {
        let list = self.access_list.get_or_insert_with(Vec::new);
        list.push(entry);
        self.transaction_type = Some(1); // EIP-2930
        self
    }

    // L2/Optimism builder methods

    /// Set L1 block number (for L2 simulations)
    #[must_use]
    pub fn l1_block_number(mut self, block: u64) -> Self {
        self.l1_block_number = Some(block);
        self
    }

    /// Set L1 timestamp (for L2 simulations)
    #[must_use]
    pub fn l1_timestamp(mut self, timestamp: u64) -> Self {
        self.l1_timestamp = Some(timestamp);
        self
    }

    /// Set L1 message sender (for L2 simulations)
    #[must_use]
    pub fn l1_message_sender(mut self, sender: impl Into<String>) -> Self {
        self.l1_message_sender = Some(sender.into());
        self
    }

    /// Mark as deposit transaction from L1 to L2
    #[must_use]
    pub fn deposit_tx(mut self, is_deposit: bool) -> Self {
        self.deposit_tx = Some(is_deposit);
        self
    }

    /// Mark as system-level transaction
    #[must_use]
    pub fn system_tx(mut self, is_system: bool) -> Self {
        self.system_tx = Some(is_system);
        self
    }

    /// Set mint amount (for L2 simulations)
    #[must_use]
    pub fn mint(mut self, amount: u64) -> Self {
        self.mint = Some(amount);
        self
    }

    /// Set amount to mint as string (for large values)
    #[must_use]
    pub fn amount_to_mint(mut self, amount: impl Into<String>) -> Self {
        self.amount_to_mint = Some(amount.into());
        self
    }
}

/// Simulation type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SimulationType {
    /// Full simulation with decoded results
    #[default]
    Full,
    /// Quick simulation with raw results
    Quick,
    /// ABI-only simulation
    Abi,
}

impl SimulationType {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Quick => "quick",
            Self::Abi => "abi",
        }
    }
}

impl std::fmt::Display for SimulationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SimulationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(Self::Full),
            "quick" => Ok(Self::Quick),
            "abi" => Ok(Self::Abi),
            _ => Err(format!(
                "Invalid simulation type: {}. Expected: full, quick, or abi",
                s
            )),
        }
    }
}

/// EIP-2930 access list entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListEntry {
    /// Contract address
    pub address: String,

    /// Storage keys to pre-warm
    #[serde(default)]
    pub storage_keys: Vec<String>,
}

impl AccessListEntry {
    /// Create a new access list entry
    #[must_use]
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            storage_keys: Vec::new(),
        }
    }

    /// Add storage keys
    #[must_use]
    pub fn storage_keys(mut self, keys: Vec<String>) -> Self {
        self.storage_keys = keys;
        self
    }

    /// Add a single storage key
    #[must_use]
    pub fn storage_key(mut self, key: impl Into<String>) -> Self {
        self.storage_keys.push(key.into());
        self
    }
}

/// State override for an account
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateOverride {
    /// Balance override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,

    /// Storage slot overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<HashMap<String, String>>,

    /// Code override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Block header overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderOverride {
    /// Timestamp override (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Block number override (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Block hash override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// State root override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_root: Option<String>,

    /// Parent hash override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_hash: Option<String>,

    /// SHA3 uncles override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha3_uncles: Option<String>,

    /// Transactions root override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions_root: Option<String>,

    /// Receipts root override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipts_root: Option<String>,

    /// Logs bloom override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_bloom: Option<String>,

    /// Difficulty override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,

    /// Gas limit override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,

    /// Gas used override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<String>,

    /// Base fee per gas override (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<String>,

    /// Miner/coinbase address override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miner: Option<String>,

    /// Extra data override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<String>,

    /// Mix hash override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix_hash: Option<String>,

    /// Nonce override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,

    /// Size override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Total difficulty override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_difficulty: Option<String>,
}

/// Response from a simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResponse {
    /// The simulation result
    pub simulation: Simulation,

    /// Transaction details
    #[serde(default)]
    pub transaction: Option<TransactionInfo>,

    /// Generated contracts (if any were created)
    #[serde(default)]
    pub contracts: Vec<serde_json::Value>,

    /// Generated access list (when generate_access_list: true was set in request)
    #[serde(default)]
    pub generated_access_list: Option<Vec<AccessListEntry>>,
}

/// Simulation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    /// Simulation ID
    pub id: String,

    /// Project ID
    #[serde(default)]
    pub project_id: Option<String>,

    /// Owner ID
    #[serde(default)]
    pub owner_id: Option<String>,

    /// Network ID
    pub network_id: String,

    /// Block number
    pub block_number: u64,

    /// Transaction index
    #[serde(default)]
    pub transaction_index: u64,

    /// Sender address
    pub from: String,

    /// Recipient address
    pub to: String,

    /// Input data
    pub input: String,

    /// Gas used
    pub gas: u64,

    /// Gas price
    #[serde(default)]
    pub gas_price: String,

    /// Gas used by simulation
    #[serde(default)]
    pub gas_used: u64,

    /// Value transferred
    pub value: String,

    /// Simulation status (true = success)
    pub status: bool,

    /// Execution queue origin
    #[serde(default)]
    pub queue_origin: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Whether simulation is shared
    #[serde(default)]
    pub shared: bool,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    /// Transaction hash
    #[serde(default)]
    pub hash: Option<String>,

    /// Block hash
    #[serde(default)]
    pub block_hash: Option<String>,

    /// Block number
    #[serde(default)]
    pub block_number: Option<u64>,

    /// Sender address
    #[serde(default)]
    pub from: Option<String>,

    /// Gas limit
    #[serde(default)]
    pub gas: Option<u64>,

    /// Gas price (can be string or number from API)
    #[serde(default)]
    pub gas_price: Option<serde_json::Value>,

    /// Gas used
    #[serde(default)]
    pub gas_used: Option<u64>,

    /// Input data
    #[serde(default)]
    pub input: Option<String>,

    /// Nonce
    #[serde(default)]
    pub nonce: Option<u64>,

    /// Recipient address
    #[serde(default)]
    pub to: Option<String>,

    /// Transaction index
    #[serde(default, rename = "index")]
    pub transaction_index: Option<u64>,

    /// Value
    #[serde(default)]
    pub value: Option<String>,

    /// Transaction status
    #[serde(default)]
    pub status: Option<bool>,

    /// Call trace
    #[serde(default)]
    pub call_trace: Option<serde_json::Value>,

    /// Transaction logs
    #[serde(default)]
    pub logs: Option<Vec<serde_json::Value>>,
}

/// Request for simulating a bundle of transactions
#[derive(Debug, Clone, Serialize)]
pub struct BundleSimulationRequest {
    /// List of simulations to run in sequence
    pub simulations: Vec<SimulationRequest>,

    /// Shared state overrides (applied to all simulations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_objects: Option<HashMap<String, StateOverride>>,
}

impl BundleSimulationRequest {
    /// Create a new bundle request
    #[must_use]
    pub fn new(simulations: Vec<SimulationRequest>) -> Self {
        Self {
            simulations,
            state_objects: None,
        }
    }

    /// Add shared state overrides
    #[must_use]
    pub fn state_overrides(mut self, overrides: HashMap<String, StateOverride>) -> Self {
        self.state_objects = Some(overrides);
        self
    }
}

/// Response from a bundle simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSimulationResponse {
    /// Results for each simulation in the bundle
    pub simulation_results: Vec<SimulationResponse>,
}

/// Summary of a saved simulation (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    /// Simulation ID
    pub id: String,

    /// Simulation status
    #[serde(default)]
    pub status: Option<bool>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Sender address
    #[serde(default)]
    pub from: Option<String>,

    /// Recipient address
    #[serde(default)]
    pub to: Option<String>,

    /// Network ID
    #[serde(default)]
    pub network_id: Option<String>,

    /// Block number
    #[serde(default)]
    pub block_number: Option<u64>,

    /// Whether shared
    #[serde(default)]
    pub shared: bool,
}

/// Response for listing simulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationListResponse {
    /// List of simulations
    pub simulations: Vec<SimulationSummary>,
}

/// Transaction trace request
#[derive(Debug, Clone, Serialize)]
pub struct TraceRequest {
    /// Transaction hash to trace
    pub hash: String,

    /// Network ID
    #[serde(default = "default_network_id")]
    pub network_id: String,
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_network_id() -> String {
    "1".to_string()
}

impl TraceRequest {
    /// Create a new trace request
    #[must_use]
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            network_id: "1".to_string(),
        }
    }

    /// Set the network ID
    #[must_use]
    pub fn network_id(mut self, id: impl Into<String>) -> Self {
        self.network_id = id.into();
        self
    }
}
