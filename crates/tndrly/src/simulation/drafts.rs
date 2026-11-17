//! Types for simulation drafts and the quick simulation info endpoint

use serde::{Deserialize, Serialize};

/// Response of [`SimulationApi::get_info`](crate::simulation::SimulationApi::get_info):
/// the transaction, its block header and the state overrides of a saved
/// simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimulationInfoResponse {
    /// The simulated transaction
    #[serde(default)]
    pub transaction_info: Option<SimulationInfoTransaction>,

    /// Header of the block the transaction was simulated in
    #[serde(default)]
    pub block_header: Option<serde_json::Value>,

    /// State overrides applied, keyed by address
    #[serde(default)]
    pub state_overrides: Option<serde_json::Value>,

    /// Creation time (Unix milliseconds in Tenderly's example)
    #[serde(default)]
    pub created_at: Option<serde_json::Value>,
}

/// The transaction of a [`SimulationInfoResponse`]
///
/// Quantities are decimal strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimulationInfoTransaction {
    /// Simulation ID
    #[serde(default)]
    pub id: Option<String>,
    /// Network ID
    #[serde(default)]
    pub network_id: Option<String>,
    /// Block hash
    #[serde(default)]
    pub block_hash: Option<String>,
    /// Block number
    #[serde(default)]
    pub block_number: Option<String>,
    /// Index in the block
    #[serde(default)]
    pub transaction_index: Option<String>,
    /// Sender
    #[serde(default)]
    pub from: Option<String>,
    /// Recipient
    #[serde(default)]
    pub to: Option<String>,
    /// Calldata
    #[serde(default)]
    pub input: Option<String>,
    /// Nonce
    #[serde(default)]
    pub nonce: Option<String>,
    /// Value in wei
    #[serde(default)]
    pub value: Option<String>,
    /// Decoded call (e.g. `method_name`)
    #[serde(default)]
    pub decoded_info: Option<serde_json::Value>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
    /// Gas price
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Gas used
    #[serde(default)]
    pub gas_used: Option<String>,
    /// Access list
    #[serde(default)]
    pub access_list: Option<serde_json::Value>,
    /// `true` if the transaction succeeded
    #[serde(default)]
    pub status: Option<bool>,
    /// Error message, empty on success
    #[serde(default)]
    pub error_message: Option<String>,
    /// Addresses touched
    #[serde(default)]
    pub addresses: Option<Vec<String>>,
    /// Contract IDs touched (`eth:{network}:{address}`)
    #[serde(default)]
    pub contract_ids: Option<Vec<String>>,
}

/// Request to store a Simulator setup as a draft
///
/// The draft only prefills the dashboard's Simulator form; nothing runs until
/// someone opens the link and clicks **Simulate**.
///
/// ```
/// use tndrly::simulation::{CreateSimulationDraftRequest, SimulationDraftPayload, SimulationDraftRow};
///
/// let row = SimulationDraftRow::new("0xdac17f958d2ee523a2206206994597c13d831ec7")
///     .from("0xab5801a7d398351b8be11c439e05c5b3259aec9b")
///     .raw_input("0xa9059cbb");
/// let request = CreateSimulationDraftRequest::new(SimulationDraftPayload::new("1").row(row));
/// ```
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateSimulationDraftRequest {
    /// The Simulator setup
    pub payload: SimulationDraftPayload,

    /// `true` lets any signed-in Tenderly user open the draft in a project of
    /// their choice; otherwise only members of this project can
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared: Option<bool>,
}

impl CreateSimulationDraftRequest {
    /// A project-scoped draft
    #[must_use]
    pub fn new(payload: SimulationDraftPayload) -> Self {
        Self {
            payload,
            shared: None,
        }
    }

    /// Make the draft openable by any signed-in Tenderly user
    #[must_use]
    pub fn shared(mut self, shared: bool) -> Self {
        self.shared = Some(shared);
        self
    }

    /// Check that the payload has at least one call
    ///
    /// # Errors
    /// If `payload.rows` is empty.
    pub fn validate(&self) -> Result<(), String> {
        if self.payload.rows.is_empty() {
            return Err("A simulation draft needs at least one row. Use .row() to add one.".into());
        }
        Ok(())
    }
}

/// A Simulator setup: one call, or several run as a bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimulationDraftPayload {
    /// Payload version (2)
    pub v: u32,

    /// Network to simulate on
    pub network: Option<SimulationDraftNetwork>,

    /// The calls, in order
    pub rows: Vec<SimulationDraftRow>,
}

impl SimulationDraftPayload {
    /// A version-2 payload on this network; add calls with [`row`](Self::row)
    pub fn new(network_id: impl Into<String>) -> Self {
        Self {
            v: 2,
            network: Some(SimulationDraftNetwork {
                id: network_id.into(),
            }),
            rows: Vec::new(),
        }
    }

    /// Add a call
    #[must_use]
    pub fn row(mut self, row: SimulationDraftRow) -> Self {
        self.rows.push(row);
        self
    }
}

/// Network of a [`SimulationDraftPayload`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimulationDraftNetwork {
    /// Network ID, e.g. `"1"`
    pub id: String,
}

/// One call of a [`SimulationDraftPayload`]
///
/// Fields the spec allows as either a string or a number are kept as JSON.
/// The builders cover common fields; set the others directly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SimulationDraftRow {
    /// Contract called
    #[serde(default)]
    pub contract_address: String,

    /// `raw` (calldata in `raw_function_input`) or `decoded`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_data_type: Option<String>,

    /// Raw calldata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_function_input: Option<String>,

    /// Function to call (`name`, `selector`, `signature`) for decoded input
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_function: Option<serde_json::Value>,

    /// Function arguments (array or object) for decoded input
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_inputs: Option<serde_json::Value>,

    /// ABI to decode with
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_abi_import: Option<String>,

    /// Sender
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    /// Gas limit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas: Option<serde_json::Value>,

    /// Gas price
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<serde_json::Value>,

    /// Value in wei
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Block number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<serde_json::Value>,

    /// Index in the block
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_index: Option<serde_json::Value>,

    /// Simulate at the end of the block
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_of_block: Option<bool>,

    /// Simulate on the pending block
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_pending_block: Option<bool>,

    /// OP-stack deposit transaction
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deposit_tx: Option<bool>,

    /// Amount to mint (deposit transactions)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,

    /// Block header overrides (`number`, `timestamp`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_header_overrides: Option<serde_json::Value>,

    /// State overrides
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_overrides: Option<serde_json::Value>,

    /// Access list
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_list: Option<serde_json::Value>,

    /// Token balances to fund
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fund_address: Option<serde_json::Value>,

    /// Edited contract source
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_source: Option<serde_json::Value>,

    /// Whether the contract source was edited
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_source_edited: Option<bool>,
}

impl SimulationDraftRow {
    /// A call to this contract
    pub fn new(contract_address: impl Into<String>) -> Self {
        Self {
            contract_address: contract_address.into(),
            ..Self::default()
        }
    }

    /// Set the sender
    #[must_use]
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Use raw calldata
    #[must_use]
    pub fn raw_input(mut self, calldata: impl Into<String>) -> Self {
        self.input_data_type = Some("raw".to_string());
        self.raw_function_input = Some(calldata.into());
        self
    }

    /// Set the value in wei
    #[must_use]
    pub fn value(mut self, wei: impl Into<String>) -> Self {
        self.value = Some(serde_json::Value::String(wei.into()));
        self
    }

    /// Set the gas limit
    #[must_use]
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Simulate at this block
    #[must_use]
    pub fn block(mut self, block: u64) -> Self {
        self.block = Some(block.into());
        self
    }
}

/// Response of creating a simulation draft
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimulationDraftCreated {
    /// Draft ID
    pub resource_id: String,
}

impl SimulationDraftCreated {
    /// Dashboard link that opens the Simulator prefilled with the draft
    #[must_use]
    pub fn dashboard_url(&self) -> String {
        format!(
            "https://dashboard.tenderly.co/simulator/new?draftId={}",
            crate::client::encode_path_segment(&self.resource_id)
        )
    }
}

/// A stored simulation draft
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SimulationDraft {
    /// The Simulator setup
    pub payload: SimulationDraftPayload,

    /// Origin account (project-scoped drafts the caller can view only)
    #[serde(default)]
    pub account_id: Option<String>,

    /// Origin project (project-scoped drafts the caller can view only)
    #[serde(default)]
    pub project_id: Option<String>,
}
